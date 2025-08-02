use log::info;
use rocket::serde::json::Json;
use rocket::{get, State};

use crate::models::SearchResult;
use crate::services::search_service;
use crate::AppState;

#[get("/?<q>&<page>&<page_size>")]
pub async fn search_captions(
    state: &State<AppState>,
    q: Option<&str>,
    page: Option<usize>,
    page_size: Option<usize>,
) -> Json<Vec<SearchResult>> {
    let query_string = q.unwrap_or("");
    let from = page.unwrap_or(0) * page_size.unwrap_or(10);
    let size = page_size.unwrap_or(10);

    if query_string.is_empty() {
        return Json(vec![]);
    }

    info!("Searching for: '{query_string}' (from={from}, size={size})");

    match search_service::search_captions(&state.es_client, query_string, from, size).await {
        Ok(results) => {
            info!("Found {} search results.", results.len());
            Json(results)
        }
        Err(e) => {
            log::error!("Search failed: {e:?}");
            Json(vec![])
        }
    }
}
