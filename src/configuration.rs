use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
#[serde(default)]
pub struct Configuration {
    pub port: u16,
    pub rust_log: Option<String>,
}

impl Default for Configuration {
    fn default() -> Self {
        Self {
            port: 3000,
            rust_log: Some("info".to_string()),
        }
    }
}

impl Configuration {
    pub fn load() -> Self {
        // 1. Load the .env file (ignore errors if it doesn't exist in production)
        dotenvy::dotenv().ok();

        // 2. Parse the environment variables into the Configuration struct
        envy::from_env::<Configuration>().expect("Failed to load the configuration")
    }
}
