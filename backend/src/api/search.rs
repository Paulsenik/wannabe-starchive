use crate::models::{ErrorResponse, SearchResponse};
use crate::services::search_service::SortBy::{CaptionMatches, Relevance};
use crate::services::search_service::SortOrder::Desc;
use crate::services::search_service::{search_captions_with_pagination, SearchOptions};
use crate::AppState;
use rocket::serde::json::Json;
use rocket::{get, State};

static PAGE_SIZE: usize = 10;
static MIN_QUERY_SIZE: usize = 3;

#[get("/?<query>&<type>&<sort>&<page>")]
pub async fn search_captions(
    query: String,
    r#type: Option<String>,
    sort: Option<String>,
    page: Option<usize>,
    state: &State<AppState>,
) -> Result<Json<SearchResponse>, ErrorResponse> {
    if query.len() < MIN_QUERY_SIZE {
        eprintln!("Search error: Query too short");
        return Err(ErrorResponse {
            error: "Query too short".to_string(),
            message: format!(
                "Search query must be at least {} characters long.",
                MIN_QUERY_SIZE
            ),
        });
    }

    let sort_by = match sort.as_deref() {
        Some("relevance") => Relevance,
        Some("caption_matches") => CaptionMatches,
        _ => Relevance,
    };

    let page = page.unwrap_or(0);

    let search_type_string = r#type.unwrap_or_else(|| "natural".to_string());
    let options = match search_type_string.as_str() {
        "natural" => SearchOptions::natural(sort_by, Desc),
        "wide" => SearchOptions::wide(sort_by, Desc),
        _ => SearchOptions::natural(sort_by, Desc),
    };

    match search_captions_with_pagination(&state.es_client, &query, page, PAGE_SIZE, &options).await
    {
        Ok(response) => Ok(Json(response)),
        Err(e) => {
            eprintln!("Search error: {}", e);
            Err(ErrorResponse {
                error: "Internal server error".to_string(),
                message: "An error occurred while processing your search request.".to_string(),
            })
        }
    }
}
