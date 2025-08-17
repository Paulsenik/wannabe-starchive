use crate::models::SearchResult;
use crate::services::search_service;
use crate::services::search_service::SearchOptions;
use crate::AppState;
use log::error;
use rocket::http::Status;
use rocket::serde::json::Json;
use rocket::{get, State};

static PAGE_SIZE: usize = 10;

#[get("/?<q>&<page>&<page_size>&<search_type>")]
pub async fn search_captions(
    state: &State<AppState>,
    q: Option<&str>,
    page: Option<usize>,
    page_size: Option<usize>,
    search_type: Option<&str>,
) -> Result<Json<Vec<SearchResult>>, Status> {
    let query_string = q.unwrap_or("");
    let from = page.unwrap_or(0) * page_size.unwrap_or(10);

    if query_string.is_empty() {
        return Err(Status::BadRequest);
    }

    // Parse search_type parameter
    let options = match search_type {
        Some("wide") => SearchOptions::wide(),
        Some("natural") => SearchOptions::natural(),
        _ => SearchOptions::natural(),
    };

    match search_service::search_captions_with_options(
        &state.es_client,
        &query_string,
        from,
        PAGE_SIZE,
        options,
    )
    .await
    {
        Ok(results) => Ok(Json(results)),
        Err(e) => {
            error!("Search failed: {e:?}");
            Err(Status::InternalServerError)
        }
    }
}
