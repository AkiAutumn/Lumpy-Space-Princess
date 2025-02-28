mod slash_commands;
mod db;
mod helper;
mod config;

use poise::serenity_prelude as serenity;
use dotenv::dotenv;
use poise::serenity_prelude::CreateMessage;
use crate::db::Database;
use crate::config::Config;
use crate::slash_commands::start_monitoring::start_monitoring;

struct Data {
    pub config: Config,
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

    // Get the database
    let database = Database::new().await.expect("Failed to initialize database");

    // Configure the bot
    let token = std::env::var("DISCORD_TOKEN").expect("No DISCORD_TOKEN in .env");
    let intents = serenity::GatewayIntents::non_privileged();

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

    // Run the bot
    let mut client = serenity::ClientBuilder::new(token, intents)
        .framework(framework)
        .await
        .unwrap();

    client.start().await.unwrap();
    
    let guilds = client.http.get_guilds(None, None).await.unwrap();
    println!("Connected to {} guilds:\r\n{}", 
             guilds.len(), 
             guilds.iter()
                 .map(|guild| guild.name.clone())
                 .collect::<Vec<_>>()
                 .join(", "));
    
    // Send a message into each guild's log channel
    for guild in client.cache.guilds() {

        let guild_id = guild.get();
        let log_channel_id = &config.guilds.get(&guild_id.to_string()).unwrap().channels.log;

        if let Some(tuple) = guild.channels(client.http.as_ref()).await.unwrap().iter().find(|tuple| {&tuple.0.get() == log_channel_id}) {
            tuple.1.send_message(&client.http, CreateMessage::default().content(":electric_plug: Connected!")).await.unwrap();
        } else {
            let guild_name = guild.name(client.cache.as_ref()).unwrap();
            println!("Unable to find log channel for guild {} ({})", guild_name, guild_id);
        }
    }

    // Start never ending monitoring task
    // Might have to export this to another thread if anything needs to be executed after this
    start_monitoring(&database.pool, &client.http, &config, &database).await;
}