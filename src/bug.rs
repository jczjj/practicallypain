use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use actix_web::{post, web, HttpResponse};
use crate::models::bug::Bug;


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

#[post("/bugs/new")]
async fn create_bug(
    pool: web::Data<sqlx::SqlitePool>,
    params: web::Json<CreateBugRequest>,
) -> actix_web::Result<HttpResponse> {
    let bug = Bug::create(
        &pool,
        &params.title,
        &params.description,
        params.reported_by,
        &params.severity,
    ).await.map_err(AppError::from)?;
    Ok(HttpResponse::Ok().json(bug))
}


impl Bug {
    pub async fn create(
        pool: &sqlx::SqlitePool,
        title: &str,
        description: &str,
        reported_by: i64,
        severity: &str,
    ) -> sqlx::Result<Self> {
        let rec = sqlx::query_as::<_, Bug>(
            "INSERT INTO bugs (title, description, reported_by, severity)
             VALUES (?, ?, ?, ?)
             RETURNING *"
        )
        .bind(title)
        .bind(description)
        .bind(reported_by)
        .bind(severity)
        .fetch_one(pool)
        .await?;
        Ok(rec)
    }
}
