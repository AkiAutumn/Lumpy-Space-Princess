use std::sync::{Arc, Mutex};
use chrono::Utc;
use sqlx::{Row};
use tokio::time::{sleep_until, Instant};
use crate::{Context, Error};
use crate::db::Suspension;
use crate::slash_commands::suspend::{restore_roles};

static mut MONITORING_ACTIVE: bool = false; // Global bool to store the flag

#[poise::command(slash_command)]
pub async fn start_monitoring(ctx: Context<'_>) -> Result<(), Error> {
    
    unsafe {
        
        if MONITORING_ACTIVE {
            ctx.reply(":x: Already monitoring!").await?;
            return Ok(());
        }
        
        MONITORING_ACTIVE = true;
    }

    // Perform the async reply operation after dropping the lock
    ctx.reply(":mag: Starting monitoring...").await?;

    let pool = &ctx.clone().data().database.pool;

    loop {
        // Find the next suspension to expire
        let next_expiration: Option<chrono::DateTime<Utc>> = sqlx::query("SELECT until FROM suspensions ORDER BY until ASC LIMIT 1")
            .fetch_optional(pool)
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

                println!("Suspension has ended for user id {}", suspension.user_id);
            }
        } else {
            // No suspensions, sleep for a longer time (e.g., an hour)
            sleep_until(Instant::now() + std::time::Duration::from_secs(3600)).await;
        }
    }
}