use std::sync::{Arc, Mutex};
use chrono::Utc;
use poise::serenity_prelude::{CreateMessage, Mentionable, UserId};
use sqlx::{Row};
use tokio::time::{sleep_until, Instant};
use crate::{helper, Context, Error};
use crate::db::Suspension;
use crate::slash_commands::suspend::{restore_roles};

static mut MONITORING_ACTIVE: bool = false; // Global bool to store the flag

#[poise::command(slash_command)]
pub async fn start_monitoring(ctx: Context<'_>) -> Result<(), Error> {
    
    unsafe {
        
        if MONITORING_ACTIVE {
            
            ctx.send(
                poise::CreateReply::default()
                    .content(":x: Already monitoring!")
                    .ephemeral(true)
            ).await?;
            
            return Ok(());
        }
        
        MONITORING_ACTIVE = true;
    }

    // Perform the async reply operation after dropping the lock
    ctx.reply(":mag: Started monitoring...").await?;

    let pool = &ctx.clone().data().database.pool;

    loop {

        println!("Checking suspensions...");
    
        // Check expired suspensions after waking up
        let expired_suspensions = sqlx::query("SELECT * FROM suspensions WHERE until_datetime <= ? AND active = TRUE")
            .bind(Utc::now().format("%Y-%m-%d %H:%M:%S").to_string())
            .fetch_all(pool)
            .await
            .unwrap_or_else(|_| vec![]);

        for row in expired_suspensions {
            let suspension = Suspension {
                id: row.get("id"),
                user_id: row.get("user_id"),
                moderator_id: row.get("moderator_id"),
                previous_roles: row.get::<String, _>("previous_roles").split(',').map(String::from).collect(),
                from_datetime: row.get("from_datetime"),
                until_datetime: row.get("until_datetime"),
                reason: row.get("reason"),
                active: row.get("active"),
            };

            // Try to restore roles
            restore_roles(ctx, &suspension).await.expect(format!("Unable to remove suspension for user id {}", suspension.user_id).as_str());

            // Set suspension inactive
            let db = &ctx.data().database;
            db.set_suspension_inactive(suspension.id).await;

            println!("Suspension ({}) has ended for user id {}", suspension.id, suspension.user_id);

            /*
            let config = &ctx.data().config;
            let guild = &ctx.guild_id().unwrap();
            
            if let Some(tuple) = guild.channels(&ctx).await.unwrap().iter().find(|tuple| {*tuple.0 == config.channels.bans_channel}) {
                tuple.1.send_message(&ctx.http(), CreateMessage::default()
                    .content(
                        format!("{}'s suspension ended!", guild.member(&ctx.http(), suspension.user_id).await.unwrap().mention())
                    )
                ).await.unwrap();
            } else {
                println!("Unable to find bans channel");
            }
             */
        }

        //sleep_until(Instant::now() + std::time::Duration::from_secs(900)).await;
        sleep_until(Instant::now() + std::time::Duration::from_secs(30)).await;
    }
}