use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AdminLoginRequest {
    pub token: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AdminLoginResponse {
    pub success: bool,
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct AdminStats {
    pub total_videos: i64,
    pub total_captions: i64,
    pub last_crawl_time: Option<String>,
    pub active_monitors: i32,
    pub queue_size: usize,
}

impl Default for AdminStats {
    fn default() -> Self {
        Self {
            total_videos: 0,
            total_captions: 0,
            last_crawl_time: None,
            active_monitors: 0,
            queue_size: 0,
        }
    }
}
