use chrono::{Duration, Local};
use poise::serenity_prelude as serenity;
use poise::serenity_prelude::{CreateMessage, Mentionable, RoleId};
use regex::Regex;
use sqlx::Row;
use crate::{Context, Error};
use crate::db::Suspension;
use crate::helper;

/// Suspends a user for a duration
#[poise::command(slash_command)]
pub async fn suspend(
    ctx: Context<'_>,
    #[description = "Selected user"] user: serenity::User,
    #[description = "Duration"] duration: String,
    #[description = "Reason"] reason: Option<String>,
) -> Result<(), Error> {
    
    let author_member = &ctx.author_member().await.unwrap();
    
    // Check if author has suspension permission
    if !helper::has_user_suspension_permission(&ctx, author_member) {
        return Ok(());
    }
    
    // Check if the user has an active suspension
    if helper::is_user_suspended(&ctx, author_member).await {
        
        ctx.send(
            poise::CreateReply::default()
                .content(format!(":x: {} is already suspended!", user.mention()))
                .ephemeral(true)
        ).await?;

        return Ok(());
    }
    
    // Evaluate the duration
    let re = Regex::new(r"^(\d+)([shdwm])$").unwrap();
    
    if let Some(caps) = re.captures(duration.as_str()) {
        let value: i64 = caps[1].parse().unwrap(); // Extract number
        let unit = &caps[2]; // Extract time unit ('d', 'w', or 'm')

        let now = Local::now().naive_local();
        let until = match unit {
            "s" => now + Duration::seconds(value),
            "h" => now + Duration::hours(value),
            "d" => now + Duration::days(value),
            "w" => now + Duration::weeks(value),
            "m" => now + Duration::days(value * 30), // Approximation of a month (30 days)
            _ => {
                println!("Unknown unit: {}", unit);
                now
            }
        };

        let guild = ctx.guild_id().unwrap();
        let guild_member = guild.member(&ctx, user.id).await.unwrap();
        let roles: Vec<String> = guild_member.roles.iter().map(|role_id| role_id.get().to_string()).collect();
        let db = &ctx.data().database;

        let until_string = &until.format("%Y-%m-%d %H:%M:%S").to_string();
        let reason_string = reason.unwrap_or_else(|| String::from("Not specified"));

        db.log_suspension(
            guild.get() as i64,
            user.id.get() as i64,
            ctx.author().id.get() as i64,
            &roles,
            &now.format("%Y-%m-%d %H:%M:%S").to_string(),
            until_string,
            &reason_string,
        )
            .await
            .expect(format!("Failed to log suspension for {}", &user.name).as_str());

        let config = &ctx.data().config;
        let guild_id = &ctx.guild_id().unwrap().get();
        let suspended_role = config.guilds.get(guild_id).unwrap().roles.suspended;

        guild_member.remove_roles(&ctx, &guild_member.roles).await?;
        guild_member.add_role(&ctx, suspended_role).await?;

        // Try to obtain the guilds log channel
        let log_channel_id = config.guilds.get(guild_id).unwrap().channels.log;
        
        if let Some(tuple) = guild.channels(&ctx).await.unwrap().iter().find(|tuple| {*tuple.0 == log_channel_id}) {
            
            // Create an embed
            let embed = serenity::CreateEmbed::default()
                .title("Suspension Log")
                .color(serenity::Colour::DARK_RED)
                .field("User", user.mention().to_string(), true)
                .field("Moderator", author_member.mention().to_string(), true)
                .field("Reason", &reason_string, false);
            
            // Send the embed
            tuple.1.send_message(&ctx, CreateMessage::default().embed(embed)).await?;
        } else {
            let guild_name = &ctx.guild_id().unwrap().name(&ctx).unwrap();
            println!("Unable to find log channel for guild {} ({})", guild_name, guild_id);
        }
        
        ctx.reply(format!(":hammer: {} has been suspended until {}!", user.mention(), helper::date_string_to_discord_timestamp(until_string))).await?;
        
        Ok(())
    } else {
        
        ctx.send(
            poise::CreateReply::default()
                .content(":x: Invalid input format!")
                .ephemeral(true)
        ).await?;

        Ok(())
    }
}

pub async fn restore_roles(ctx: Context<'_>, suspension: &Suspension) -> Result<(), Error> {

    let guild = ctx.guild_id().unwrap();
    let guild_member = guild.member(&ctx, suspension.user_id as u64).await.unwrap();
    let config = &ctx.data().config;
    let guild_id = &ctx.guild_id().unwrap().get();
    let suspended_role_id = config.guilds.get(guild_id).unwrap().roles.suspended;
    let suspended_role = RoleId::from(suspended_role_id);
    let role_ids = suspension.previous_roles.clone();
    let role_ids_serenity: Vec<RoleId> = role_ids.iter()
        .filter_map(|id| id.parse::<u64>().ok())
        .map(RoleId::from)
        .collect();

    guild_member.remove_role(&ctx, suspended_role).await?;
    guild_member.add_roles(&ctx, &*role_ids_serenity).await?;

    Ok(())
}
