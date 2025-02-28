use serde::Deserialize;
use std::fs;

#[derive(Debug, Deserialize)]
pub(crate) struct Config {
    pub(crate) guilds: std::collections::HashMap<u64, GuildConfig>,
    pub(crate) monitoring_interval_in_seconds: u64,
}

#[derive(Debug, Deserialize)]
pub(crate) struct GuildConfig {
    pub(crate) channels: Channels,
    pub(crate) roles: Roles,
}

#[derive(Debug, Deserialize)]
pub(crate) struct Channels {
    pub(crate)log: u64,
}

#[derive(Debug, Deserialize)]
pub(crate) struct Roles {
    pub(crate) suspended: u64,
    pub(crate) suspend_permitted: Vec<u64>,
}

impl Config {
    pub fn from_file(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path)?;
        let config: Config = toml::from_str(&content)?;
        Ok(config)
    }
}

