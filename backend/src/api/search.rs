use crate::models::{SearchResponse, SearchResult};
use crate::services::search_service;
use crate::services::search_service::SearchType::{Natural, Wide};
use crate::services::search_service::SortBy::Relevance;
use crate::services::search_service::SortOrder::{Asc, Desc};
use crate::services::search_service::{search_captions_with_pagination, SearchOptions, SortBy};
use crate::AppState;
use log::error;
use rocket::http::Status;
use rocket::serde::json::Json;
use rocket::{get, State};

static PAGE_SIZE: usize = 10;

#[get("/?<query>&<type>&<sort>&<page>")]
pub async fn search_captions(
    query: String,
    r#type: Option<String>,
    sort: Option<String>,
    page: Option<usize>,
    state: &State<AppState>,
) -> Result<Json<SearchResponse>, rocket::serde::json::Value> {
    let page = page.unwrap_or(0);
    let per_page = 10; // Limit max per_page to 50

    let sort_by = match sort.as_deref() {
        Some("relevance") => SortBy::Relevance,
        Some("caption_matches") => SortBy::CaptionMatches,
        _ => SortBy::Relevance,
    };

    let search_type_string = r#type.unwrap_or_else(|| "natural".to_string());
    let options = match search_type_string.as_str() {
        "natural" => SearchOptions::natural(sort_by, Desc),
        "wide" => SearchOptions::wide(sort_by, Desc),
        _ => SearchOptions::natural(sort_by, Desc),
    };

    match search_captions_with_pagination(&state.es_client, &query, page, per_page, &options).await
    {
        Ok(response) => Ok(Json(response)),
        Err(e) => {
            eprintln!("Search error: {}", e);
            Err(rocket::serde::json::json!({
                "error": "Search failed",
                "details": e.to_string()
            }))
        }
    }
}
