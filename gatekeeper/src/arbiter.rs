use std::path::PathBuf;

use iroh::NodeId;
use sqlx::{SqlitePool, sqlite::SqliteConnectOptions};

use crate::{Error, Node, db};

#[derive(Clone, Debug)]
pub struct Arbiter {
    db: SqlitePool,
    remote_setup: bool,
}

impl Arbiter {
    pub async fn new(db_path: PathBuf, remote_setup: bool) -> Result<Self, sqlx::Error> {
        let db = init_db(db_path).await?;
        Ok(Self { db, remote_setup })
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

    pub async fn allow(&self, caller: NodeId) -> Result<bool, Error> {
        if self.remote_setup && !db::Node::any(&self.db).await? {
            return Ok(true);
        }

        let res = db::Node::find(&self.db, &format!("{caller}"))
            .await?
            .map(|n| n.superadmin)
            .unwrap_or(false);

        Ok(res)
    }

    async fn get_node(&self, node: &str) -> Result<db::Node, Error> {
        match db::Node::find(&self.db, node).await? {
            None => Err(Error::NoSuchNodeError),
            Some(node) => Ok(node),
        }
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
