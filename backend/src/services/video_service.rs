use anyhow::Result;
use elasticsearch::{Elasticsearch, SearchParts};
use serde_json::{json, Value};

pub async fn list_all_videos(es_client: &Elasticsearch) -> Result<Vec<String>> {
    let search_body = json!({
        "size": 10000,
        "query": {
            "match_all": {}
        },
        "_source": false
    });

    let response = es_client
        .search(SearchParts::Index(&["youtube_videos"]))
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
    let mut video_ids = Vec::new();

    if let Some(hits) = json_response["hits"]["hits"].as_array() {
        for hit in hits {
            if let Some(id) = hit["_id"].as_str() {
                video_ids.push(id.to_string());
            }
        }
    }

    Ok(video_ids)
}
