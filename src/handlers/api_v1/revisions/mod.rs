use crate::handlers::api_v1::revisions::revision_id::revision_id_routes;
use crate::state::AppState;
use axum::routing::get;
use axum::Router;

mod latest;
mod revision_id;

pub fn revisions_routes() -> Router<AppState> {
    Router::new()
        .route("/latest", get(revision_id::get_revision::get_revision))
        .route("/latest-revision-id", get(latest::latest_revision))
        .nest("/{revision_id}", revision_id_routes())
}
