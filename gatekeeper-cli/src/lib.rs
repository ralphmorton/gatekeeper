mod cli;

use std::str::FromStr;

pub use cli::Cli;
use gatekeeper::{Client, Cmd};
use iroh::{Endpoint, NodeId};

pub async fn exec(cli: Cli) -> anyhow::Result<()> {
    let endpoint = Endpoint::builder()
        .discovery_n0()
        .secret_key(cli.sk)
        .bind()
        .await?;

    let client = Client::new(endpoint, cli.server);

    match cli.cmd {
        Cmd::Roles => {
            let roles = client.roles().await?;
            println!("{}", roles.join("\n"));
        }
        Cmd::Nodes => {
            for node in client.nodes().await?.iter() {
                println!("{} {} {}", node.node, node.superadmin, node.name);
            }
        }
        Cmd::NodeRoles { node } => {
            let node = NodeId::from_str(&node)?;
            let roles = client.node_roles(node).await?;
            println!("{}", roles.join("\n"));
        }
        Cmd::CreateNode {
            name,
            node,
            superadmin,
        } => {
            let node = NodeId::from_str(&node)?;
            let node = client.create_node(name, node, superadmin).await?;
            println!("{} {} {}", node.node, node.superadmin, node.name);
        }
        Cmd::DeleteNode { node } => {
            let node = NodeId::from_str(&node)?;
            client.delete_node(node).await?;
            println!("ok");
        }
        Cmd::GrantRole { node, role } => {
            let node = NodeId::from_str(&node)?;
            client.grant_role(node, role).await?;
            println!("ok");
        }
        Cmd::RevokeRole { node, role } => {
            let node = NodeId::from_str(&node)?;
            client.revoke_role(node, role).await?;
            println!("ok");
        }
    }

    Ok(())
}
