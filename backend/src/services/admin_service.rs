use crate::config::ADMIN_TOKEN;
use crate::models::{
    AdminEnqueueResponse, AdminLoginResponse, AdminQueueResponse, AdminStats,
    AdminVideoListResponse, VideoMetadata,
};
use crate::services::crawler::VideoQueue;
use crate::services::monitoring_service::{
    get_monitored_channels_list, get_monitored_playlist_list,
};
use crate::utils;
use anyhow::Result;
use elasticsearch::{DeleteByQueryParts, DeleteParts, Elasticsearch, SearchParts};
use serde_json::{json, Value};
use std::sync::Arc;

pub async fn authenticate_admin(token: &str) -> Result<AdminLoginResponse> {
    if token == &*ADMIN_TOKEN {
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

pub async fn get_admin_stats(
    es_client: &Elasticsearch,
    video_queue: &VideoQueue,
) -> Result<AdminStats> {
    let total_videos = get_index_count(es_client, "youtube_videos").await;
    let total_captions = get_index_count(es_client, "youtube_captions").await;
    let last_crawl_time = get_last_crawl_time(es_client).await;

    let channels = get_monitored_channels_list(es_client).await;
    let playlists = get_monitored_playlist_list(es_client).await;
    let active_monitors = channels.iter().filter(|c| c.active).count() as i32
        + playlists.iter().filter(|c| c.active).count() as i32;
    let queue_size = video_queue.get_size();

    Ok(AdminStats {
        total_videos,
        total_captions,
        last_crawl_time,
        active_monitors,
        queue_size,
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
    let video_id = utils::extract_youtube_video_id(url)
        .ok_or_else(|| anyhow::anyhow!("Invalid YouTube URL"))?;

    video_queue.add_video(video_id.clone());

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
    let count_body = json!({
        "query": {
            "match_all": {}
        }
    });

    match es_client
        .count(elasticsearch::CountParts::Index(&[index]))
        .body(count_body)
        .send()
        .await
    {
        Ok(response) => {
            if response.status_code().is_success() {
                if let Ok(json_response) = response.json::<Value>().await {
                    return json_response["count"].as_i64().unwrap_or(0);
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
