use std::borrow::Cow;
use chrono::{Local, NaiveDateTime};
use poise::serenity_prelude::{GuildId, Http, Member, RoleId};
use crate::config::Config;
use crate::{Context, Error};
use crate::db::Suspension;

pub fn date_string_to_discord_timestamp(date_string: &str) -> String {
    let datetime = NaiveDateTime::parse_from_str(date_string, "%Y-%m-%d %H:%M:%S%.9f").expect("Failed to parse datetime");
    format!("<t:{}>", datetime.and_local_timezone(Local).unwrap().timestamp())
}

pub async fn user_has_suspension_permission(ctx: &Context<'_>, member: &Cow<'_, Member>) -> bool {

    let config = &ctx.data().config;
    let guild_id = &ctx.guild_id().unwrap().get();
    let guild_config = Config::get_guild_config(&config, *guild_id).unwrap();
    let permitted_roles = &guild_config.roles.suspend_permitted;
    
    let members_permitted_roles = member.roles.iter().filter(|role_id| permitted_roles.contains(&role_id.get())).collect::<Vec<_>>();
    
    if members_permitted_roles.is_empty() && !member.permissions.unwrap().administrator() {

        ctx.send(
            poise::CreateReply::default()
                .content(":x: You don't have permission to do that!")
                .ephemeral(true)
        ).await.expect("Failed to send no-permission-reply");
        
        return false;
    }
    
    true
}

pub async fn user_is_suspended(ctx: &Context<'_>, member: &Member) -> bool {
    let guild_id = ctx.guild_id().unwrap().get();
    let db = &ctx.data().database;
    let active_suspensions = db.get_active_suspensions(guild_id as i64, member.user.id.get() as i64).await.unwrap();
    
    active_suspensions.len() > 0
}

pub async fn restore_roles(http: &Http, guild: GuildId, suspended_role_id: u64, suspension: &Suspension) -> Result<(), Error> {

    let guild_member = guild.member(&http, suspension.user_id as u64).await.unwrap();
    let suspended_role = RoleId::from(suspended_role_id);
    let role_ids = suspension.previous_roles.clone();
    let role_ids_serenity: Vec<RoleId> = role_ids.iter()
        .filter_map(|id| id.parse::<u64>().ok())
        .map(RoleId::from)
        .collect();

    guild_member.remove_role(&http, suspended_role).await?;

    for role_id in role_ids_serenity {
        // Check if the role still exists
        if guild.role(&http, role_id).await.is_ok() {
            // Give the role back to the member
            guild_member.add_role(&http, &role_id).await?;
        }
    }

    Ok(())
}
