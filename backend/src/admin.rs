use elasticsearch::Elasticsearch;
use rocket::http::Status;
use rocket::request::{self, FromRequest, Outcome};
use rocket::serde::{Deserialize, Serialize};
use rocket::{Request, State};
use serde_json::json;
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

#[rocket::post("/admin/login", data = "<login_request>")]
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

#[rocket::get("/admin/stats")]
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

#[rocket::post("/admin/trigger-crawl")]
pub async fn trigger_crawl(_token: AdminToken) -> rocket::serde::json::Json<AdminLoginResponse> {
    // This is a placeholder - you should implement actual crawl triggering
    rocket::serde::json::Json(AdminLoginResponse {
        success: true,
        message: "Crawl triggered successfully".to_string(),
    })
}
