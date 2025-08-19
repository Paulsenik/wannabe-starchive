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
mod utils;

use crate::api::{
    activate_channel, activate_playlist, add_channel, add_playlist, check_channel, check_playlist,
    deactivate_channel, deactivate_playlist, get_channels, get_playlists, get_videos_metadata,
    remove_channel, remove_playlist,
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
        .mount(
            "/video",
            routes![list_videos, get_video_metadata, get_videos_metadata],
        )
        .mount(
            "/monitor",
            routes![
                add_channel,
                get_channels,
                remove_channel,
                activate_channel,
                deactivate_channel,
                check_channel,
                add_playlist,
                get_playlists,
                remove_playlist,
                activate_playlist,
                deactivate_playlist,
                check_playlist,
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
