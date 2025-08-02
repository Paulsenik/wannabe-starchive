use log::info;
use rocket::http::Status;
use rocket::serde::json::Json;
use rocket::{delete, get, post, State};

use crate::models::{
    AdminEnqueueRequest, AdminEnqueueResponse, AdminLoginRequest, AdminLoginResponse,
    AdminQueueResponse, AdminStats, AdminToken, AdminVideoListResponse,
};
use crate::services::admin_service;
use crate::AppState;

#[post("/login", data = "<login_request>")]
pub async fn admin_login(login_request: Json<AdminLoginRequest>) -> Json<AdminLoginResponse> {
    match admin_service::authenticate_admin(&login_request.token).await {
        Ok(response) => Json(response),
        Err(e) => {
            log::error!("Admin login failed: {e:?}");
            Json(AdminLoginResponse {
                success: false,
                message: "Authentication failed".to_string(),
            })
        }
    }
}

#[get("/stats")]
pub async fn admin_stats(_token: AdminToken, state: &State<AppState>) -> Json<AdminStats> {
    match admin_service::get_admin_stats(&state.es_client).await {
        Ok(stats) => {
            info!("Admin stats retrieved successfully");
            Json(stats)
        }
        Err(e) => {
            log::error!("Failed to get admin stats: {e:?}");
            Json(AdminStats {
                total_videos: 0,
                total_captions: 0,
                last_crawl_time: None,
            })
        }
    }
}

#[get("/queue")]
pub async fn get_queue(_token: AdminToken, state: &State<AppState>) -> Json<AdminQueueResponse> {
    match admin_service::get_admin_queue(&state.video_queue).await {
        Ok(response) => Json(response),
        Err(e) => {
            log::error!("Failed to get admin queue: {e:?}");
            Json(AdminQueueResponse {
                success: false,
                message: "Failed to retrieve queue".to_string(),
                items: vec![],
            })
        }
    }
}

#[post("/queue", data = "<enqueue_request>")]
pub async fn admin_enqueue(
    _token: AdminToken,
    state: &State<AppState>,
    enqueue_request: Json<AdminEnqueueRequest>,
) -> Json<AdminEnqueueResponse> {
    match admin_service::enqueue_video(&state.video_queue, &enqueue_request.url).await {
        Ok(response) => {
            info!("Video enqueued successfully: {}", enqueue_request.url);
            Json(response)
        }
        Err(e) => {
            log::error!("Failed to enqueue video: {e:?}");
            Json(AdminEnqueueResponse {
                success: false,
                message: format!("Failed to enqueue video: {}", e),
            })
        }
    }
}

#[delete("/queue/<id>")]
pub async fn remove_queue_item(
    _token: AdminToken,
    state: &State<AppState>,
    id: &str,
) -> Json<AdminLoginResponse> {
    match admin_service::remove_from_queue(&state.video_queue, id).await {
        Ok(_) => {
            info!("Queue item removed successfully: {}", id);
            Json(AdminLoginResponse {
                success: true,
                message: "Item removed from queue".to_string(),
            })
        }
        Err(e) => {
            log::error!("Failed to remove queue item: {e:?}");
            Json(AdminLoginResponse {
                success: false,
                message: format!("Failed to remove item: {}", e),
            })
        }
    }
}

#[delete("/video/<video_id>")]
pub async fn delete_video_endpoint(
    _token: AdminToken,
    state: &State<AppState>,
    video_id: &str,
) -> Result<Status, Status> {
    match admin_service::delete_video(&state.es_client, video_id).await {
        Ok(_) => {
            info!("Video deleted successfully: {}", video_id);
            Ok(Status::Ok)
        }
        Err(e) => {
            log::error!("Failed to delete video: {e:?}");
            Err(Status::InternalServerError)
        }
    }
}

#[get("/videos?<page>&<per_page>")]
pub async fn get_videos(
    _token: AdminToken,
    state: &State<AppState>,
    page: Option<i64>,
    per_page: Option<i64>,
) -> Json<AdminVideoListResponse> {
    let page = page.unwrap_or(1);
    let per_page = per_page.unwrap_or(20);

    match admin_service::get_videos_paginated(&state.es_client, page, per_page).await {
        Ok(response) => {
            info!(
                "Retrieved {} videos for page {}",
                response.videos.len(),
                page
            );
            Json(response)
        }
        Err(e) => {
            log::error!("Failed to get videos: {e:?}");
            Json(AdminVideoListResponse {
                videos: vec![],
                total: 0,
                page,
                per_page,
            })
        }
    }
}
