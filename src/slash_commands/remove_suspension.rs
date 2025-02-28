use poise::serenity_prelude as serenity;
use poise::serenity_prelude::Mentionable;
use crate::{helper, Context, Error};
use crate::slash_commands::suspend::restore_roles;

/// Removes a users active suspension
#[poise::command(slash_command)]
pub async fn remove_suspension(
    ctx: Context<'_>,
    #[description = "Selected user"] user: serenity::User,
) -> Result<(), Error> {

    let author_member = &ctx.author_member().await.unwrap();

    if !helper::has_user_suspension_permission(&ctx, author_member) {
        return Ok(());
    }
    
    let db = &ctx.data().database;
    let guild_id = ctx.guild_id().unwrap().get();
    let suspensions = db.get_active_suspensions(guild_id as i64, user.id.get() as i64).await?;

    // Extract the Guild ID instead of keeping `CacheRef`
    let guild_id = ctx.guild_id().unwrap();

    let member = guild_id.member(ctx, user.id).await?;

    for suspension in &suspensions {
        restore_roles(ctx, &suspension).await?;
        db.set_suspension_inactive(suspension.id).await;
    }

    if suspensions.len() > 0 {
        ctx.reply(format!(":broken_chain: {} is no longer suspended!", member.mention())).await?;
    } else {
        ctx.reply(format!(":sparkles: {} has no active suspensions!", member.mention())).await?;
    }

    Ok(())
}
