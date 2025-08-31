# (Name tbd)

Inspired by [Starchives](https://github.com/kyjackson/starchives?tab=readme-ov-file)

**Rust-Stack:**

- [Rocket](https://rocket.rs/)
- [Yew](https://yew.rs/docs/next/getting-started/introduction)
- [yt-stranscript-rs](https://crates.io/crates/yt-transcript-rs)
- ElasticSearch

## Deploy

```bash
# Create the directory if it doesn't exist
sudo mkdir -p /deployment/wannabe-starchive/data

# Change ownership to UID 1000 (elasticsearch user)
sudo chown -R 1000:1000 /deployment/wannabe-starchive/data

# Set proper permissions
sudo chmod -R 755 /deployment/wannabe-starchive/data
```

```bash
docker compose up --build
```

## Dependencies

- Docker Compose
- [Rust](https://www.rust-lang.org/tools/install)
    - cargo install trunk
    - rustup target add wasm32-unknown-unknown
- OpenSSL
    - Debian:
        ```bash
        sudo apt install pkg-config libssl-dev
        ```

## Local Deploy

setup `.env` file:

```bash
touch backend/.env
```

`.env`-Variables

```toml
ELASTICSEARCH_URL = "http://localhost:9200"
# https://console.cloud.google.com/apis/api/youtube.googleapis.com/credentials
YOUTUBE_API_KEY = "YOUR-GOOGLE-API-KEY"
ADMIN_TOKEN = "BENE_KANN_KEIN_COUNTER_STRIKE"
LANGUAGE_PRIORITY = "en,en-GB,en-US,de,de-DE"

FRONTEND_URL = "http://localhost:8080"
BACKEND_URL = "http://localhost:8000"

# Optional
CRAWL_BURST_MAX = 1
MONITOR_CHECK_SCHEDULE = "0 */10 * * * *"
CRAWL_QUEUE_SCHEDULE = "*/30 * * * * *"
```

Deploy:

1. Elasticsearch
    ```bash
    docker compose up elasticsearch
    ```
2. Backend
    ```bash
    cd backend
    cargo run
    ```
3. Frontend
    ```bash
    cd frontend
    trunk serve --release
    ```

## (Proxy?)

Circumvent Youtube IP-ban with Proxy, if necessary:

- https://www.webshare.io/?referral_code=w0xno53eb50g
