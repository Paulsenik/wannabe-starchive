use crate::models::AdminToken;
use crate::services::crawler::{crawl_youtube_video, VideoQueue};
use crate::services::elasticsearch_service::create_es_index;
use crate::services::monitoring_service::setup_monitoring;
use crate::AppState;
use anyhow::Result;
use elasticsearch::{
    http::transport::{SingleNodeConnectionPool, TransportBuilder},
    Elasticsearch,
};
use env_logger::Builder;
use lazy_static::lazy_static;
use log::{info, LevelFilter};
use rocket::http::{Method, Status};
use rocket::request::{FromRequest, Outcome};
use rocket::Request;
use rocket_cors::{AllowedHeaders, AllowedOrigins, CorsOptions};
use std::env;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio_cron_scheduler::{Job, JobScheduler};

lazy_static! {
    pub static ref YOUTUBE_API_KEY: String =
        env::var("YOUTUBE_API_KEY").expect("YOUTUBE_API_KEY environment variable must be set");
    pub static ref ADMIN_TOKEN: String =
        env::var("ADMIN_TOKEN").expect("ADMIN_TOKEN environment variable must be set");
    pub static ref ELASTICSEARCH_URL: String =
        env::var("ELASTICSEARCH_URL").unwrap_or_else(|_| "http://localhost:9200".to_string());
    pub static ref CRAWL_BURST_MAX: i32 = env::var("CRAWL_BURST_MAX")
        .unwrap_or_else(|_| "1".to_string())
        .parse::<i32>()
        .unwrap_or(1);
    pub static ref MONITOR_CHECK_SCHEDULE: String =
        env::var("MONITOR_CHECK_SCHEDULE").unwrap_or_else(|_| "0 */10 * * * *".to_string());
    pub static ref CRAWL_QUEUE_SCHEDULE: String =
        env::var("CRAWL_QUEUE_SCHEDULE").unwrap_or_else(|_| "*/30 * * * * *".to_string());
}

pub fn init_logger() {
    Builder::new().filter_level(LevelFilter::Info).init();
    info!("Starting Rocket backend...");
}

pub fn load_environment() {
    dotenv::dotenv().ok();
}

pub fn create_elasticsearch_client() -> Result<Elasticsearch> {
    let es_url = &*ELASTICSEARCH_URL;
    info!("Connecting to Elasticsearch at: {es_url}");

    let transport =
        TransportBuilder::new(SingleNodeConnectionPool::new(es_url.parse()?)).build()?;

    Ok(Elasticsearch::new(transport))
}

pub async fn setup_queue_scheduler(
    es_client: Elasticsearch,
    video_queue: Arc<VideoQueue>,
) -> Result<JobScheduler> {
    let scheduler = JobScheduler::new().await?;
    let es_client_clone = es_client.clone();
    let video_queue_clone = video_queue.clone();
    let craw_burst_max = CRAWL_BURST_MAX.clone();

    let crawl_job = Job::new_async(CRAWL_QUEUE_SCHEDULE.as_str(), move |_uuid, _l| {
        let es_client_for_job = es_client_clone.clone();
        let queue = video_queue_clone.clone();
        Box::pin(async move {
            if queue.get_size() == 0 {
                return;
            }
            crawl_youtube_video(&es_client_for_job, &queue, craw_burst_max).await;
        })
    })?;

    scheduler.add(crawl_job).await?;
    scheduler.start().await?;
    info!("Crawler scheduler started.");

    Ok(scheduler)
}

pub async fn create_app_state() -> Result<AppState> {
    let es_client = create_elasticsearch_client()?;
    let video_queue = Arc::new(VideoQueue::new());

    create_es_index(&es_client).await;

    let scheduler = setup_queue_scheduler(es_client.clone(), video_queue.clone()).await?;

    let es_client_arc = Arc::new(es_client.clone());

    setup_monitoring(es_client_arc, video_queue.clone())
        .await
        .expect("Monitoring setup failed.");

    Ok(AppState {
        es_client,
        scheduler: Mutex::new(scheduler),
        video_queue,
    })
}

pub fn create_cors() -> Result<rocket_cors::Cors> {
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
        .map_err(|e| anyhow::anyhow!("Failed to create CORS options: {}", e))?;

    Ok(cors)
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for AdminToken {
    type Error = &'static str;

    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let token = request
            .headers()
            .get_one("Authorization")
            .and_then(|auth| auth.strip_prefix("Bearer "));

        match token {
            Some(t) => {
                if t == &*ADMIN_TOKEN {
                    Outcome::Success(AdminToken(t.to_string()))
                } else {
                    Outcome::Error((Status::Unauthorized, "Invalid token"))
                }
            }
            None => Outcome::Error((Status::Unauthorized, "Missing token")),
        }
    }
}
