use crate::api::MonitoredChannelStats;
use crate::config::YOUTUBE_API_KEY;
use crate::models::MonitoredChannel;
use crate::services::crawler::VideoQueue;
use elasticsearch::{DeleteParts, Elasticsearch, SearchParts};
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

pub async fn get_monitored_channels_list(es_client: &Elasticsearch) -> Vec<MonitoredChannelStats> {
    let channels = MONITORED_CHANNELS.read().await.clone();

    let mut result = Vec::new();
    for channel in channels {
        let response = es_client
            .count(elasticsearch::CountParts::Index(&["youtube_videos"]))
            .body(json!({
                "query": {
                    "match": {
                        "channel_id": channel.channel_id
                    }
                }
            }))
            .send()
            .await;

        let video_count = match response {
            Ok(r) => {
                let count: Value = r.json().await.unwrap_or(json!({"count": 0}));
                count["count"].as_i64().unwrap_or(0) as i32
            }
            Err(_) => 0,
        };

        result.push(MonitoredChannelStats {
            channel_id: channel.channel_id,
            channel_name: channel.channel_name,
            active: channel.active,
            created_at: channel.created_at,
            videos_indexed: video_count,
        });
    }
    result
}

pub async fn remove_monitored_channel(
    channel_id: &str,
    es_client: &Elasticsearch,
) -> Result<(), anyhow::Error> {
    info!("Removing monitored channel: {}", channel_id);

    es_client
        .delete(DeleteParts::IndexId("monitored_channels", channel_id))
        .send()
        .await?;

    let mut channels = MONITORED_CHANNELS.write().await;
    channels.retain(|channel| channel.channel_id != channel_id);

    info!("Successfully removed monitored channel");
    Ok(())
}

async fn fetch_monitored_channel(input: &str) -> Result<MonitoredChannel, anyhow::Error> {
    let client = Client::new();
    let api_key = &*YOUTUBE_API_KEY;

    // Extract channel ID from different URL formats
    let channel_id = if input.contains("/channel/") {
        // Format: https://www.youtube.com/channel/UCTeLqJq1mXUX5WWoNXLmOIA
        input
            .split("/channel/")
            .nth(1)
            .ok_or_else(|| anyhow::anyhow!("Invalid channel URL"))?
            .to_string()
    } else if input.contains("/@") {
        // Format: https://youtube.com/@RobertsSpaceInd
        let handle = input
            .split("/@")
            .nth(1)
            .ok_or_else(|| anyhow::anyhow!("Invalid handle URL"))?;
        // Get channel ID from handle via API
        let url = format!(
            "https://www.googleapis.com/youtube/v3/channels?part=id&forHandle={}&key={}",
            handle, api_key
        );
        let response = client.get(&url).send().await?.json::<Value>().await?;
        response["items"][0]["id"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Invalid API response"))?
            .to_string()
    } else if input.contains("/c/") {
        // Format: https://www.youtube.com/c/RobertsSpaceInd
        let custom = input
            .split("/c/")
            .nth(1)
            .ok_or_else(|| anyhow::anyhow!("Invalid custom URL"))?;
        // Get channel ID from custom URL via API
        let url = format!(
            "https://www.googleapis.com/youtube/v3/channels?part=id&forUsername={}&key={}",
            custom, api_key
        );
        let response = client.get(&url).send().await?.json::<Value>().await?;
        response["items"][0]["id"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Invalid API response"))?
            .to_string()
    } else {
        return Err(anyhow::anyhow!("Invalid channel URL format"));
    };

    let url = format!(
        "https://www.googleapis.com/youtube/v3/channels?part=snippet,statistics&id={}&key={}",
        channel_id, api_key
    );

    let response = client.get(&url).send().await?.json::<Value>().await?;
    let channel = &response["items"][0];

    Ok(MonitoredChannel {
        channel_id,
        channel_name: channel["snippet"]["title"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Invalid channel title"))?
            .to_string(),
        active: true,
        created_at: chrono::Utc::now().to_rfc3339(),
    })
}

pub async fn add_monitored_channel(
    channel_input: &str,
    es_client: &Elasticsearch,
) -> Result<(), anyhow::Error> {
    info!("Adding new monitored channel: {}", channel_input);

    let new_channel;

    match fetch_monitored_channel(channel_input).await {
        Ok(channel) => {
            new_channel = channel;
        }
        Err(e) => {
            error!("Failed to fetch monitored channel from youtube: {}", e);
            return Err(e);
        }
    }

    es_client
        .index(elasticsearch::IndexParts::IndexId(
            "monitored_channels",
            &new_channel.channel_id,
        ))
        .body(json!(new_channel))
        .send()
        .await?;

    info!(
        "Successfully added new monitored channel: {} ({})",
        new_channel.channel_name, new_channel.channel_id
    );

    let mut channels = MONITORED_CHANNELS.write().await;
    channels.push(new_channel);
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
            "size": 1000
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
                            channels.push(channel);
                        }
                    }
                }

                info!("Loaded {} monitored channels", channels.len());
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
        info!(
            "Checking channel: {} ({}) - active: {}",
            channel.channel_name, channel.channel_id, channel.active
        );

        if channel.active {
            check_channel_for_new_videos(&channel.channel_id, &es_client, &video_queue).await;
        }
    }
    info!("Finished checking monitored channels!");
}

pub async fn check_channel_for_new_videos(
    channel_id: &str,
    es_client: &Elasticsearch,
    video_queue: &VideoQueue,
) {
    match get_channel_playlist_id(&channel_id).await {
        Ok(playlist_id) => {
            check_playlist_for_new_videos(&playlist_id, &es_client, &video_queue).await;
        }
        Err(e) => {
            error!(
                "Failed to get upload playlist for channel {}: {}",
                channel_id, e
            );
        }
    }
}

pub async fn check_playlist_for_new_videos(
    playlist_id: &str,
    es_client: &Elasticsearch,
    video_queue: &VideoQueue,
) {
    let all_playlist_videos = match fetch_all_playlist_videos(playlist_id).await {
        Ok(videos) => videos,
        Err(e) => {
            error!("Failed to fetch playlist videos: {}", e);
            return;
        }
    };

    info!("Found {} videos in playlist", all_playlist_videos.len());

    let mut added_videos = 0;
    for video_id in all_playlist_videos {
        let search_response = es_client
            .get(elasticsearch::GetParts::IndexId(
                "youtube_videos",
                &video_id,
            ))
            .send()
            .await;

        match search_response {
            Ok(response) => {
                // Video doesn't exist, add to queue
                if !response.status_code().is_success() {
                    video_queue.add_video(video_id.clone());
                    added_videos += 1;
                    info!("Added video to queue: {}", video_id);
                } else {
                    info!("Video already exists: {}", video_id);
                }
            }
            Err(e) => {
                error!("Failed to check video existence: {}", e);
            }
        }
    }
    info!("Enqueued {} videos from Playlist", added_videos);
}

// returns the complete video-library-playlist (as list-id) of a channel with the given channel-id
pub async fn get_channel_playlist_id(channel_id: &str) -> Result<String, anyhow::Error> {
    let client = Client::new();
    let api_key = &*YOUTUBE_API_KEY;

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

// Returns list of YT-Videos of a given playlist.
pub async fn fetch_all_playlist_videos(
    playlist_id: &str,
) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let client = Client::new();
    let api_key = &*YOUTUBE_API_KEY;
    let mut all_video_ids = Vec::new();
    let mut next_page_token: Option<String> = None;

    loop {
        // https://developers.google.com/youtube/v3/docs/playlistItems
        let mut url = format!(
            "https://www.googleapis.com/youtube/v3/playlistItems?playlistId={}&key={}&part=snippet",
            playlist_id, api_key
        );

        if let Some(token) = &next_page_token {
            url.push_str(&format!("&pageToken={}", token));
        }

        let response = client
            .get(&url)
            .send()
            .await?
            .json::<serde_json::Value>()
            .await?;

        if let Some(items) = response["items"].as_array() {
            for item in items {
                if let Some(video_id) = item["snippet"]["resourceId"]["videoId"].as_str() {
                    all_video_ids.push(video_id.to_string());
                }
            }
        }

        // Check for next page
        if let Some(token) = response["nextPageToken"].as_str() {
            next_page_token = Some(token.to_string());
        } else {
            break; // No more pages
        }
    }

    Ok(all_video_ids)
}

pub async fn set_active(
    channel_id: &str,
    active: bool,
    es_client: &Elasticsearch,
) -> Result<(), anyhow::Error> {
    es_client
        .update(elasticsearch::UpdateParts::IndexId(
            "monitored_channels",
            channel_id,
        ))
        .body(json!({
            "doc": {
                "active": active
            }
        }))
        .send()
        .await?;

    let mut channels = MONITORED_CHANNELS.write().await;
    if let Some(channel) = channels.iter_mut().find(|c| c.channel_id == channel_id) {
        channel.active = active;
        Ok(())
    } else {
        Err(anyhow::anyhow!("Channel not found"))
    }
}
