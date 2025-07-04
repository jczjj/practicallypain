use actix_web::{post, web, HttpResponse, Responder};
use sha2::{Sha256, Digest};
use crate::models::user::{LoginRequest, Claims};
use jsonwebtoken::{EncodingKey, DecodingKey, Header, Validation, TokenData, encode, decode, errors::Error};

const SALT: &str = "bugtrack2025";

//For jwt
const SECRET_KEY: &[u8] = b"8f3a7d1e5b924f60a3c2e7d9b1f04c12";

// Hardcoded password hash: hash("bugtrack2025admin123")
const ADMIN_HASH: &str = "fc6e4582a0162f9642bf852d3fa902014c8fe15e6fde8cd3377561ce513603ae";


#[post("/login")]
pub async fn login(data: web::Json<LoginRequest>) -> impl Responder {
    let username = &data.username;
    let password = &data.password;

if username == "admin" && hash_password(password) == ADMIN_HASH {
        // admin login
        match create_jwt(username, true) {
            Ok(token) => HttpResponse::Ok().json(serde_json::json!({
                "status": "success",
                "token": token
            })),
            Err(_) => HttpResponse::InternalServerError().finish(),
        }
    } else if username == "user" && password == "user123" {
        // user login (no hashing example)
        match create_jwt(username, false) {
            Ok(token) => HttpResponse::Ok().json(serde_json::json!({
                "status": "success",
                "token": token
            })),
            Err(_) => HttpResponse::InternalServerError().finish(),
        }
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

pub fn create_jwt(username: &str, is_admin: bool) -> Result<String, Error> {
    use std::time::{SystemTime, UNIX_EPOCH};

    let expiration = SystemTime::now()
        .checked_add(std::time::Duration::from_secs(60 * 60)) // 1 hour expiration
        .unwrap()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as usize;

    let claims = Claims {
        sub: username.to_owned(),
        is_admin,
        exp: expiration,
    };

    encode(&Header::default(), &claims, &EncodingKey::from_secret(SECRET_KEY))
}

pub fn validate_jwt(token: &str) -> Result<TokenData<Claims>, Error> {
    decode::<Claims>(
        token,
        &DecodingKey::from_secret(SECRET_KEY),
        &Validation::default(),
    )
}