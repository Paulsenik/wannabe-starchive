use serde::{Deserialize, Serialize};

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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SearchResult {
    pub video_id: String,
    pub text: String,
    pub start_time: f64,
    pub end_time: f64,
    pub highlighted_text: Option<String>,
}
