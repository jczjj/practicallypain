use actix_web::{post, web, HttpResponse, Responder};
use sha2::{Sha256, Digest};
use crate::models::user::LoginRequest;

const SALT: &str = "bugtrack2025";

// Hardcoded password hash: hash("bugtrack2025admin123")
const ADMIN_HASH: &str = "fc6e4582a0162f9642bf852d3fa902014c8fe15e6fde8cd3377561ce513603ae";


#[post("/login")]
pub async fn login(data: web::Json<LoginRequest>) -> impl Responder {
    let username = &data.username;
    let password = &data.password;

    let computed_hash = hash_password(password);
    print!("{}", computed_hash);

    if username == "admin" && computed_hash == ADMIN_HASH {
        HttpResponse::Ok().json(serde_json::json!({
            "status": "success",
            "token": "fake-session-token-123"
        }))
    } else {
        HttpResponse::Unauthorized().json(serde_json::json!({
            "status": "failure"
        }))
    }
}

fn hash_password(password: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(format!("{}{}", SALT, password));
    let result = hasher.finalize();
    format!("{:x}", result)
}