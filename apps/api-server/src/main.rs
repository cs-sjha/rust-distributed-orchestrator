use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    routing::{delete, get, patch, post},
    Json, Router,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::{postgres::PgPoolOptions, PgPool};
use std::{env, net::SocketAddr};
use tower_http::trace::TraceLayer;
use uuid::Uuid;

#[derive(Clone)]
struct AppState {
    db: PgPool,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
struct Job {
    id: Uuid,
    kind: String,
    payload: Value,
    priority: i16,
    status: String,
    attempts: i32,
    max_attempts: i32,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
struct CreateJob {
    kind: String,
    #[serde(default = "empty_payload")]
    payload: Value,
    #[serde(default)]
    priority: i16,
    #[serde(default = "default_max_attempts")]
    max_attempts: i32,
}

fn empty_payload() -> Value { serde_json::json!({}) }
fn default_max_attempts() -> i32 { 3 }

#[derive(Debug, Deserialize)]
struct ListQuery {
    limit: Option<i64>,
    offset: Option<i64>,
}

#[derive(Debug, Deserialize)]
struct UpdateStatus {
    status: String,
}

#[derive(Debug, Serialize)]
struct ErrorResponse {
    error: String,
}

fn error(status: StatusCode, message: impl Into<String>) -> (StatusCode, Json<ErrorResponse>) {
    (status, Json(ErrorResponse { error: message.into() }))
}

async fn health() -> &'static str {
    "ok"
}

async fn create_job(
    State(state): State<AppState>,
    Json(input): Json<CreateJob>,
) -> Result<(StatusCode, Json<Job>), (StatusCode, Json<ErrorResponse>)> {
    if input.kind.trim().is_empty() {
        return Err(error(StatusCode::BAD_REQUEST, "kind must not be empty"));
    }
    if input.max_attempts < 1 {
        return Err(error(StatusCode::BAD_REQUEST, "max_attempts must be at least 1"));
    }

    let id = Uuid::new_v4();
    let job = sqlx::query_as::<_, Job>(
        r#"INSERT INTO jobs (id, kind, payload, priority, max_attempts)
           VALUES ($1, $2, $3, $4, $5)
           RETURNING id, kind, payload, priority, status, attempts, max_attempts, created_at, updated_at"#,
    )
    .bind(id)
    .bind(input.kind.trim())
    .bind(input.payload)
    .bind(input.priority)
    .bind(input.max_attempts)
    .fetch_one(&state.db)
    .await
    .map_err(|e| error(StatusCode::INTERNAL_SERVER_ERROR, format!("database error: {e}")))?;

    Ok((StatusCode::CREATED, Json(job)))
}

async fn get_job(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Job>, (StatusCode, Json<ErrorResponse>)> {
    let job = sqlx::query_as::<_, Job>(
        "SELECT id, kind, payload, priority, status, attempts, max_attempts, created_at, updated_at FROM jobs WHERE id = $1"
    )
    .bind(id)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| error(StatusCode::INTERNAL_SERVER_ERROR, format!("database error: {e}")))?
    .ok_or_else(|| error(StatusCode::NOT_FOUND, "job not found"))?;

    Ok(Json(job))
}

async fn list_jobs(
    State(state): State<AppState>,
    Query(query): Query<ListQuery>,
) -> Result<Json<Vec<Job>>, (StatusCode, Json<ErrorResponse>)> {
    let limit = query.limit.unwrap_or(50).clamp(1, 100);
    let offset = query.offset.unwrap_or(0).max(0);
    let jobs = sqlx::query_as::<_, Job>(
        r#"SELECT id, kind, payload, priority, status, attempts, max_attempts, created_at, updated_at
           FROM jobs ORDER BY created_at DESC LIMIT $1 OFFSET $2"#
    )
    .bind(limit)
    .bind(offset)
    .fetch_all(&state.db)
    .await
    .map_err(|e| error(StatusCode::INTERNAL_SERVER_ERROR, format!("database error: {e}")))?;

    Ok(Json(jobs))
}

async fn cancel_job(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Job>, (StatusCode, Json<ErrorResponse>)> {
    let job = sqlx::query_as::<_, Job>(
        r#"UPDATE jobs SET status = 'cancelled', updated_at = NOW()
           WHERE id = $1 AND status = 'pending'
           RETURNING id, kind, payload, priority, status, attempts, max_attempts, created_at, updated_at"#
    )
    .bind(id)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| error(StatusCode::INTERNAL_SERVER_ERROR, format!("database error: {e}")))?
    .ok_or_else(|| error(StatusCode::CONFLICT, "job not found or is not pending"))?;

    Ok(Json(job))
}

async fn update_status(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(input): Json<UpdateStatus>,
) -> Result<Json<Job>, (StatusCode, Json<ErrorResponse>)> {
    const ALLOWED: &[&str] = &["pending", "scheduled", "running", "completed", "failed"];
    if !ALLOWED.contains(&input.status.as_str()) {
        return Err(error(StatusCode::BAD_REQUEST, "unsupported job status"));
    }

    let job = sqlx::query_as::<_, Job>(
        r#"UPDATE jobs SET status = $2, updated_at = NOW()
           WHERE id = $1
           RETURNING id, kind, payload, priority, status, attempts, max_attempts, created_at, updated_at"#
    )
    .bind(id)
    .bind(input.status)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| error(StatusCode::INTERNAL_SERVER_ERROR, format!("database error: {e}")))?
    .ok_or_else(|| error(StatusCode::NOT_FOUND, "job not found"))?;

    Ok(Json(job))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt()
        .with_env_filter(env::var("RUST_LOG").unwrap_or_else(|_| "info,tower_http=info".into()))
        .init();

    let database_url = env::var("DATABASE_URL")?;
    let db = PgPoolOptions::new()
        .max_connections(10)
        .connect(&database_url)
        .await?;

    sqlx::migrate!("../../migrations").run(&db).await?;

    let state = AppState { db };
    let app = Router::new()
        .route("/api/v1/health", get(health))
        .route("/api/v1/jobs", post(create_job).get(list_jobs))
        .route("/api/v1/jobs/{id}", get(get_job).delete(cancel_job))
        .route("/api/v1/jobs/{id}/status", patch(update_status))
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    let host = env::var("HOST").unwrap_or_else(|_| "127.0.0.1".into());
    let port: u16 = env::var("PORT").unwrap_or_else(|_| "8080".into()).parse()?;
    let addr: SocketAddr = format!("{host}:{port}").parse()?;

    let listener = tokio::net::TcpListener::bind(addr).await?;
    tracing::info!(%addr, "API server listening");
    axum::serve(listener, app).await?;
    Ok(())
}
