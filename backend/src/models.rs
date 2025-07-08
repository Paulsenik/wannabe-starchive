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
