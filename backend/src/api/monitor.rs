use crate::services::monitoring_service::{
    add_monitored_channel, add_monitored_playlist, check_channel_for_new_videos,
    check_playlist_for_new_videos, get_monitored_channels_list, get_monitored_playlist_list,
    remove_monitored_channel, remove_monitored_playlist, set_channel_active, set_playlist_active,
};
use crate::AppState;
use rocket::http::Status;
use rocket::serde::json::Json;
use rocket::{delete, get, post, State};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct NewChannel {
    input: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NewPlaylist {
    input: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MonitoredChannelStats {
    pub channel_id: String,
    pub channel_name: String,
    pub active: bool,
    pub created_at: String,
    pub videos_indexed: i32,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct MonitoredPlaylistStats {
    pub playlist_id: String,
    pub playlist_name: String,
    pub active: bool,
    pub created_at: String,
    pub videos_indexed: i32,
}

#[post("/channel", data = "<channel>")]
pub async fn add_channel(
    channel: Json<NewChannel>,
    state: &State<AppState>,
) -> Result<Status, Status> {
    match add_monitored_channel(&channel.into_inner().input, &state.es_client).await {
        Ok(_) => Ok(Status::Created),
        Err(_) => Err(Status::InternalServerError),
    }
}

#[get("/channel")]
pub async fn get_channels(
    state: &State<AppState>,
) -> Result<Json<Vec<MonitoredChannelStats>>, Status> {
    Ok(Json(get_monitored_channels_list(&state.es_client).await))
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

#[post("/channel/<channel_id>/activate")]
pub async fn activate_channel(channel_id: &str, state: &State<AppState>) -> Result<Status, Status> {
    match set_channel_active(&channel_id, true, &state.es_client).await {
        Ok(_) => Ok(Status::Ok),
        Err(_) => Err(Status::InternalServerError),
    }
}

#[post("/channel/<channel_id>/deactivate")]
pub async fn deactivate_channel(
    channel_id: &str,
    state: &State<AppState>,
) -> Result<Status, Status> {
    match set_channel_active(&channel_id, false, &state.es_client).await {
        Ok(_) => Ok(Status::Ok),
        Err(_) => Err(Status::InternalServerError),
    }
}

#[post("/playlist", data = "<playlist>")]
pub async fn add_playlist(
    playlist: Json<NewPlaylist>,
    state: &State<AppState>,
) -> Result<Status, Status> {
    match add_monitored_playlist(&playlist.into_inner().input, &state.es_client).await {
        Ok(_) => Ok(Status::Created),
        Err(_) => Err(Status::InternalServerError),
    }
}

#[get("/playlist")]
pub async fn get_playlists() -> Result<Json<Vec<MonitoredPlaylistStats>>, Status> {
    Ok(Json(get_monitored_playlist_list().await))
}

#[delete("/playlist/<playlist_id>")]
pub async fn remove_playlist(playlist_id: &str, state: &State<AppState>) -> Result<Status, Status> {
    if playlist_id.is_empty() {
        return Err(Status::BadRequest);
    }

    match remove_monitored_playlist(&playlist_id, &state.es_client).await {
        Ok(_) => Ok(Status::NoContent),
        Err(_) => Err(Status::InternalServerError),
    }
}

#[post("/playlist/<playlist_id>/activate")]
pub async fn activate_playlist(
    playlist_id: &str,
    state: &State<AppState>,
) -> Result<Status, Status> {
    match set_playlist_active(&playlist_id, true, &state.es_client).await {
        Ok(_) => Ok(Status::Ok),
        Err(_) => Err(Status::InternalServerError),
    }
}

#[post("/playlist/<playlist_id>/deactivate")]
pub async fn deactivate_playlist(
    playlist_id: &str,
    state: &State<AppState>,
) -> Result<Status, Status> {
    match set_playlist_active(&playlist_id, false, &state.es_client).await {
        Ok(_) => Ok(Status::Ok),
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
