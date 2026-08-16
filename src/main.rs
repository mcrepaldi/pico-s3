//! Binary entrypoint for the pico-s3 server.

use pico_s3::config::Config;
use pico_s3::error::AppResult;
use pico_s3::server::Server;

/// Starts the S3-compatible local development server.
#[tokio::main]
async fn main() -> AppResult<()> {
    let config = Config::from_env();
    let server = Server::new(config).await?;
    server.run().await
}
