use actix_web::{post, web, HttpResponse, Responder};
use sha2::{Sha256, Digest};
use crate::models::user::{LoginRequest, Claims, User,RegisterRequest};
use jsonwebtoken::{EncodingKey, DecodingKey, Header, Validation, TokenData, encode, decode, errors::Error};
use sqlx::SqlitePool;

const SALT: &str = "bugtrack2025";

//For jwt
const SECRET_KEY: &[u8] = b"8f3a7d1e5b924f60a3c2e7d9b1f04c12";

#[post("/register")]
pub async fn register(
    data: web::Json<RegisterRequest>,
    pool: web::Data<SqlitePool>,
) -> impl Responder {
    let username = data.username.trim();
    let password_hash = hash_password(&data.password);
    let is_admin = data.is_admin.unwrap_or(false);

    // Check if user already exists
    match User::find_by_username(&pool, username).await {
        Ok(Some(_)) => {
            return HttpResponse::Conflict().json(serde_json::json!({
                "status": "error",
                "message": "Username already taken"
            }));
        }
        Ok(None) => {
            // Create new user
            match User::create_user(&pool, username, &password_hash, is_admin).await {
                Ok(_) => HttpResponse::Ok().json(serde_json::json!({
                    "status": "success",
                    "message": "User registered successfully"
                })),
                Err(e) => {
                    eprintln!("Error inserting user: {:?}", e);
                    HttpResponse::InternalServerError().finish()
                }
            }
        }
        Err(e) => {
            eprintln!("DB error: {:?}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}


#[post("/login")]
pub async fn login(data: web::Json<LoginRequest>,pool: web::Data<SqlitePool>) -> impl Responder {
    let username = data.username.clone();
    let password = hash_password(&data.password.clone());

    match User::find_by_username(&pool, &username).await {
        Ok(Some(user)) =>{
            if password == user.password_hash {
                print!("{}", user.is_admin);
                match create_jwt(&username, user.is_admin) {
                    Ok(token) => HttpResponse::Ok().json(serde_json::json!({
                    "status": "success",
                    "token": token
                })),
                Err(_) => HttpResponse::InternalServerError().finish(),
                }
            } else {
                HttpResponse::Unauthorized().body("Invalid credentials")
            }
        }
        Ok(None) => HttpResponse::NotFound().body("Invalid credentials"),
        Err(err) => {
            eprintln!("Database error: {:?}", err);
            HttpResponse::InternalServerError().body("Internal Error")
        }
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