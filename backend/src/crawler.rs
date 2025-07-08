use crate::Caption;
use elasticsearch::{Elasticsearch, IndexParts};
use log::{error, info};
use serde_json::json;
use yt_transcript_rs::api::YouTubeTranscriptApi;

// This function will be called periodically by the scheduler.
// In a real application, you'd fetch video IDs from a more dynamic source
// (e.g., YouTube Data API, a list of channels, or a queue).
pub async fn crawl_youtube_captions(es_client: &Elasticsearch) {
    info!("Starting YouTube caption crawl...");

    // Example video IDs. Replace with your actual video discovery logic.
    // For a real application, you'd use the YouTube Data API to find new videos
    // from channels you're interested in, or a list of popular videos.
    let video_ids = vec![
        "dQw4w9WgXcQ", // Rick Astley - Never Gonna Give You Up
        "jNQXAC9MKEB", // Me at the zoo (first YouTube video)
        "k_9_gQ_f6_Q", // Example of a tech talk or educational video
        "q_9_gQ_f6_Q", // Another example video
    ];

    let api =
        YouTubeTranscriptApi::new(None, None, None).expect("Failed to create YouTubeTranscriptApi");

    let languages = &["en"];

    for video_id in video_ids {
        info!("Processing video ID: {video_id}");
        match api.fetch_transcript(video_id, languages, false).await {
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
                    match es_client
                        .index(IndexParts::Index("youtube_captions"))
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
