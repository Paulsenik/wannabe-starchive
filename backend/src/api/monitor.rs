use crate::models::MonitoredChannel;
use crate::services::monitoring_service::{
    add_monitored_channel, check_channel_for_new_videos, check_playlist_for_new_videos,
    get_monitored_channels_list, remove_monitored_channel, set_active,
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

#[post("/channel/<channel_id>/activate")]
pub async fn activate_channel(channel_id: &str, state: &State<AppState>) -> Result<Status, Status> {
    match set_active(&channel_id, true, &state.es_client).await {
        Ok(_) => Ok(Status::Ok),
        Err(_) => Err(Status::InternalServerError),
    }
}

#[post("/channel/<channel_id>/deactivate")]
pub async fn deactivate_channel(
    channel_id: &str,
    state: &State<AppState>,
) -> Result<Status, Status> {
    match set_active(&channel_id, false, &state.es_client).await {
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

#[derive(Debug, Serialize, Deserialize)]
pub struct PlaylistVideosResponse {
    pub video_ids: Vec<String>,
}
