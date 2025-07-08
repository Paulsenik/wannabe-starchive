#![allow(unused_imports)] // Allow unused imports for now as we build out the project
#[macro_use]
extern crate rocket;

use elasticsearch::indices::IndicesCreateParts;
use elasticsearch::{
    http::transport::{SingleNodeConnectionPool, TransportBuilder},
    // Corrected import path for IndicesCreateRequest for elasticsearch v9.x
    DeleteByQueryParts,
    Elasticsearch,
    SearchParts,
};
use env_logger::Builder;
use log::{error, info, LevelFilter};
use rocket::serde::{json::Json, Deserialize, Serialize};
use rocket::{get, launch, post, routes, State};
use serde_json::{json, Value};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use tokio_cron_scheduler::{Job, JobScheduler};
// Explicitly import CorsOptions and AllowedOrigins from rocket_cors
use rocket_cors::{AllowedOrigins, CorsOptions};

mod crawler; // We'll define this module for the YouTube crawler
mod models; // We'll define data models here

use models::{Caption, SearchResult}; // <--- ENSURED THIS IS CORRECT

use crawler::crawl_youtube_captions;

// State struct to hold the Elasticsearch client and other shared resources
pub struct AppState {
    pub es_client: Elasticsearch,
    pub scheduler: Mutex<JobScheduler>,
}

// --- API Endpoints ---

/// Root endpoint
#[get("/")]
async fn index() -> &'static str {
    "Welcome to the YouTube Caption Search Backend!"
}

/// Search endpoint
/// Takes a search query and returns matching captions from Elasticsearch.
#[get("/search?<q>&<page>&<page_size>")]
async fn search_captions(
    state: &State<AppState>,
    q: Option<&str>,
    page: Option<usize>,
    page_size: Option<usize>,
) -> Json<Vec<SearchResult>> {
    let client = &state.es_client;
    let query_string = q.unwrap_or("");
    let from = page.unwrap_or(0) * page_size.unwrap_or(10);
    let size = page_size.unwrap_or(10);

    if query_string.is_empty() {
        return Json(vec![]);
    }

    info!("Searching for: '{query_string}' (from={from}, size={size})");

    let search_body = json!({
        "query": {
            "match": {
                "text": {
                    "query": query_string,
                    "fuzziness": "AUTO" // Allow for some typos
                }
            }
        },
        "from": from,
        "size": size,
        "highlight": {
            "fields": {
                "text": {}
            }
        }
    });

    match client
        .search(SearchParts::Index(&["youtube_captions"]))
        .body(search_body)
        .send()
        .await
    {
        Ok(response) => {
            if response.status_code().is_success() {
                match response.json::<Value>().await {
                    Ok(json_response) => {
                        let mut results: Vec<SearchResult> = Vec::new();
                        if let Some(hits) = json_response["hits"]["hits"].as_array() {
                            for hit in hits {
                                if let Some(source) = hit["_source"].as_object() {
                                    let video_id =
                                        source["video_id"].as_str().unwrap_or("N/A").to_string();
                                    let text = source["text"].as_str().unwrap_or("N/A").to_string();
                                    let start_time = source["start_time"].as_f64().unwrap_or(0.0);
                                    let end_time = source["end_time"].as_f64().unwrap_or(0.0);
                                    let mut highlighted_text = None;

                                    if let Some(highlight) = hit["highlight"]["text"].as_array() {
                                        if let Some(first_highlight) = highlight.first() {
                                            highlighted_text = Some(
                                                first_highlight.as_str().unwrap_or("").to_string(),
                                            );
                                        }
                                    }

                                    results.push(SearchResult {
                                        video_id,
                                        text,
                                        start_time,
                                        end_time,
                                        highlighted_text,
                                    });
                                }
                            }
                        }
                        info!("Found {} search results.", results.len());
                        Json(results)
                    }
                    Err(e) => {
                        error!("Failed to parse Elasticsearch response: {e:?}");
                        Json(vec![])
                    }
                }
            } else {
                error!(
                    "Elasticsearch search failed with status: {}",
                    response.status_code()
                );
                error!("Response body: {:?}", response.text().await);
                Json(vec![])
            }
        }
        Err(e) => {
            error!("Failed to connect to Elasticsearch for search: {e:?}");
            Json(vec![])
        }
    }
}

// --- Rocket Launch ---

#[launch]
async fn rocket() -> _ {
    // Initialize logger
    Builder::new().filter_level(LevelFilter::Info).init();

    info!("Starting Rocket backend...");

    // Load environment variables from .env file
    dotenv::dotenv().ok();

    // Initialize Elasticsearch client
    let es_url =
        std::env::var("ELASTICSEARCH_URL").unwrap_or_else(|_| "http://localhost:9200".to_string());
    info!("Connecting to Elasticsearch at: {es_url}");

    let transport = TransportBuilder::new(SingleNodeConnectionPool::new(es_url.parse().unwrap()))
        .build()
        .unwrap();
    let es_client = Elasticsearch::new(transport);

    // Create Elasticsearch index if it doesn't exist
    // Define the mapping for the 'youtube_captions' index
    let create_index_body = json!({
        "mappings": {
            "properties": {
                "video_id": { "type": "keyword" },
                "text": { "type": "text" },
                "start_time": { "type": "float" },
                "end_time": { "type": "float" }
            }
        }
    });

    match es_client
        .indices()
        .create(IndicesCreateParts::Index("youtube_captions"))
        .body(create_index_body)
        .send()
        .await
    {
        Ok(response) => {
            if response.status_code().is_success() {
                info!("Elasticsearch index 'youtube_captions' created or already exists.");
            } else {
                // Check if the error is "resource_already_exists_exception"
                let response_text = response.text().await.unwrap_or_default();
                if response_text.contains("resource_already_exists_exception") {
                    info!("Elasticsearch index 'youtube_captions' already exists.");
                } else {
                    error!("Failed to create Elasticsearch index: {response_text}");
                }
            }
        }
        Err(e) => {
            error!("Failed to connect to Elasticsearch to create index: {e:?}");
        }
    }

    // Initialize job scheduler for the crawler
    let scheduler = JobScheduler::new().await.unwrap();
    let es_client_clone = es_client.clone(); // Clone client for the job

    // Add a cron job to crawl YouTube captions every 30 minutes (for example)
    // In a real application, you'd want a more sophisticated way to get video IDs.
    // For now, we'll use a hardcoded list in the crawler module.
    let crawl_job = Job::new_async("0 */30 * * * *", move |_uuid, _l| {
        let es_client_for_job = es_client_clone.clone();
        Box::pin(async move {
            info!("Running YouTube caption crawl job...");
            crawl_youtube_captions(&es_client_for_job).await;
            info!("YouTube caption crawl job finished.");
        })
    })
    .unwrap();

    scheduler.add(crawl_job).await.unwrap();
    scheduler.start().await.unwrap();
    info!("Crawler scheduler started.");

    // Set up CORS for Rocket
    let cors = CorsOptions::default()
        .allowed_origins(AllowedOrigins::all())
        .allowed_methods(
            vec![rocket::http::Method::Get, rocket::http::Method::Post]
                .into_iter()
                .map(From::from)
                .collect(),
        )
        .allow_credentials(true)
        .to_cors()
        .expect("Failed to create CORS options");

    rocket::build()
        .manage(AppState {
            es_client,
            scheduler: Mutex::new(scheduler),
        })
        .mount("/", routes![index, search_captions])
        .attach(cors)
}
