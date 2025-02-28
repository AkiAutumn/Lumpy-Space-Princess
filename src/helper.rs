use std::borrow::Cow;
use chrono::{Local, NaiveDateTime};
use poise::serenity_prelude::Member;
use crate::Context;

pub fn date_string_to_discord_timestamp(date_string: &str) -> String {
    let datetime = NaiveDateTime::parse_from_str(date_string, "%Y-%m-%d %H:%M:%S%.9f").expect("Failed to parse datetime");
    format!("<t:{}>", datetime.and_local_timezone(Local).unwrap().timestamp())
}

pub async fn has_user_suspension_permission(ctx: &Context<'_>, member: &Cow<'_, Member>) -> bool {

    let config = &ctx.data().config;
    let guild_id = &ctx.guild_id().unwrap().get();
    let permitted_roles = &config.guilds.get(&guild_id.to_string()).unwrap().roles.suspend_permitted;
    
    let members_permitted_roles = member.roles.iter().filter(|role_id| permitted_roles.contains(&role_id.get())).collect::<Vec<_>>();
    
    if members_permitted_roles.is_empty() {

        ctx.send(
            poise::CreateReply::default()
                .content(":x: You don't have permission to do that!")
                .ephemeral(true)
        ).await.expect("Failed to send no-permission-reply");
        
        return false;
    }
    
    true
}

pub async fn is_user_suspended(ctx: &Context<'_>, member: &Member) -> bool {
    let guild_id = ctx.guild_id().unwrap().get();
    let db = &ctx.data().database;
    let active_suspensions = db.get_active_suspensions(guild_id as i64, member.user.id.get() as i64).await.unwrap();
    
    active_suspensions.len() > 0
}