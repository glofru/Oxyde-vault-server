use crate::git::git_client::GitClient;

#[derive(Clone)]
pub struct AppState {
    pub app_name: String,
    pub git_client: GitClient,
}
