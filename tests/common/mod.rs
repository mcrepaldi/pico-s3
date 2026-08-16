use std::net::SocketAddr;

use aws_credential_types::Credentials;
use aws_sdk_s3::Client;
use tempfile::TempDir;
use tokio::net::TcpListener;

use pico_s3::config::Config;
use pico_s3::server::Server;

pub struct TestServer {
    pub client: Client,
    pub _tmp: TempDir,
}

pub async fn start() -> TestServer {
    let tmp = tempfile::tempdir().expect("tempdir");
    let listener = TcpListener::bind("127.0.0.1:0").await.expect("bind");
    let addr: SocketAddr = listener.local_addr().expect("local_addr");

    let mut config = Config::from_env();
    config.host = "127.0.0.1".into();
    config.port = addr.port();
    config.data_dir = tmp.path().to_path_buf();
    config.db_path = tmp.path().join("store.db");

    let server = Server::new(config).await.expect("server new");
    tokio::spawn(async move {
        let _ = server.run_with_listener(listener).await;
    });

    let sdk_config = aws_config::defaults(aws_config::BehaviorVersion::latest())
        .endpoint_url(format!("http://127.0.0.1:{}", addr.port()))
        .region(aws_config::Region::new("us-east-1"))
        .credentials_provider(Credentials::new("dummy", "dummy", None, None, "test"))
        .load()
        .await;

    let s3_config = aws_sdk_s3::config::Builder::from(&sdk_config)
        .force_path_style(true)
        .build();
    let client = Client::from_conf(s3_config);

    TestServer { client, _tmp: tmp }
}
