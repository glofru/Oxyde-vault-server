use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use crate::configuration::Configuration;

mod configuration;
mod errors;
mod handlers;
mod router;
mod state;

#[tokio::main]
async fn main() {
    let configuration = Configuration::load();
    
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            configuration.rust_log
                .unwrap_or_else(|| "info,tower_http=debug".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let state = state::AppState {
        app_name: "OxydeVault".to_string(),
    };

    let app = router::create_router(state).layer(tower_http::trace::TraceLayer::new_for_http());

    let listener = tokio::net::TcpListener::bind(format!(
        "0.0.0.0:{}",
        configuration.port
    ))
    .await
    .unwrap();
    tracing::info!("Listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}
