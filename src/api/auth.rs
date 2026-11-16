use crate::{
    db::{AppState, with_db},
    error::{ApiError, ApiResult},
    time::now_ms,
};
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier, password_hash::SaltString};
use axum::{
    Json,
    extract::State,
    http::{HeaderMap, StatusCode, header},
};
use rand_core::OsRng;
use rusqlite::{OptionalExtension, params};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use uuid::Uuid;

const SESSION_LIFETIME_MS: i64 = 30 * 24 * 60 * 60 * 1000;

fn token_hash(token: &str) -> String {
    format!("{:x}", Sha256::digest(token.as_bytes()))
}

fn bearer(headers: &HeaderMap) -> ApiResult<String> {
    let value = headers
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .filter(|v| !v.is_empty() && v.len() <= 128)
        .ok_or_else(ApiError::unauthorized)?;
    Ok(token_hash(value))
}

pub(crate) async fn authenticated_user(state: &AppState, headers: &HeaderMap) -> ApiResult<String> {
    let hash = bearer(headers)?;
    with_db(state, move |db| {
        db.query_row(
            "SELECT user_id FROM sessions WHERE token_hash = ?1 AND expires_at > ?2",
            params![hash, now_ms()],
            |row| row.get(0),
        )
        .optional()
        .map_err(|_| ApiError::internal())?
        .ok_or_else(ApiError::unauthorized)
    })
    .await
}

#[derive(Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct Credentials {
    /// Email của tài xế.
    email: String,
    /// Mật khẩu 12–128 ký tự khi đăng ký.
    password: String,
}

#[derive(Serialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub(crate) struct AuthResponse {
    user_id: String,
    email: String,
    token: String,
    expires_at: i64,
}

fn normalized_email(value: &str) -> ApiResult<String> {
    let email = value.trim().to_lowercase();
    if email.len() > 254 || email.len() < 3 || !email.contains('@') || email.contains(' ') {
        return Err(ApiError::bad("Invalid email"));
    }
    Ok(email)
}

#[utoipa::path(post, path = "/api/v1/auth/register", tag = "Tài khoản",
    request_body = Credentials,
    responses((status = 201, body = AuthResponse), (status = 400), (status = 409)),
    security(()))]
pub(crate) async fn register(
    State(state): State<AppState>,
    Json(input): Json<Credentials>,
) -> ApiResult<(StatusCode, Json<AuthResponse>)> {
    let email = normalized_email(&input.email)?;
    if input.password.len() < 12 || input.password.len() > 128 {
        return Err(ApiError::bad("Password must be 12 to 128 characters"));
    }
    let password_hash = tokio::task::spawn_blocking(move || {
        Argon2::default()
            .hash_password(input.password.as_bytes(), &SaltString::generate(&mut OsRng))
            .map(|hash| hash.to_string())
    })
    .await
    .map_err(|_| ApiError::internal())?
    .map_err(|_| ApiError::internal())?;
    let user_id = Uuid::new_v4().to_string();
    let token = format!("{}{}", Uuid::new_v4().simple(), Uuid::new_v4().simple());
    let hash = token_hash(&token);
    let expires_at = now_ms() + SESSION_LIFETIME_MS;
    let saved_user_id = user_id.clone();
    let saved_email = email.clone();
    with_db(&state, move |db| {
        let tx = db.transaction().map_err(|_| ApiError::internal())?;
        if tx
            .execute(
                "INSERT INTO users (id, email, password_hash, created_at) VALUES (?1, ?2, ?3, ?4)",
                params![saved_user_id, saved_email, password_hash, now_ms()],
            )
            .is_err()
        {
            return Err(ApiError::conflict("Account already exists"));
        }
        tx.execute(
            "INSERT INTO sessions (token_hash, user_id, expires_at) VALUES (?1, ?2, ?3)",
            params![hash, saved_user_id, expires_at],
        )
        .map_err(|_| ApiError::internal())?;
        tx.commit().map_err(|_| ApiError::internal())
    })
    .await?;
    Ok((
        StatusCode::CREATED,
        Json(AuthResponse {
            user_id,
            email,
            token,
            expires_at,
        }),
    ))
}

#[utoipa::path(post, path = "/api/v1/auth/login", tag = "Tài khoản",
    request_body = Credentials,
    responses((status = 200, body = AuthResponse), (status = 401)),
    security(()))]
pub(crate) async fn login(
    State(state): State<AppState>,
    Json(input): Json<Credentials>,
) -> ApiResult<Json<AuthResponse>> {
    let email = normalized_email(&input.email)?;
    let saved_email = email.clone();
    let row: Option<(String, String)> = with_db(&state, move |db| {
        db.query_row(
            "SELECT id, password_hash FROM users WHERE email = ?1",
            [saved_email],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .optional()
        .map_err(|_| ApiError::internal())
    })
    .await?;
    let (user_id, hash) = row.ok_or_else(ApiError::unauthorized)?;
    let valid = tokio::task::spawn_blocking(move || {
        PasswordHash::new(&hash).ok().is_some_and(|parsed| {
            Argon2::default()
                .verify_password(input.password.as_bytes(), &parsed)
                .is_ok()
        })
    })
    .await
    .map_err(|_| ApiError::internal())?;
    if !valid {
        return Err(ApiError::unauthorized());
    }
    let token = format!("{}{}", Uuid::new_v4().simple(), Uuid::new_v4().simple());
    let hash = token_hash(&token);
    let expires_at = now_ms() + SESSION_LIFETIME_MS;
    let saved_user_id = user_id.clone();
    with_db(&state, move |db| {
        db.execute(
            "INSERT INTO sessions (token_hash, user_id, expires_at) VALUES (?1, ?2, ?3)",
            params![hash, saved_user_id, expires_at],
        )
        .map_err(|_| ApiError::internal())?;
        Ok(())
    })
    .await?;
    Ok(Json(AuthResponse {
        user_id,
        email,
        token,
        expires_at,
    }))
}

#[utoipa::path(post, path = "/api/v1/auth/logout", tag = "Tài khoản",
    responses((status = 204), (status = 401)))]
pub(crate) async fn logout(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> ApiResult<StatusCode> {
    let hash = bearer(&headers)?;
    with_db(&state, move |db| {
        db.execute("DELETE FROM sessions WHERE token_hash = ?1", [hash])
            .map_err(|_| ApiError::internal())?;
        Ok(())
    })
    .await?;
    Ok(StatusCode::NO_CONTENT)
}

#[derive(Serialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Profile {
    id: String,
    email: String,
}

#[utoipa::path(get, path = "/api/v1/auth/me", tag = "Tài khoản",
    responses((status = 200, body = Profile), (status = 401)))]
pub(crate) async fn me(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> ApiResult<Json<Profile>> {
    let user_id = authenticated_user(&state, &headers).await?;
    with_db(&state, move |db| {
        db.query_row(
            "SELECT id, email FROM users WHERE id = ?1",
            [user_id],
            |row| {
                Ok(Profile {
                    id: row.get(0)?,
                    email: row.get(1)?,
                })
            },
        )
        .map(Json)
        .map_err(|_| ApiError::internal())
    })
    .await
}
