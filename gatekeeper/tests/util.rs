use std::path::PathBuf;

use gatekeeper::{ALPN, Arbiter, Client, Server};
use iroh::{Endpoint, SecretKey, Watcher, protocol::Router};
use uuid::Uuid;

pub struct TestInfra {
    pub db_path: PathBuf,
}

impl Drop for TestInfra {
    fn drop(&mut self) {
        std::fs::remove_file(&self.db_path).unwrap();
    }
}

#[allow(dead_code)]
impl TestInfra {
    pub async fn new() -> Self {
        let db_path = PathBuf::from(format!("test-{}.db", Uuid::new_v4().to_string()));
        TestInfra { db_path }
    }
}

#[allow(dead_code)]
pub struct ClientServer {
    pub infra: TestInfra,
    pub client: Client,
    pub client_sk: SecretKey,
    pub server: Router,
    pub server_sk: SecretKey,
}

impl ClientServer {
    pub async fn new(infra: TestInfra, remote_setup: bool) -> Self {
        let mut rng = rand::thread_rng();
        let server_sk = SecretKey::generate(&mut rng);
        let client_sk = SecretKey::generate(&mut rng);

        let server_endpoint = Endpoint::builder()
            .discovery_n0()
            .secret_key(server_sk.clone())
            .bind()
            .await
            .unwrap();

        let arbiter = Arbiter::new(infra.db_path.clone(), remote_setup)
            .await
            .unwrap();

        let server = Router::builder(server_endpoint)
            .accept(ALPN, Server::new(arbiter))
            .spawn();

        let server_addr = server.endpoint().node_addr().initialized().await;

        let client_endpoint = Endpoint::builder()
            .discovery_n0()
            .secret_key(client_sk.clone())
            .bind()
            .await
            .unwrap();

        let client = Client::with_addr(client_endpoint, server_addr);

        Self {
            infra,
            client,
            client_sk,
            server,
            server_sk,
        }
    }
}
