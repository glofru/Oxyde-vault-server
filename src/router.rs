use crate::handlers::health_check::health_check_routes;
use crate::state::AppState;
use axum::Router;

pub fn create_router(state: AppState) -> Router {
    let api_v1 = Router::new().nest("/health", health_check_routes());

    Router::new().nest("/api/v1", api_v1).with_state(state)
}
