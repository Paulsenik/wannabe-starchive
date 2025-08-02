use elasticsearch::{indices::IndicesCreateParts, Elasticsearch};
use log::{error, info};
use serde_json::json;

pub async fn create_es_index(es_client: &Elasticsearch) {
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
