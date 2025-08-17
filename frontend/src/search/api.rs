use crate::models::{SearchResult, VideoMetadata};
use gloo_net::http::Request;
use yew::prelude::*;

pub async fn perform_search_request(
    query: &str,
    search_type: &str,
) -> Result<gloo_net::http::Response, gloo_net::Error> {
    let backend_url = "http://localhost:8000";
    let url = format!("{backend_url}/search?q={query}&search_type={search_type}");
    Request::get(&url).send().await
}

pub async fn get_raw_video_metadata(
    video_id: &str,
) -> Result<gloo_net::http::Response, gloo_net::Error> {
    let backend_url = "http://localhost:8000";
    let url = format!("{backend_url}/video/{video_id}");
    Request::get(&url).send().await
}

pub async fn handle_search_response(
    response: Result<gloo_net::http::Response, gloo_net::Error>,
    search_results: &UseStateHandle<Vec<SearchResult>>,
    error_message: &UseStateHandle<Option<String>>,
) {
    match response {
        Ok(response) => {
            if response.ok() {
                match response.json::<Vec<SearchResult>>().await {
                    Ok(results) => search_results.set(results),
                    Err(e) => handle_error(
                        error_message,
                        format!("Failed to parse search results: {e}"),
                    ),
                }
            } else {
                let status = response.status();
                let text = response.text().await.unwrap_or_default();
                handle_error(
                    error_message,
                    format!("Search failed: HTTP {status} - {text}"),
                );
            }
        }
        Err(e) => handle_error(error_message, format!("Failed to connect to backend: {e}")),
    }
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
    search_results: UseStateHandle<Vec<SearchResult>>,
    error_message: UseStateHandle<Option<String>>,
    loading: UseStateHandle<bool>,
) {
    if let Some(window) = web_sys::window() {
        if let Ok(history) = window.history() {
            let url = format!("/?q={}&t={}", query, search_type);
            let _ = history.push_state_with_url(&wasm_bindgen::JsValue::NULL, "", Some(&url));
        }
    }

    let response = perform_search_request(&query, &search_type).await;
    handle_search_response(response, &search_results, &error_message).await;
    loading.set(false);
}

fn handle_error(error_message: &UseStateHandle<Option<String>>, error: String) {
    error_message.set(Some(error.clone()));
    web_sys::console::error_1(&error.into());
}
