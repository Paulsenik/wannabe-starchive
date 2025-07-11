use serde::{Deserialize, Serialize};

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
    pub highlighted_text: Option<String>, // For displaying highlighted matches
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct VideoMetadata {
    pub title: String,
    pub channel_name: String,
    pub upload_date: String,
    pub likes: i64,
    pub views: i64,
    pub duration: String,
    pub comment_count: i64,
}
