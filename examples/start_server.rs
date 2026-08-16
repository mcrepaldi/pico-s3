//! Minimal example: start a pico-s3 server and print the listening address.
//!
//! Run with:
//!
//! ```text
//! cargo run --example start_server
//! ```
//!
//! The server listens on `127.0.0.1:7331` by default.  Override the port and
//! storage paths via environment variables:
//!
//! ```text
//! S3_PORT=9999 S3_DATA_DIR=/tmp/my-data cargo run --example start_server
//! ```
//!
//! Once running, point any S3-compatible client at `http://127.0.0.1:7331`
//! with any credentials (pico-s3 does not validate them).

use std::path::PathBuf;

use pico_s3::config::Config;
use pico_s3::server::Server;

#[tokio::main]
async fn main() {
    // Build a config directly, or call `Config::from_env()` to read
    // S3_PORT / S3_HOST / S3_DATA_DIR / S3_DB_PATH from the environment.
    let config = Config {
        port: 7331,
        host: "127.0.0.1".to_string(),
        data_dir: PathBuf::from("./data"),
        db_path: PathBuf::from("./data/store.db"),
        log_level: "info".to_string(),
    };

    let address = config.bind_address();
    let server = Server::new(config)
        .await
        .expect("failed to initialise server");

    tracing::info!(address = %address, "starting pico-s3");

    server.run().await.expect("server error");
}
