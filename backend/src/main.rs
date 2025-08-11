extern crate rocket;

use elasticsearch::Elasticsearch;
use rocket::{launch, routes};
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio_cron_scheduler::JobScheduler;

mod api;
mod config;
mod models;
mod services;

use crate::api::{
    activate_channel, add_channel, check_channel, check_playlist, deactivate_channel, get_channels,
    remove_channel,
};
use api::{
    admin_enqueue, admin_login, admin_stats, delete_video_endpoint, get_queue, get_video_metadata,
    get_videos, list_videos, remove_queue_item, search_captions,
};
use config::{create_app_state, create_cors, init_logger, load_environment};
use services::crawler::VideoQueue;

pub struct AppState {
    pub es_client: Elasticsearch,
    pub scheduler: Mutex<JobScheduler>,
    pub video_queue: Arc<VideoQueue>,
}

#[launch]
async fn rocket() -> _ {
    init_logger();
    load_environment();

    let app_state = create_app_state()
        .await
        .expect("Failed to create application state");

    let cors = create_cors().expect("Failed to create CORS configuration");

    rocket::build()
        .manage(app_state)
        .mount("/search", routes![search_captions])
        .mount("/video", routes![list_videos, get_video_metadata])
        .mount(
            "/monitor",
            routes![
                add_channel,
                get_channels,
                remove_channel,
                check_playlist,
                check_channel,
                activate_channel,
                deactivate_channel,
            ],
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
