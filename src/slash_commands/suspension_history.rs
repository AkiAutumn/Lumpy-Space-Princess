use poise::serenity_prelude as serenity;
use poise::serenity_prelude::Mentionable;
use crate::{Context, Error};

/// Returns the history of suspensions for a user
#[poise::command(slash_command)]
pub async fn suspension_history(
    ctx: Context<'_>,
    #[description = "Selected user"] user: serenity::User,
) -> Result<(), Error> {

    let db = &ctx.data().database;

    let suspensions = db.get_suspensions(user.id.get() as i64).await?;

    let mut message = format!("Suspension history for {}\r\n", user.mention());
    let mut count = 1;

    if suspensions.len() == 0 {
        message = format!("{} has never been suspended. What a good boy/girl!", user.mention());
    }

    for suspension in suspensions {
        
        message += format!("\r\n**{count}. Suspension {}**\r\nIssued by: {}\r\nFrom: {}\r\nUntil: {}\r\nReason:{}",
                            { if suspension.active {"(Active)"} else {""} },
                            ctx.guild_id().unwrap().member(ctx, suspension.moderator_id as u64).await?.mention(),
                            suspension.from_datetime,
                            suspension.until_datetime,
                            suspension.reason.as_deref().unwrap_or("None")
        ).as_str();

        count += 1;
    }

    ctx.reply(message).await?;

    Ok(())
}