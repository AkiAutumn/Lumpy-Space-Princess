mod slash_commands;
mod db;
mod helper;
mod config;

use poise::serenity_prelude as serenity;
use dotenv::dotenv;
use crate::db::Database;
use crate::config::Config;

struct Data {
    pub config: config::Config,
    pub database: Database
} // User data, which is stored and accessible in all command invocations
type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

#[tokio::main]
async fn main() {
    // Load the environment variables from the .env file
    dotenv().ok();

    // Load the config
    let config = Config::from_file("config.toml").expect("Unable to access config.toml"); 

    // Configure the bot
    let token = std::env::var("DISCORD_TOKEN").expect("No DISCORD_TOKEN in .env");
    let intents = serenity::GatewayIntents::non_privileged();

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![
                slash_commands::suspend::suspend(),
                slash_commands::remove_suspension::remove_suspension(),
                slash_commands::suspension_history::suspension_history()
            ],
            ..Default::default()
        })
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;

                let database = Database::new().await.expect("Failed to initialize database");
                //tokio::spawn(slash_commands::suspend::monitor_suspensions(ctx.clone(), &ctx.data().database.clone()));

                Ok(Data { config, database })
            })
        })
        .build();

    // Run the bot
    let mut client = serenity::ClientBuilder::new(token, intents)
        .framework(framework)
        .await
        .unwrap();
    
    client.start().await.unwrap();
}