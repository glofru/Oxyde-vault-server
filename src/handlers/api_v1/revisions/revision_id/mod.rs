pub mod get_revision;

use crate::state::AppState;
use axum::routing::get;
use axum::Router;

pub fn revision_id_routes() -> Router<AppState> {
    Router::new().route("/", get(get_revision::get_revision))
}
