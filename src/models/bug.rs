use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, SqlitePool};
use actix_web::{
    post, get, web,
    HttpResponse, Result as ActixResult,
    error::{ErrorInternalServerError, ErrorNotFound},
};

/// JSON body for creating a new bug
#[derive(Debug, Deserialize)]
pub struct CreateBugRequest {
    pub title: String,
    pub description: String,
    pub reported_by: i64,
    pub severity: String,
}

/// The Bug model, matching your `bugs` table
#[derive(Debug, FromRow, Serialize, Deserialize)]
pub struct Bug {
    pub bug_id:       i64,
    pub title:        String,
    pub description:  String,
    pub reported_by:  i64,
    pub severity:     String,
    pub developer_id: Option<i64>,
    pub status:       String,
    pub created_at:   DateTime<Utc>,
}

impl Bug {
    /// Insert a new bug (other columns default in SQL schema) and fetch it back.
    pub async fn create(
        pool: &SqlitePool,
        title: &str,
        description: &str,
        reported_by: i64,
        severity: &str,
    ) -> sqlx::Result<Self> {
        // 1. INSERT, ignore developer_id/status/created_at (use your SQL defaults)
        let res = sqlx::query(
            r#"
            INSERT INTO bugs (title, description, reported_by, severity)
             VALUES (?, ?, ?, ?)
            "#
        )
        .bind(title)
        .bind(description)
        .bind(reported_by)
        .bind(severity)
        .execute(pool)
        .await?;

        // 2. Grab the last inserted ID
        let last_id = res.last_insert_rowid();

        // 3. Query the full row
        sqlx::query_as::<_, Bug>(
            r#"
            SELECT bug_id, title, description, reported_by,
                   severity, developer_id, status, created_at
              FROM bugs
             WHERE bug_id = ?
            "#
        )
        .bind(last_id)
        .fetch_one(pool)
        .await
    }

    /// Return _all_ bugs in the table
    pub async fn list(pool: &SqlitePool) -> sqlx::Result<Vec<Self>> {
        sqlx::query_as::<_, Bug>(
            r#"
            SELECT bug_id, title, description, reported_by,
                   severity, developer_id, status, created_at
              FROM bugs
            "#
        )
        .fetch_all(pool)
        .await
    }

    /// Return one bug by its ID, or None if missing
    pub async fn find_by_id(pool: &SqlitePool, id: i64) -> sqlx::Result<Option<Self>> {
        sqlx::query_as::<_, Bug>(
            r#"
            SELECT bug_id, title, description, reported_by,
                   severity, developer_id, status, created_at
              FROM bugs
             WHERE bug_id = ?
            "#
        )
        .bind(id)
        .fetch_optional(pool)
        .await
    }
}

/// === Actix-Web handlers ===

/// POST /bugs/new → create & return the new bug
#[post("/bugs/new")]
pub async fn create_bug(
    pool: web::Data<SqlitePool>,
    params: web::Json<CreateBugRequest>,
) -> ActixResult<HttpResponse> {
    let bug = Bug::create(
        &pool,
        &params.title,
        &params.description,
        params.reported_by,
        &params.severity,
    )
    .await
    .map_err(ErrorInternalServerError)?;
    Ok(HttpResponse::Created().json(bug))
}

/// GET /bugs → list all bugs
#[get("/bugs")]
pub async fn list_bugs(
    pool: web::Data<SqlitePool>,
) -> ActixResult<HttpResponse> {
    let bugs = Bug::list(&pool)
        .await
        .map_err(ErrorInternalServerError)?;
    Ok(HttpResponse::Ok().json(bugs))
}

/// GET /bugs/{id} → fetch one bug by ID
#[get("/bugs/{id}")]
pub async fn get_bug(
    pool: web::Data<SqlitePool>,
    path: web::Path<i64>,
) -> ActixResult<HttpResponse> {
    let id = path.into_inner();
    match Bug::find_by_id(&pool, id)
        .await
        .map_err(ErrorInternalServerError)?
    {
        Some(bug) => Ok(HttpResponse::Ok().json(bug)),
        None => Err(ErrorNotFound("Bug not found")),
    }
}
