use crate::models::{Caption, SearchResult};
use anyhow::{Context, Result};
use elasticsearch::{Elasticsearch, SearchParts};
use log::debug;
use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value};

/// Fragmenting
const DEFAULT_FRAGMENT_SIZE: usize = 400;
const DEFAULT_NUM_FRAGMENTS: usize = 1;
const DEFAULT_BOUNDARY_MAX_SCAN: usize = 50;
const DEFAULT_NO_MATCH_SIZE: usize = 250;

/// Neighbor settings
const DEFAULT_NEIGHBORS_BEFORE: usize = 2;
const DEFAULT_NEIGHBORS_AFTER: usize = 2;
const MAX_COMBINED_CHARS: usize = 800;

/// HTML tags for highlighting
const PRE_TAG: &str = "<strong>";
const POST_TAG: &str = "</strong>";

/// Search options for different search strategies
#[derive(Debug, Clone)]
pub struct SearchOptions {
    pub search_type: SearchType,
    pub fuzzy_distance: Option<String>, // "AUTO", "1", "2", etc.
    pub sort_by: SortBy,
    pub sort_order: SortOrder,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SortBy {
    Relevance, // default sort after search-score
    UploadDate,
    Duration,
    Views,
    Likes,
    CaptionMatches, // amount of matches per video
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SortOrder {
    Asc,
    Desc,
}

#[derive(Debug, Clone)]
pub enum SearchType {
    Natural, // Exact phrase + basic stemming
    Wide,    // Flexible word matching + fuzzy + stemming
}

impl SearchOptions {
    pub fn natural() -> Self {
        Self {
            search_type: SearchType::Natural,
            fuzzy_distance: None,
            sort_by: SortBy::Relevance,
            sort_order: SortOrder::Asc,
        }
    }

    pub fn wide() -> Self {
        Self {
            search_type: SearchType::Wide,
            fuzzy_distance: Some("2".to_string()),
            sort_by: SortBy::Relevance,
            sort_order: SortOrder::Asc,
        }
    }
}

pub async fn search_captions_with_options(
    es_client: &Elasticsearch,
    query_string: &str,
    from: usize,
    size: usize,
    options: SearchOptions,
) -> Result<Vec<SearchResult>> {
    // Step 1: Get unique video IDs with pagination
    let video_ids = get_paginated_video_ids(es_client, query_string, from, size, &options).await?;

    if video_ids.is_empty() {
        return Ok(Vec::new());
    }

    // Step 2: For each video, get ALL matching captions
    let mut all_results = Vec::new();

    for video_id in video_ids {
        let video_captions =
            get_all_captions_for_video(es_client, query_string, &video_id, &options).await?;
        all_results.extend(video_captions);
    }

    // Step 3: Process each result with neighbors
    for res in all_results.iter_mut() {
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
        let prev_text = join_neighbor_text(&prev);
        let next_text = join_neighbor_text(&next);

        // Combine with improved sentence awareness
        let combined = stitch_with_neighbors_enhanced(&prev_text, &res.snippet_html, &next_text);

        // Trim to a max length while keeping the highlight in view
        res.snippet_html =
            truncate_around_highlight(&combined, MAX_COMBINED_CHARS, PRE_TAG, POST_TAG);
    }

    Ok(all_results)
}

/// Get unique video IDs with video-level pagination
async fn get_paginated_video_ids(
    es_client: &Elasticsearch,
    query_string: &str,
    from: usize,
    size: usize,
    options: &SearchOptions,
) -> Result<Vec<String>> {
    let main_query = build_main_query_by_type(query_string, options);

    // Choose aggregation order based on sort option
    let agg_order = match options.sort_by {
        SortBy::Relevance => {
            json!({
                "avg_score": "desc"  // Sort by average score of all matches in video
            })
        }
        SortBy::CaptionMatches => {
            json!({
                "_count": "desc"  // Sort by number of matching captions
            })
        }
        // For other sort types, we'll fall back to max score for now
        // You could extend this to join with video metadata for upload date, views, etc.
        _ => {
            json!({
                "max_score": "desc"
            })
        }
    };

    let query_body = json!({
        "size": 0,  // We don't need the hits, just the aggregation
        "query": main_query,
        "aggs": {
            "unique_videos": {
                "terms": {
                    "field": "video_id",
                    "size": from + size + 50,  // Get extra to ensure we have enough after sorting
                    "order": agg_order
                },
                "aggs": {
                    "max_score": {
                        "max": {
                            "script": "_score"
                        }
                    },
                    "avg_score": {
                        "avg": {
                            "script": "_score"
                        }
                    },
                    "total_score": {
                        "sum": {
                            "script": "_score"
                        }
                    }
                }
            }
        }
    });

    let response = es_client
        .search(SearchParts::Index(&["youtube_captions"]))
        .body(query_body)
        .send()
        .await
        .context("Elasticsearch aggregation request failed")?
        .json::<Value>()
        .await
        .context("Failed to parse Elasticsearch aggregation response as JSON")?;

    // Extract video IDs from aggregation, applying pagination
    let empty_vec = vec![];
    let buckets = response
        .get("aggregations")
        .and_then(|aggs| aggs.get("unique_videos"))
        .and_then(|unique_videos| unique_videos.get("buckets"))
        .and_then(|buckets| buckets.as_array())
        .unwrap_or(&empty_vec);

    let video_ids: Vec<String> = buckets
        .iter()
        .skip(from) // Apply video-level pagination
        .take(size)
        .filter_map(|bucket| {
            bucket
                .get("key")
                .and_then(|key| key.as_str())
                .map(|s| s.to_string())
        })
        .collect();

    Ok(video_ids)
}

/// Get all matching captions for a specific video
async fn get_all_captions_for_video(
    es_client: &Elasticsearch,
    query_string: &str,
    video_id: &str,
    options: &SearchOptions,
) -> Result<Vec<SearchResult>> {
    let main_query = build_main_query_by_type(query_string, options);

    // Combine the main query with a video filter
    let combined_query = json!({
        "bool": {
            "must": [
                main_query,
                {
                    "term": {
                        "video_id": video_id  // No .keyword suffix needed since video_id is already keyword type
                    }
                }
            ]
        }
    });

    let query_body = json!({
        "size": 1000,  // Large size to get all captions for this video
        "query": combined_query,
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
                    "boundary_chars": ".,!?;",
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
    });

    let response = es_client
        .search(SearchParts::Index(&["youtube_captions"]))
        .body(query_body)
        .send()
        .await
        .context("Elasticsearch video captions request failed")?
        .json::<Value>()
        .await
        .context("Failed to parse Elasticsearch video captions response as JSON")?;

    let results = process_search_response(response).await;
    Ok(results)
}

fn build_main_query_by_type(query_string: &str, options: &SearchOptions) -> Value {
    match options.search_type {
        SearchType::Natural => {
            json!({
                "bool": {
                    "should": [
                        // Exact phrase match (highest priority)
                        {
                            "match_phrase": {
                                "text": {
                                    "query": query_string,
                                    "boost": 3.0
                                }
                            }
                        },
                        // Exact phrase match on stemmed field (for basic stemming)
                        {
                            "match_phrase": {
                                "text.stemmed": {
                                    "query": query_string,
                                    "boost": 2.0,
                                    "slop": 0  // No word reordering allowed
                                }
                            }
                        }
                    ],
                    "minimum_should_match": 1
                }
            })
        }
        SearchType::Wide => {
            let fuzzy_setting = options.fuzzy_distance.as_deref().unwrap_or("AUTO");

            json!({
                "bool": {
                    "should": [
                        // Exact phrase match (highest boost)
                        {
                            "match_phrase": {
                                "text": {
                                    "query": query_string,
                                    "boost": 4.0
                                }
                            }
                        },
                        // Phrase with some slop (words can be reordered/separated)
                        {
                            "match_phrase": {
                                "text": {
                                    "query": query_string,
                                    "slop": 3,  // Allow up to 3 words between terms
                                    "boost": 3.0
                                }
                            }
                        },
                        // All words must be present (any order) - stemmed
                        {
                            "multi_match": {
                                "query": query_string,
                                "fields": ["text^2", "text.stemmed"],
                                "type": "best_fields",
                                "operator": "and",  // All words must be present
                                "boost": 2.5
                            }
                        },
                        // All words must be present with fuzzy matching
                        {
                            "multi_match": {
                                "query": query_string,
                                "fields": ["text^1.5", "text.stemmed"],
                                "type": "best_fields",
                                "operator": "and",
                                "fuzziness": fuzzy_setting,
                                "boost": 2.0
                            }
                        },
                        // At least most words present (for partial matches)
                        {
                            "multi_match": {
                                "query": query_string,
                                "fields": ["text", "text.stemmed"],
                                "type": "best_fields",
                                "operator": "or",
                                "minimum_should_match": "75%",  // At least 75% of words
                                "boost": 1.5
                            }
                        },
                        // Fuzzy matching for typos (lowest priority)
                        {
                            "multi_match": {
                                "query": query_string,
                                "fields": ["text", "text.stemmed"],
                                "type": "best_fields",
                                "operator": "or",
                                "fuzziness": fuzzy_setting,
                                "minimum_should_match": "50%",
                                "boost": 1.0
                            }
                        }
                    ],
                    "minimum_should_match": 1
                }
            })
        }
    }
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
    let window_seconds = ((before + after) as f64 * 6.0).max(30.0);
    let start_window = anchor_start_time - window_seconds;
    let end_window = anchor_end_time + window_seconds;

    let window_query = json!({
        "_source": ["text", "start_time", "end_time"],
        "size": ((before + after + 1) * 3).max(50),
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

    let mut anchor_index = None;
    for (i, caption) in all_captions.iter().enumerate() {
        if (caption.start_time - anchor_start_time).abs() < 0.1 {
            anchor_index = Some(i);
            break;
        }
    }

    let (prev_captions, next_captions) = match anchor_index {
        Some(anchor_idx) => {
            let prev_start = if anchor_idx >= before {
                anchor_idx - before
            } else {
                0
            };
            let prev_captions = all_captions[prev_start..anchor_idx].to_vec();

            let next_start = anchor_idx + 1;
            let next_end = (next_start + after).min(all_captions.len());
            let next_captions = all_captions[next_start..next_end].to_vec();

            (prev_captions, next_captions)
        }
        None => {
            let mut prev_captions = Vec::new();
            let mut next_captions = Vec::new();

            for caption in all_captions {
                if caption.start_time < anchor_start_time {
                    prev_captions.push(caption);
                } else if caption.start_time > anchor_end_time {
                    next_captions.push(caption);
                }
            }

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

fn join_neighbor_text(prev: &Vec<Caption>) -> String {
    let texts: Vec<String> = prev
        .iter()
        .map(|d| clean_caption_text(&d.text))
        .filter(|s| !s.trim().is_empty())
        .collect();
    texts.join(" ")
}

fn clean_caption_text(text: &str) -> String {
    text.trim()
        .replace("  ", " ") // Collapse multiple spaces
        .replace(" ,", ",") // Fix spacing around punctuation
        .replace(" .", ".")
        .replace(" ?", "?")
        .replace(" !", "!")
        .to_string()
}

/// Enhanced stitching with better sentence awareness
fn stitch_with_neighbors_enhanced(prev: &str, anchor_html: &str, next: &str) -> String {
    let mut parts = Vec::new();

    if !prev.is_empty() {
        let prev_clean = clean_caption_text(prev);
        // Only add ellipsis if previous doesn't end with punctuation
        if prev_clean.ends_with(&['.', '!', '?', ':'][..]) {
            parts.push(prev_clean);
        } else {
            parts.push(format!("…{}", prev_clean));
        }
    }

    parts.push(clean_caption_text(anchor_html));

    if !next.is_empty() {
        let next_clean = clean_caption_text(next);
        // Only add ellipsis if next doesn't start with punctuation
        if next_clean.starts_with(&['.', ',', '!', '?', ':'][..]) {
            parts.push(next_clean);
        } else {
            parts.push(format!("{}…", next_clean));
        }
    }

    parts.join(" ")
}

fn truncate_around_highlight(s: &str, max_chars: usize, pre_tag: &str, post_tag: &str) -> String {
    if s.chars().count() <= max_chars {
        return s.to_string();
    }

    if let Some(pre_idx) = s.find(pre_tag) {
        let after_pre = &s[pre_idx + pre_tag.len()..];
        if let Some(rel_post_idx) = after_pre.find(post_tag) {
            let hl_start = pre_idx;
            let hl_end = pre_idx + pre_tag.len() + rel_post_idx + post_tag.len();

            let total_chars = s.chars().count();
            let s_chars: Vec<char> = s.chars().collect();

            let hl_start_chars = s[..hl_start].chars().count();
            let hl_chars = s[hl_start..hl_end].chars().count();
            let hl_end_chars = hl_start_chars + hl_chars;

            let remaining = max_chars.saturating_sub(hl_chars);
            let side = remaining / 2;
            let extra_buffer = 20;

            let mut prefix_take = (side + extra_buffer).min(hl_start_chars);
            let mut suffix_take = (side + extra_buffer).min(total_chars - hl_end_chars);

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

            let mut actual_start = start_char;
            let mut actual_end = end_char;

            // Find sentence boundaries for more natural breaks
            if start_char > 0 {
                for i in (0..=start_char.min(start_char + 30)).rev() {
                    if i < s_chars.len() && matches!(s_chars[i], '.' | '!' | '?') {
                        actual_start = (i + 1).min(s_chars.len() - 1);
                        break;
                    }
                }
                // Fallback to word boundary
                if actual_start == start_char {
                    for i in (0..=start_char.min(start_char + 20)).rev() {
                        if i < s_chars.len() && s_chars[i] == ' ' {
                            actual_start = i + 1;
                            break;
                        }
                    }
                }
            }

            if end_char < total_chars {
                for i in end_char..=(end_char + 30).min(total_chars - 1) {
                    if i < s_chars.len() && matches!(s_chars[i], '.' | '!' | '?') {
                        actual_end = (i + 1).min(s_chars.len());
                        break;
                    }
                }
                // Fallback to word boundary
                if actual_end == end_char {
                    for i in end_char..=(end_char + 20).min(total_chars - 1) {
                        if i < s_chars.len() && s_chars[i] == ' ' {
                            actual_end = i;
                            break;
                        }
                    }
                }
            }

            let trimmed: String = s_chars[actual_start..actual_end].iter().collect();

            let mut with_ellipses = trimmed;
            if actual_start > 0 {
                with_ellipses = format!("…{}", with_ellipses.trim_start());
            }
            if actual_end < total_chars {
                with_ellipses = format!("{}…", with_ellipses.trim_end());
            }

            return with_ellipses;
        }
    }

    let prefix: String = s.chars().take(max_chars.saturating_sub(2)).collect();
    format!("{}…", prefix.trim_end())
}
