use crate::errors::AppError;
use crate::state::AppState;
use axum::extract::State;
use axum::routing::get;
use axum::{Json, Router};
use serde::Serialize;

#[derive(Serialize)]
struct HealthCheckResponse {
    app_name: String,
    status: String,
}

async fn health_check(
    State(state): State<AppState>,
) -> Result<Json<HealthCheckResponse>, AppError> {
    Ok(Json(HealthCheckResponse {
        app_name: state.app_name,
        status: "healthy".to_string(),
    }))
}

pub fn health_check_routes() -> Router<AppState> {
    Router::new().route("/", get(health_check))
}
