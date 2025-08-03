use chrono::NaiveDateTime;
use sqlx::{Executor, Sqlite, SqlitePool, prelude::FromRow, query_as};

#[allow(dead_code)]
#[derive(Debug, FromRow)]
pub struct Role {
    pub id: i64,
    pub role: String,
    pub created: NaiveDateTime,
}

impl Role {
    pub async fn all<'a, E: Executor<'a, Database = Sqlite>>(
        conn: E,
    ) -> Result<Vec<Role>, sqlx::Error> {
        query_as::<_, Role>("SELECT * FROM roles ORDER BY role")
            .fetch_all(conn)
            .await
    }

    pub async fn ensure(pool: &SqlitePool, role: &str) -> Result<Role, sqlx::Error> {
        match Self::find(pool, role).await? {
            Some(existing) => Ok(existing),
            None => Self::insert(pool, role).await,
        }
    }

    pub async fn find<'a, E: Executor<'a, Database = Sqlite>>(
        conn: E,
        role: &str,
    ) -> Result<Option<Role>, sqlx::Error> {
        query_as::<_, Role>("SELECT * FROM roles WHERE role = $1")
            .bind(role)
            .fetch_optional(conn)
            .await
    }

    pub async fn insert<'a, E: Executor<'a, Database = Sqlite>>(
        conn: E,
        role: &str,
    ) -> Result<Role, sqlx::Error> {
        query_as::<_, Role>(
            "INSERT INTO roles (role, created) VALUES ($1, datetime('now')) RETURNING *",
        )
        .bind(role)
        .fetch_one(conn)
        .await
    }
}
