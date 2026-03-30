use crate::errors::AppError;
use crate::git::git_client::File;
use crate::state::AppState;
use axum::extract::{Path, Query, State};
use axum::Json;
use base64::engine::general_purpose;
use base64::Engine;
use serde::{Deserialize, Serialize};

const MAXIMUM_RESPONSE_SIZE: usize = 5_242_880;

#[derive(Deserialize)]
pub struct GetRevisionQueryParameters {
    page: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GetRevisionFile {
    path: String,
    content: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GetRevisionResponse {
    revision_id: String,
    files: Vec<GetRevisionFile>,
    next_page: Option<String>,
}

pub async fn get_revision(
    State(mut state): State<AppState>,
    path: Option<Path<String>>,
    Query(parameters): Query<GetRevisionQueryParameters>,
) -> Result<Json<GetRevisionResponse>, AppError> {
    let revision_id = match path {
        None => state.git_client.get_latest_commit_id()?.to_string(),
        Some(Path(revision_id)) => revision_id,
    };

    let commit_data = state.git_client.get_commit_data(
        revision_id.as_str(),
        parameters.page,
        MAXIMUM_RESPONSE_SIZE,
    )?;

    Ok(Json(GetRevisionResponse {
        revision_id,
        files: commit_data.files.into_iter().map(Into::into).collect(),
        next_page: commit_data.last_file_id,
    }))
}

impl From<File> for GetRevisionFile {
    fn from(value: File) -> Self {
        let content = general_purpose::URL_SAFE_NO_PAD.encode(value.content);
        Self {
            path: value.path,
            content,
        }
    }
}
