//! HTTP server assembly and runtime lifecycle.

use std::sync::Arc;

use tokio::net::TcpListener;
use tracing_subscriber::EnvFilter;

use crate::config::Config;
use crate::error::AppResult;
use crate::router::create_router;
use crate::state::AppState;
use crate::storage::db::DBStore;
use crate::store::{FilesystemObjectStore, RedbMetadataStore};

/// The S3-compatible server.
///
/// Owns the [`AppState`] (stores + config) and exposes two run methods:
/// [`run`](Server::run) which binds its own listener from the config, and
/// [`run_with_listener`](Server::run_with_listener) which accepts a
/// pre-bound [`TcpListener`], useful for tests where you need to know the
/// port before the server starts.
pub struct Server {
    config: Arc<Config>,
    state: AppState,
}

impl Server {
    /// Create a new server instance from `config`.
    ///
    /// Initialises tracing, creates the data directory if it does not exist,
    /// opens (or creates) the redb database, and wires up the metadata and
    /// object stores.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use std::path::PathBuf;
    /// use pico_s3::config::Config;
    /// use pico_s3::server::Server;
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let config = Config {
    ///         port:      7331,
    ///         host:      "127.0.0.1".to_string(),
    ///         data_dir:  PathBuf::from("./data"),
    ///         db_path:   PathBuf::from("./data/store.db"),
    ///         log_level: "info".to_string(),
    ///     };
    ///     let server = Server::new(config).await.expect("server failed to start");
    ///     server.run().await.expect("server error");
    /// }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns [`S3Error::InternalError`](crate::error::S3Error::InternalError)
    /// if the data directory cannot be created or the redb database cannot be
    /// opened.
    pub async fn new(config: Config) -> AppResult<Self> {
        init_tracing(&config.log_level);

        tracing::info!(
            data_dir = %config.data_dir.display(),
            db_path = %config.db_path.display(),
            "initialising pico-s3"
        );

        tokio::fs::create_dir_all(&config.data_dir).await?;
        let db = Arc::new(DBStore::new(&config.db_path)?);
        let metadata = Arc::new(RedbMetadataStore::new(db));
        let objects = Arc::new(FilesystemObjectStore::new(&config.data_dir));

        let config = Arc::new(config);
        let state = AppState {
            metadata,
            objects,
            config: config.clone(),
        };

        Ok(Self { config, state })
    }

    /// Start the server and listen for incoming requests.
    ///
    /// Binds to the address in `config.bind_address()` and serves requests
    /// until the process is killed.  This method never returns under normal
    /// operation.
    ///
    /// Prefer [`run_with_listener`](Server::run_with_listener) in tests so
    /// you can bind to port `0` and learn the assigned port before the server
    /// starts.
    ///
    /// # Errors
    ///
    /// Returns [`S3Error::InternalError`](crate::error::S3Error::InternalError)
    /// if the TCP listener cannot be bound or if axum encounters a fatal
    /// serve error.
    pub async fn run(self) -> AppResult<()> {
        let listener = TcpListener::bind(self.config.bind_address()).await?;
        tracing::info!(
            address = %self.config.bind_address(),
            "listening for S3 requests"
        );
        self.run_with_listener(listener).await
    }

    /// Start the server on a specific listener.
    ///
    /// Accepts a pre-bound [`TcpListener`], which lets you bind to port `0`
    /// (OS-assigned ephemeral port) and inspect the address before the server
    /// starts accepting connections.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use std::path::PathBuf;
    /// use tokio::net::TcpListener;
    /// use pico_s3::config::Config;
    /// use pico_s3::server::Server;
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    ///     let port = listener.local_addr().unwrap().port();
    ///
    ///     let config = Config {
    ///         port,
    ///         host:      "127.0.0.1".to_string(),
    ///         data_dir:  PathBuf::from("./data"),
    ///         db_path:   PathBuf::from("./data/store.db"),
    ///         log_level: "info".to_string(),
    ///     };
    ///
    ///     let server = Server::new(config).await.unwrap();
    ///     // Spawn so the rest of the program can continue while the server runs.
    ///     tokio::spawn(async move { server.run_with_listener(listener).await });
    ///     println!("pico-s3 listening on port {port}");
    /// }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns [`S3Error::InternalError`](crate::error::S3Error::InternalError)
    /// if axum encounters a fatal serve error.
    pub async fn run_with_listener(self, listener: TcpListener) -> AppResult<()> {
        let app = create_router(self.state);
        match axum::serve(listener, app).await {
            Ok(()) => {
                tracing::info!("server shut down");
                Ok(())
            }
            Err(e) => {
                tracing::error!(error = %e, "fatal serve error");
                Err(e.into())
            }
        }
    }
}

/// Initialise the global tracing subscriber from the configured log level.
///
/// The level string is used as an [`EnvFilter`] directive, so it may be a
/// plain level (`info`, `debug`) or a full filter expression
/// (`pico_s3=debug,axum=warn`).  A second call is a no-op: the first
/// subscriber installed wins.
fn init_tracing(log_level: &str) {
    let filter = EnvFilter::try_new(log_level).unwrap_or_else(|_| EnvFilter::new("info"));
    let _ = tracing_subscriber::fmt().with_env_filter(filter).try_init();
}
