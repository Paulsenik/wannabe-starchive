use crate::Caption;
use elasticsearch::{Elasticsearch, IndexParts};
use log::{error, info};
use serde_json::json;
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use yt_transcript_rs::api::YouTubeTranscriptApi;

// This function will be called periodically by the scheduler.
// In a real application, you'd fetch video IDs from a more dynamic source
// (e.g., YouTube Data API, a list of channels, or a queue).
static VIDEO_IDS: &[&str] = &[];

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
