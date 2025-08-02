use crate::models::VideoMetadata;
use crate::services::video_service;
use crate::AppState;
use log::{error, info};
use rocket::serde::json::Json;
use rocket::{get, State};
use serde_json::Value;

#[get("/")]
pub async fn list_videos(state: &State<AppState>) -> Json<Vec<String>> {
    match video_service::list_all_videos(&state.es_client).await {
        Ok(video_ids) => {
            info!("Found {} registered videos.", video_ids.len());
            Json(video_ids)
        }
        Err(e) => {
            log::error!("Failed to list videos: {e:?}");
            Json(vec![])
        }
    }
}

#[get("/<id>")]
pub async fn get_video_metadata(state: &State<AppState>, id: &str) -> Json<Option<VideoMetadata>> {
    match state
        .es_client
        .get(elasticsearch::GetParts::IndexId("youtube_videos", id))
        .send()
        .await
    {
        Ok(response) => {
            if response.status_code().is_success() {
                match response.json::<Value>().await {
                    Ok(json_response) => {
                        if let Some(source) = json_response.get("_source") {
                            if let Ok(metadata) = serde_json::from_value(source.clone()) {
                                return Json(Some(metadata));
                            }
                        }
                    }
                    Err(e) => {
                        error!("Failed to parse Elasticsearch response: {e:?}");
                    }
                }
            }
        }
        Err(e) => {
            error!("Failed to fetch video metadata: {e:?}");
        }
    }
    Json(None)
}
