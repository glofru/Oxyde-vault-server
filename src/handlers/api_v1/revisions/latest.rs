use crate::errors::AppError;
use crate::state::AppState;
use axum::extract::State;
use axum::Json;
use serde::Serialize;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LatestRevisionResponse {
    revision_id: String,
}

pub async fn latest_revision(
    State(mut state): State<AppState>,
) -> Result<Json<LatestRevisionResponse>, AppError> {
    let revision_id = state.git_client.get_latest_commit_id()?.to_string();
    Ok(Json(LatestRevisionResponse { revision_id }))
}
