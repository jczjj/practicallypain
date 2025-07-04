use serde::{Deserialize, Serialize};
use sqlx::{FromRow, SqlitePool};
use actix_web::{get, web, HttpResponse, Result as ActixResult};


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

    pub async fn create_user(
        pool: &sqlx::SqlitePool,
        username: &str,
        password_hash: &str,
        is_admin: bool,
    ) -> sqlx::Result<Option<Self>> {
        let rec = sqlx::query_as::<_, User>(
            "INSERT INTO users (username, password_hash, is_admin)
             VALUES (?, ?, ?)"
        )
        .bind(username)
        .bind(password_hash)
        .bind(is_admin)
        .fetch_optional(pool)
        .await?;

        Ok(rec)
    }
        /// List all users in the database
    pub async fn list(
        pool: &SqlitePool,
    ) -> sqlx::Result<Vec<Self>> {
        sqlx::query_as::<_, User>(
            r#"
            SELECT user_id, username, password_hash, is_admin
              FROM users
            "#,
        )
        .fetch_all(pool)
        .await
    }

}


#[derive(Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

// To register user, only done by admin
#[derive(Deserialize)]
pub struct RegisterRequest {
    pub username: String,
    pub password: String,
    pub is_admin: Option<bool>, 
}


#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,     // Subject (username)
    pub is_admin: bool,  // admin flag
    pub exp: usize,      // expiration timestamp (unix)
}

#[derive(Serialize)]
struct UserInfo {
    user_id: i64,
    username: String,
}

/// GET /users → list all users (admin‐only if you like)
#[get("/users")]
pub async fn list_users(pool: web::Data<SqlitePool>) -> ActixResult<HttpResponse> {
    let users = User::list(&pool)
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?;

    let infos: Vec<UserInfo> = users.into_iter()
        .map(|u| UserInfo { user_id: u.user_id, username: u.username })
        .collect();

    Ok(HttpResponse::Ok().json(infos))
}
