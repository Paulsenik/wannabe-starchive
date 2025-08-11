use anyhow::Result;
use elasticsearch::{Elasticsearch, SearchParts};
use serde_json::{json, Value};

use crate::models::SearchResult;

pub async fn search_captions(
    es_client: &Elasticsearch,
    query_string: &str,
    from: usize,
    size: usize,
) -> Result<Vec<SearchResult>> {
    let search_body = build_exact_search_query(query_string, from, size);
    //let search_body = build_search_query(query_string, from, size);

    let response = es_client
        .search(SearchParts::Index(&["youtube_captions"]))
        .body(search_body)
        .send()
        .await?;

    if !response.status_code().is_success() {
        return Err(anyhow::anyhow!(
            "Elasticsearch search failed with status: {}",
            response.status_code()
        ));
    }

    let json_response: Value = response.json().await?;
    let results = process_search_response(json_response).await;

    Ok(results)
}

#[allow(dead_code)]
fn build_search_query(query_string: &str, from: usize, size: usize) -> Value {
    json!({
        "size": size,
        "query": {
            "bool": {
                "should": [
                    {
                        "match": {
                            "text": {
                                "query": query_string,
                                "boost": 3.0  // Highest priority for exact matches
                            }
                        }
                    },
                    {
                        "match": {
                            "text": {
                                "query": query_string,
                                "fuzziness": 1,  // Only 1 character difference allowed
                                "prefix_length": 2,  // First 2 characters must match exactly
                                "max_expansions": 10,  // Limit expansions for performance
                                "boost": 1.5  // Medium priority for minor typos
                            }
                        }
                    }
                ],
                "minimum_should_match": 1
            }
        },
        "collapse": {
            "field": "video_id",
            "inner_hits": {
                "name": "captions",
                "size": 10000
            }
        },
        "sort": ["_score"],
        "from": from,
        "_source": ["video_id", "text", "start_time", "end_time"],
        "highlight": {
            "fields": {
                "text": {}
            }
        }
    })
}

fn build_exact_search_query(query_string: &str, from: usize, size: usize) -> Value {
    json!({
        "size": size,
        "query": {
            "match_phrase": {
                "text": {
                    "query": query_string
                }
            }
        },
        "collapse": {
            "field": "video_id",
            "inner_hits": {
                "name": "captions",
                "size": 10000
            }
        },
        "sort": ["_score"],
        "from": from,
        "_source": ["video_id", "text", "start_time", "end_time"],
        "highlight": {
            "fields": {
                "text": {}
            }
        }
    })
}

fn parse_search_result(source: &serde_json::Map<String, Value>, inner_hit: &Value) -> SearchResult {
    let video_id = source["video_id"].as_str().unwrap_or("N/A").to_string();
    let text = source["text"].as_str().unwrap_or("N/A").to_string();
    let start_time = source["start_time"].as_f64().unwrap_or(0.0);
    let end_time = source["end_time"].as_f64().unwrap_or(0.0);

    let highlighted_text = inner_hit["highlight"]["text"]
        .as_array()
        .and_then(|highlight| highlight.first())
        .and_then(|first_highlight| first_highlight.as_str())
        .map(String::from);

    SearchResult {
        video_id,
        text,
        start_time,
        end_time,
        highlighted_text,
    }
}

async fn process_search_response(response: Value) -> Vec<SearchResult> {
    let mut results = Vec::new();

    if let Some(hits) = response["hits"]["hits"].as_array() {
        for hit in hits {
            if let Some(inner_hits) = hit["inner_hits"]["captions"]["hits"]["hits"].as_array() {
                for inner_hit in inner_hits {
                    if let Some(source) = inner_hit["_source"].as_object() {
                        results.push(parse_search_result(source, inner_hit));
                    }
                }
            }
        }
    }

    results
}
