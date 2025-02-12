mod slash_commands;
mod db;
mod helper;

use poise::serenity_prelude as serenity;
use dotenv::dotenv;
use crate::db::Database;

struct Data {
    pub database: Database
} // User data, which is stored and accessible in all command invocations
type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

#[tokio::main]
async fn main() {
    // Load the environment variables from the .env file
    dotenv().ok();

    // Configure the bot
    let token = std::env::var("DISCORD_TOKEN").expect("Missing DISCORD_TOKEN");
    let intents = serenity::GatewayIntents::non_privileged();

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![
                slash_commands::suspend::suspend(),
                slash_commands::suspension_history::suspension_history()
            ],
            ..Default::default()
        })
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;

                let database = Database::new().await.expect("Failed to initialize database");

                Ok(Data { database })
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