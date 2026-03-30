use crate::state::AppState;
use axum::Router;

mod health_check;
mod revisions;

pub fn api_v1_routes() -> Router<AppState> {
    Router::new()
        .nest("/revisions", revisions::revisions_routes())
        .nest("/health", health_check::health_check_routes())
}
