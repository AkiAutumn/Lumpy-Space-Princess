use chrono::{Duration, Local, Utc};
use poise::serenity_prelude as serenity;
use poise::serenity_prelude::{Mentionable, RoleId};
use regex::Regex;
use tokio::time::{sleep_until, Instant};
use sqlx::{SqlitePool, Row};
use crate::{Context, Error};
use crate::db::Suspension;

/// Suspends a user for a duration
#[poise::command(slash_command)]
pub async fn suspend(
    ctx: Context<'_>,
    #[description = "Selected user"] user: serenity::User,
    #[description = "Duration"] duration: String,
    #[description = "Reason"] reason: Option<String>,
) -> Result<(), Error> {
    
    let re = Regex::new(r"^(\d+)([dwm])$").unwrap();
    
    if let Some(caps) = re.captures(duration.as_str()) {
        let value: i64 = caps[1].parse().unwrap(); // Extract number
        let unit = &caps[2]; // Extract time unit ('d', 'w', or 'm')

        let now = Local::now().naive_local();
        let until = match unit {
            "d" => now + Duration::days(value),
            "w" => now + Duration::weeks(value),
            "m" => now + Duration::days(value * 30), // Approximation of a month (30 days)
            _ => {
                println!("Unknown unit: {}", unit);
                now
            }
        };

        println!("--- BEGIN OF SUSPENSION ---\r\nUser: {} \r\nModerator: {} \r\nFrom: {} \r\nUntil: {} \r\nReason: {} \r\n--- END OF SUSPENSION ---",
        user.name, ctx.author().name, now.to_string(), until.to_string(), reason.clone().unwrap_or_else(|| String::from("(None)")));

        let guild = ctx.guild_id().unwrap();
        let guild_member = guild.member(&ctx, user.id).await.unwrap();
        let roles: Vec<String> = guild_member.roles.iter().map(|role_id| role_id.get().to_string()).collect();
        let db = &ctx.data().database;

        db.log_suspension(
            user.id.get() as i64,
            ctx.author().id.get() as i64,
            &roles,
            &now.format("%Y-%m-%d %H:%M:%S").to_string(),
            &until.format("%Y-%m-%d %H:%M:%S").to_string(),
            reason.unwrap_or_else(|| String::from("NULL")).as_str(),
        )
            .await
            .expect(format!("Failed to log suspension for {}", &user.name).as_str());

        let config = &ctx.data().config;
        let suspended_role = config.roles.suspended_role;

        guild_member.remove_roles(&ctx, &guild_member.roles).await?;
        guild_member.add_role(&ctx, suspended_role).await?;

        ctx.send(
            poise::CreateReply::default()
                .content(format!("{} has been suspended until {}!", user.mention(), until))
                .ephemeral(true)
        ).await?;
        
        Ok(())
    } else {
        
        ctx.send(
            poise::CreateReply::default()
                .content("Invalid input format!")
                .ephemeral(true)
        ).await?;

        Ok(())
    }
}

pub async fn restore_roles(ctx: Context<'_>, suspension: &Suspension) -> Result<(), Error> {

    let guild = ctx.guild_id().unwrap();
    let guild_member = guild.member(&ctx, suspension.user_id as u64).await.unwrap();
    let config = &ctx.data().config;
    let suspended_role = RoleId::from(config.roles.suspended_role);
    let role_ids = suspension.previous_roles.clone();
    let role_ids_serenity: Vec<RoleId> = role_ids.iter()
        .filter_map(|id| id.parse::<u64>().ok())
        .map(RoleId::from)
        .collect();

    guild_member.remove_role(&ctx, suspended_role).await?;
    guild_member.add_roles(&ctx, &*role_ids_serenity).await?;

    Ok(())
}

pub(crate) async fn monitor_suspensions(ctx: Context<'_>, db: SqlitePool) {
    loop {
        // Find the next suspension to expire
        let next_expiration: Option<chrono::DateTime<Utc>> = sqlx::query("SELECT until FROM suspensions ORDER BY until ASC LIMIT 1")
            .fetch_optional(&db)
            .await
            .ok()
            .flatten()
            .map(|row: sqlx::sqlite::SqliteRow| row.get("until"));

        if let Some(expiration_time) = next_expiration {
            let now = Utc::now();
            let duration_until_next = (expiration_time - now).to_std().unwrap_or_default();

            // Sleep until the next suspension expires
            sleep_until(Instant::now() + duration_until_next).await;

            // Re-check expired suspensions after waking up
            let expired_suspensions = sqlx::query("SELECT user_id FROM suspensions WHERE until <= ?")
                .bind(Utc::now())
                .fetch_all(&db)
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

                println!("Suspension has ended for user id {}", suspension.user_id);
            }
        } else {
            // No suspensions, sleep for a longer time (e.g., an hour)
            sleep_until(Instant::now() + std::time::Duration::from_secs(3600)).await;
        }
    }
}
