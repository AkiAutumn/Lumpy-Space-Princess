use chrono::{Duration, Local};
use poise::serenity_prelude as serenity;
use poise::serenity_prelude::Mentionable;
use regex::Regex;
use crate::{Context, Error};

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

        let db = &ctx.data().database;

        db.log_suspension(
            user.id.get() as i64,
            ctx.author().id.get() as i64,
            &[], // You'll need to fetch user roles here
            &now.format("%Y-%m-%d %H:%M:%S").to_string(),
            &until.format("%Y-%m-%d %H:%M:%S").to_string(),
            reason.unwrap_or_else(|| String::from("NULL")).as_str(),
        )
            .await
            .expect(format!("Failed to log suspension for {}", &user.name).as_str());

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