use serde::Deserialize;
use std::fs;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub roles: RoleConfig,
    pub channels: ChannelConfig,
}

#[derive(Debug, Deserialize)]
pub struct RoleConfig {
    pub suspended_role: u64,
    pub suspension_permitted_roles: Vec<u64>,
}

#[derive(Debug, Deserialize)]
pub struct ChannelConfig {
    pub bans_channel: u64,
    pub ban_logs_channel: u64,
}

impl Config {
    pub fn from_file(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path)?;
        let config: Config = toml::from_str(&content)?;
        Ok(config)
    }
}

