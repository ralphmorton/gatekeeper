use chrono::NaiveDateTime;
use sqlx::{Executor, Sqlite, prelude::FromRow, query, query_as};

#[allow(dead_code)]
#[derive(Debug, FromRow)]
pub struct Node {
    pub id: i64,
    pub name: String,
    pub node: String,
    pub superadmin: bool,
    pub created: NaiveDateTime,
}

#[derive(FromRow)]
pub struct Count {
    count: i64,
}

#[derive(FromRow)]
pub struct Role {
    role: String,
}

impl Node {
    pub async fn all<'a, E: Executor<'a, Database = Sqlite>>(
        conn: E,
    ) -> Result<Vec<Node>, sqlx::Error> {
        query_as::<_, Node>("SELECT * FROM nodes ORDER BY node")
            .fetch_all(conn)
            .await
    }

    pub async fn any<'a, E: Executor<'a, Database = Sqlite>>(conn: E) -> Result<bool, sqlx::Error> {
        query_as::<_, Count>("SELECT COUNT(*) AS count FROM nodes")
            .fetch_one(conn)
            .await
            .map(|c| c.count > 0)
    }

    pub async fn find<'a, E: Executor<'a, Database = Sqlite>>(
        conn: E,
        node: &str,
    ) -> Result<Option<Node>, sqlx::Error> {
        query_as::<_, Node>("SELECT * FROM nodes WHERE node = $1")
            .bind(node)
            .fetch_optional(conn)
            .await
    }

    pub async fn roles<'a, E: Executor<'a, Database = Sqlite>>(
        conn: E,
        node: &str,
    ) -> Result<Vec<String>, sqlx::Error> {
        let roles = query_as::<_, Role>(
            r#"
                SELECT r.role AS role FROM nodes n
                JOIN node_roles nr ON nr.node_id = n.id
                JOIN roles r ON nr.role_id = r.id
                WHERE n.node = $1
                ORDER BY r.role
            "#,
        )
        .bind(node)
        .fetch_all(conn)
        .await?
        .into_iter()
        .map(|r| r.role)
        .collect();

        Ok(roles)
    }

    pub async fn insert<'a, E: Executor<'a, Database = Sqlite>>(
        conn: E,
        name: &str,
        node: &str,
        superadmin: bool,
    ) -> Result<Node, sqlx::Error> {
        query_as::<_, Node>(
            "INSERT INTO nodes (name, node, superadmin, created) VALUES ($1, $2, $3, datetime('now')) RETURNING *",
        )
        .bind(name)
        .bind(node)
        .bind(superadmin)
        .fetch_one(conn)
        .await
    }

    pub async fn delete<'a, E: Executor<'a, Database = Sqlite>>(
        conn: E,
        id: i64,
    ) -> Result<u64, sqlx::Error> {
        query("DELETE FROM nodes WHERE id = $1")
            .bind(id)
            .execute(conn)
            .await
            .map(|r| r.rows_affected())
    }
}
