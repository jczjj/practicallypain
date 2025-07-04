use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, FromRow, Serialize, Deserialize)]
pub struct Project {
    pub project_id: i64,
    pub name:       String,
    pub description: Option<String>,
}

impl Project {
    pub async fn list(pool: &sqlx::SqlitePool) -> sqlx::Result<Vec<Self>> {
        sqlx::query_as::<_, Project>("SELECT project_id, name, description FROM projects")
            .fetch_all(pool)
            .await
    }
}
