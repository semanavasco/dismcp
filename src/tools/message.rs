use rmcp::{
    handler::server::wrapper::Parameters,
    model::{CallToolResult, ErrorData},
    tool, tool_router,
};
use schemars::JsonSchema;
use serde::Deserialize;
use serde_json::{Map, Value, json};
use serenity::{
    http::MessagePagination,
    model::{
        channel::ReactionType,
        id::{ChannelId, MessageId, UserId},
    },
};

use crate::{
    server::{Server, structured},
    tools::{ChannelIdParams, ChannelMessageIdParams, parse_id_list, parse_snowflake},
};

#[derive(Debug, Deserialize, JsonSchema)]
struct GetMessagesParams {
    #[schemars(description = "Channel ID (snowflake, as string).")]
    channel_id: String,
    #[schemars(
        description = "Maximum number of messages to fetch (1-100). Defaults to 50.",
        range(min = 1, max = 100)
    )]
    limit: Option<u64>,
    #[schemars(description = "Fetch messages before this message ID (snowflake, as string).")]
    before: Option<String>,
    #[schemars(description = "Fetch messages after this message ID (snowflake, as string).")]
    after: Option<String>,
    #[schemars(description = "Fetch messages around this message ID (snowflake, as string).")]
    around: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct SendMessageParams {
    #[schemars(description = "Channel ID (snowflake, as string). Can be a DM channel ID.")]
    channel_id: String,
    #[schemars(description = "Message content. Optional when sending embeds/components only.")]
    content: Option<String>,
    #[schemars(description = "Whether the message is text-to-speech. Defaults to false.")]
    tts: Option<bool>,
    #[schemars(description = "Embeds as raw Discord JSON array.")]
    embeds: Option<Value>,
    #[schemars(description = "Components as raw Discord JSON array.")]
    components: Option<Value>,
    #[schemars(description = "Allowed mention settings.")]
    allowed_mentions: Option<AllowedMentionsParams>,
    #[schemars(description = "Message reference for replies.")]
    message_reference: Option<MessageReferenceParams>,
    #[schemars(description = "Discord message flags bitfield.")]
    flags: Option<u64>,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct EditMessageParams {
    #[schemars(description = "Channel ID (snowflake, as string). Can be a DM channel ID.")]
    channel_id: String,
    #[schemars(description = "Message ID (snowflake, as string).")]
    message_id: String,
    #[schemars(description = "New message content. Optional when editing embeds/components only.")]
    content: Option<String>,
    #[schemars(description = "Embeds as raw Discord JSON array.")]
    embeds: Option<Value>,
    #[schemars(description = "Components as raw Discord JSON array.")]
    components: Option<Value>,
    #[schemars(description = "Allowed mention settings.")]
    allowed_mentions: Option<AllowedMentionsParams>,
    #[schemars(description = "Message reference for replies.")]
    message_reference: Option<MessageReferenceParams>,
    #[schemars(description = "Discord message flags bitfield.")]
    flags: Option<u64>,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct ReactionParams {
    #[schemars(description = "Channel ID (snowflake, as string).")]
    channel_id: String,
    #[schemars(description = "Message ID (snowflake, as string).")]
    message_id: String,
    #[schemars(
        description = "Emoji reaction. Supports unicode (👍) and custom formats (<:name:id> or name:id)."
    )]
    emoji: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct RemoveUserReactionParams {
    #[schemars(description = "Channel ID (snowflake, as string).")]
    channel_id: String,
    #[schemars(description = "Message ID (snowflake, as string).")]
    message_id: String,
    #[schemars(description = "User ID whose reaction should be removed (snowflake, as string).")]
    user_id: String,
    #[schemars(
        description = "Emoji reaction. Supports unicode (👍) and custom formats (<:name:id> or name:id)."
    )]
    emoji: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct GetReactionUsersParams {
    #[schemars(description = "Channel ID (snowflake, as string).")]
    channel_id: String,
    #[schemars(description = "Message ID (snowflake, as string).")]
    message_id: String,
    #[schemars(
        description = "Emoji reaction. Supports unicode (👍) and custom formats (<:name:id> or name:id)."
    )]
    emoji: String,
    #[schemars(
        description = "Max users to fetch (1-100). Defaults to 25.",
        range(min = 1, max = 100)
    )]
    limit: Option<u64>,
    #[schemars(description = "Fetch users after this user ID (snowflake, as string).")]
    after: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct AllowedMentionsParams {
    #[schemars(description = "Allowed mention categories (roles, users, everyone).")]
    parse: Option<Vec<String>>,
    #[schemars(description = "Role IDs explicitly allowed to mention (snowflakes, as strings).")]
    roles: Option<Vec<String>>,
    #[schemars(description = "User IDs explicitly allowed to mention (snowflakes, as strings).")]
    users: Option<Vec<String>>,
    #[schemars(description = "Whether a reply should ping the referenced author.")]
    replied_user: Option<bool>,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct MessageReferenceParams {
    #[schemars(description = "Referenced message ID (snowflake, as string).")]
    message_id: String,
    #[schemars(description = "Referenced channel ID (snowflake, as string).")]
    channel_id: Option<String>,
    #[schemars(description = "Referenced guild ID (snowflake, as string).")]
    guild_id: Option<String>,
    #[schemars(description = "Whether send should fail if referenced message does not exist.")]
    fail_if_not_exists: Option<bool>,
}

fn parse_reaction(param_name: &'static str, value: &str) -> Result<ReactionType, ErrorData> {
    ReactionType::try_from(value).map_err(|_| {
        ErrorData::invalid_params(
            format!(
                "Parameter '{param_name}' must be a valid emoji (unicode or custom emoji format)."
            ),
            None,
        )
    })
}

fn merge_array(
    target: &mut Map<String, Value>,
    key: &'static str,
    value: Option<Value>,
) -> Result<(), ErrorData> {
    if let Some(value) = value {
        let Value::Array(_) = value else {
            return Err(ErrorData::invalid_params(
                format!("Parameter '{key}' must be a JSON array."),
                None,
            ));
        };
        target.insert(key.to_string(), value);
    }

    Ok(())
}

fn add_allowed_mentions(
    target: &mut Map<String, Value>,
    allowed_mentions: Option<AllowedMentionsParams>,
) -> Result<(), ErrorData> {
    if let Some(allowed_mentions) = allowed_mentions {
        let mut map = Map::new();

        if let Some(parse) = allowed_mentions.parse {
            map.insert(
                "parse".to_string(),
                Value::Array(parse.into_iter().map(Value::String).collect()),
            );
        }
        if let Some(roles) = allowed_mentions.roles {
            map.insert(
                "roles".to_string(),
                Value::Array(
                    parse_id_list("allowed_mentions.roles[]", roles)?
                        .into_iter()
                        .map(|id| Value::Number(id.into()))
                        .collect(),
                ),
            );
        }
        if let Some(users) = allowed_mentions.users {
            map.insert(
                "users".to_string(),
                Value::Array(
                    parse_id_list("allowed_mentions.users[]", users)?
                        .into_iter()
                        .map(|id| Value::Number(id.into()))
                        .collect(),
                ),
            );
        }
        if let Some(replied_user) = allowed_mentions.replied_user {
            map.insert("replied_user".to_string(), Value::Bool(replied_user));
        }

        target.insert("allowed_mentions".to_string(), Value::Object(map));
    }

    Ok(())
}

fn add_message_reference(
    target: &mut Map<String, Value>,
    message_reference: Option<MessageReferenceParams>,
) -> Result<(), ErrorData> {
    if let Some(message_reference) = message_reference {
        let mut map = Map::new();
        map.insert(
            "message_id".to_string(),
            Value::Number(
                parse_snowflake(
                    "message_reference.message_id",
                    &message_reference.message_id,
                )?
                .into(),
            ),
        );
        if let Some(channel_id) = message_reference.channel_id {
            map.insert(
                "channel_id".to_string(),
                Value::Number(parse_snowflake("message_reference.channel_id", &channel_id)?.into()),
            );
        }
        if let Some(guild_id) = message_reference.guild_id {
            map.insert(
                "guild_id".to_string(),
                Value::Number(parse_snowflake("message_reference.guild_id", &guild_id)?.into()),
            );
        }
        if let Some(fail_if_not_exists) = message_reference.fail_if_not_exists {
            map.insert(
                "fail_if_not_exists".to_string(),
                Value::Bool(fail_if_not_exists),
            );
        }

        target.insert("message_reference".to_string(), Value::Object(map));
    }

    Ok(())
}

fn ensure_message_payload_has_fields(
    content: &Option<String>,
    embeds: &Option<Value>,
    components: &Option<Value>,
    allowed_mentions: &Option<AllowedMentionsParams>,
    message_reference: &Option<MessageReferenceParams>,
    flags: &Option<u64>,
) -> Result<(), ErrorData> {
    if let Some(content) = content
        && content.len() > 2000
    {
        return Err(ErrorData::invalid_params(
            "Parameter 'content' must be 2000 characters or fewer.",
            None,
        ));
    }

    if content.is_none()
        && embeds.is_none()
        && components.is_none()
        && allowed_mentions.is_none()
        && message_reference.is_none()
        && flags.is_none()
    {
        return Err(ErrorData::invalid_params(
            "Provide at least one message field (content, embeds, components, allowed_mentions, message_reference, or flags).",
            None,
        ));
    }

    Ok(())
}

#[tool_router(router = message_router, vis = "pub(crate)")]
impl Server {
    #[tool(description = "Get a message by channel and message IDs.")]
    async fn get_message(
        &self,
        Parameters(ChannelMessageIdParams {
            channel_id,
            message_id,
        }): Parameters<ChannelMessageIdParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let channel_id = ChannelId::new(parse_snowflake("channel_id", &channel_id)?);
        let message_id = MessageId::new(parse_snowflake("message_id", &message_id)?);

        structured(
            self.bot_http()
                .get_message(channel_id, message_id)
                .await
                .map_err(|error| {
                    ErrorData::internal_error(format!("Failed to fetch message: {error}"), None)
                })?,
        )
    }

    #[tool(description = "List messages in a channel with optional pagination.")]
    async fn get_messages(
        &self,
        Parameters(GetMessagesParams {
            channel_id,
            limit,
            before,
            after,
            around,
        }): Parameters<GetMessagesParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let channel_id = ChannelId::new(parse_snowflake("channel_id", &channel_id)?);
        let limit = limit.unwrap_or(50);

        if !(1..=100).contains(&limit) {
            return Err(ErrorData::invalid_params(
                "Parameter 'limit' must be between 1 and 100.",
                None,
            ));
        }

        let target_count = before.is_some() as u8 + after.is_some() as u8 + around.is_some() as u8;
        if target_count > 1 {
            return Err(ErrorData::invalid_params(
                "Use only one of 'before', 'after', or 'around'.",
                None,
            ));
        }

        let target = match (before, after, around) {
            (Some(value), None, None) => Some(MessagePagination::Before(MessageId::new(
                parse_snowflake("before", &value)?,
            ))),
            (None, Some(value), None) => Some(MessagePagination::After(MessageId::new(
                parse_snowflake("after", &value)?,
            ))),
            (None, None, Some(value)) => Some(MessagePagination::Around(MessageId::new(
                parse_snowflake("around", &value)?,
            ))),
            (None, None, None) => None,
            _ => unreachable!(),
        };

        structured(
            self.bot_http()
                .get_messages(channel_id, target, Some(limit as u8))
                .await
                .map_err(|error| {
                    ErrorData::internal_error(format!("Failed to fetch messages: {error}"), None)
                })?,
        )
    }

    #[tool(description = "Send a message to a channel or DM channel.")]
    async fn send_message(
        &self,
        Parameters(SendMessageParams {
            channel_id,
            content,
            tts,
            embeds,
            components,
            allowed_mentions,
            message_reference,
            flags,
        }): Parameters<SendMessageParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let channel_id = ChannelId::new(parse_snowflake("channel_id", &channel_id)?);
        ensure_message_payload_has_fields(
            &content,
            &embeds,
            &components,
            &allowed_mentions,
            &message_reference,
            &flags,
        )?;

        let mut body = Map::new();
        if let Some(content) = content {
            body.insert("content".to_string(), Value::String(content));
        }
        if let Some(tts) = tts {
            body.insert("tts".to_string(), Value::Bool(tts));
        }
        if let Some(flags) = flags {
            body.insert("flags".to_string(), Value::Number(flags.into()));
        }
        merge_array(&mut body, "embeds", embeds)?;
        merge_array(&mut body, "components", components)?;
        add_allowed_mentions(&mut body, allowed_mentions)?;
        add_message_reference(&mut body, message_reference)?;

        structured(
            self.bot_http()
                .send_message(channel_id, vec![], &Value::Object(body))
                .await
                .map_err(|error| {
                    ErrorData::internal_error(format!("Failed to send message: {error}"), None)
                })?,
        )
    }

    #[tool(description = "Edit an existing message in a channel or DM channel.")]
    async fn edit_message(
        &self,
        Parameters(EditMessageParams {
            channel_id,
            message_id,
            content,
            embeds,
            components,
            allowed_mentions,
            message_reference,
            flags,
        }): Parameters<EditMessageParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let channel_id = ChannelId::new(parse_snowflake("channel_id", &channel_id)?);
        let message_id = MessageId::new(parse_snowflake("message_id", &message_id)?);
        ensure_message_payload_has_fields(
            &content,
            &embeds,
            &components,
            &allowed_mentions,
            &message_reference,
            &flags,
        )?;

        let mut body = Map::new();
        if let Some(content) = content {
            body.insert("content".to_string(), Value::String(content));
        }
        if let Some(flags) = flags {
            body.insert("flags".to_string(), Value::Number(flags.into()));
        }
        merge_array(&mut body, "embeds", embeds)?;
        merge_array(&mut body, "components", components)?;
        add_allowed_mentions(&mut body, allowed_mentions)?;
        add_message_reference(&mut body, message_reference)?;

        structured(
            self.bot_http()
                .edit_message(channel_id, message_id, &Value::Object(body), vec![])
                .await
                .map_err(|error| {
                    ErrorData::internal_error(format!("Failed to edit message: {error}"), None)
                })?,
        )
    }

    #[tool(description = "Delete a message in a channel or DM channel.")]
    async fn delete_message(
        &self,
        Parameters(ChannelMessageIdParams {
            channel_id,
            message_id,
        }): Parameters<ChannelMessageIdParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let channel_id = ChannelId::new(parse_snowflake("channel_id", &channel_id)?);
        let message_id = MessageId::new(parse_snowflake("message_id", &message_id)?);

        self.bot_http()
            .delete_message(channel_id, message_id, None)
            .await
            .map_err(|error| {
                ErrorData::internal_error(format!("Failed to delete message: {error}"), None)
            })?;

        structured(json!({
            "deleted": true,
            "channel_id": channel_id,
            "message_id": message_id,
        }))
    }

    #[tool(description = "List pinned messages in a channel.")]
    async fn get_pinned_messages(
        &self,
        Parameters(ChannelIdParams { channel_id }): Parameters<ChannelIdParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let channel_id = ChannelId::new(parse_snowflake("channel_id", &channel_id)?);

        structured(
            self.bot_http()
                .get_pins(channel_id)
                .await
                .map_err(|error| {
                    ErrorData::internal_error(
                        format!("Failed to fetch pinned messages: {error}"),
                        None,
                    )
                })?,
        )
    }

    #[tool(description = "Pin a message in a channel.")]
    async fn pin_message(
        &self,
        Parameters(ChannelMessageIdParams {
            channel_id,
            message_id,
        }): Parameters<ChannelMessageIdParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let channel_id = ChannelId::new(parse_snowflake("channel_id", &channel_id)?);
        let message_id = MessageId::new(parse_snowflake("message_id", &message_id)?);

        self.bot_http()
            .pin_message(channel_id, message_id, None)
            .await
            .map_err(|error| {
                ErrorData::internal_error(format!("Failed to pin message: {error}"), None)
            })?;

        structured(json!({
            "updated": true,
            "action": "pin",
            "channel_id": channel_id,
            "message_id": message_id,
        }))
    }

    #[tool(description = "Unpin a message in a channel.")]
    async fn unpin_message(
        &self,
        Parameters(ChannelMessageIdParams {
            channel_id,
            message_id,
        }): Parameters<ChannelMessageIdParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let channel_id = ChannelId::new(parse_snowflake("channel_id", &channel_id)?);
        let message_id = MessageId::new(parse_snowflake("message_id", &message_id)?);

        self.bot_http()
            .unpin_message(channel_id, message_id, None)
            .await
            .map_err(|error| {
                ErrorData::internal_error(format!("Failed to unpin message: {error}"), None)
            })?;

        structured(json!({
            "updated": true,
            "action": "unpin",
            "channel_id": channel_id,
            "message_id": message_id,
        }))
    }

    #[tool(description = "Add a reaction to a message.")]
    async fn add_message_reaction(
        &self,
        Parameters(ReactionParams {
            channel_id,
            message_id,
            emoji,
        }): Parameters<ReactionParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let channel_id = ChannelId::new(parse_snowflake("channel_id", &channel_id)?);
        let message_id = MessageId::new(parse_snowflake("message_id", &message_id)?);
        let emoji = parse_reaction("emoji", &emoji)?;

        self.bot_http()
            .create_reaction(channel_id, message_id, &emoji)
            .await
            .map_err(|error| {
                ErrorData::internal_error(format!("Failed to add reaction: {error}"), None)
            })?;

        structured(json!({
            "updated": true,
            "action": "add_reaction",
            "channel_id": channel_id,
            "message_id": message_id,
            "emoji": emoji.as_data(),
        }))
    }

    #[tool(description = "Remove the bot's reaction from a message.")]
    async fn remove_own_message_reaction(
        &self,
        Parameters(ReactionParams {
            channel_id,
            message_id,
            emoji,
        }): Parameters<ReactionParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let channel_id = ChannelId::new(parse_snowflake("channel_id", &channel_id)?);
        let message_id = MessageId::new(parse_snowflake("message_id", &message_id)?);
        let emoji = parse_reaction("emoji", &emoji)?;

        self.bot_http()
            .delete_reaction_me(channel_id, message_id, &emoji)
            .await
            .map_err(|error| {
                ErrorData::internal_error(format!("Failed to remove own reaction: {error}"), None)
            })?;

        structured(json!({
            "updated": true,
            "action": "remove_own_reaction",
            "channel_id": channel_id,
            "message_id": message_id,
            "emoji": emoji.as_data(),
        }))
    }

    #[tool(description = "Remove another user's reaction from a message.")]
    async fn remove_user_message_reaction(
        &self,
        Parameters(RemoveUserReactionParams {
            channel_id,
            message_id,
            user_id,
            emoji,
        }): Parameters<RemoveUserReactionParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let channel_id = ChannelId::new(parse_snowflake("channel_id", &channel_id)?);
        let message_id = MessageId::new(parse_snowflake("message_id", &message_id)?);
        let user_id = UserId::new(parse_snowflake("user_id", &user_id)?);
        let emoji = parse_reaction("emoji", &emoji)?;

        self.bot_http()
            .delete_reaction(channel_id, message_id, user_id, &emoji)
            .await
            .map_err(|error| {
                ErrorData::internal_error(format!("Failed to remove user reaction: {error}"), None)
            })?;

        structured(json!({
            "updated": true,
            "action": "remove_user_reaction",
            "channel_id": channel_id,
            "message_id": message_id,
            "user_id": user_id,
            "emoji": emoji.as_data(),
        }))
    }

    #[tool(description = "Clear all reactions from a message.")]
    async fn clear_message_reactions(
        &self,
        Parameters(ChannelMessageIdParams {
            channel_id,
            message_id,
        }): Parameters<ChannelMessageIdParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let channel_id = ChannelId::new(parse_snowflake("channel_id", &channel_id)?);
        let message_id = MessageId::new(parse_snowflake("message_id", &message_id)?);

        self.bot_http()
            .delete_message_reactions(channel_id, message_id)
            .await
            .map_err(|error| {
                ErrorData::internal_error(
                    format!("Failed to clear message reactions: {error}"),
                    None,
                )
            })?;

        structured(json!({
            "updated": true,
            "action": "clear_reactions",
            "channel_id": channel_id,
            "message_id": message_id,
        }))
    }

    #[tool(description = "Clear one emoji's reactions from a message.")]
    async fn clear_message_emoji_reactions(
        &self,
        Parameters(ReactionParams {
            channel_id,
            message_id,
            emoji,
        }): Parameters<ReactionParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let channel_id = ChannelId::new(parse_snowflake("channel_id", &channel_id)?);
        let message_id = MessageId::new(parse_snowflake("message_id", &message_id)?);
        let emoji = parse_reaction("emoji", &emoji)?;

        self.bot_http()
            .delete_message_reaction_emoji(channel_id, message_id, &emoji)
            .await
            .map_err(|error| {
                ErrorData::internal_error(
                    format!("Failed to clear emoji reactions on message: {error}"),
                    None,
                )
            })?;

        structured(json!({
            "updated": true,
            "action": "clear_emoji_reactions",
            "channel_id": channel_id,
            "message_id": message_id,
            "emoji": emoji.as_data(),
        }))
    }

    #[tool(description = "List users who reacted to a message with a specific emoji.")]
    async fn get_message_reaction_users(
        &self,
        Parameters(GetReactionUsersParams {
            channel_id,
            message_id,
            emoji,
            limit,
            after,
        }): Parameters<GetReactionUsersParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let channel_id = ChannelId::new(parse_snowflake("channel_id", &channel_id)?);
        let message_id = MessageId::new(parse_snowflake("message_id", &message_id)?);
        let emoji = parse_reaction("emoji", &emoji)?;
        let limit = limit.unwrap_or(25);

        if !(1..=100).contains(&limit) {
            return Err(ErrorData::invalid_params(
                "Parameter 'limit' must be between 1 and 100.",
                None,
            ));
        }

        let after = after
            .as_deref()
            .map(|value| parse_snowflake("after", value))
            .transpose()?;

        structured(
            self.bot_http()
                .get_reaction_users(channel_id, message_id, &emoji, limit as u8, after)
                .await
                .map_err(|error| {
                    ErrorData::internal_error(
                        format!("Failed to fetch reaction users: {error}"),
                        None,
                    )
                })?,
        )
    }
}
