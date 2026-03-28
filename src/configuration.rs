use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
pub struct Configuration {
    pub branch: String,
    pub personal_access_token: String,
    pub port: u16,
    pub repository_name: String,
    pub rust_log: Option<String>,
    pub username: String,
}

impl Configuration {
    pub fn load() -> Self {
        // 1. Load the .env file (ignore errors if it doesn't exist in production)
        dotenvy::dotenv().ok();

        // 2. Parse the environment variables into the Configuration struct
        envy::from_env::<Configuration>().expect("Fail to load the configuration")
    }
}
