use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, SqlitePool};
use actix_web::{
    post, get, patch, delete, web,
    HttpResponse, Result as ActixResult,
    error::{ErrorInternalServerError, ErrorNotFound},
};
use sqlx::{QueryBuilder, Sqlite};

/// JSON for updating an existing bug (PATCH semantics)
#[derive(Debug, Deserialize)]
pub struct UpdateBugRequest {
    pub title:        Option<String>,
    pub description:  Option<String>,
    pub severity:     Option<String>,
    pub status:       Option<String>,
    pub developer_id: Option<i64>,
}

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

/// Update fields on an existing bug; returns the updated row
    pub async fn update(
        pool: &SqlitePool,
        id: i64,
        changes: UpdateBugRequest,
    ) -> sqlx::Result<Option<Self>> {
        // Collect the non-None fields
        let mut builder = QueryBuilder::<Sqlite>::new("UPDATE bugs SET ");
        let mut first = true;

        if let Some(title) = changes.title {
            if !first { builder.push(", "); }
            builder.push("title = ").push_bind(title);
            first = false;
        }
        if let Some(description) = changes.description {
            if !first { builder.push(", "); }
            builder.push("description = ").push_bind(description);
            first = false;
        }
        if let Some(severity) = changes.severity {
            if !first { builder.push(", "); }
            builder.push("severity = ").push_bind(severity);
            first = false;
        }
        if let Some(status) = changes.status {
            if !first { builder.push(", "); }
            builder.push("status = ").push_bind(status);
            first = false;
        }
        if let Some(dev) = changes.developer_id {
            if !first { builder.push(", "); }
            builder.push("developer_id = ").push_bind(dev);
        }

        // If nothing to update, just return the existing record
        if first {
            return Bug::find_by_id(pool, id).await;
        }

    // Add the WHERE clause and execute
    builder.push(" WHERE bug_id = ").push_bind(id);
    builder.build().execute(pool).await?;

    // Fetch & return the updated bug
    Bug::find_by_id(pool, id).await
}

    /// Delete a bug by its ID; returns true if a row was deleted
    pub async fn delete(pool: &SqlitePool, id: i64) -> sqlx::Result<bool> {
        let res = sqlx::query(
            r#"
            DELETE FROM bugs WHERE bug_id = ?
            "#
        )
        .bind(id)
        .execute(pool)
        .await?;
        Ok(res.rows_affected() > 0)
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

/// PATCH /bugs/{id} → update an existing bug
#[patch("/bugs/{id}")]
pub async fn update_bug(
    pool: web::Data<SqlitePool>,
    path: web::Path<i64>,
    params: web::Json<UpdateBugRequest>,
) -> ActixResult<HttpResponse> {
    let id = path.into_inner();
    match Bug::update(&pool, id, params.into_inner())
        .await
        .map_err(ErrorInternalServerError)?
    {
        Some(bug) => Ok(HttpResponse::Ok().json(bug)),
        None => Err(ErrorNotFound("Bug not found")),
    }
}


/// DELETE /bugs/{id} → delete a bug
#[delete("/bugs/{id}")]
pub async fn delete_bug(
    pool: web::Data<SqlitePool>,
    path: web::Path<i64>,
) -> ActixResult<HttpResponse> {
    let id = path.into_inner();
    let deleted = Bug::delete(&pool, id)
        .await
        .map_err(ErrorInternalServerError)?;

    if deleted {
        Ok(HttpResponse::NoContent().finish())
    } else {
        Err(ErrorNotFound("Bug not found"))
    }
}

