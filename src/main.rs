mod slash_commands;
mod db;
mod helper;
mod config;
pub(crate) mod start_monitoring;
mod event_handler;

use poise::serenity_prelude as serenity;
use dotenv::dotenv;
use crate::db::Database;
use crate::config::Config;
use start_monitoring::start_monitoring;
use event_handler::Handler;
use once_cell::sync::Lazy;
use toml;
use std::fs;
use std::sync::RwLock;

struct Data {
    pub config: Config,
    pub database: Database
}

type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

pub static CONFIG: Lazy<RwLock<Config>> = Lazy::new(|| {
    let content = fs::read_to_string("config.toml").unwrap();
    toml::from_str(&content).unwrap()
});

#[tokio::main]
async fn main() {
    
    // Load the environment variables from the .env file
    dotenv().ok();

    // Load the config
    let config = CONFIG.read().unwrap();

    // Get the database
    let database = Database::new().await.expect("Failed to initialize database");

    // Configure the bot
    let token = std::env::var("DISCORD_TOKEN").expect("No DISCORD_TOKEN in .env");
    let intents = serenity::GatewayIntents::non_privileged();

    // Build the framework
    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![
                slash_commands::suspend::suspend(),
                slash_commands::remove_suspension::remove_suspension(),
                slash_commands::suspension_history::suspension_history(),
            ],
            ..Default::default()
        })
        .setup({
            // Clone the config and database because we need them later
            let config = config.clone();
            let database = database.clone();
            
            move |ctx, _ready, framework| {
                Box::pin(async move {
                    poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                    Ok(Data { config, database })
                })
            }
        })
        .build();
    
    // Build the client
    let mut client = serenity::ClientBuilder::new(token, intents)
        .framework(framework)
        .event_handler(Handler)
        .await
        .unwrap();
    
    // Print active guilds
    let guilds = client.http.get_guilds(None, None).await.unwrap();
    println!("Connected to {} guild(s):\r\n{}", 
             guilds.len(), 
             guilds.iter()
                 .map(|guild| guild.name.clone())
                 .collect::<Vec<_>>()
                 .join(", "));
    
    // Spawn monitoring task
    let http = client.http.clone();
    let _ = tokio::spawn( async move {
        start_monitoring(&database.pool, &http, &config, &database).await;
    });
    
    // Run the bot
    client.start().await.unwrap();
}