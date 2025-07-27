use crate::crawler::QueueItem;
use elasticsearch::Elasticsearch;
use rocket::http::Status;
use rocket::request::{self, FromRequest, Outcome};
use rocket::serde::{Deserialize, Serialize};
use rocket::{Request, State};
use std::env;

#[derive(Debug, Serialize, Deserialize)]
pub struct AdminToken(pub String);

#[derive(Serialize, Deserialize)]
pub struct AdminLoginRequest {
    pub token: String,
}

#[derive(Serialize, Deserialize)]
pub struct AdminLoginResponse {
    pub success: bool,
    pub message: String,
}

#[derive(Serialize, Deserialize)]
pub struct AdminStats {
    pub total_videos: i64,
    pub total_captions: i64,
    pub last_crawl_time: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AdminEnqueueRequest {
    pub url: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AdminEnqueueResponse {
    pub success: bool,
    pub message: String,
}

#[derive(Serialize, Deserialize)]
pub struct AdminQueueResponse {
    pub success: bool,
    pub message: String,
    pub items: Vec<QueueItem>,
}

fn extract_youtube_video_id(url: &str) -> Option<String> {
    use url::Url;

    let parsed_url = Url::parse(url).ok()?;
    let host = parsed_url.host_str()?;

    // Handle different YouTube URL formats
    match host {
        "www.youtube.com" | "youtube.com" | "m.youtube.com" => {
            // Standard YouTube URLs: https://www.youtube.com/watch?v=VIDEO_ID
            if parsed_url.path() == "/watch" {
                parsed_url
                    .query_pairs()
                    .find(|(key, _)| key == "v")
                    .map(|(_, value)| value.to_string())
            } else {
                None
            }
        }
        "youtu.be" => {
            // Short YouTube URLs: https://youtu.be/VIDEO_ID
            parsed_url
                .path_segments()
                .and_then(|segments| segments.last())
                .map(|id| id.to_string())
        }
        _ => None,
    }
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for AdminToken {
    type Error = &'static str;

    async fn from_request(request: &'r Request<'_>) -> request::Outcome<Self, Self::Error> {
        let token = request
            .headers()
            .get_one("Authorization")
            .and_then(|header| header.strip_prefix("Bearer "));

        match token {
            Some(token) => {
                let admin_token =
                    env::var("ADMIN_TOKEN").unwrap_or_else(|_| "default_admin_token".to_string());

                if token == admin_token {
                    Outcome::Success(AdminToken(token.to_string()))
                } else {
                    Outcome::Error((Status::Unauthorized, "Invalid admin token"))
                }
            }
            None => Outcome::Error((Status::Unauthorized, "Missing admin token")),
        }
    }
}

#[rocket::post("/login", data = "<login_request>")]
pub async fn admin_login(
    login_request: rocket::serde::json::Json<AdminLoginRequest>,
) -> rocket::serde::json::Json<AdminLoginResponse> {
    let admin_token = env::var("ADMIN_TOKEN").unwrap_or_else(|_| "default_admin_token".to_string());

    if login_request.token == admin_token {
        rocket::serde::json::Json(AdminLoginResponse {
            success: true,
            message: "Login successful".to_string(),
        })
    } else {
        rocket::serde::json::Json(AdminLoginResponse {
            success: false,
            message: "Invalid admin token".to_string(),
        })
    }
}

async fn get_index_count(es_client: &Elasticsearch, index: &str) -> i64 {
    match es_client
        .count(elasticsearch::CountParts::Index(&[index]))
        .send()
        .await
    {
        Ok(response) => {
            let response_body = response
                .json::<serde_json::Value>()
                .await
                .unwrap_or_default();
            response_body["count"].as_i64().unwrap_or(0)
        }
        Err(_) => 0,
    }
}

async fn get_last_crawl_time(es_client: &Elasticsearch) -> Option<String> {
    use serde_json::json;

    match es_client
        .search(elasticsearch::SearchParts::Index(&["youtube_videos"]))
        .body(json!({
            "size": 1,
            "sort": [{"crawl_date": {"order": "desc"}}],
            "_source": ["crawl_date"]
        }))
        .send()
        .await
    {
        Ok(response) => {
            let response_body = response
                .json::<serde_json::Value>()
                .await
                .unwrap_or_default();
            response_body["hits"]["hits"][0]["_source"]["crawl_date"]
                .as_str()
                .map(String::from)
        }
        Err(_) => None,
    }
}

#[rocket::get("/stats")]
pub async fn admin_stats(
    _token: AdminToken,
    state: &State<crate::AppState>,
) -> rocket::serde::json::Json<AdminStats> {
    let es_client = &state.es_client;

    let total_videos = get_index_count(es_client, "youtube_videos").await;
    let total_captions = get_index_count(es_client, "youtube_captions").await;
    let last_crawl_time = get_last_crawl_time(es_client).await;

    log::info!("Stats: captions={total_captions}; videos={total_videos}; last_crawl_time={last_crawl_time:?};");

    rocket::serde::json::Json(AdminStats {
        total_videos,
        total_captions,
        last_crawl_time,
    })
}

#[rocket::get("/queue")]
pub async fn get_queue(
    _token: AdminToken,
    state: &State<crate::AppState>,
) -> rocket::serde::json::Json<AdminQueueResponse> {
    let items = state.video_queue.get_all_items();

    rocket::serde::json::Json(AdminQueueResponse {
        success: true,
        message: "Queue items retrieved successfully".to_string(),
        items,
    })
}

#[rocket::post("/queue", data = "<enqueue_request>")]
pub async fn admin_enqueue(
    _token: AdminToken,
    state: &State<crate::AppState>,
    enqueue_request: rocket::serde::json::Json<AdminEnqueueRequest>,
) -> rocket::serde::json::Json<AdminEnqueueResponse> {
    let url = &enqueue_request.url;

    // Extract video ID from URL
    let video_id = match extract_youtube_video_id(url) {
        Some(id) => id,
        None => {
            return rocket::serde::json::Json(AdminEnqueueResponse {
                success: false,
                message: "Invalid YouTube URL format".to_string(),
            });
        }
    };

    // Add video to the queue
    let item_id = state.video_queue.add_video(url.clone(), video_id);

    if !item_id.is_empty() {
        rocket::serde::json::Json(AdminEnqueueResponse {
            success: true,
            message: "URL added to queue successfully".to_string(),
        })
    } else {
        rocket::serde::json::Json(AdminEnqueueResponse {
            success: false,
            message: "Failed to add URL to queue".to_string(),
        })
    }
}

#[rocket::delete("/queue/<id>")]
pub async fn remove_queue_item(
    _token: AdminToken,
    state: &State<crate::AppState>,
    id: &str,
) -> rocket::serde::json::Json<AdminLoginResponse> {
    if state.video_queue.remove_item(&id) {
        rocket::serde::json::Json(AdminLoginResponse {
            success: true,
            message: "Queue item removed successfully".to_string(),
        })
    } else {
        rocket::serde::json::Json(AdminLoginResponse {
            success: false,
            message: "Queue item not found".to_string(),
        })
    }
}

#[rocket::options("/queue/<_id>")]
pub fn delete_queue_item_options(_id: String) -> rocket::response::status::NoContent {
    rocket::response::status::NoContent
}
