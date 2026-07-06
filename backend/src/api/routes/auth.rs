use axum::{extract::State, routing::post, Json, Router};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{auth::issue_access_token, shared::AppError, AppState};

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

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/login", post(login))
        .route("/register", post(register))
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
    let refresh_token = issue_access_token(user_id, &role, &state.settings.jwt_refresh_secret)?;
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
    let refresh_token = issue_access_token(user_id, &role, &state.settings.jwt_refresh_secret)?;

    Ok(Json(AuthResponse {
        access_token,
        refresh_token,
        user_id,
        role,
    }))
}
