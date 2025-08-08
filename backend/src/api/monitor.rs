use crate::models::{MonitoredChannel, VideoMetadata};
use crate::services::crawler::{fetch_all_playlist_videos, VideoQueue};
use crate::services::monitoring::get_channel_uploads_playlist_id;
use rocket::http::Status;
use rocket::serde::json::Json;
use rocket::{delete, get, post, State};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

// Add to routes
#[post("/channel", data = "<channel>")]
pub async fn add_monitored_channel(channel: Json<MonitoredChannel>) {
    // Store in Elasticsearch
}

#[get("/channel")]
pub async fn get_monitored_channels() -> Json<Vec<MonitoredChannel>> {
    // Fetch from Elasticsearch
    Json(vec![MonitoredChannel {
        channel_id: "placeholder_id".to_string(),
        channel_name: "".to_string(),
        last_video_id: None,
        check_frequency: "".to_string(),
        active: false,
        created_at: "".to_string(),
    }])
}

#[delete("/channel/<channel_id>")]
pub async fn remove_monitored_channel(channel_id: String) -> Result<Status, Status> {
    // Remove from Elasticsearch
    if channel_id.is_empty() {
        return Err(Status::BadRequest);
    }
    Ok(Status::NoContent)
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PlaylistVideosResponse {
    pub video_ids: Vec<String>,
}

// TEMPORARY - TODO remove, only for testing
#[get("/playlist/<playlist_id>/videos")]
pub async fn get_playlist_videos(
    playlist_id: &str,
) -> Result<Json<PlaylistVideosResponse>, Status> {
    match fetch_all_playlist_videos(&playlist_id).await {
        Ok(video_ids) => Ok(Json(PlaylistVideosResponse { video_ids })),
        Err(_) => Err(Status::InternalServerError),
    }
}

// TEMPORARY - TODO remove, only for testing
#[get("/channel/<channel_id>/video-playlist")]
pub async fn get_channel_upload_playlist(channel_id: &str) -> Result<Json<String>, Status> {
    match get_channel_uploads_playlist_id(&channel_id).await {
        Ok(playlist_id) => Ok(Json(playlist_id)),
        Err(_) => Err(Status::InternalServerError),
    }
}
