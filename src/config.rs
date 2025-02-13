use serde::Deserialize;
use std::fs;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub bot: BotConfig,
    pub roles: RoleConfig,
}

#[derive(Debug, Deserialize)]
pub struct BotConfig {

}

#[derive(Debug, Deserialize)]
pub struct RoleConfig {
    pub suspended_role: u64,
}

impl Config {
    pub fn from_file(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path)?;
        let config: Config = toml::from_str(&content)?;
        Ok(config)
    }
}
