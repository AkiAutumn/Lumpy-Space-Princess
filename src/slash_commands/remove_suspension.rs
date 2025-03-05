use poise::serenity_prelude as serenity;
use poise::serenity_prelude::Mentionable;
use crate::{helper, Context, Error};
use crate::config::Config;
use crate::helper::restore_roles;

/// Removes a users active suspension
#[poise::command(slash_command)]
pub async fn remove_suspension(
    ctx: Context<'_>,
    #[description = "Selected user"] user: serenity::User,
) -> Result<(), Error> {

    let author_member = &ctx.author_member().await.unwrap();

    if !helper::user_has_suspension_permission(&ctx, author_member).await {
        return Ok(());
    }
    
    let db = &ctx.data().database;
    let guild = ctx.guild_id().unwrap();
    let guild_id = guild.get();
    let suspensions = db.get_active_suspensions(guild_id as i64, user.id.get() as i64).await?;
    let member = guild.member(ctx, user.id).await?;
    let config = &ctx.data().config;
    let guild_config = Config::get_guild_config(&config, *guild_id).unwrap();
    let suspended_role_id = guild_config.roles.suspended;

    for suspension in &suspensions {
        // Try to restore roles
        restore_roles(ctx.http(), guild, suspended_role_id, &suspension).await.expect(format!("Unable to restore roles for user id {}", suspension.user_id).as_str());
        db.set_suspension_inactive(suspension.id).await;
    }

    if suspensions.len() > 0 {
        ctx.reply(format!(":broken_chain: {} is no longer suspended!", member.mention())).await?;
    } else {
        ctx.reply(format!(":sparkles: {} has no active suspensions!", member.mention())).await?;
    }

    Ok(())
}
