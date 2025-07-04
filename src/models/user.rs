use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, FromRow, Serialize, Deserialize)]
pub struct User {
    pub user_id:    i64,
    pub username:   String,
    pub password_hash: String,
    pub is_admin:   bool,
}

impl User {
    pub async fn find_by_username(
        pool: &sqlx::SqlitePool,
        username: &str,
    ) -> sqlx::Result<Option<Self>> {
        let rec = sqlx::query_as::<_, User>(
            "SELECT user_id, username, password_hash, is_admin 
             FROM users WHERE username = ?"
        )
        .bind(username)
        .fetch_optional(pool)
        .await?;
        Ok(rec)
    }
}

#[derive(Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}


#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,     // Subject (username)
    pub is_admin: bool,  // admin flag
    pub exp: usize,      // expiration timestamp (unix)
}