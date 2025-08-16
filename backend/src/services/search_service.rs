use crate::models::{Caption, SearchResult};
use anyhow::{Context, Result};
use elasticsearch::{Elasticsearch, SearchParts};
use log::{debug, error};
use serde_json::{json, Map, Value};

/// Fragmenting
const DEFAULT_FRAGMENT_SIZE: usize = 400;
const DEFAULT_NUM_FRAGMENTS: usize = 1;
const DEFAULT_BOUNDARY_MAX_SCAN: usize = 50;
const DEFAULT_NO_MATCH_SIZE: usize = 250;

/// Captions to include before/after the anchor
const DEFAULT_NEIGHBORS_BEFORE: usize = 2;
const DEFAULT_NEIGHBORS_AFTER: usize = 2;

/// Max combined snippet length
const MAX_COMBINED_CHARS: usize = 800;

/// HTML tags for highlighting
const PRE_TAG: &str = "<strong>";
const POST_TAG: &str = "</strong>";

pub async fn search_captions(
    es_client: &Elasticsearch,
    query_string: &str,
    from: usize,
    size: usize,
) -> Result<Vec<SearchResult>> {
    let query_body = build_search_query(
        query_string,
        from,
        size,
        DEFAULT_FRAGMENT_SIZE,
        DEFAULT_NUM_FRAGMENTS,
    );

    let response = es_client
        .search(SearchParts::Index(&["youtube_captions"]))
        .from(from as i64)
        .size(size as i64)
        .body(query_body)
        .send()
        .await
        .context("Elasticsearch search request failed")?
        .json::<Value>()
        .await
        .context("Failed to parse Elasticsearch search response as JSON")?;

    // Parse base results with highlighted anchor snippet
    let mut results = process_search_response(response).await;

    // For each result, fetch neighbors and build the combined snippet
    for res in results.iter_mut() {
        let (prev, next) = fetch_neighbors_for_hit(
            es_client,
            &res.video_id,
            res.start_time,
            res.end_time,
            DEFAULT_NEIGHBORS_BEFORE,
            DEFAULT_NEIGHBORS_AFTER,
        )
        .await
        .unwrap_or_default();

        // Build neighbor text blocks
        let prev_text = join_neighbor_text_prev(&prev);
        let next_text = join_neighbor_text_next(&next);

        // Combine: prev + highlighted anchor (res.snippet_html) + next
        let combined = stitch_with_neighbors(&prev_text, &res.snippet_html, &next_text);

        // Trim to a max length while keeping the highlight in view
        res.snippet_html =
            truncate_around_highlight(&combined, MAX_COMBINED_CHARS, PRE_TAG, POST_TAG);

        res.start_time = prev.first().map(|c| c.start_time).unwrap_or(res.start_time);
    }

    Ok(results)
}

#[allow(dead_code)]
fn build_search_query(
    query_string: &str,
    from: usize,
    size: usize,
    fragment_size: usize,
    number_of_fragments: usize,
) -> Value {
    // Primary query: multi_match with operator "and" for intent-like matches
    let main_query = json!({
        "multi_match": {
            "query": query_string,
            "fields": ["text"],
            "operator": "and",
            "type": "best_fields"
        }
    });

    json!({
        "from": from,
        "size": size,
        "query": main_query,
        "_source": ["video_id", "text", "start_time", "end_time"],
        "highlight": {
            "pre_tags": [PRE_TAG],
            "post_tags": [POST_TAG],
            "fields": {
                "text": {
                    "type": "unified",
                    "number_of_fragments": number_of_fragments,
                    "fragment_size": fragment_size,
                    "order": "score",
                    "boundary_scanner": "sentence",
                    "boundary_max_scan": DEFAULT_BOUNDARY_MAX_SCAN,
                    "no_match_size": DEFAULT_NO_MATCH_SIZE,
                    // >>> Keep highlight logic aligned with the main query
                    "highlight_query": main_query,
                    "fragmenter": "simple", // For more predictable results
                    "max_analyzed_offset": 1000000 // Allow highlighting in longer texts
                }
            },
            "require_field_match": true
        },
        "sort": [
            { "_score": { "order": "desc" } },
            { "start_time": { "order": "asc" } }
        ]
    })
}

fn build_exact_search_query(query_string: &str, from: usize, size: usize) -> Value {
    // Optional alternative: enforce phrase matching for tighter highlights
    let main_query = json!({
        "match_phrase": { "text": { "query": query_string } }
    });

    json!({
        "from": from,
        "size": size,
        "query": main_query,
        "_source": ["video_id", "text", "start_time", "end_time"],
        "highlight": {
            "pre_tags": [PRE_TAG],
            "post_tags": [POST_TAG],
            "fields": {
                "text": {
                    "type": "unified",
                    "number_of_fragments": DEFAULT_NUM_FRAGMENTS,
                    "fragment_size": DEFAULT_FRAGMENT_SIZE,
                    "order": "score",
                    "boundary_scanner": "sentence",
                    "boundary_max_scan": DEFAULT_BOUNDARY_MAX_SCAN,
                    "no_match_size": DEFAULT_NO_MATCH_SIZE,
                    "highlight_query": main_query,
                    "fragmenter": "simple",
                    "max_analyzed_offset": 1000000
                }
            },
            "require_field_match": true
        },
        "sort": [
            { "_score": { "order": "desc" } },
            { "start_time": { "order": "asc" } }
        ]
    })
}

fn parse_search_result(source: &Map<String, Value>, hit: &Value) -> SearchResult {
    let video_id = source
        .get("video_id")
        .and_then(|v| v.as_str())
        .unwrap_or_default()
        .to_string();

    let start_time = source
        .get("start_time")
        .and_then(|v| v.as_f64())
        .unwrap_or_default();

    let end_time = source
        .get("end_time")
        .and_then(|v| v.as_f64())
        .unwrap_or_default();

    // Prefer highlight if present; fallback to the raw text
    let snippet_html = hit
        .get("highlight")
        .and_then(|hl| hl.get("text"))
        .and_then(|arr| arr.as_array())
        .and_then(|arr| arr.first())
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .or_else(|| {
            // Fallback: if no highlight, use the full raw text
            source
                .get("text")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
        })
        .unwrap_or_default();

    SearchResult {
        video_id,
        start_time,
        end_time,
        snippet_html,
    }
}

async fn process_search_response(response: Value) -> Vec<SearchResult> {
    let mut out = Vec::new();

    let hits = response
        .get("hits")
        .and_then(|h| h.get("hits"))
        .and_then(|arr| arr.as_array())
        .cloned()
        .unwrap_or_default();

    for hit in hits {
        let source = hit
            .get("_source")
            .and_then(|s| s.as_object())
            .cloned()
            .unwrap_or_else(|| Map::new());

        let result = parse_search_result(&source, &hit);
        out.push(result);
    }

    out
}

async fn fetch_neighbors_for_hit(
    es_client: &Elasticsearch,
    video_id: &str,
    anchor_start_time: f64,
    anchor_end_time: f64,
    before: usize,
    after: usize,
) -> Result<(Vec<Caption>, Vec<Caption>)> {
    // Strategy: Find all captions for the video around the anchor timeframe
    // Then slice out the correct neighbors to avoid gaps

    // Calculate a reasonable window around the anchor to fetch captions
    // Assuming captions are typically 2-6 seconds each, fetch a wider window
    let window_seconds = ((before + after) as f64 * 6.0).max(30.0); // At least 30 seconds window
    let start_window = anchor_start_time - window_seconds;
    let end_window = anchor_end_time + window_seconds;

    let window_query = json!({
        "_source": ["text", "start_time", "end_time"],
        "size": ((before + after + 1) * 3).max(50), // Fetch more than needed to ensure coverage
        "sort": [{ "start_time": { "order": "asc" } }],
        "query": {
            "bool": {
                "filter": [
                    { "term": { "video_id": video_id }},
                    { "range": { "start_time": { "gte": start_window, "lte": end_window } } }
                ]
            }
        }
    });

    let resp = es_client
        .search(SearchParts::Index(&["youtube_captions"]))
        .body(window_query)
        .send()
        .await
        .context("Elasticsearch window search failed")?
        .json::<Value>()
        .await
        .context("Failed to parse window response JSON")?;

    let all_captions = parse_neighbor_hits(resp);

    // Find the anchor caption in the results
    let mut anchor_index = None;
    for (i, caption) in all_captions.iter().enumerate() {
        // Match by start_time (since that's unique per video)
        if (caption.start_time - anchor_start_time).abs() < 0.1 {
            anchor_index = Some(i);
            break;
        }
    }

    let (prev_captions, next_captions) = match anchor_index {
        Some(anchor_idx) => {
            // Get neighbors before the anchor
            let prev_start = if anchor_idx >= before {
                anchor_idx - before
            } else {
                0
            };
            let prev_captions = all_captions[prev_start..anchor_idx].to_vec();

            // Get neighbors after the anchor
            let next_start = anchor_idx + 1;
            let next_end = (next_start + after).min(all_captions.len());
            let next_captions = all_captions[next_start..next_end].to_vec();

            (prev_captions, next_captions)
        }
        None => {
            // Fallback: couldn't find exact anchor, split around the time
            let mut prev_captions = Vec::new();
            let mut next_captions = Vec::new();

            for caption in all_captions {
                if caption.start_time < anchor_start_time {
                    prev_captions.push(caption);
                } else if caption.start_time > anchor_end_time {
                    next_captions.push(caption);
                }
            }

            // Take the last N previous and first N next
            if prev_captions.len() > before {
                prev_captions = prev_captions[prev_captions.len() - before..].to_vec();
            }
            if next_captions.len() > after {
                next_captions.truncate(after);
            }

            (prev_captions, next_captions)
        }
    };

    debug!(
        "Found {} prev neighbors and {} next neighbors for video {} at {}s",
        prev_captions.len(),
        next_captions.len(),
        video_id,
        anchor_start_time
    );

    Ok((prev_captions, next_captions))
}

fn parse_neighbor_hits(resp: Value) -> Vec<Caption> {
    resp.get("hits")
        .and_then(|h| h.get("hits"))
        .and_then(|arr| arr.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|hit| {
                    let src = hit.get("_source")?.as_object()?;
                    let text = src.get("text")?.as_str()?.to_string();
                    let start_time = src
                        .get("start_time")
                        .and_then(|v| v.as_f64())
                        .unwrap_or(0.0);
                    let end_time = src.get("end_time").and_then(|v| v.as_f64()).unwrap_or(0.0);

                    // Extract video_id from the document or use placeholder
                    let video_id = src
                        .get("video_id")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string();

                    Some(Caption {
                        video_id,
                        text,
                        start_time,
                        end_time,
                    })
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default()
}

fn join_neighbor_text_prev(prev: &Vec<Caption>) -> String {
    // prev is already in chronological order from the window query
    let mut texts: Vec<&str> = prev.iter().map(|d| d.text.as_str()).collect();
    texts.retain(|s| !s.trim().is_empty());
    texts.join(" ")
}

fn join_neighbor_text_next(next: &Vec<Caption>) -> String {
    // next is already in chronological order from the window query
    let mut texts: Vec<&str> = next.iter().map(|d| d.text.as_str()).collect();
    texts.retain(|s| !s.trim().is_empty());
    texts.join(" ")
}

fn stitch_with_neighbors(prev: &str, anchor_html: &str, next: &str) -> String {
    let mut parts = Vec::new();
    if !prev.is_empty() {
        parts.push(prev.trim().to_string());
    }
    parts.push(anchor_html.trim().to_string());
    if !next.is_empty() {
        parts.push(next.trim().to_string());
    }
    parts.join(" ")
}

fn truncate_around_highlight(s: &str, max_chars: usize, pre_tag: &str, post_tag: &str) -> String {
    if s.chars().count() <= max_chars {
        return s.to_string();
    }

    // Try to find the highlighted region
    if let Some(pre_idx) = s.find(pre_tag) {
        let after_pre = &s[pre_idx + pre_tag.len()..];
        if let Some(rel_post_idx) = after_pre.find(post_tag) {
            let hl_start = pre_idx;
            let hl_end = pre_idx + pre_tag.len() + rel_post_idx + post_tag.len();

            let total_chars = s.chars().count();
            let s_chars: Vec<char> = s.chars().collect();

            // Calculate character positions
            let hl_start_chars = s[..hl_start].chars().count();
            let hl_chars = s[hl_start..hl_end].chars().count();
            let hl_end_chars = hl_start_chars + hl_chars;

            // Be more generous with context around the highlight
            let remaining = max_chars.saturating_sub(hl_chars);
            let side = remaining / 2;
            let extra_buffer = 20; // Extra characters to ensure we don't cut mid-word

            let mut prefix_take = (side + extra_buffer).min(hl_start_chars);
            let mut suffix_take = (side + extra_buffer).min(total_chars - hl_end_chars);

            // Adjust if we have room to expand one side
            let total_take = prefix_take + hl_chars + suffix_take;
            if total_take < max_chars {
                let extra = max_chars - total_take;
                if prefix_take < hl_start_chars {
                    let can_expand_prefix = (hl_start_chars - prefix_take).min(extra / 2);
                    prefix_take += can_expand_prefix;
                }
                if suffix_take < (total_chars - hl_end_chars) {
                    let can_expand_suffix =
                        (total_chars - hl_end_chars - suffix_take).min(extra / 2);
                    suffix_take += can_expand_suffix;
                }
            }

            let start_char = hl_start_chars - prefix_take;
            let end_char = (hl_end_chars + suffix_take).min(total_chars);

            // Try to break at word boundaries
            let mut actual_start = start_char;
            let mut actual_end = end_char;

            // Find word boundary for start (look backwards for space)
            if start_char > 0 {
                for i in (0..=start_char.min(start_char + 20)).rev() {
                    if i < s_chars.len() && (s_chars[i] == ' ' || s_chars[i] == '\n') {
                        actual_start = i + 1;
                        break;
                    }
                }
            }

            // Find word boundary for end (look forwards for space)
            if end_char < total_chars {
                for i in end_char..=(end_char + 20).min(total_chars - 1) {
                    if i < s_chars.len() && (s_chars[i] == ' ' || s_chars[i] == '\n') {
                        actual_end = i;
                        break;
                    }
                }
            }

            let trimmed: String = s_chars[actual_start..actual_end].iter().collect();

            let mut with_ellipses = trimmed;
            if actual_start > 0 {
                with_ellipses = format!("… {}", with_ellipses.trim_start());
            }
            if actual_end < total_chars {
                with_ellipses = format!("{} …", with_ellipses.trim_end());
            }

            return with_ellipses;
        }
    }

    // Fallback: simple head truncation with ellipsis
    let prefix: String = s.chars().take(max_chars.saturating_sub(2)).collect();
    format!("{} …", prefix.trim_end())
}
