use poise::serenity_prelude as serenity;
use poise::serenity_prelude::Mentionable;
use crate::{Context, Error};
use crate::helper;

/// Returns the history of suspensions for a user
#[poise::command(slash_command)]
pub async fn suspension_history(
    ctx: Context<'_>,
    #[description = "Selected user"] user: serenity::User,
) -> Result<(), Error> {

    let author_member = &ctx.author_member().await.unwrap();

    if !helper::member_has_suspension_permission(&ctx, author_member).await {
        return Ok(());
    }

    let db = &ctx.data().database;
    let guild_id = ctx.guild_id().unwrap().get();
    let suspensions = db.get_suspensions(guild_id as i64, user.id.get() as i64).await?;

    let mut message = format!("## :open_file_folder: Suspension history for {}\r\n", user.mention());
    let mut count = 1;

    if suspensions.len() == 0 {
        message = format!(":sparkles: {} has never been suspended. What a good boy/girl!", user.mention());
    }

    for suspension in suspensions {
        
        message += format!("\r\n### {count}. Suspension {}\r\nIssued by: {}\r\nFrom: {}\r\nUntil: {}\r\nReason: {}",
                            { if suspension.active.unwrap_or_else(|| false) {"(Active)"} else {""} },
                            ctx.guild_id().unwrap().member(ctx, suspension.moderator_id as u64).await?.mention(),
                            helper::date_string_to_discord_timestamp(&suspension.from_datetime),
                            helper::date_string_to_discord_timestamp(&suspension.until_datetime),
                            suspension.reason.as_deref().unwrap_or("None")
        ).as_str();

        count += 1;
    }
    
    ctx.send(
        poise::CreateReply::default()
            .content(message)
            .ephemeral(true)
    ).await?;

    Ok(())
}