use crate::env_variable_utils::BACKEND_URL;
use crate::models::{ErrorResponse, SearchResponse, SearchResult, VideoMetadata};
use crate::search::search_options::{SortBy, SortOrder};
use gloo_net::http::Request;
use serde::{Deserialize, Serialize};
use yew::prelude::*;

pub async fn get_raw_video_metadata(
    video_id: &str,
) -> Result<gloo_net::http::Response, gloo_net::Error> {
    let backend_url = &*BACKEND_URL;
    let url = format!("{backend_url}/video/{video_id}");
    Request::get(&url).send().await
}

#[derive(Debug, Deserialize, Serialize)]
pub struct BatchVideoRequest {
    pub video_ids: Vec<String>,
}

pub async fn get_video_metadata(
    video_id: String,
    video_metadata: UseStateHandle<Option<VideoMetadata>>,
    error_message: UseStateHandle<Option<String>>,
    loading: UseStateHandle<bool>,
) {
    let response = get_raw_video_metadata(&video_id).await;

    match response {
        Ok(response) => {
            if response.ok() {
                match response.json::<Option<VideoMetadata>>().await {
                    Ok(results) => video_metadata.set(results),
                    Err(e) => {
                        handle_error(&error_message, format!("Failed to parse video-id: {e}"))
                    }
                }
            } else {
                let status = response.status();
                let text = response.text().await.unwrap_or_default();
                handle_error(
                    &error_message,
                    format!("Search failed: HTTP {status} - {text}"),
                );
            }
        }
        Err(e) => handle_error(&error_message, format!("Failed to connect to backend: {e}")),
    }

    loading.set(false);
}

pub async fn execute_search(
    query: String,
    search_type: &str,
    sort_by: SortBy,
    sort_order: SortOrder,
    page: usize,
    search_results: UseStateHandle<Vec<SearchResult>>,
    total_results: UseStateHandle<Option<(usize, usize)>>, // (videos, captions)
    error_message: UseStateHandle<Option<String>>,
    loading: UseStateHandle<bool>,
) {
    let sort_by_str = match sort_by {
        SortBy::Relevance => "relevance",
        SortBy::UploadDate => "upload_date",
        SortBy::Duration => "duration",
        SortBy::Views => "views",
        SortBy::Likes => "likes",
        SortBy::CaptionMatches => "caption_matches",
    };

    let order_by_str = match sort_order {
        SortOrder::Asc => "asc",
        SortOrder::Desc => "desc",
    };

    let url = format!(
        "{}/search/?query={}&type={}&sort={}&order={}&page={}",
        &*BACKEND_URL,
        urlencoding::encode(&query),
        search_type,
        sort_by_str,
        order_by_str,
        page
    );

    match Request::get(&url).send().await {
        Ok(response) => {
            if response.ok() {
                match response.json::<SearchResponse>().await {
                    Ok(search_response) => {
                        search_results.set(search_response.results);
                        total_results.set(Some((
                            search_response.total_videos,
                            search_response.total_captions,
                        )));
                        error_message.set(None);
                    }
                    Err(e) => {
                        error_message.set(Some(format!("Failed to parse response: {}", e)));
                    }
                }
            } else {
                let status = response.status();
                match response.text().await {
                    Ok(error_text) => {
                        // Try to parse as structured error response first
                        match serde_json::from_str::<ErrorResponse>(&error_text) {
                            Ok(error_response) => {
                                error_message.set(Some(error_response.message));
                            }
                            Err(_) => {
                                // Fallback to raw error text
                                error_message.set(Some(format!(
                                    "Search failed ({}): {}",
                                    status, error_text
                                )));
                            }
                        }
                    }
                    Err(_) => {
                        error_message.set(Some(format!("Search failed with status: {}", status)));
                    }
                }
            }
        }
        Err(e) => {
            error_message.set(Some(format!("Network error: {}", e)));
        }
    }

    loading.set(false);
}

fn handle_error(error_message: &UseStateHandle<Option<String>>, error: String) {
    error_message.set(Some(error.clone()));
    web_sys::console::error_1(&error.into());
}
