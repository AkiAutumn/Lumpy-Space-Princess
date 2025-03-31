use serde::Deserialize;
use std::fs;

#[derive(Debug, Deserialize, Clone)]
pub(crate) struct Config {
    pub(crate) monitoring_interval_in_seconds: u64,
    pub(crate) guilds: Vec<GuildConfig>,
}

#[derive(Debug, Deserialize, Clone)]
pub(crate) struct GuildConfig {
    pub(crate) id: u64,
    pub(crate) channels: Channels,
    pub(crate) roles: Roles,
}

#[derive(Debug, Deserialize, Clone)]
pub(crate) struct Channels {
    pub(crate) ban_log: u64,
    pub(crate) ban_log_staff: u64,
    pub(crate) event_log: u64,
}

#[derive(Debug, Deserialize, Clone)]
pub(crate) struct Roles {
    pub(crate) suspended: u64,
    pub(crate) suspend_permitted: Vec<u64>,
}

impl Config {
    pub fn get_guild_config(&self, guild_id: u64) -> Option<&GuildConfig> {
        self.guilds.iter().find(|g| g.id == guild_id)
    }
}

