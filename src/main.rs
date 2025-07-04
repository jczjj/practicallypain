use actix_web::{App, HttpServer, web};
use sqlx::SqlitePool;
use std::fs;
use anyhow::Result;
use tera::Tera;

mod handlers {
    pub mod auth;
    pub mod assign;
}

mod models {
    pub mod user;
    pub mod project;
    pub mod bug;
}

mod middleware {
    pub mod auth_middleware;
    pub mod admin_guard;
}

use handlers::auth::{login, register, login_page};
use handlers::assign::{get_assign_form, post_assign_form};
use middleware::auth_middleware::AuthMiddleware;
use middleware::admin_guard::AdminGuard;
use models::bug::{create_bug, list_bugs, get_bug};

#[actix_web::main]
async fn main() -> Result<()> {

    // 1) Read & apply your schema:
    let sql = fs::read_to_string("schema.sql")?;
    let pool = SqlitePool::connect("sqlite://bugtrack.db").await?;
    sqlx::query(&sql).execute(&pool).await?;
    let tera = Tera::new("templates/**/*").unwrap();

    println!("Starting server on http://127.0.0.1:8080");


    HttpServer::new(move || {
        App::new()
            // shared DB pool
            .app_data(web::Data::new(pool.clone()))
            .app_data(web::Data::new(tera.clone()))
            .service(create_bug)
            .service(list_bugs)
            .service(get_bug)
            .service(login)
            .service(register)


            .service(login) //if using curlx
            .route("/login", web::get().to(login_page)) //if using endpoint
            .service(
                web::scope("/admin")
                .wrap(AdminGuard)
                .service(register)
                // Add your admin routes here, e.g.
            )
            .wrap(AuthMiddleware) // AuthMiddlewarehere, need add routes to it also
            .route("/bugs/assign", web::get().to(get_assign_form))
            .route("/bugs/assign", web::post().to(post_assign_form))
            
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await?;

    Ok(())
}
