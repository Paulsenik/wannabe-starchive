# (Name tbd)

Inspired by [Starchives](https://github.com/kyjackson/starchives?tab=readme-ov-file)

**Rust-Stack:**

- [Rocket](https://rocket.rs/)
- [Yew](https://yew.rs/docs/next/getting-started/introduction)
- [yt-stranscript-rs](https://crates.io/crates/yt-transcript-rs)
- ElasticSearch

## Dependencies

- Docker Compose
- [Rust](https://www.rust-lang.org/tools/install)
  - cargo install trunk
- OpenSSL
    - Debian:
        ```bash
        sudo apt install pkg-config libssl-dev
        ```

## Local Deploy

1. Elasticsearch
    ```bash
    docker compose up
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
