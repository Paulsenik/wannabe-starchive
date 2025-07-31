#[macro_use]
extern crate rocket;

use elasticsearch::indices::IndicesCreateParts;
use elasticsearch::{
    http::transport::{SingleNodeConnectionPool, TransportBuilder},
    DeleteByQueryParts, DeleteParts, Elasticsearch, SearchParts,
};
use env_logger::Builder;
use log::{error, info, LevelFilter};
use rocket::serde::{json::Json, Deserialize, Serialize};
use rocket::{get, launch, post, routes, State};
use rocket_cors::{AllowedOrigins, CorsOptions};
use serde_json::{json, Value};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use tokio_cron_scheduler::{Job, JobScheduler};

mod admin;
mod crawler;
mod models;

use models::{Caption, SearchResult, VideoMetadata};

use crate::admin::{
    admin_enqueue, admin_login, admin_stats, delete_video_endpoint, get_queue, get_videos,
    remove_queue_item,
};
use crate::crawler::VideoQueue;
use crawler::crawl_youtube_video;

pub struct AppState {
    pub es_client: Elasticsearch,
    pub scheduler: Mutex<JobScheduler>,
    pub video_queue: Arc<VideoQueue>,
}

#[get("/")]
async fn index() -> &'static str {
    "Welcome to the YouTube Caption Search Backend!"
}

/// Search endpoint
/// Takes a search query and returns matching captions from Elasticsearch.
fn build_search_query(query_string: &str, from: usize, size: usize) -> Value {
    json!({
        "size": size,
        "query": {
            "bool": {
                "should": [
                    {
                        "match": {
                            "text": {
                                "query": query_string,
                                "boost": 3.0  // Highest priority for exact matches
                            }
                        }
                    },
                    {
                        "match": {
                            "text": {
                                "query": query_string,
                                "fuzziness": 1,  // Only 1 character difference allowed
                                "prefix_length": 2,  // First 2 characters must match exactly
                                "max_expansions": 10,  // Limit expansions for performance
                                "boost": 1.5  // Medium priority for minor typos
                            }
                        }
                    }
                ],
                "minimum_should_match": 1
            }
        },
        "collapse": {
            "field": "video_id",
            "inner_hits": {
                "name": "captions",
                "size": 10000
            }
        },
        "sort": ["_score"],
        "from": from,
        "_source": ["video_id", "text", "start_time", "end_time"],
        "highlight": {
            "fields": {
                "text": {}
            }
        }
    })
}

fn parse_search_result(source: &serde_json::Map<String, Value>, inner_hit: &Value) -> SearchResult {
    let video_id = source["video_id"].as_str().unwrap_or("N/A").to_string();
    let text = source["text"].as_str().unwrap_or("N/A").to_string();
    let start_time = source["start_time"].as_f64().unwrap_or(0.0);
    let end_time = source["end_time"].as_f64().unwrap_or(0.0);

    let highlighted_text = inner_hit["highlight"]["text"]
        .as_array()
        .and_then(|highlight| highlight.first())
        .and_then(|first_highlight| first_highlight.as_str())
        .map(String::from);

    SearchResult {
        video_id,
        text,
        start_time,
        end_time,
        highlighted_text,
    }
}

async fn process_search_response(response: Value) -> Vec<SearchResult> {
    let mut results = Vec::new();

    if let Some(hits) = response["hits"]["hits"].as_array() {
        for hit in hits {
            if let Some(inner_hits) = hit["inner_hits"]["captions"]["hits"]["hits"].as_array() {
                for inner_hit in inner_hits {
                    if let Some(source) = inner_hit["_source"].as_object() {
                        results.push(parse_search_result(source, inner_hit));
                    }
                }
            }
        }
    }

    results
}

#[get("/video/<id>")]
async fn get_video_metadata(state: &State<AppState>, id: &str) -> Json<Option<VideoMetadata>> {
    match state
        .es_client
        .get(elasticsearch::GetParts::IndexId("youtube_videos", id))
        .send()
        .await
    {
        Ok(response) => {
            if response.status_code().is_success() {
                match response.json::<Value>().await {
                    Ok(json_response) => {
                        if let Some(source) = json_response.get("_source") {
                            if let Ok(metadata) = serde_json::from_value(source.clone()) {
                                return Json(Some(metadata));
                            }
                        }
                    }
                    Err(e) => {
                        error!("Failed to parse Elasticsearch response: {e:?}");
                    }
                }
            }
        }
        Err(e) => {
            error!("Failed to fetch video metadata: {e:?}");
        }
    }
    Json(None)
}

#[get("/search?<q>&<page>&<page_size>")]
async fn search_captions(
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
    let search_body = build_search_query(query_string, from, size);

    match state
        .es_client
        .search(SearchParts::Index(&["youtube_captions"]))
        .body(search_body)
        .send()
        .await
    {
        Ok(response) => {
            if response.status_code().is_success() {
                match response.json::<Value>().await {
                    Ok(json_response) => {
                        let results = process_search_response(json_response).await;
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

#[get("/video")]
async fn list_videos(state: &State<AppState>) -> Json<Vec<String>> {
    let search_body = json!({
        "size": 10000,
        "query": {
            "match_all": {}
        },
        "_source": false
    });

    match state
        .es_client
        .search(SearchParts::Index(&["youtube_videos"]))
        .body(search_body)
        .send()
        .await
    {
        Ok(response) => {
            if response.status_code().is_success() {
                match response.json::<Value>().await {
                    Ok(json_response) => {
                        let mut video_ids = Vec::new();

                        if let Some(hits) = json_response["hits"]["hits"].as_array() {
                            for hit in hits {
                                if let Some(id) = hit["_id"].as_str() {
                                    video_ids.push(id.to_string());
                                }
                            }
                        }

                        info!("Found {} registered videos.", video_ids.len());
                        Json(video_ids)
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
                Json(vec![])
            }
        }
        Err(e) => {
            error!("Failed to connect to Elasticsearch for video listing: {e:?}");
            Json(vec![])
        }
    }
}

async fn create_es_index(es_client: &Elasticsearch) {
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
}

#[launch]
async fn rocket() -> _ {
    Builder::new().filter_level(LevelFilter::Info).init();
    info!("Starting Rocket backend...");
    dotenv::dotenv().ok();

    let es_url =
        std::env::var("ELASTICSEARCH_URL").unwrap_or_else(|_| "http://localhost:9200".to_string());
    info!("Connecting to Elasticsearch at: {es_url}");

    let transport = TransportBuilder::new(SingleNodeConnectionPool::new(es_url.parse().unwrap()))
        .build()
        .unwrap();
    let es_client = Elasticsearch::new(transport);
    let video_queue = Arc::new(VideoQueue::new());

    create_es_index(&es_client).await;

    let scheduler = JobScheduler::new().await.unwrap();
    let es_client_clone = es_client.clone();


    let video_queue_clone = video_queue.clone();
    let crawl_job = Job::new_async("*/30 * * * * *", move |_uuid, _l| {
        let es_client_for_job = es_client_clone.clone();
        let queue = video_queue_clone.clone();
        Box::pin(async move {
            if queue.get_size() == 0 {
                return;
            }
            crawl_youtube_video(&es_client_for_job, &queue).await;
        })
    })
    .unwrap();

    scheduler.add(crawl_job).await.unwrap();
    scheduler.start().await.unwrap();
    info!("Crawler scheduler started.");

    use rocket::http::Method;
    use rocket_cors::AllowedHeaders;

    let cors = CorsOptions::default()
        .allowed_origins(AllowedOrigins::some_exact(&["http://localhost:8080"]))
        .allowed_methods(
            vec![
                Method::Get,
                Method::Post,
                Method::Put,
                Method::Delete,
                Method::Options,
            ]
            .into_iter()
            .map(From::from)
            .collect(),
        )
        .allowed_headers(AllowedHeaders::some(&[
            "Authorization",
            "Accept",
            "Content-Type",
        ]))
        .allow_credentials(true)
        .to_cors()
        .expect("Failed to create CORS options");

    rocket::build()
        .manage(AppState {
            es_client,
            scheduler: Mutex::new(scheduler),
            video_queue,
        })
        .mount(
            "/",
            routes![index, search_captions, get_video_metadata, list_videos],
        )
        .mount(
            "/admin",
            routes![
                admin_login,
                admin_stats,
                get_queue,
                admin_enqueue,
                remove_queue_item,
                delete_video_endpoint,
                get_videos,
            ],
        )
        .attach(cors)
}
