use std::fmt::Debug;

use bincode::Encode;
use iroh::{
    NodeId,
    endpoint::Connection,
    protocol::{AcceptError, ProtocolHandler},
};

use crate::{Arbiter, Cmd, Error};

#[derive(Clone)]
pub struct Server {
    arbiter: Arbiter,
    bincode_config: bincode::config::Configuration,
}

impl Debug for Server {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Server {{ arbiter: {:?} }}", self.arbiter)?;
        Ok(())
    }
}

impl Server {
    pub fn new(arbiter: Arbiter) -> Self {
        Self {
            arbiter,
            bincode_config: bincode::config::standard(),
        }
    }

    async fn handle(&self, caller: NodeId, cmd: Cmd) -> Result<Vec<u8>, Error> {
        if !self.arbiter.allow(caller).await? {
            return Err(Error::UnauthorizedError);
        }

        match cmd {
            Cmd::Roles => self.exec(self.arbiter.roles()).await,
            Cmd::Nodes => self.exec(self.arbiter.nodes()).await,
            Cmd::NodeRoles { node } => self.exec(self.arbiter.node_roles(&node)).await,
            Cmd::CreateNode {
                name,
                node,
                superadmin,
            } => {
                self.exec(self.arbiter.create_node(&name, &node, superadmin))
                    .await
            }
            Cmd::DeleteNode { node } => self.exec(self.arbiter.delete_node(&node)).await,
            Cmd::GrantRole { node, role } => self.exec(self.arbiter.grant_role(&node, &role)).await,
            Cmd::RevokeRole { node, role } => {
                self.exec(self.arbiter.revoke_role(&node, &role)).await
            }
        }
    }

    async fn exec<R: Encode, F: Future<Output = Result<R, Error>>>(
        &self,
        f: F,
    ) -> Result<Vec<u8>, Error> {
        let rsp = f.await?;
        let res = bincode::encode_to_vec(&rsp, self.bincode_config)?;
        Ok(res)
    }
}

impl ProtocolHandler for Server {
    async fn accept(&self, connection: Connection) -> Result<(), AcceptError> {
        let node_id = connection.remote_node_id()?;
        tracing::info!(node_id = ?node_id, "accept");

        let (mut tx, mut rx) = connection.accept_bi().await?;

        let mut data = vec![];
        while let Some(chunk) = rx
            .read_chunk(100_000, true)
            .await
            .map_err(AcceptError::from_err)?
        {
            let mut bytes = chunk.bytes.to_vec();
            data.append(&mut bytes);
        }

        let cmd: Cmd = bincode::decode_from_slice(&data, self.bincode_config)
            .map_err(AcceptError::from_err)?
            .0;

        let rsp = self.handle(node_id, cmd.clone()).await;
        if rsp.is_err() {
            tracing::warn!(cmd = ?cmd, rsp = ?rsp, "handle_failed");
        }

        let rsp = rsp.map_err(AcceptError::from_err)?;

        tx.write_all(&rsp).await.map_err(AcceptError::from_err)?;
        tx.finish()?;
        connection.closed().await;

        Ok(())
    }
}
