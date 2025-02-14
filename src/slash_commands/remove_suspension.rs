use poise::serenity_prelude as serenity;
use poise::serenity_prelude::Mentionable;
use crate::{Context, Error};
use crate::slash_commands::suspend::restore_roles;

/// Removes a users active suspension
#[poise::command(slash_command)]
pub async fn remove_suspension(
    ctx: Context<'_>,
    #[description = "Selected user"] user: serenity::User,
) -> Result<(), Error> {
    let db = &ctx.data().database;
    let suspensions = db.get_active_suspensions(user.id.get() as i64).await?;

    // Extract the Guild ID instead of keeping `CacheRef`
    let guild_id = ctx.guild_id().unwrap();

    let member = guild_id.member(ctx, user.id).await?;

    for suspension in suspensions {
        restore_roles(ctx, &suspension).await?;
        db.set_suspension_inactive(suspension.id).await;
    }

    ctx.reply(format!(":white_check_mark: {} is no longer suspended!", member.mention())).await?;

    Ok(())
}
