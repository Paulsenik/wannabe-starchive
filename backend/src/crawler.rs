use crate::models::VideoMetadata;
use crate::Caption;
use elasticsearch::{Elasticsearch, IndexParts};
use log::{error, info};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use yt_transcript_rs::api::YouTubeTranscriptApi;

// This function will be called periodically by the scheduler.
// In a real application, you'd fetch video IDs from a more dynamic source
// (e.g., YouTube Data API, a list of channels, or a queue).
static VIDEO_IDS: &[&str] = &[];

async fn fetch_video_metadata(video_id: &str) -> Result<VideoMetadata, Box<dyn std::error::Error>> {
    let client = Client::new();
    let api_key = std::env::var("YOUTUBE_API_KEY")?;

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
    })
}

pub struct VideoQueue {
    queue: Arc<Mutex<VecDeque<String>>>,
}

impl Default for VideoQueue {
    fn default() -> Self {
        Self::new()
    }
}

impl VideoQueue {
    pub fn new() -> Self {
        let mut queue = VecDeque::new();
        for &id in VIDEO_IDS {
            queue.push_back(id.to_string());
        }
        VideoQueue {
            queue: Arc::new(Mutex::new(queue)),
        }
    }

    pub fn add_video(&self, video_id: String) {
        if let Ok(mut queue) = self.queue.lock() {
            queue.push_back(video_id);
        }
    }

    pub fn pop_next_video(&self) -> Option<String> {
        if let Ok(mut queue) = self.queue.lock() {
            queue.pop_front()
        } else {
            None
        }
    }

    pub fn get_size(&self) -> usize {
        if let Ok(queue) = self.queue.lock() {
            queue.len()
        } else {
            0
        }
    }
}

pub async fn crawl_youtube_captions(es_client: &Elasticsearch, video_queue: &VideoQueue) {
    info!("Starting YouTube caption crawl...");

    let api =
        YouTubeTranscriptApi::new(None, None, None).expect("Failed to create YouTubeTranscriptApi");

    let languages = &["en"];

    while let Some(video_id) = video_queue.pop_next_video() {
        info!("Processing video ID: {video_id}");
        match api.fetch_transcript(&video_id, languages, false).await {
            Ok(transcript) => {
                let mut captions_to_index: Vec<Caption> = Vec::new();
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
                    }
                });
                info!(
                    "Video metadata - Title: {}, Channel: {}, Upload Date: {}, Likes: {}, Views: {}, Duration: {}, Comments: {}",
                    metadata.title,
                    metadata.channel_name,
                    metadata.upload_date,
                    metadata.likes,
                    metadata.views,
                    metadata.duration,
                    metadata.comment_count
                );

                // First index video metadata
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
                        }
                    }
                    Err(e) => {
                        error!(
                            "Failed to send metadata to Elasticsearch for video ID {}: {e:?}",
                            video_id
                        );
                    }
                }

                // Then index captions
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

                // Index captions into Elasticsearch
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
    info!("YouTube caption crawl completed.");
}
