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
    pub total_transcripts: i64,
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

#[rocket::get("/admin/stats")]
pub async fn admin_stats(_token: AdminToken) -> rocket::serde::json::Json<AdminStats> {
    // This is a placeholder - you should implement actual stats from your database
    rocket::serde::json::Json(AdminStats {
        total_videos: 0,
        total_transcripts: 0,
        last_crawl_time: None,
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
