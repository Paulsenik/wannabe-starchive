use crate::services::crawler::VideoQueue;
use elasticsearch::Elasticsearch;
use std::sync::Arc;
use tokio_cron_scheduler::{Job, JobScheduler};

// TODO - complete
pub async fn setup_channel_monitoring(
    es_client: Arc<Elasticsearch>,
    video_queue: Arc<VideoQueue>,
) -> Result<(), Box<dyn std::error::Error>> {
    let sched = JobScheduler::new().await?;

    // Check channels every hour
    let es_client_clone = es_client.clone();
    let queue_clone = video_queue.clone();

    sched
        .add(Job::new_async("0 0 * * * *", move |_uuid, _l| {
            let es_client = es_client_clone.clone();
            let queue = queue_clone.clone();
            Box::pin(async move {
                check_monitored_channels(&es_client, &queue).await;
            })
        })?)
        .await?;

    sched.start().await?;
    Ok(())
}

async fn check_monitored_channels(es_client: &Elasticsearch, video_queue: &VideoQueue) {
    // TODO - complete
    // Fetch monitored channels from Elasticsearch
    // Check each channel for new videos
    // Add new videos to queue
}
