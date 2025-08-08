use crate::models::{MonitoredChannel, MonitoredChannelModify, VideoMetadata};
use crate::services::crawler::VideoQueue;
use crate::services::monitoring::{
    add_monitored_channel, check_channel_for_new_videos, check_playlist_for_new_videos,
    fetch_all_playlist_videos, get_channel_playlist_id, get_monitored_channels_list,
    remove_monitored_channel,
};
use crate::AppState;
use elasticsearch::Elasticsearch;
use rocket::http::Status;
use rocket::serde::json::Json;
use rocket::{delete, get, post, State};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

// Add to routes
#[post("/channel", data = "<channel>")]
pub async fn add_channel(
    channel: Json<MonitoredChannelModify>,
    state: &State<AppState>,
) -> Result<Status, Status> {
    match add_monitored_channel(channel.into_inner(), &state.es_client).await {
        Ok(_) => Ok(Status::Created),
        Err(_) => Err(Status::InternalServerError),
    }
}

#[get("/channel")]
pub async fn get_channels() -> Result<Json<Vec<MonitoredChannel>>, Status> {
    Ok(Json(get_monitored_channels_list().await))
}

#[delete("/channel/<channel_id>")]
pub async fn remove_channel(channel_id: &str, state: &State<AppState>) -> Result<Status, Status> {
    if channel_id.is_empty() {
        return Err(Status::BadRequest);
    }

    match remove_monitored_channel(&channel_id, &state.es_client).await {
        Ok(_) => Ok(Status::NoContent),
        Err(_) => Err(Status::InternalServerError),
    }
}

#[post("/channel/<channel_id>/check")]
pub async fn check_channel(channel_id: &str, state: &State<AppState>) -> Result<Status, Status> {
    check_channel_for_new_videos(&channel_id, &state.es_client, &state.video_queue).await;
    Ok(Default::default())
}

#[post("/playlist/<playlist_id>/check")]
pub async fn check_playlist(playlist_id: &str, state: &State<AppState>) -> Result<Status, Status> {
    check_playlist_for_new_videos(&playlist_id, &state.es_client, &state.video_queue).await;
    Ok(Default::default())
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
    match get_channel_playlist_id(&channel_id).await {
        Ok(playlist_id) => Ok(Json(playlist_id)),
        Err(_) => Err(Status::InternalServerError),
    }
}
