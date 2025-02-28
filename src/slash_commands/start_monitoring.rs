use chrono::Utc;
use poise::serenity_prelude as serenity;
use poise::serenity_prelude::{CreateEmbedFooter, Mentionable, UserId};
use sqlx::{Row};
use tokio::time::{sleep_until, Instant};
use crate::{helper, Context, Error};
use crate::db::Suspension;
use crate::slash_commands::suspend::{restore_roles};

static mut MONITORING_ACTIVE: bool = false; // Global bool to store the flag

#[poise::command(slash_command)]
pub async fn start_monitoring(ctx: Context<'_>) -> Result<(), Error> {

    let author_member = &ctx.author_member().await.unwrap();

    if !helper::has_user_suspension_permission(&ctx, author_member).await {
        return Ok(());
    }
    
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
    let guild = ctx.guild_id().unwrap();
    let guild_id = guild.get();
    let config = &ctx.data().config;
    let log_channel_id = config.guilds.get(&guild_id.to_string()).unwrap().channels.log;

    loop {
    
        // Check expired suspensions after waking up
        let expired_suspensions = sqlx::query("SELECT * FROM suspensions WHERE until_datetime <= ? AND active = TRUE")
            .bind(Utc::now().format("%Y-%m-%d %H:%M:%S").to_string())
            .fetch_all(pool)
            .await
            .unwrap_or_else(|_| vec![]);

        for row in expired_suspensions {
            let suspension = Suspension {
                id: row.get("id"),
                guild_id: row.get("guild_id"),
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

            if let Some(tuple) = guild.channels(&ctx).await.unwrap().iter().find(|tuple| {*tuple.0 == log_channel_id}) {
                
                let member_id = UserId::new(suspension.user_id as u64);
                let member = guild.member(&ctx, member_id).await?;
                let avatar_url = member.avatar_url().unwrap_or_else(|| member.user.default_avatar_url().to_string());

                // Create an embed
                let embed = serenity::CreateEmbed::default()
                    .title("Suspension expired")
                    .thumbnail(avatar_url)
                    .color(serenity::Colour::ROSEWATER)
                    .field("User", member.mention().to_string(), false)
                    .footer(CreateEmbedFooter::new(format!("ID: {}", suspension.id)));

                // Send the embed
                tuple.1.send_message(&ctx, serenity::CreateMessage::default().embed(embed)).await?;
            } else {
                let guild_name = &ctx.guild_id().unwrap().name(&ctx).unwrap();
                println!("Unable to find log channel for guild {} ({})", guild_name, guild_id);
            }
        }
        
        sleep_until(Instant::now() + std::time::Duration::from_secs(config.monitoring_interval_in_seconds)).await;
    }
}