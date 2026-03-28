use crate::configuration::Configuration;
use crate::git::git_client::GitClient;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

mod configuration;
mod errors;
mod git;
mod handlers;
mod router;
mod state;

#[tokio::main]
async fn main() {
    let configuration = Configuration::load();

    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            configuration
                .rust_log
                .unwrap_or_else(|| "info,tower_http=debug".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let git_client = GitClient::new(
        configuration.branch.clone(),
        configuration.personal_access_token.clone(),
        configuration.username.clone(),
        configuration.repository_name.clone(),
    )
    .expect("Fail to create the Git Client");
    git_client.pull().expect(
        format!(
            "Fail to pull origin/{} for {}",
            configuration.branch, configuration.repository_name
        )
        .as_str(),
    );

    let state = state::AppState {
        app_name: "OxydeVault".to_string(),
        git_client,
    };

    let app = router::create_router(state).layer(tower_http::trace::TraceLayer::new_for_http());

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", configuration.port))
        .await
        .unwrap();
    tracing::info!("Listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}
