use anyhow::Result;
use elasticsearch::{DeleteByQueryParts, DeleteParts, Elasticsearch, SearchParts};
use serde_json::{json, Value};
use std::sync::Arc;

use crate::models::{
    AdminEnqueueResponse, AdminLoginResponse, AdminQueueResponse, AdminStats,
    AdminVideoListResponse, VideoMetadata,
};
use crate::services::crawler::VideoQueue;

const ADMIN_TOKEN: &str = "your-secret-admin-token"; // In production, use env variable

pub async fn authenticate_admin(token: &str) -> Result<AdminLoginResponse> {
    let expected_token = std::env::var("ADMIN_TOKEN").unwrap_or_else(|_| ADMIN_TOKEN.to_string());

    if token == expected_token {
        Ok(AdminLoginResponse {
            success: true,
            message: "Authentication successful".to_string(),
        })
    } else {
        Ok(AdminLoginResponse {
            success: false,
            message: "Invalid admin token".to_string(),
        })
    }
}

pub async fn get_admin_stats(es_client: &Elasticsearch) -> Result<AdminStats> {
    let total_videos = get_index_count(es_client, "youtube_videos").await;
    let total_captions = get_index_count(es_client, "youtube_captions").await;
    let last_crawl_time = get_last_crawl_time(es_client).await;

    Ok(AdminStats {
        total_videos,
        total_captions,
        last_crawl_time,
    })
}

pub async fn get_admin_queue(video_queue: &Arc<VideoQueue>) -> Result<AdminQueueResponse> {
    let items = video_queue.get_all_items();

    Ok(AdminQueueResponse {
        success: true,
        message: format!("Retrieved {} queue items", items.len()),
        items,
    })
}

pub async fn enqueue_video(
    video_queue: &Arc<VideoQueue>,
    url: &str,
) -> Result<AdminEnqueueResponse> {
    let video_id =
        extract_youtube_video_id(url).ok_or_else(|| anyhow::anyhow!("Invalid YouTube URL"))?;

    video_queue.add_video(url.to_string(), video_id.clone());

    Ok(AdminEnqueueResponse {
        success: true,
        message: format!("Video {} added to queue", video_id),
    })
}

pub async fn remove_from_queue(video_queue: &Arc<VideoQueue>, id: &str) -> Result<()> {
    video_queue.remove_item(id);
    Ok(())
}

pub async fn delete_video(es_client: &Elasticsearch, video_id: &str) -> Result<()> {
    // Delete from youtube_videos index
    let delete_video_response = es_client
        .delete(DeleteParts::IndexId("youtube_videos", video_id))
        .send()
        .await?;

    if !delete_video_response.status_code().is_success() {
        return Err(anyhow::anyhow!("Failed to delete video metadata"));
    }

    let delete_captions_body = json!({
        "query": {
            "term": {
                "video_id": video_id
            }
        }
    });

    let delete_captions_response = es_client
        .delete_by_query(DeleteByQueryParts::Index(&["youtube_captions"]))
        .body(delete_captions_body)
        .send()
        .await?;

    if !delete_captions_response.status_code().is_success() {
        return Err(anyhow::anyhow!("Failed to delete video captions"));
    }

    Ok(())
}

pub async fn get_videos_paginated(
    es_client: &Elasticsearch,
    page: i64,
    per_page: i64,
) -> Result<AdminVideoListResponse> {
    let from = (page - 1) * per_page;

    let search_body = json!({
        "size": per_page,
        "from": from,
        "query": {
            "match_all": {}
        },
        "sort": [
            {
                "upload_date": {
                    "order": "desc"
                }
            }
        ]
    });

    let response = es_client
        .search(SearchParts::Index(&["youtube_videos"]))
        .body(search_body)
        .send()
        .await?;

    if !response.status_code().is_success() {
        return Err(anyhow::anyhow!("Elasticsearch search failed"));
    }

    let json_response: Value = response.json().await?;
    let mut videos = Vec::new();
    let total = json_response["hits"]["total"]["value"]
        .as_i64()
        .unwrap_or(0);

    if let Some(hits) = json_response["hits"]["hits"].as_array() {
        for hit in hits {
            if let Some(source) = hit["_source"].as_object() {
                if let Ok(video) =
                    serde_json::from_value::<VideoMetadata>(Value::Object(source.clone()))
                {
                    videos.push(video);
                }
            }
        }
    }

    Ok(AdminVideoListResponse {
        videos,
        total,
        page,
        per_page,
    })
}

async fn get_index_count(es_client: &Elasticsearch, index: &str) -> i64 {
    let search_body = json!({
        "size": 0,
        "query": {
            "match_all": {}
        }
    });

    match es_client
        .search(SearchParts::Index(&[index]))
        .body(search_body)
        .send()
        .await
    {
        Ok(response) => {
            if response.status_code().is_success() {
                if let Ok(json_response) = response.json::<Value>().await {
                    return json_response["hits"]["total"]["value"]
                        .as_i64()
                        .unwrap_or(0);
                }
            }
        }
        Err(e) => {
            log::error!("Failed to get count for index {}: {e:?}", index);
        }
    }
    0
}

async fn get_last_crawl_time(es_client: &Elasticsearch) -> Option<String> {
    let search_body = json!({
        "size": 1,
        "query": {
            "match_all": {}
        },
        "sort": [
            {
                "crawl_date": {
                    "order": "desc"
                }
            }
        ],
        "_source": ["crawl_date"]
    });

    match es_client
        .search(SearchParts::Index(&["youtube_videos"]))
        .body(search_body)
        .send()
        .await
    {
        Ok(response) => {
            if response.status_code().is_success() {
                if let Ok(json_response) = response.json::<Value>().await {
                    if let Some(hits) = json_response["hits"]["hits"].as_array() {
                        if let Some(first_hit) = hits.first() {
                            return first_hit["_source"]["crawl_date"]
                                .as_str()
                                .map(String::from);
                        }
                    }
                }
            }
        }
        Err(e) => {
            log::error!("Failed to get last crawl time: {e:?}");
        }
    }
    None
}

fn extract_youtube_video_id(url: &str) -> Option<String> {
    if let Some(captures) = regex::Regex::new(
        r"(?:youtube\.com/watch\?v=|youtu\.be/|youtube\.com/embed/)([a-zA-Z0-9_-]{11})",
    )
    .ok()?
    .captures(url)
    {
        return captures.get(1).map(|m| m.as_str().to_string());
    }
    None
}
