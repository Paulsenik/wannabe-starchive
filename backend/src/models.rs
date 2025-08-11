// Add these to your existing models.rs file

use rocket::request::{FromRequest, Outcome, Request};
use rocket::serde::{Deserialize, Serialize};

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

#[derive(Debug, Serialize, Deserialize)]
pub struct AdminVideoListResponse {
    pub videos: Vec<VideoMetadata>,
    pub total: i64,
    pub page: i64,
    pub per_page: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueueItem {
    pub id: String,
    pub video_id: String,
    pub status: String,
    pub added_at: String,
    pub processed_at: Option<String>,
    pub error_message: Option<String>,
}

// AdminToken FromRequest implementation
#[rocket::async_trait]
impl<'r> FromRequest<'r> for AdminToken {
    type Error = &'static str;

    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let token = request
            .headers()
            .get_one("Authorization")
            .and_then(|auth| auth.strip_prefix("Bearer "));

        match token {
            Some(t) => {
                let expected_token = std::env::var("ADMIN_TOKEN")
                    .unwrap_or_else(|_| "your-secret-admin-token".to_string());

                if t == expected_token {
                    Outcome::Success(AdminToken(t.to_string()))
                } else {
                    Outcome::Error((rocket::http::Status::Unauthorized, "Invalid token"))
                }
            }
            None => Outcome::Error((rocket::http::Status::Unauthorized, "Missing token")),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Caption {
    pub video_id: String,
    pub text: String,
    pub start_time: f64,
    pub end_time: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub video_id: String,
    pub text: String,
    pub start_time: f64,
    pub end_time: f64,
    pub highlighted_text: Option<String>, // For displaying highlighted matches TODO remove
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct VideoMetadata {
    pub title: String,
    pub channel_name: String,
    pub channel_id: String,
    pub upload_date: String,
    pub crawl_date: String,
    pub duration: String,
    pub likes: i64,
    pub views: i64,
    pub comment_count: i64,
    pub has_captions: bool,
    pub tags: Vec<String>,
    pub video_id: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct MonitoredChannel {
    pub channel_id: String,
    pub channel_name: String,
    pub last_video_id: Option<String>, // Track the latest video processed
    pub active: bool,
    pub created_at: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct MonitoredPlaylist {
    pub playlist_id: String,
    pub playlist_name: String,
    pub last_video_id: Option<String>,
    pub active: bool,
    pub created_at: String,
}
