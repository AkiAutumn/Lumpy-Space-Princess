use std::future::Future;
use poise::serenity_prelude::{Context, ChannelId, CreateEmbed, EventHandler, GuildId, audit_log, MessageId, Message, MessageUpdateEvent, CreateEmbedAuthor, CreateEmbedFooter, User, Member, AuditLogEntry, CreateMessage};
use crate::CONFIG;

pub struct Handler;

#[poise::serenity_prelude::async_trait]
impl EventHandler for Handler {

    async fn message_delete(&self, ctx: Context, channel_id: ChannelId, deleted_message_id: MessageId, guild_id: Option<GuildId>) -> () {

        if guild_id.is_none() {
            return;
        }

        let config = CONFIG.read().unwrap();
        let guild_id = guild_id.unwrap();
        let guild_config = config.get_guild_config(guild_id.get());

        if let Some(guild_config) = guild_config {

            let event_log_channel_id = ChannelId::new(guild_config.channels.event_log);
            let deleted_message = ctx.cache.message(channel_id, deleted_message_id).expect("Deleted message not found in cache");
            let mut user_who_deleted = None;

            // Get the guild's audit logs
            if let Ok(audit_logs) = guild_id.audit_logs(&ctx, Option::from(audit_log::Action::Message(audit_log::MessageAction::Delete)), None, None, None).await {
                if let Some(log_entry) = audit_logs.entries.first() {
                    user_who_deleted = ctx.cache.user(log_entry.user_id);
                }
            }

            let embed = CreateEmbed::default()
                .title("Message Delete")
                .author(CreateEmbedAuthor::new(&deleted_message.author.name).icon_url(&deleted_message.author.avatar_url().unwrap_or_default()))
                .field("Content", &deleted_message.content, false)
                .field("Deleted by", if user_who_deleted.is_some() {
                    &user_who_deleted.unwrap().name
                } else {
                    "Unknown"
                }, false)
                .footer(CreateEmbedFooter::new(format!("<t:{}:f>", &deleted_message.timestamp.timestamp().to_string())));

            event_log_channel_id.send_message(&ctx.http, CreateMessage::default().embed(embed)).await.unwrap();
        }

        Ok(())
    }

    async fn message_update(&self, ctx: Context, old_if_available: Option<Message>, new: Option<Message>, event: MessageUpdateEvent) {

        if old_if_available.is_none() || new.is_none() || event.guild_id.is_none() {
            return;
        }

        let config = CONFIG.read().unwrap();
        let guild_id = event.guild_id.unwrap();
        let guild_config = config.get_guild_config(guild_id.get());

        if let Some(guild_config) = guild_config {

            let event_log_channel_id = ChannelId::new(guild_config.channels.event_log);
            let user = event.author.unwrap();

            let embed = CreateEmbed::default()
                .title("Message Update")
                .author(CreateEmbedAuthor::new(&user.name).icon_url(user.avatar_url().unwrap_or_default()))
                .field("Old", old_if_available.unwrap().content, false)
                .field("New", new.unwrap().content, false)
                .footer(CreateEmbedFooter::new(format!("<t:{}:f>",  &event.timestamp.unwrap().timestamp().to_string())));

            let _ = event_log_channel_id.send_message(&ctx.http, CreateMessage::default().embed(embed));
        }
    }

    async fn guild_member_removal(&self, ctx: Context, guild_id: GuildId, user: User, member_data_if_available: Option<Member>) {

        let config = CONFIG.read().unwrap();
        let guild_config = config.get_guild_config(guild_id.get());

        if let Some(guild_config) = guild_config {

            let event_log_channel_id = ChannelId::new(guild_config.channels.event_log);

            let embed = CreateEmbed::default()
                .title("Member left")
                .author(CreateEmbedAuthor::new(&user.name).icon_url(user.avatar_url().unwrap_or_default()))
                .footer(CreateEmbedFooter::new(format!("Joined at <t:{}:f>",  &member_data_if_available.unwrap().joined_at.unwrap().to_string())));

            let _ = event_log_channel_id.send_message(&ctx.http, CreateMessage::default().embed(embed));
        }
    }

    async fn guild_audit_log_entry_create(&self, ctx: Context, entry: AuditLogEntry, guild_id: GuildId) {

        let config = CONFIG.read().unwrap();
        let guild_config = config.get_guild_config(guild_id.get());
        let user = ctx.cache.user(entry.user_id).unwrap();

        let embed = match entry.action {
            audit_log::Action::Member(audit_log::MemberAction::Kick) => {

                let kicked_user = ctx.cache.user(entry.target_id.unwrap().get()).unwrap();

                Some (
                    CreateEmbed::default()
                    .title("Member kicked")
                    .author(CreateEmbedAuthor::new(&kicked_user.name).icon_url(kicked_user.avatar_url().unwrap_or_default()))
                    .field("Kicked by", &user.name, false)
                )
            }
            _ => { None }
        };

        if let Some(guild_config) = guild_config {

            let event_log_channel_id = ChannelId::new(guild_config.channels.event_log);

            if let Some(embed) = embed {
                let _ = event_log_channel_id.send_message(&ctx.http, CreateMessage::default().embed(embed));
            }
        }
    }
}