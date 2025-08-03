use sqlx::{Executor, Sqlite, SqlitePool, prelude::FromRow, query, query_as};

#[allow(dead_code)]
#[derive(Debug, FromRow)]
pub struct NodeRole {
    pub id: i64,
    pub node_id: i64,
    pub role_id: i64,
}

impl NodeRole {
    pub async fn ensure(
        pool: &SqlitePool,
        node_id: i64,
        role_id: i64,
    ) -> Result<NodeRole, sqlx::Error> {
        match Self::find(pool, node_id, role_id).await? {
            Some(existing) => Ok(existing),
            None => Self::insert(pool, node_id, role_id).await,
        }
    }

    pub async fn find<'a, E: Executor<'a, Database = Sqlite>>(
        conn: E,
        node_id: i64,
        role_id: i64,
    ) -> Result<Option<NodeRole>, sqlx::Error> {
        query_as::<_, NodeRole>("SELECT * FROM node_roles WHERE node_id = $1 AND role_id = $2")
            .bind(node_id)
            .bind(role_id)
            .fetch_optional(conn)
            .await
    }

    pub async fn insert<'a, E: Executor<'a, Database = Sqlite>>(
        conn: E,
        node_id: i64,
        role_id: i64,
    ) -> Result<NodeRole, sqlx::Error> {
        query_as::<_, NodeRole>(
            "INSERT INTO node_roles (node_id, role_id) VALUES ($1, $2) RETURNING *",
        )
        .bind(node_id)
        .bind(role_id)
        .fetch_one(conn)
        .await
    }

    pub async fn delete<'a, E: Executor<'a, Database = Sqlite>>(
        conn: E,
        id: i64,
    ) -> Result<u64, sqlx::Error> {
        query("DELETE FROM node_roles WHERE id = $1")
            .bind(id)
            .execute(conn)
            .await
            .map(|r| r.rows_affected())
    }
}
