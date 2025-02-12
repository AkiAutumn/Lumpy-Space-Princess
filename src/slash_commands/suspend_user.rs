use chrono::{Duration, Local};
use poise::serenity_prelude as serenity;
use poise::serenity_prelude::Error;
use regex::Regex;
use crate::Context;

/// Suspends a user for a duration
#[poise::command(slash_command)]
pub async fn suspend_user(
    ctx: Context<'_>,
    #[description = "Selected user"] user: Option<serenity::User>,
    #[description = "Duration"] duration: String,
    #[description = "Reason"] reason: String,
) -> Result<(), Error> {

    let u = user.as_ref().unwrap();
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
        u.name, ctx.author().name, now.to_string(), until.to_string(), reason);

        let db = ctx.data();

        db.log_suspension(
            u.id.get() as i64,
            ctx.author().id.get() as i64,
            &[], // You'll need to fetch user roles here
            &now.format("%Y-%m-%d %H:%M:%S").to_string(),
            &until.format("%Y-%m-%d %H:%M:%S").to_string(),
            reason,
        ).expect("Failed to log suspension for {}", &u.name);

        ctx.send(|m| {
            m.content(format!("{} has been suspended until {}!", u.tag(), until))
                .ephemeral(true)
        }).await?;

        Ok(())
    } else {
        ctx.send(|m| {
            m.content("Invalid input format!")
                .ephemeral(true)
        }).await?;

        Ok(())
    }
}