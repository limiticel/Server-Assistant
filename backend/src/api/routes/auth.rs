use axum::{
    extract::State,
    http::HeaderMap,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    auth::{issue_access_token, issue_refresh_token, verify_access_token, Claims},
    shared::AppError,
    AppState,
};

#[derive(Deserialize)]
struct AuthRequest {
    email: String,
    password: String,
    name: Option<String>,
}

#[derive(Serialize)]
struct AuthResponse {
    access_token: String,
    refresh_token: String,
    user_id: Uuid,
    role: String,
}

#[derive(Deserialize)]
struct ProfileUpdateRequest {
    name: String,
}

#[derive(Deserialize)]
struct RefreshRequest {
    refresh_token: String,
}

#[derive(Serialize)]
struct ProfileResponse {
    id: Uuid,
    email: String,
    name: String,
    role: String,
}

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/login", post(login))
        .route("/register", post(register))
        .route("/refresh", post(refresh))
        .route("/me", get(me).put(update_me))
}

async fn login(
    State(state): State<AppState>,
    Json(payload): Json<AuthRequest>,
) -> Result<Json<AuthResponse>, AppError> {
    let row: Option<(Uuid, String)> = sqlx::query_as("select id, role from users where email = $1 and password_hash = crypt($2, password_hash) and active = true")
        .bind(&payload.email)
        .bind(&payload.password)
        .fetch_optional(&state.db)
        .await?;

    let (user_id, role) = row.ok_or(AppError::Unauthorized)?;
    let access_token = issue_access_token(user_id, &role, &state.settings.jwt_secret)?;
    let refresh_token = issue_refresh_token(user_id, &role, &state.settings.jwt_refresh_secret)?;
    Ok(Json(AuthResponse {
        access_token,
        refresh_token,
        user_id,
        role,
    }))
}

async fn register(
    State(state): State<AppState>,
    Json(payload): Json<AuthRequest>,
) -> Result<Json<AuthResponse>, AppError> {
    let user_id = Uuid::new_v4();
    let role = "user".to_owned();
    sqlx::query("insert into users (id, email, name, password_hash, role, active) values ($1, $2, $3, crypt($4, gen_salt('bf')), $5, true)")
        .bind(user_id)
        .bind(&payload.email)
        .bind(payload.name.unwrap_or_else(|| payload.email.clone()))
        .bind(&payload.password)
        .bind(&role)
        .execute(&state.db)
        .await?;

    let access_token = issue_access_token(user_id, &role, &state.settings.jwt_secret)?;
    let refresh_token = issue_refresh_token(user_id, &role, &state.settings.jwt_refresh_secret)?;

    Ok(Json(AuthResponse {
        access_token,
        refresh_token,
        user_id,
        role,
    }))
}

async fn refresh(
    State(state): State<AppState>,
    Json(payload): Json<RefreshRequest>,
) -> Result<Json<AuthResponse>, AppError> {
    let claims = verify_access_token(&payload.refresh_token, &state.settings.jwt_refresh_secret)?;
    let profile = load_profile(&state, claims.sub).await?;
    let access_token = issue_access_token(profile.id, &profile.role, &state.settings.jwt_secret)?;
    let refresh_token = issue_refresh_token(
        profile.id,
        &profile.role,
        &state.settings.jwt_refresh_secret,
    )?;

    Ok(Json(AuthResponse {
        access_token,
        refresh_token,
        user_id: profile.id,
        role: profile.role,
    }))
}

async fn me(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<ProfileResponse>, AppError> {
    let claims = claims_from_headers(&headers, &state)?;
    let profile = load_profile(&state, claims.sub).await?;
    Ok(Json(profile))
}

async fn update_me(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<ProfileUpdateRequest>,
) -> Result<Json<ProfileResponse>, AppError> {
    let claims = claims_from_headers(&headers, &state)?;
    let name = payload.name.trim();
    if name.is_empty() {
        return Err(AppError::Validation("name is required".to_owned()));
    }

    sqlx::query("update users set name = $2, updated_at = now() where id = $1 and active = true")
        .bind(claims.sub)
        .bind(name)
        .execute(&state.db)
        .await?;

    let profile = load_profile(&state, claims.sub).await?;
    Ok(Json(profile))
}

async fn load_profile(state: &AppState, user_id: Uuid) -> Result<ProfileResponse, AppError> {
    let row: Option<(Uuid, String, String, String)> = sqlx::query_as(
        "select id, email, name, role
         from users
         where id = $1 and active = true",
    )
    .bind(user_id)
    .fetch_optional(&state.db)
    .await?;

    row.map(|(id, email, name, role)| ProfileResponse {
        id,
        email,
        name,
        role,
    })
    .ok_or(AppError::Unauthorized)
}

fn claims_from_headers(headers: &HeaderMap, state: &AppState) -> Result<Claims, AppError> {
    let token = headers
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.strip_prefix("Bearer "))
        .ok_or(AppError::Unauthorized)?;

    verify_access_token(token, &state.settings.jwt_secret)
}
