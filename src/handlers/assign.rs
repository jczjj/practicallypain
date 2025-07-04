use actix_web::{web, HttpResponse, Responder};
use tera::{Tera, Context};
use sqlx::SqlitePool;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct AssignForm{
    pub bug_id: i32,
    pub developer_id: i32,
}

pub async fn get_assign_form(tmpl: web::Data<Tera>) -> impl Responder{
    let ctx = Context::new();
    match tmpl.render("assign_bug.html", &ctx){
        Ok(rendered) => HttpResponse::Ok().content_type("text/html").body(rendered),
        Err(e) => {
            eprintln!("Tera render error: {}", e);
            HttpResponse::InternalServerError().body(format!("Render error: {}", e))}
    }
}

pub async fn post_assign_form(pool: web::Data<SqlitePool>, tmpl: web::Data<Tera>, form: web::Form<AssignForm>,)
-> impl Responder{
    let result = sqlx::query("UPDATE bugs SET developer_id = ? WHERE bug_id = ?")
        .bind(form.developer_id)
        .bind(form.bug_id)
        .execute(pool.get_ref())
        .await;

    match result{
        Ok(res) if res.rows_affected() > 0 =>{
            let mut ctx = Context::new();
            ctx.insert("bug_id", &form.bug_id);
            ctx.insert("developer_id", &form.developer_id);
            match tmpl.render("assign_confirm.html", &ctx){
                Ok(html) => HttpResponse::Ok().content_type("text/html").body(html),
                Err(_) => HttpResponse::InternalServerError().body("Error rendering confirmation."),
            }
        }
        _ => {
            let mut ctx = Context::new();
            ctx.insert("message", "Invalid Bug ID or Developer ID");
            match tmpl.render("assign_error.html", &ctx){
                Ok(html) => HttpResponse::BadRequest().content_type("text/html").body(html),
                Err(_) => HttpResponse::InternalServerError().body("Error rendering error page."),
            }
        }
    }
}