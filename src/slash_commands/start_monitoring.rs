use chrono::Utc;
use poise::serenity_prelude as serenity;
use poise::serenity_prelude::{CreateEmbedFooter, GuildId, Http, Mentionable, UserId};
use sqlx::{Row, SqlitePool};
use tokio::time::{sleep_until, Instant};
use crate::config::Config;
use crate::db::{Database, Suspension};
use crate::slash_commands::suspend::{restore_roles};

pub async fn start_monitoring(pool: &SqlitePool, http: &Http, config: &Config, db: &Database) {

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

            let guild = http.get_guild(GuildId::new(suspension.guild_id as u64)).await.unwrap();
            let guild_id = guild.id;
            let log_channel_id = config.guilds.get(&guild_id.to_string()).unwrap().channels.log;
            let suspended_role_id = config.guilds.get(&guild_id.to_string()).unwrap().roles.suspended;

            // Try to restore roles
            restore_roles(&http, guild_id, suspended_role_id, &suspension).await.expect(format!("Unable to restore roles for user id {}", suspension.user_id).as_str());

            // Set suspension inactive
            db.set_suspension_inactive(suspension.id).await;

            println!("Suspension ({}) has ended for user id {}", suspension.id, suspension.user_id);

            if let Some(tuple) = guild.channels(&http).await.unwrap().iter().find(|tuple| {*tuple.0 == log_channel_id}) {
                
                let member_id = UserId::new(suspension.user_id as u64);
                let member = guild.member(&http, member_id).await
                    .expect(&format!("Failed to get member ({}) from guild {}", member_id, guild.name));
                let avatar_url = member.avatar_url().unwrap_or_else(|| member.user.default_avatar_url().to_string());

                // Create an embed
                let embed = serenity::CreateEmbed::default()
                    .title("Suspension expired")
                    .thumbnail(avatar_url)
                    .color(serenity::Colour::ROSEWATER)
                    .field("User", member.mention().to_string(), false)
                    .footer(CreateEmbedFooter::new(format!("ID: {}", suspension.id)));

                // Send the embed
                tuple.1.send_message(&http, serenity::CreateMessage::default().embed(embed)).await
                    .expect(&format!("Failed to send message to log-channel of guild {}", guild.name));
            } else {
                println!("Unable to find log channel for guild {} ({})", guild.name, guild_id);
            }
        }
        
        sleep_until(Instant::now() + std::time::Duration::from_secs(config.monitoring_interval_in_seconds)).await;
    }
}