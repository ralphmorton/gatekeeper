mod util;

use gatekeeper::Client;
use iroh::{Endpoint, SecretKey, Watcher};
use util::{ClientServer, TestInfra};

#[tokio::test]
async fn non_privileged_node_creation() {
    let infra = TestInfra::new().await;
    let client_server = ClientServer::new(infra, true).await;
    let client = client_server.client;

    let client_pk = client_server.client_sk.public();
    let res = client
        .create_node("self".to_string(), client_pk, true)
        .await;
    assert!(matches!(res, Ok(_)));

    let mut rng = rand::thread_rng();
    let other_sk = SecretKey::generate(&mut rng);
    let other_id = other_sk.public();

    let other = client
        .create_node("other".to_string(), other_id, false)
        .await
        .unwrap();

    assert_eq!(other.name, "other".to_string());
    assert_eq!(other.node, format!("{other_id}"));
    assert_eq!(other.superadmin, false);

    let server_addr = client_server
        .server
        .endpoint()
        .node_addr()
        .initialized()
        .await;

    let other_client_endpoint = Endpoint::builder()
        .discovery_n0()
        .secret_key(other_sk.clone())
        .bind()
        .await
        .unwrap();

    let other_client = Client::with_addr(other_client_endpoint, server_addr);

    let res = other_client.roles().await;
    assert!(matches!(res, Err(_)));
}

#[tokio::test]
async fn privileged_node_creation() {
    let infra = TestInfra::new().await;
    let client_server = ClientServer::new(infra, true).await;
    let client = client_server.client;

    let client_pk = client_server.client_sk.public();
    let res = client
        .create_node("self".to_string(), client_pk, true)
        .await;
    assert!(matches!(res, Ok(_)));

    let mut rng = rand::thread_rng();
    let other_sk = SecretKey::generate(&mut rng);
    let other_id = other_sk.public();

    let other = client
        .create_node("other".to_string(), other_id, true)
        .await
        .unwrap();

    assert_eq!(other.name, "other".to_string());
    assert_eq!(other.node, format!("{other_id}"));
    assert_eq!(other.superadmin, true);

    let server_addr = client_server
        .server
        .endpoint()
        .node_addr()
        .initialized()
        .await;

    let other_client_endpoint = Endpoint::builder()
        .discovery_n0()
        .secret_key(other_sk.clone())
        .bind()
        .await
        .unwrap();

    let other_client = Client::with_addr(other_client_endpoint, server_addr);

    let res = other_client.roles().await;
    assert!(matches!(res, Ok(_)));
}
