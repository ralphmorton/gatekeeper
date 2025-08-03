use std::{fmt::Debug, path::PathBuf};

use bincode::Encode;
use iroh::{
    NodeId,
    endpoint::Connection,
    protocol::{AcceptError, ProtocolHandler},
};
use sqlx::{SqlitePool, sqlite::SqliteConnectOptions};

use crate::{Cmd, Error, Node, db};

#[derive(Clone)]
pub struct Server {
    db: SqlitePool,
    remote_setup: bool,
    bincode_config: bincode::config::Configuration,
}

impl Debug for Server {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Server {{ db: {:?} }}", self.db)?;
        Ok(())
    }
}

impl Server {
    pub async fn new(db_path: PathBuf, remote_setup: bool) -> Result<Self, sqlx::Error> {
        let db = init_db(db_path).await?;

        let i = Self {
            db,
            remote_setup,
            bincode_config: bincode::config::standard(),
        };

        Ok(i)
    }

    async fn handle(&self, caller: NodeId, cmd: Cmd) -> Result<Vec<u8>, Error> {
        if !self.allow(caller).await? {
            return Err(Error::UnauthorizedError);
        }

        match cmd {
            Cmd::Roles => self.exec(self.roles()).await,
            Cmd::Nodes => self.exec(self.nodes()).await,
            Cmd::NodeRoles { node } => self.exec(self.node_roles(&node)).await,
            Cmd::CreateNode {
                name,
                node,
                superadmin,
            } => self.exec(self.create_node(&name, &node, superadmin)).await,
            Cmd::DeleteNode { node } => self.exec(self.delete_node(&node)).await,
            Cmd::GrantRole { node, role } => self.exec(self.grant_role(&node, &role)).await,
            Cmd::RevokeRole { node, role } => self.exec(self.revoke_role(&node, &role)).await,
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

    pub async fn roles(&self) -> Result<Vec<String>, Error> {
        let res = db::Role::all(&self.db)
            .await?
            .into_iter()
            .map(|r| r.role)
            .collect();

        Ok(res)
    }

    pub async fn nodes(&self) -> Result<Vec<Node>, Error> {
        let res = db::Node::all(&self.db)
            .await?
            .into_iter()
            .map(Node::from)
            .collect();

        Ok(res)
    }

    pub async fn node_roles(&self, node: &str) -> Result<Vec<String>, Error> {
        let res = db::Node::roles(&self.db, node).await?;
        Ok(res)
    }

    pub async fn create_node(
        &self,
        name: &str,
        node: &str,
        superadmin: bool,
    ) -> Result<Node, Error> {
        let res = db::Node::insert(&self.db, name, node, superadmin)
            .await?
            .into();

        Ok(res)
    }

    pub async fn delete_node(&self, node: &str) -> Result<(), Error> {
        let node = self.get_node(node).await?;
        db::Node::delete(&self.db, node.id).await?;

        Ok(())
    }

    pub async fn grant_role(&self, node: &str, role: &str) -> Result<(), Error> {
        let node = self.get_node(node).await?;
        let role = db::Role::ensure(&self.db, role).await?;

        db::NodeRole::ensure(&self.db, node.id, role.id).await?;
        Ok(())
    }

    pub async fn revoke_role(&self, node: &str, role: &str) -> Result<(), Error> {
        let node = self.get_node(node).await?;
        let role = db::Role::ensure(&self.db, role).await?;

        if let Some(node_role) = db::NodeRole::find(&self.db, node.id, role.id).await? {
            db::NodeRole::delete(&self.db, node_role.id).await?;
        }

        Ok(())
    }

    async fn get_node(&self, node: &str) -> Result<db::Node, Error> {
        match db::Node::find(&self.db, node).await? {
            None => Err(Error::NoSuchNodeError),
            Some(node) => Ok(node),
        }
    }

    async fn allow(&self, caller: NodeId) -> Result<bool, Error> {
        if self.remote_setup && !db::Node::any(&self.db).await? {
            return Ok(true);
        }

        let res = db::Node::find(&self.db, &format!("{caller}"))
            .await?
            .map(|n| n.superadmin)
            .unwrap_or(false);

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

async fn init_db(path: PathBuf) -> Result<SqlitePool, sqlx::Error> {
    let opts = SqliteConnectOptions::new()
        .filename(path)
        .create_if_missing(true);

    let pool = SqlitePool::connect_with(opts).await?;
    sqlx::migrate!("./migrations").run(&pool).await?;
    Ok(pool)
}
