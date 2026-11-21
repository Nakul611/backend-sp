use chrono::Utc;
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};

use crate::error::AppError;

pub const JWT_SECRET: &[u8] = b"stackplane-super-secure-jwt-secret-key-2026-production";
pub const ACCESS_TOKEN_EXPIRY_SECONDS: i64 = 15 * 60; // 15 minutes (Session Token)
pub const REFRESH_TOKEN_EXPIRY_SECONDS: i64 = 7 * 24 * 3600; // 7 days (Refresh Token)

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: String,       // user id
    pub username: String,
    pub email: String,
    pub role: String,      // "admin" | "member"
    pub token_type: String, // "access" | "refresh"
    pub exp: i64,
    pub iat: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AuthResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub token_type: String,
    pub expires_in: i64,
    pub user: AuthUser,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RefreshResponse {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AuthUser {
    pub id: String,
    pub username: String,
    pub email: String,
    pub role: String,
    pub assigned_projects: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct RefreshRequest {
    pub refresh_token: String,
}

pub fn generate_tokens(user: &AuthUser) -> Result<(String, String), AppError> {
    let now = Utc::now().timestamp();

    // Access token (session token) - 15 minutes
    let access_claims = Claims {
        sub: user.id.clone(),
        username: user.username.clone(),
        email: user.email.clone(),
        role: user.role.clone(),
        token_type: "access".to_string(),
        exp: now + ACCESS_TOKEN_EXPIRY_SECONDS,
        iat: now,
    };

    let enc_key = EncodingKey::from_secret(JWT_SECRET)
        .map_err(|e| AppError::Internal(format!("Invalid secret key: {}", e)))?;

    let access_token = encode(
        &Header::default(),
        &access_claims,
        &enc_key,
    )
    .map_err(|e| AppError::Internal(format!("Failed to sign access token: {}", e)))?;

    // Refresh token - 7 days
    let refresh_claims = Claims {
        sub: user.id.clone(),
        username: user.username.clone(),
        email: user.email.clone(),
        role: user.role.clone(),
        token_type: "refresh".to_string(),
        exp: now + REFRESH_TOKEN_EXPIRY_SECONDS,
        iat: now,
    };

    let refresh_token = encode(
        &Header::default(),
        &refresh_claims,
        &enc_key,
    )
    .map_err(|e| AppError::Internal(format!("Failed to sign refresh token: {}", e)))?;

    Ok((access_token, refresh_token))
}

pub fn verify_token(token: &str, expected_type: &str) -> Result<Claims, AppError> {
    let dec_key = DecodingKey::from_secret(JWT_SECRET)
        .map_err(|e| AppError::Internal(format!("Invalid secret key: {}", e)))?;

    let token_data = decode::<Claims>(
        token,
        &dec_key,
        &Validation::default(),
    )
    .map_err(|e| AppError::Unauthorized(format!("Invalid or expired token: {}", e)))?;

    if token_data.claims.token_type != expected_type {
        return Err(AppError::Unauthorized(format!(
            "Invalid token type: expected '{}', got '{}'",
            expected_type, token_data.claims.token_type
        )));
    }

    Ok(token_data.claims)
}
