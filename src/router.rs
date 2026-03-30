use crate::handlers::api_v1::api_v1_routes;
use crate::state::AppState;
use axum::Router;

pub fn create_router(state: AppState) -> Router {
    Router::new()
        .nest("/api/v1", api_v1_routes())
        .with_state(state)
}
