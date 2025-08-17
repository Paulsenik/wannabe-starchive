// Add these to your existing models.rs file

use crate::config::ADMIN_TOKEN;
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
    pub active_monitors: i32,
    pub queue_size: usize,
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
    pub playlist_id: Option<String>,
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
                if t == &*ADMIN_TOKEN {
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

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchResponse {
    pub results: Vec<SearchResult>,
    pub total_videos: usize,
    pub total_captions: usize,
    pub page: usize,
    pub per_page: usize,
    pub total_pages: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub video_id: String,
    pub start_time: f64,
    pub end_time: f64,
    /// Combined snippet with neighbors included. Highlight tags are preserved on the anchor snippet.
    pub snippet_html: String,
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
    pub playlists: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct MonitoredChannel {
    pub channel_id: String,
    pub channel_name: String,
    pub active: bool,
    pub created_at: String,
    pub videos_uploaded: i64,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct MonitoredPlaylist {
    pub playlist_id: String,
    pub playlist_name: String,
    pub active: bool,
    pub created_at: String,
    pub videos_added: i64,
}
