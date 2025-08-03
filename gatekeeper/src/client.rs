use bincode::Decode;
use iroh::{Endpoint, NodeAddr, NodeId};

use crate::{ALPN, Cmd, Either, Error, Node};

const CHUNK_SIZE: usize = 1_000_000;

#[derive(Clone)]
pub struct Client {
    endpoint: Endpoint,
    server: Either<NodeAddr, NodeId>,
    bincode_config: bincode::config::Configuration,
}

impl Client {
    pub fn new(endpoint: Endpoint, server: NodeId) -> Self {
        Self {
            endpoint,
            server: Either::Right(server),
            bincode_config: bincode::config::standard(),
        }
    }

    pub fn with_addr(endpoint: Endpoint, server: NodeAddr) -> Self {
        Self {
            endpoint,
            server: Either::Left(server),
            bincode_config: bincode::config::standard(),
        }
    }

    pub async fn roles(&self) -> Result<Vec<String>, Error> {
        self.send(Cmd::Roles).await
    }

    pub async fn nodes(&self) -> Result<Vec<Node>, Error> {
        self.send(Cmd::Nodes).await
    }

    pub async fn node_roles(&self, node: NodeId) -> Result<Vec<String>, Error> {
        self.send(Cmd::NodeRoles {
            node: format!("{node}"),
        })
        .await
    }

    pub async fn create_node(
        &self,
        name: String,
        node: NodeId,
        superadmin: bool,
    ) -> Result<Node, Error> {
        self.send(Cmd::CreateNode {
            name,
            node: format!("{node}"),
            superadmin,
        })
        .await
    }

    pub async fn delete_node(&self, node: NodeId) -> Result<(), Error> {
        self.send(Cmd::DeleteNode {
            node: format!("{node}"),
        })
        .await
    }

    pub async fn grant_role(&self, node: NodeId, role: String) -> Result<(), Error> {
        self.send(Cmd::GrantRole {
            node: format!("{node}"),
            role,
        })
        .await
    }

    pub async fn revoke_role(&self, node: NodeId, role: String) -> Result<(), Error> {
        self.send(Cmd::RevokeRole {
            node: format!("{node}"),
            role,
        })
        .await
    }

    async fn send<R: Decode<()>>(&self, cmd: Cmd) -> Result<R, Error> {
        let json = bincode::encode_to_vec(&cmd, self.bincode_config)?;
        let conn = match &self.server {
            Either::Left(node_id) => self.endpoint.connect(node_id.clone(), ALPN).await?,
            Either::Right(node_addr) => self.endpoint.connect(node_addr.clone(), ALPN).await?,
        };

        let (mut tx, mut rx) = conn.open_bi().await?;
        tx.write_all(&json).await?;
        tx.finish()?;

        let mut data = vec![];
        while let Some(chunk) = rx.read_chunk(CHUNK_SIZE, true).await? {
            let mut bytes = chunk.bytes.to_vec();
            data.append(&mut bytes);
        }

        conn.close(0u32.into(), b"bye");

        let rsp = bincode::decode_from_slice(&data, self.bincode_config)?.0;
        Ok(rsp)
    }
}
