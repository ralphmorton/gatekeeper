mod util;

use util::{ClientServer, TestInfra};

#[tokio::test]
async fn with_remote_setup() {
    let infra = TestInfra::new().await;
    let client_server = ClientServer::new(infra, true).await;
    let client = client_server.client;

    let client_pk = client_server.client_sk.public();

    let res = client
        .create_node("self".to_string(), client_pk, true)
        .await;
    assert!(matches!(res, Ok(_)));

    let res = client.roles().await;
    assert!(matches!(res, Ok(_)));
}

#[tokio::test]
async fn without_remote_setup() {
    let infra = TestInfra::new().await;
    let client_server = ClientServer::new(infra, false).await;
    let client = client_server.client;

    let client_pk = client_server.client_sk.public();

    let res = client
        .create_node("self".to_string(), client_pk, true)
        .await;
    assert!(matches!(res, Err(_)));

    let res = client.roles().await;
    assert!(matches!(res, Err(_)));
}
