use poise::serenity_prelude as serenity;
use poise::serenity_prelude::Mentionable;
use crate::{Context, Error};
use crate::slash_commands::suspend;
use crate::slash_commands::suspend::remove_suspension;

/// Removes a users active suspension
#[poise::command(slash_command)]
pub async fn suspension_history(
    ctx: Context<'_>,
    #[description = "Selected user"] user: serenity::User,
) -> Result<(), Error> {

    let db = &ctx.data().database;
    let suspensions = db.get_active_suspensions(user.id.get() as i64).await?;
    let guild = ctx.guild().unwrap();
    let member = guild.member(ctx, user.id).await?;

    for(suspension) in suspensions {
        remove_suspension(ctx, &suspension).await?;
    }

    ctx.reply(format!("{} is no longer suspended!", member.mention())).await?;

    Ok(())
}