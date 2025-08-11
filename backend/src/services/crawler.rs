use crate::config::YOUTUBE_API_KEY;
use crate::models::{Caption, QueueItem, VideoMetadata};
use elasticsearch::{Elasticsearch, IndexParts};
use log::{error, info};
use reqwest::Client;
use serde_json::json;
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use yt_transcript_rs::api::YouTubeTranscriptApi;

pub struct VideoQueue {
    queue: Arc<Mutex<VecDeque<QueueItem>>>,
}

impl Default for VideoQueue {
    fn default() -> Self {
        Self::new()
    }
}

impl VideoQueue {
    pub fn new() -> Self {
        let queue = VecDeque::new();
        VideoQueue {
            queue: Arc::new(Mutex::new(queue)),
        }
    }

    pub fn add_video(&self, video_id: String) -> String {
        if let Ok(mut queue) = self.queue.lock() {
            let item_id = format!("{}_{}", chrono::Utc::now().timestamp(), video_id);
            let item = QueueItem {
                id: item_id.clone(),
                video_id,
                status: "pending".to_string(),
                added_at: chrono::Utc::now().to_rfc3339(),
                processed_at: None,
                error_message: None,
            };
            queue.push_back(item);
            item_id
        } else {
            String::new()
        }
    }

    pub fn pop_next_video(&self) -> Option<QueueItem> {
        if let Ok(mut queue) = self.queue.lock() {
            if let Some(mut item) = queue.pop_front() {
                item.status = "processing".to_string();
                Some(item)
            } else {
                None
            }
        } else {
            None
        }
    }

    pub fn mark_completed(&self, item_id: &str) {
        if let Ok(mut queue) = self.queue.lock() {
            for item in queue.iter_mut() {
                if item.id == item_id {
                    item.status = "completed".to_string();
                    item.processed_at = Some(chrono::Utc::now().to_rfc3339());
                    break;
                }
            }
        }
    }

    pub fn mark_failed(&self, item_id: &str, error_message: String) {
        if let Ok(mut queue) = self.queue.lock() {
            for item in queue.iter_mut() {
                if item.id == item_id {
                    item.status = "failed".to_string();
                    item.processed_at = Some(chrono::Utc::now().to_rfc3339());
                    item.error_message = Some(error_message);
                    break;
                }
            }
        }
    }

    pub fn get_all_items(&self) -> Vec<QueueItem> {
        if let Ok(queue) = self.queue.lock() {
            queue.iter().cloned().collect()
        } else {
            Vec::new()
        }
    }

    pub fn remove_item(&self, item_id: &str) -> bool {
        if let Ok(mut queue) = self.queue.lock() {
            if let Some(pos) = queue.iter().position(|item| item.id == item_id) {
                queue.remove(pos);
                return true;
            }
        }
        false
    }

    pub fn get_size(&self) -> usize {
        if let Ok(queue) = self.queue.lock() {
            queue.len()
        } else {
            0
        }
    }
}

async fn fetch_video_metadata(video_id: &str) -> Result<VideoMetadata, Box<dyn std::error::Error>> {
    let client = Client::new();
    let api_key = &*YOUTUBE_API_KEY;

    // Documentation: https://developers.google.com/youtube/v3/docs/videos
    let url = format!(
        "https://www.googleapis.com/youtube/v3/videos?id={video_id}&key={api_key}&part=snippet,statistics,contentDetails"
    );

    let response = client
        .get(&url)
        .send()
        .await?
        .json::<serde_json::Value>()
        .await?;
    let item = &response["items"][0];

    Ok(VideoMetadata {
        title: item["snippet"]["title"].as_str().unwrap_or("").to_string(),
        channel_id: item["snippet"]["channelId"]
            .as_str()
            .unwrap_or("")
            .to_string(),
        channel_name: item["snippet"]["channelTitle"]
            .as_str()
            .unwrap_or("")
            .to_string(),
        upload_date: item["snippet"]["publishedAt"]
            .as_str()
            .unwrap_or("")
            .to_string(),
        likes: item["statistics"]["likeCount"]
            .as_str()
            .unwrap_or("0")
            .parse()
            .unwrap_or(0),
        views: item["statistics"]["viewCount"]
            .as_str()
            .unwrap_or("0")
            .parse()
            .unwrap_or(0),
        duration: item["contentDetails"]["duration"]
            .as_str()
            .unwrap_or("")
            .to_string(),
        comment_count: item["statistics"]["commentCount"]
            .as_str()
            .unwrap_or("0")
            .parse()
            .unwrap_or(0),
        tags: item["snippet"]["tags"]
            .as_array()
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str())
                    .map(String::from)
                    .collect()
            })
            .unwrap_or_default(),
        has_captions: item["contentDetails"]["caption"]
            .as_str()
            .map(|s| s == "true")
            .unwrap_or(false),
        crawl_date: chrono::Utc::now().to_rfc3339(),
        video_id: video_id.to_string(),
    })
}

pub async fn process_video_metadata(es_client: &Elasticsearch, video_id: &str) {
    let metadata = fetch_video_metadata(&video_id).await.unwrap_or_else(|e| {
        error!("Failed to fetch metadata for video {}: {:?}", video_id, e);
        VideoMetadata {
            title: String::new(),
            channel_name: String::new(),
            channel_id: String::new(),
            upload_date: String::new(),
            likes: 0,
            views: 0,
            duration: String::new(),
            comment_count: 0,
            tags: Vec::new(),
            has_captions: false,
            crawl_date: String::new(),
            video_id: String::new(),
        }
    });

    match es_client
        .index(IndexParts::IndexId("youtube_videos", &video_id))
        .body(json!(metadata))
        .send()
        .await
    {
        Ok(response) => {
            if !response.status_code().is_success() {
                error!(
                    "Failed to index metadata for video ID {}: {:?}",
                    video_id,
                    response.text().await
                );
            } else {
                info!(
                    "Processed YT-Video: {}\nChannel: {} -> {}, Upload Date: {}, Crawl Date: {}\nDuration: {}, Views: {}, Likes: {}, Comments: {} Captions: {},\nTags: {}",
                    metadata.title,
                    metadata.channel_name,
                    metadata.channel_id,
                    metadata.upload_date,
                    metadata.crawl_date,
                    metadata.duration,
                    metadata.views,
                    metadata.likes,
                    metadata.comment_count,
                    metadata.has_captions,
                    metadata.tags.join(", "),
                );
            }
        }
        Err(e) => {
            error!(
                "Failed to send metadata to Elasticsearch for video ID {}: {e:?}",
                video_id
            );
        }
    }
}

pub async fn process_video_captions(es_client: &Elasticsearch, video_id: &str) {
    let languages = &["en"];

    let api =
        YouTubeTranscriptApi::new(None, None, None).expect("Failed to create YouTubeTranscriptApi");

    match api.fetch_transcript(&video_id, languages, false).await {
        Ok(transcript) => {
            let mut captions_to_index: Vec<Caption> = Vec::new();

            for entry in transcript {
                captions_to_index.push(Caption {
                    video_id: video_id.to_string(),
                    text: entry.text,
                    start_time: entry.start,
                    end_time: entry.start + entry.duration,
                });
            }
            info!(
                "Fetched {} captions for video ID: {video_id}",
                captions_to_index.len()
            );

            for caption in captions_to_index {
                let doc_id = format!("{}_{}", caption.video_id, caption.start_time);
                match es_client
                    .index(IndexParts::IndexId("youtube_captions", &doc_id))
                    .body(json!(caption))
                    .send()
                    .await
                {
                    Ok(response) => {
                        if response.status_code().is_success() {
                            // info!("Indexed caption for video ID: {}", caption.video_id);
                        } else {
                            error!(
                                "Failed to index caption for video ID {}: {:?}",
                                caption.video_id,
                                response.text().await
                            );
                        }
                    }
                    Err(e) => {
                        error!(
                            "Failed to send caption to Elasticsearch for video ID {}: {e:?}",
                            caption.video_id
                        );
                    }
                }
            }
        }
        Err(e) => {
            error!("Failed to fetch transcript for video ID {video_id}: {e:?}");
        }
    }
}

pub async fn crawl_youtube_video(
    es_client: &Elasticsearch,
    video_queue: &VideoQueue,
    maxcount: i32,
) {
    info!("Starting YouTube caption crawl...");

    let mut count = 0;
    while let Some(item) = video_queue.pop_next_video() {
        info!("Processing video ID: {}", item.video_id);

        process_video_metadata(es_client, &item.video_id).await;
        process_video_captions(es_client, &item.video_id).await;

        video_queue.mark_completed(&item.id);

        count += 1;
        if count >= maxcount {
            info!("YouTube caption crawl maxcount reached. ");
            break;
        }
    }
    info!("YouTube caption crawl completed.");
}
