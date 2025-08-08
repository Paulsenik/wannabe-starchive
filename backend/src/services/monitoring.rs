use crate::models::MonitoredChannel;
use crate::services::crawler::VideoQueue;
use elasticsearch::{Elasticsearch, SearchParts};
use log::{error, info};
use reqwest::Client;
use serde_json::{json, Value};
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio_cron_scheduler::{Job, JobScheduler};

lazy_static::lazy_static! {
    pub static ref MONITORED_CHANNELS: Arc<RwLock<Vec<MonitoredChannel>>> = Arc::new(RwLock::new(Vec::new()));
}

pub async fn setup_channel_monitoring(
    es_client: Arc<Elasticsearch>,
    video_queue: Arc<VideoQueue>,
) -> Result<(), Box<dyn std::error::Error>> {
    info!("Setting up channel monitoring scheduler...");

    let sched = JobScheduler::new().await?;

    // Load monitored channels from Elasticsearch on startup
    load_monitored_channels(&es_client).await;

    // Check channels every 10 minutes
    let es_client_clone = es_client.clone();
    let queue_clone = video_queue.clone();

    let monitor_job = Job::new_async("0 */10 * * * *", move |_uuid, _l| {
        let es_client = es_client_clone.clone();
        let queue = queue_clone.clone();
        Box::pin(async move {
            check_monitored_channels(&es_client, &queue).await;
        })
    })?;

    sched.add(monitor_job).await?;

    sched.start().await?;
    info!("Channel monitoring scheduler started.");
    Ok(())
}

async fn load_monitored_channels(es_client: &Elasticsearch) {
    info!("Loading monitored channels from Elasticsearch...");

    let search_response = es_client
        .search(SearchParts::Index(&["monitored_channels"]))
        .body(json!({
            "query": {
                "match_all": {}
            },
            "size": 100
        }))
        .send()
        .await;

    match search_response {
        Ok(response) => {
            let response_body: Value = response.json().await.unwrap_or_default();

            if let Some(hits) = response_body["hits"]["hits"].as_array() {
                let mut channels = MONITORED_CHANNELS.write().await;
                channels.clear();

                for hit in hits {
                    if let Some(source) = hit["_source"].as_object() {
                        if let Ok(channel) =
                            serde_json::from_value::<MonitoredChannel>(source.clone().into())
                        {
                            if channel.active {
                                channels.push(channel);
                            }
                        }
                    }
                }

                info!("Loaded {} active monitored channels", channels.len());
            }
        }
        Err(e) => {
            error!("Failed to load monitored channels: {}", e);
        }
    }
}

async fn check_monitored_channels(es_client: &Elasticsearch, video_queue: &VideoQueue) {
    info!("Checking monitored channels for new videos...");

    let channels = MONITORED_CHANNELS.read().await;

    for channel in channels.iter() {
        if !channel.active {
            continue;
        }

        info!(
            "Checking channel: {} ({})",
            channel.channel_name, channel.channel_id
        );

        // Get the channel's upload playlist
        match get_channel_uploads_playlist_id(&channel.channel_id).await {
            Ok(playlist_id) => {
                // Check for new videos in the playlist
                match check_playlist_for_new_videos(&playlist_id, channel).await {
                    Ok(new_video_urls) => {
                        for url in new_video_urls.clone() {
                            info!("Found new video for monitoring: {}", url);
                            video_queue.add_video(url);
                        }

                        // Update the last_video_id if we found new videos
                        if !new_video_urls.is_empty() {
                            // You might want to update the channel's last_video_id in Elasticsearch here
                        }
                    }
                    Err(e) => {
                        error!(
                            "Failed to check playlist for channel {}: {}",
                            channel.channel_id, e
                        );
                    }
                }
            }
            Err(e) => {
                error!(
                    "Failed to get upload playlist for channel {}: {}",
                    channel.channel_id, e
                );
            }
        }
    }
}

async fn check_playlist_for_new_videos(
    playlist_id: &str,
    channel: &MonitoredChannel,
) -> Result<Vec<String>, anyhow::Error> {
    let api_key = std::env::var("YOUTUBE_API_KEY")?;
    let url = format!(
        "https://www.googleapis.com/youtube/v3/playlistItems?part=snippet&playlistId={}&maxResults=10&key={}",
        playlist_id, api_key
    );

    let client = reqwest::Client::new();
    let response: Value = client.get(&url).send().await?.json().await?;

    let mut new_videos = Vec::new();

    if let Some(items) = response["items"].as_array() {
        let mut found_last_processed = channel.last_video_id.is_none();

        for item in items {
            if let Some(video_id) = item["snippet"]["resourceId"]["videoId"].as_str() {
                // If we haven't found the last processed video yet, check if this is it
                if !found_last_processed {
                    if Some(video_id.to_string()) == channel.last_video_id {
                        found_last_processed = true;
                    }
                    continue;
                }

                // This is a new video
                let video_url = format!("https://www.youtube.com/watch?v={}", video_id);
                new_videos.push(video_url);
            }
        }
    }

    Ok(new_videos)
}

pub async fn get_monitored_channels_list() -> Vec<MonitoredChannel> {
    MONITORED_CHANNELS.read().await.clone()
}

// returns the complete video-library-playlist (as list-id) of a channel with the given channel-id
pub async fn get_channel_uploads_playlist_id(channel_id: &str) -> Result<String, anyhow::Error> {
    let client = Client::new();
    let api_key = std::env::var("YOUTUBE_API_KEY")?;

    let url = format!(
        "https://www.googleapis.com/youtube/v3/channels?id={}&key={}&part=contentDetails",
        channel_id, api_key
    );

    let response = client
        .get(&url)
        .send()
        .await?
        .json::<serde_json::Value>()
        .await?;

    let uploads_playlist_id = response["items"][0]["contentDetails"]["relatedPlaylists"]["uploads"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("No uploads playlist found"))?;

    Ok(uploads_playlist_id.to_string())
}
