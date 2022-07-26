use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    pub authentication: AuthConfig,
    pub database: DatabaseConfig,
    pub storage: StorageConfig,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AuthConfig {
    pub username: String,
    pub password: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DatabaseConfig {
    pub path: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct StorageConfig {
    pub storage_dir: String,
    pub path_prefix: String,
}

impl Config {
    pub fn from_toml(file_name: &str) -> Self {
        toml::from_str(&std::fs::read_to_string(file_name).unwrap()).unwrap()
    }
}
