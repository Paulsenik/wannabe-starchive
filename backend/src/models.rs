use rocket::http::{ContentType, Status};
use rocket::request::Request;
use rocket::response::Responder;
use rocket::serde::{Deserialize, Serialize};
use rocket::{response, Response};
use std::io::Cursor;

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
    pub page_size: usize,
    pub total_pages: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub video_id: String,
    pub start_time: f64,
    pub end_time: f64,
    pub snippet_html: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct VideoMetadata {
    pub title: String,
    pub channel_name: String,
    pub channel_id: String,
    pub upload_date: i64, // unix
    pub crawl_date: i64,  // unix
    pub duration: i64,    // in seconds
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

#[derive(Serialize, Deserialize)]
pub struct ErrorResponse {
    pub error: String,
    pub message: String,
}

impl<'r> Responder<'r, 'static> for ErrorResponse {
    fn respond_to(self, _: &'r Request<'_>) -> response::Result<'static> {
        let json = serde_json::to_string(&self).unwrap();
        Response::build()
            .status(Status::BadRequest)
            .header(ContentType::JSON)
            .sized_body(json.len(), Cursor::new(json))
            .ok()
    }
}
