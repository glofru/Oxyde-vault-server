use crate::errors::AppError;
use crate::state::AppState;
use axum::extract::State;
use axum::routing::get;
use axum::{Json, Router};
use serde::Serialize;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct LatestRevisionResponse {
    revision_id: String,
}

async fn latest_revision(
    State(state): State<AppState>,
) -> Result<Json<LatestRevisionResponse>, AppError> {
    let revision_id = state.git_client.fetch()?.to_string();
    Ok(Json(LatestRevisionResponse { revision_id }))
}

pub fn revisions_routes() -> Router<AppState> {
    Router::new().route("/latest", get(latest_revision))
}
