use actix_web::{App, HttpServer, web};
use sqlx::SqlitePool;
use std::fs;
use anyhow::Result;
use actix_web::dev::{Service, ServiceRequest, ServiceResponse};

mod handlers {
    pub mod auth;
}

mod models {
    pub mod user;
}

mod middleware {
    pub mod auth_middleware;
    pub mod admin_guard;
}

use handlers::auth::login;
use middleware::auth_middleware::AuthMiddleware;
use middleware::admin_guard::AdminGuard;



#[actix_web::main]
async fn main() -> Result<()> {
    // 1) Read & apply your schema:
    let sql = fs::read_to_string("schema.sql")?;
    let pool = SqlitePool::connect("sqlite://bugtrack.db").await?;
    sqlx::query(&sql).execute(&pool).await?;

    println!("▶️  Starting server on http://127.0.0.1:8080");

    HttpServer::new(move || {
        App::new()
            // === Temporary logger ===
            .wrap_fn(|req: ServiceRequest, srv| {
                println!("➡️  {} {}", req.method(), req.uri());
                let fut = srv.call(req);
                async move {
                    let res: ServiceResponse = fut.await?;
                    println!("⬅️  {} {}", res.status(), res.request().uri());
                    Ok(res)
                }
            })

            // shared DB pool
            .app_data(web::Data::new(pool.clone()))
            // ← Here: mount your routes/handlers!
            //   .service(create_bug) 

            .service(login)

            .service(
                web::scope("/admin")
                .wrap(AdminGuard)
                // Add your admin routes here, e.g.
            )
            .wrap(AuthMiddleware) // AUthMiddlewarehere, need add routes to it also
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await?;

    Ok(())
}
