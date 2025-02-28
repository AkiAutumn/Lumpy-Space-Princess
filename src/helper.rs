use std::borrow::Cow;
use chrono::{Local, NaiveDateTime};
use poise::serenity_prelude::Member;
use crate::Context;

pub fn date_string_to_discord_timestamp(date_string: &str) -> String {
    let datetime = NaiveDateTime::parse_from_str(date_string, "%Y-%m-%d %H:%M:%S%.9f").expect("Failed to parse datetime");
    format!("<t:{}>", datetime.and_local_timezone(Local).unwrap().timestamp())
}

pub fn has_user_suspension_permission(ctx: &Context<'_>, member: &Cow<Member>) -> bool {

    let config = &ctx.data().config;
    let guild_id = &ctx.guild_id().unwrap().get();
    let permitted_roles = &config.guilds.get(guild_id).unwrap().roles.suspend_permitted;
    
    let members_permitted_roles = member.roles.iter().filter(|role_id| permitted_roles.contains(&role_id.get())).collect::<Vec<_>>();
    
    if members_permitted_roles.is_empty() {
        return false;
    }
    
    true
}