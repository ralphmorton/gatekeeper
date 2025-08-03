mod util;

use iroh::SecretKey;
use util::{ClientServer, TestInfra};

#[tokio::test]
async fn node_deletion() {
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

    client
        .create_node("other".to_string(), other_id, false)
        .await
        .unwrap();

    let nodes = client.nodes().await.unwrap();
    assert_eq!(nodes.len(), 2);

    let mut node_ids: Vec<String> = nodes.into_iter().map(|n| n.node).collect();
    node_ids.sort();
    let mut expected_node_ids = vec![
        format!("{other_id}"),
        format!("{}", client_server.client_sk.public()),
    ];
    expected_node_ids.sort();
    assert_eq!(node_ids, expected_node_ids);

    client.delete_node(other_id).await.unwrap();

    let nodes = client.nodes().await.unwrap();
    let node_ids: Vec<String> = nodes.into_iter().map(|n| n.node).collect();
    let expected_node_ids = vec![format!("{}", client_server.client_sk.public())];
    assert_eq!(node_ids, expected_node_ids);
}
