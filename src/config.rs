//! Server configuration loaded from environment variables.

use std::path::PathBuf;

/// Server configuration loaded from environment variables.
#[derive(Debug, Clone)]
pub struct Config {
    /// HTTP port.
    pub port: u16,
    /// Base data directory.
    pub data_dir: PathBuf,
    /// redb path.
    pub db_path: PathBuf,
    /// Bind host.
    pub host: String,
    /// Tracing log level.
    pub log_level: String,
}

impl Config {
    /// Load configuration from environment variables with defaults.
    ///
    /// Reads a `.env` file if one is present, then falls back to the
    /// environment, then to hardcoded defaults.
    ///
    /// | Variable      | Default               | Description              |
    /// |---------------|-----------------------|--------------------------|
    /// | `S3_PORT`     | `7331`                | TCP port to listen on    |
    /// | `S3_HOST`     | `127.0.0.1`           | Bind address             |
    /// | `S3_DATA_DIR` | `./data`              | Object storage root      |
    /// | `S3_DB_PATH`  | `./data/store.db`     | redb database file path  |
    /// | `RUST_LOG`    | `info`                | Tracing log level        |
    ///
    /// # Example
    ///
    /// ```no_run
    /// // Reads environment variables; defaults are used when a variable is unset.
    /// let config = pico_s3::config::Config::from_env();
    /// println!("Binding to {}", config.bind_address());
    /// ```
    pub fn from_env() -> Self {
        let _ = dotenv::dotenv();
        let data_dir = std::env::var("S3_DATA_DIR").unwrap_or_else(|_| "./data".to_string());
        Self {
            port: std::env::var("S3_PORT")
                .ok()
                .and_then(|v| v.parse::<u16>().ok())
                .unwrap_or(7331),
            db_path: std::env::var("S3_DB_PATH")
                .map(PathBuf::from)
                .unwrap_or_else(|_| PathBuf::from(format!("{data_dir}/store.db"))),
            host: std::env::var("S3_HOST").unwrap_or_else(|_| "127.0.0.1".to_string()),
            log_level: std::env::var("RUST_LOG").unwrap_or_else(|_| "info".to_string()),
            data_dir: PathBuf::from(data_dir),
        }
    }

    /// Returns the full bind address as host:port.
    ///
    /// # Example
    ///
    /// ```
    /// use std::path::PathBuf;
    /// use pico_s3::config::Config;
    ///
    /// let config = Config {
    ///     port: 8080,
    ///     host: "0.0.0.0".to_string(),
    ///     data_dir: PathBuf::from("./data"),
    ///     db_path: PathBuf::from("./data/store.db"),
    ///     log_level: "info".to_string(),
    /// };
    /// assert_eq!(config.bind_address(), "0.0.0.0:8080");
    /// ```
    pub fn bind_address(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{Mutex, OnceLock};

    use super::Config;

    fn env_lock() -> &'static Mutex<()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(()))
    }

    #[test]
    fn has_defaults() {
        let _guard = env_lock().lock().expect("lock");
        unsafe {
            std::env::remove_var("S3_PORT");
            std::env::remove_var("S3_DATA_DIR");
            std::env::remove_var("S3_DB_PATH");
            std::env::remove_var("S3_HOST");
            std::env::remove_var("RUST_LOG");
        }
        let c = Config::from_env();
        assert_eq!(c.port, 7331);
        assert_eq!(c.host, "127.0.0.1");
        assert_eq!(c.log_level, "info");
        assert_eq!(c.bind_address(), "127.0.0.1:7331");
    }

    #[test]
    fn supports_overrides() {
        let _guard = env_lock().lock().expect("lock");
        unsafe {
            std::env::set_var("S3_PORT", "9911");
            std::env::set_var("S3_DATA_DIR", "/tmp/pico-data");
            std::env::set_var("S3_DB_PATH", "/tmp/pico-data/custom.db");
            std::env::set_var("S3_HOST", "0.0.0.0");
            std::env::set_var("RUST_LOG", "debug");
        }

        let c = Config::from_env();
        assert_eq!(c.port, 9911);
        assert_eq!(c.data_dir.to_string_lossy(), "/tmp/pico-data");
        assert_eq!(c.db_path.to_string_lossy(), "/tmp/pico-data/custom.db");
        assert_eq!(c.host, "0.0.0.0");
        assert_eq!(c.log_level, "debug");
        assert_eq!(c.bind_address(), "0.0.0.0:9911");

        unsafe {
            std::env::remove_var("S3_PORT");
            std::env::remove_var("S3_DATA_DIR");
            std::env::remove_var("S3_DB_PATH");
            std::env::remove_var("S3_HOST");
            std::env::remove_var("RUST_LOG");
        }
    }
}
