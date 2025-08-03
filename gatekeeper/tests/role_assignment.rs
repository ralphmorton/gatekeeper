mod util;

use util::{ClientServer, TestInfra};

#[tokio::test]
async fn role_assignment() {
    let infra = TestInfra::new().await;
    let client_server = ClientServer::new(infra, true).await;
    let client = client_server.client;

    let client_pk = client_server.client_sk.public();
    let res = client
        .create_node("self".to_string(), client_pk, true)
        .await;
    assert!(matches!(res, Ok(_)));

    client
        .grant_role(client_pk, "foo".to_string())
        .await
        .unwrap();
    client
        .grant_role(client_pk, "bar".to_string())
        .await
        .unwrap();
    client
        .grant_role(client_pk, "baz".to_string())
        .await
        .unwrap();

    let roles = client.roles().await.unwrap();
    assert_eq!(
        roles,
        vec!["bar".to_string(), "baz".to_string(), "foo".to_string()]
    );

    let node_roles = client.node_roles(client_pk).await.unwrap();
    assert_eq!(
        node_roles,
        vec!["bar".to_string(), "baz".to_string(), "foo".to_string()]
    );

    client
        .revoke_role(client_pk, "baz".to_string())
        .await
        .unwrap();

    let roles = client.roles().await.unwrap();
    assert_eq!(
        roles,
        vec!["bar".to_string(), "baz".to_string(), "foo".to_string()]
    );

    let node_roles = client.node_roles(client_pk).await.unwrap();
    assert_eq!(node_roles, vec!["bar".to_string(), "foo".to_string()]);
}
