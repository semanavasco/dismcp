use rmcp::{
    handler::server::wrapper::Parameters,
    model::{CallToolResult, ErrorData},
    tool, tool_router,
};
use schemars::JsonSchema;
use serde::Deserialize;
use serde_json::{Map, Value, json};
use serenity::model::id::{ChannelId, GuildId, UserId};

use crate::{
    server::{Server, structured},
    tools::{
        ChannelIdParams, GuildIdParams, UserIdParams, matches_query, normalize_query,
        parse_id_list, parse_snowflake,
    },
};

#[derive(Debug, Deserialize, JsonSchema)]
struct CreateGuildChannelParams {
    #[schemars(description = "Guild ID (snowflake, as string).")]
    guild_id: String,
    #[schemars(description = "Channel name.")]
    name: String,
    #[schemars(
        description = "Discord channel type number (e.g., 0 text, 2 voice, 4 category, 13 stage, 15 forum)."
    )]
    channel_type: Option<u8>,
    #[schemars(description = "Parent category ID (snowflake, as string).")]
    parent_id: Option<String>,
    #[schemars(description = "Channel topic.")]
    topic: Option<String>,
    #[schemars(description = "Whether the channel is NSFW.")]
    nsfw: Option<bool>,
    #[schemars(description = "Voice bitrate, for voice/stage channels.")]
    bitrate: Option<u64>,
    #[schemars(description = "Voice user limit, for voice/stage channels.")]
    user_limit: Option<u64>,
    #[schemars(description = "Slowmode in seconds.")]
    rate_limit_per_user: Option<u64>,
    #[schemars(description = "Channel position in the guild.")]
    position: Option<u64>,
    #[schemars(description = "Forum tags available in this channel.")]
    available_tags: Option<Vec<ForumTagParams>>,
    #[schemars(description = "Default reaction emoji for forum posts.")]
    default_reaction_emoji: Option<DefaultReactionEmojiParams>,
    #[schemars(description = "Default auto-archive duration (60, 1440, 4320, 10080).")]
    default_auto_archive_duration: Option<u64>,
    #[schemars(description = "Default slowmode for newly created threads (0-21600).")]
    default_thread_rate_limit_per_user: Option<u64>,
    #[schemars(
        description = "Default sort order for forum posts (0 latest activity, 1 creation date)."
    )]
    default_sort_order: Option<u8>,
    #[schemars(description = "Default forum layout (0 not set, 1 list view, 2 gallery view).")]
    default_forum_layout: Option<u8>,
    #[schemars(description = "Channel permission overwrites.")]
    permission_overwrites: Option<Vec<PermissionOverwriteParams>>,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct EditChannelParams {
    #[schemars(description = "Channel ID (snowflake, as string).")]
    channel_id: String,
    #[schemars(description = "New channel name.")]
    name: Option<String>,
    #[schemars(description = "New channel topic.")]
    topic: Option<String>,
    #[schemars(description = "Whether the channel is NSFW.")]
    nsfw: Option<bool>,
    #[schemars(description = "Parent category ID (snowflake, as string).")]
    parent_id: Option<String>,
    #[schemars(description = "Voice bitrate, for voice/stage channels.")]
    bitrate: Option<u64>,
    #[schemars(description = "Voice user limit, for voice/stage channels.")]
    user_limit: Option<u64>,
    #[schemars(description = "Slowmode in seconds.")]
    rate_limit_per_user: Option<u64>,
    #[schemars(description = "Channel position in the guild.")]
    position: Option<u64>,
    #[schemars(description = "Forum tags available in this channel.")]
    available_tags: Option<Vec<ForumTagParams>>,
    #[schemars(description = "Default reaction emoji for forum posts.")]
    default_reaction_emoji: Option<DefaultReactionEmojiParams>,
    #[schemars(description = "Default auto-archive duration (60, 1440, 4320, 10080).")]
    default_auto_archive_duration: Option<u64>,
    #[schemars(description = "Default slowmode for newly created threads (0-21600).")]
    default_thread_rate_limit_per_user: Option<u64>,
    #[schemars(
        description = "Default sort order for forum posts (0 latest activity, 1 creation date)."
    )]
    default_sort_order: Option<u8>,
    #[schemars(description = "Default forum layout (0 not set, 1 list view, 2 gallery view).")]
    default_forum_layout: Option<u8>,
    #[schemars(description = "Channel permission overwrites.")]
    permission_overwrites: Option<Vec<PermissionOverwriteParams>>,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct ForumTagParams {
    #[schemars(
        description = "Optional forum tag ID when editing existing tags (snowflake, as string)."
    )]
    id: Option<String>,
    #[schemars(description = "Forum tag name.")]
    name: String,
    #[schemars(description = "Whether this tag can only be added by moderators.")]
    moderated: Option<bool>,
    #[schemars(description = "Emoji ID for this tag (snowflake, as string).")]
    emoji_id: Option<String>,
    #[schemars(description = "Unicode emoji name for this tag.")]
    emoji_name: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct DefaultReactionEmojiParams {
    #[schemars(description = "Emoji ID (snowflake, as string).")]
    emoji_id: Option<String>,
    #[schemars(description = "Unicode emoji name.")]
    emoji_name: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct PermissionOverwriteParams {
    #[schemars(description = "Overwrite target ID (snowflake, as string).")]
    id: String,
    #[schemars(description = "Overwrite type: 0 role, 1 member.")]
    kind: u8,
    #[schemars(description = "Allowed permissions bitset as a stringified integer.")]
    allow: String,
    #[schemars(description = "Denied permissions bitset as a stringified integer.")]
    deny: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct CreateForumPostParams {
    #[schemars(description = "Forum channel ID (snowflake, as string).")]
    forum_channel_id: String,
    #[schemars(description = "Post title.")]
    title: String,
    #[schemars(description = "Post body text.")]
    content: String,
    #[schemars(description = "Applied forum tag IDs (snowflakes, as strings).")]
    applied_tag_ids: Option<Vec<String>>,
    #[schemars(description = "Auto archive duration in minutes.")]
    auto_archive_duration: Option<u64>,
    #[schemars(description = "Slowmode in seconds for the post thread.")]
    rate_limit_per_user: Option<u64>,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct EditForumPostTagsParams {
    #[schemars(description = "Forum post channel/thread ID (snowflake, as string).")]
    post_channel_id: String,
    #[schemars(description = "Applied forum tag IDs (snowflakes, as strings).")]
    applied_tag_ids: Vec<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct ArchivedThreadsParams {
    #[schemars(description = "Channel ID (snowflake, as string).")]
    channel_id: String,
    #[schemars(description = "Optional thread ID cursor (numeric).")]
    before: Option<u64>,
    #[schemars(description = "Maximum number of threads to fetch.")]
    limit: Option<u64>,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct SearchGuildChannelsParams {
    #[schemars(description = "Guild ID (snowflake, as string).")]
    guild_id: String,
    #[schemars(description = "Channel name search query.")]
    query: String,
    #[schemars(description = "Exact name match instead of contains. Defaults to false.")]
    exact: Option<bool>,
    #[schemars(
        description = "Discord channel type number to filter by (e.g., 0 text, 2 voice, 15 forum)."
    )]
    channel_type: Option<u8>,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct CreateChannelInviteParams {
    #[schemars(description = "Channel ID (snowflake, as string).")]
    channel_id: String,
    #[schemars(description = "How long the invite lasts in seconds. 0 means never expires.")]
    max_age: Option<u64>,
    #[schemars(description = "How many uses before invite expires. 0 means unlimited.")]
    max_uses: Option<u64>,
    #[schemars(description = "Whether this invite only grants temporary membership.")]
    temporary: Option<bool>,
    #[schemars(description = "Whether this should be a unique invite code.")]
    unique: Option<bool>,
    #[schemars(description = "Invite target type: 1 stream, 2 embedded application.")]
    target_type: Option<u8>,
    #[schemars(description = "Target user ID for stream invites (snowflake, as string).")]
    target_user_id: Option<String>,
    #[schemars(
        description = "Target application ID for embedded application invites (snowflake, as string)."
    )]
    target_application_id: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct InviteCodeParam {
    #[schemars(description = "Invite code (not full URL).")]
    code: String,
}

fn add_default_reaction_emoji(
    body: &mut Map<String, Value>,
    value: Option<DefaultReactionEmojiParams>,
) -> Result<(), ErrorData> {
    if let Some(value) = value {
        if value.emoji_id.is_none() && value.emoji_name.is_none() {
            return Err(ErrorData::invalid_params(
                "Parameter 'default_reaction_emoji' must include 'emoji_id' or 'emoji_name'.",
                None,
            ));
        }

        let mut emoji = Map::new();
        if let Some(emoji_id) = value.emoji_id {
            emoji.insert(
                "emoji_id".to_string(),
                Value::Number(
                    parse_snowflake("default_reaction_emoji.emoji_id", &emoji_id)?.into(),
                ),
            );
        }
        if let Some(emoji_name) = value.emoji_name {
            emoji.insert("emoji_name".to_string(), Value::String(emoji_name));
        }
        body.insert("default_reaction_emoji".to_string(), Value::Object(emoji));
    }

    Ok(())
}

fn add_available_tags(
    body: &mut Map<String, Value>,
    value: Option<Vec<ForumTagParams>>,
) -> Result<(), ErrorData> {
    if let Some(tags) = value {
        let mut out = Vec::with_capacity(tags.len());
        for tag in tags {
            if tag.name.trim().is_empty() || tag.name.len() > 20 {
                return Err(ErrorData::invalid_params(
                    "Each forum tag name must be between 1 and 20 characters.",
                    None,
                ));
            }
            let mut map = Map::new();
            if let Some(id) = tag.id {
                map.insert(
                    "id".to_string(),
                    Value::Number(parse_snowflake("available_tags[].id", &id)?.into()),
                );
            }
            map.insert("name".to_string(), Value::String(tag.name));
            if let Some(moderated) = tag.moderated {
                map.insert("moderated".to_string(), Value::Bool(moderated));
            }
            if let Some(emoji_id) = tag.emoji_id {
                map.insert(
                    "emoji_id".to_string(),
                    Value::Number(parse_snowflake("available_tags[].emoji_id", &emoji_id)?.into()),
                );
            }
            if let Some(emoji_name) = tag.emoji_name {
                map.insert("emoji_name".to_string(), Value::String(emoji_name));
            }
            out.push(Value::Object(map));
        }
        body.insert("available_tags".to_string(), Value::Array(out));
    }
    Ok(())
}

fn add_permission_overwrites(
    body: &mut Map<String, Value>,
    value: Option<Vec<PermissionOverwriteParams>>,
) -> Result<(), ErrorData> {
    if let Some(overwrites) = value {
        let mut out = Vec::with_capacity(overwrites.len());
        for overwrite in overwrites {
            if overwrite.kind > 1 {
                return Err(ErrorData::invalid_params(
                    "Each permission overwrite 'kind' must be 0 (role) or 1 (member).",
                    None,
                ));
            }

            let allow = overwrite.allow.parse::<u64>().map_err(|_| {
                ErrorData::invalid_params(
                    "Each permission overwrite 'allow' must be a stringified unsigned integer.",
                    None,
                )
            })?;
            let deny = overwrite.deny.parse::<u64>().map_err(|_| {
                ErrorData::invalid_params(
                    "Each permission overwrite 'deny' must be a stringified unsigned integer.",
                    None,
                )
            })?;

            let mut map = Map::new();
            map.insert(
                "id".to_string(),
                Value::Number(parse_snowflake("permission_overwrites[].id", &overwrite.id)?.into()),
            );
            map.insert("type".to_string(), Value::Number(overwrite.kind.into()));
            map.insert("allow".to_string(), Value::String(allow.to_string()));
            map.insert("deny".to_string(), Value::String(deny.to_string()));
            out.push(Value::Object(map));
        }
        body.insert("permission_overwrites".to_string(), Value::Array(out));
    }
    Ok(())
}

fn validate_channel_type(value: u8) -> Result<(), ErrorData> {
    const KNOWN_CHANNEL_TYPES: [u8; 12] = [0, 1, 2, 3, 4, 5, 10, 11, 12, 13, 15, 16];
    if KNOWN_CHANNEL_TYPES.contains(&value) {
        Ok(())
    } else {
        Err(ErrorData::invalid_params(
            format!("Parameter 'channel_type' has unsupported value '{value}'."),
            None,
        ))
    }
}

fn validate_rate_limit_per_user(value: u64, param_name: &'static str) -> Result<(), ErrorData> {
    if value > 21_600 {
        return Err(ErrorData::invalid_params(
            format!("Parameter '{param_name}' must be between 0 and 21600."),
            None,
        ));
    }
    Ok(())
}

fn validate_auto_archive_duration(value: u64) -> Result<(), ErrorData> {
    const ALLOWED: [u64; 4] = [60, 1_440, 4_320, 10_080];
    if ALLOWED.contains(&value) {
        Ok(())
    } else {
        Err(ErrorData::invalid_params(
            "Parameter 'auto_archive_duration' must be one of 60, 1440, 4320, or 10080.",
            None,
        ))
    }
}

#[tool_router(router = channel_router, vis = "pub(crate)")]
impl Server {
    #[tool(description = "Get details for a specific channel.")]
    async fn get_channel(
        &self,
        Parameters(ChannelIdParams { channel_id }): Parameters<ChannelIdParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let channel_id = ChannelId::new(parse_snowflake("channel_id", &channel_id)?);

        structured(
            self.bot_http()
                .get_channel(channel_id)
                .await
                .map_err(|error| {
                    ErrorData::internal_error(format!("Failed to fetch channel: {error}"), None)
                })?,
        )
    }

    #[tool(description = "List DM channels for the authenticated user.")]
    async fn get_dm_channels(&self) -> Result<CallToolResult, ErrorData> {
        structured(
            self.bot_http()
                .get_user_dm_channels()
                .await
                .map_err(|error| {
                    ErrorData::internal_error(format!("Failed to fetch DM channels: {error}"), None)
                })?,
        )
    }

    #[tool(description = "Create or get a DM channel with a specific user.")]
    async fn create_dm_channel(
        &self,
        Parameters(UserIdParams { user_id }): Parameters<UserIdParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let user_id = UserId::new(parse_snowflake("user_id", &user_id)?);
        let body = json!({
            "recipient_id": user_id,
        });

        structured(
            self.bot_http()
                .create_private_channel(&body)
                .await
                .map_err(|error| {
                    ErrorData::internal_error(format!("Failed to create DM channel: {error}"), None)
                })?,
        )
    }

    #[tool(description = "List channels in a specific guild.")]
    async fn get_guild_channels(
        &self,
        Parameters(GuildIdParams { guild_id }): Parameters<GuildIdParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let guild_id = GuildId::new(parse_snowflake("guild_id", &guild_id)?);

        structured(
            self.bot_http()
                .get_channels(guild_id)
                .await
                .map_err(|error| {
                    ErrorData::internal_error(
                        format!("Failed to fetch guild channels: {error}"),
                        None,
                    )
                })?,
        )
    }

    #[tool(description = "Search guild channels by name, with optional channel type filter.")]
    async fn search_guild_channels(
        &self,
        Parameters(SearchGuildChannelsParams {
            guild_id,
            query,
            exact,
            channel_type,
        }): Parameters<SearchGuildChannelsParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let guild_id = GuildId::new(parse_snowflake("guild_id", &guild_id)?);
        let exact = exact.unwrap_or(false);
        let query = normalize_query(&query);

        if let Some(channel_type) = channel_type {
            validate_channel_type(channel_type)?;
        }

        if query.is_empty() {
            return Err(ErrorData::invalid_params(
                "Parameter 'query' cannot be empty.",
                None,
            ));
        }

        let channels = self
            .bot_http()
            .get_channels(guild_id)
            .await
            .map_err(|error| {
                ErrorData::internal_error(format!("Failed to fetch guild channels: {error}"), None)
            })?;

        let filtered: Vec<_> = channels
            .into_iter()
            .filter(|channel| {
                let name = normalize_query(&channel.name);
                let name_matches = matches_query(&name, &query, exact);
                let type_matches =
                    channel_type.is_none_or(|type_id| u8::from(channel.kind) == type_id);

                name_matches && type_matches
            })
            .collect();

        structured(filtered)
    }

    #[tool(description = "Create a guild channel (text/voice/forum/category/stage etc).")]
    async fn create_guild_channel(
        &self,
        Parameters(CreateGuildChannelParams {
            guild_id,
            name,
            channel_type,
            parent_id,
            topic,
            nsfw,
            bitrate,
            user_limit,
            rate_limit_per_user,
            position,
            available_tags,
            default_reaction_emoji,
            default_auto_archive_duration,
            default_thread_rate_limit_per_user,
            default_sort_order,
            default_forum_layout,
            permission_overwrites,
        }): Parameters<CreateGuildChannelParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let guild_id = GuildId::new(parse_snowflake("guild_id", &guild_id)?);
        let mut body = Map::new();
        body.insert("name".to_string(), Value::String(name));

        if let Some(channel_type) = channel_type {
            validate_channel_type(channel_type)?;
            body.insert("type".to_string(), Value::Number(channel_type.into()));
        }
        if let Some(parent_id) = parent_id {
            body.insert(
                "parent_id".to_string(),
                Value::Number(parse_snowflake("parent_id", &parent_id)?.into()),
            );
        }
        if let Some(topic) = topic {
            body.insert("topic".to_string(), Value::String(topic));
        }
        if let Some(nsfw) = nsfw {
            body.insert("nsfw".to_string(), Value::Bool(nsfw));
        }
        if let Some(bitrate) = bitrate {
            body.insert("bitrate".to_string(), Value::Number(bitrate.into()));
        }
        if let Some(user_limit) = user_limit {
            body.insert("user_limit".to_string(), Value::Number(user_limit.into()));
        }
        if let Some(rate_limit_per_user) = rate_limit_per_user {
            validate_rate_limit_per_user(rate_limit_per_user, "rate_limit_per_user")?;
            body.insert(
                "rate_limit_per_user".to_string(),
                Value::Number(rate_limit_per_user.into()),
            );
        }
        if let Some(position) = position {
            body.insert("position".to_string(), Value::Number(position.into()));
        }
        if let Some(value) = default_auto_archive_duration {
            validate_auto_archive_duration(value)?;
            body.insert(
                "default_auto_archive_duration".to_string(),
                Value::Number(value.into()),
            );
        }
        if let Some(value) = default_thread_rate_limit_per_user {
            validate_rate_limit_per_user(value, "default_thread_rate_limit_per_user")?;
            body.insert(
                "default_thread_rate_limit_per_user".to_string(),
                Value::Number(value.into()),
            );
        }
        if let Some(value) = default_sort_order {
            if value > 1 {
                return Err(ErrorData::invalid_params(
                    "Parameter 'default_sort_order' must be 0 or 1.",
                    None,
                ));
            }
            body.insert(
                "default_sort_order".to_string(),
                Value::Number(value.into()),
            );
        }
        if let Some(value) = default_forum_layout {
            if value > 2 {
                return Err(ErrorData::invalid_params(
                    "Parameter 'default_forum_layout' must be 0, 1, or 2.",
                    None,
                ));
            }
            body.insert(
                "default_forum_layout".to_string(),
                Value::Number(value.into()),
            );
        }
        add_default_reaction_emoji(&mut body, default_reaction_emoji)?;
        add_available_tags(&mut body, available_tags)?;
        add_permission_overwrites(&mut body, permission_overwrites)?;

        structured(
            self.bot_http()
                .create_channel(guild_id, &Value::Object(body), None)
                .await
                .map_err(|error| {
                    ErrorData::internal_error(
                        format!("Failed to create guild channel: {error}"),
                        None,
                    )
                })?,
        )
    }

    #[tool(description = "Edit a guild channel (supports typed forum channel fields).")]
    async fn edit_channel(
        &self,
        Parameters(EditChannelParams {
            channel_id,
            name,
            topic,
            nsfw,
            parent_id,
            bitrate,
            user_limit,
            rate_limit_per_user,
            position,
            available_tags,
            default_reaction_emoji,
            default_auto_archive_duration,
            default_thread_rate_limit_per_user,
            default_sort_order,
            default_forum_layout,
            permission_overwrites,
        }): Parameters<EditChannelParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let channel_id = ChannelId::new(parse_snowflake("channel_id", &channel_id)?);

        if name.is_none()
            && topic.is_none()
            && nsfw.is_none()
            && parent_id.is_none()
            && bitrate.is_none()
            && user_limit.is_none()
            && rate_limit_per_user.is_none()
            && position.is_none()
            && available_tags.is_none()
            && default_reaction_emoji.is_none()
            && default_auto_archive_duration.is_none()
            && default_thread_rate_limit_per_user.is_none()
            && default_sort_order.is_none()
            && default_forum_layout.is_none()
            && permission_overwrites.is_none()
        {
            return Err(ErrorData::invalid_params(
                "Provide at least one channel field to update.",
                None,
            ));
        }

        let mut body = Map::new();

        if let Some(name) = name {
            body.insert("name".to_string(), Value::String(name));
        }
        if let Some(topic) = topic {
            body.insert("topic".to_string(), Value::String(topic));
        }
        if let Some(nsfw) = nsfw {
            body.insert("nsfw".to_string(), Value::Bool(nsfw));
        }
        if let Some(parent_id) = parent_id {
            body.insert(
                "parent_id".to_string(),
                Value::Number(parse_snowflake("parent_id", &parent_id)?.into()),
            );
        }
        if let Some(bitrate) = bitrate {
            body.insert("bitrate".to_string(), Value::Number(bitrate.into()));
        }
        if let Some(user_limit) = user_limit {
            body.insert("user_limit".to_string(), Value::Number(user_limit.into()));
        }
        if let Some(rate_limit_per_user) = rate_limit_per_user {
            validate_rate_limit_per_user(rate_limit_per_user, "rate_limit_per_user")?;
            body.insert(
                "rate_limit_per_user".to_string(),
                Value::Number(rate_limit_per_user.into()),
            );
        }
        if let Some(position) = position {
            body.insert("position".to_string(), Value::Number(position.into()));
        }
        if let Some(value) = default_auto_archive_duration {
            validate_auto_archive_duration(value)?;
            body.insert(
                "default_auto_archive_duration".to_string(),
                Value::Number(value.into()),
            );
        }
        if let Some(value) = default_thread_rate_limit_per_user {
            validate_rate_limit_per_user(value, "default_thread_rate_limit_per_user")?;
            body.insert(
                "default_thread_rate_limit_per_user".to_string(),
                Value::Number(value.into()),
            );
        }
        if let Some(value) = default_sort_order {
            if value > 1 {
                return Err(ErrorData::invalid_params(
                    "Parameter 'default_sort_order' must be 0 or 1.",
                    None,
                ));
            }
            body.insert(
                "default_sort_order".to_string(),
                Value::Number(value.into()),
            );
        }
        if let Some(value) = default_forum_layout {
            if value > 2 {
                return Err(ErrorData::invalid_params(
                    "Parameter 'default_forum_layout' must be 0, 1, or 2.",
                    None,
                ));
            }
            body.insert(
                "default_forum_layout".to_string(),
                Value::Number(value.into()),
            );
        }
        add_default_reaction_emoji(&mut body, default_reaction_emoji)?;
        add_available_tags(&mut body, available_tags)?;
        add_permission_overwrites(&mut body, permission_overwrites)?;

        structured(
            self.bot_http()
                .edit_channel(channel_id, &Value::Object(body), None)
                .await
                .map_err(|error| {
                    ErrorData::internal_error(format!("Failed to edit channel: {error}"), None)
                })?,
        )
    }

    #[tool(description = "Delete a channel (guild or DM).")]
    async fn delete_channel(
        &self,
        Parameters(ChannelIdParams { channel_id }): Parameters<ChannelIdParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let channel_id = ChannelId::new(parse_snowflake("channel_id", &channel_id)?);

        structured(
            self.bot_http()
                .delete_channel(channel_id, None)
                .await
                .map_err(|error| {
                    ErrorData::internal_error(format!("Failed to delete channel: {error}"), None)
                })?,
        )
    }

    #[tool(description = "Create a post in a forum channel.")]
    async fn create_forum_post(
        &self,
        Parameters(CreateForumPostParams {
            forum_channel_id,
            title,
            content,
            applied_tag_ids,
            auto_archive_duration,
            rate_limit_per_user,
        }): Parameters<CreateForumPostParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let forum_channel_id =
            ChannelId::new(parse_snowflake("forum_channel_id", &forum_channel_id)?);
        let mut body = Map::new();
        body.insert("name".to_string(), Value::String(title));
        body.insert("message".to_string(), json!({ "content": content }));

        if let Some(applied_tag_ids) = applied_tag_ids {
            body.insert(
                "applied_tags".to_string(),
                serde_json::to_value(parse_id_list("applied_tag_ids[]", applied_tag_ids)?)
                    .map_err(|error| {
                        ErrorData::internal_error(
                            format!("Failed to serialize forum post tags: {error}"),
                            None,
                        )
                    })?,
            );
        }
        if let Some(auto_archive_duration) = auto_archive_duration {
            validate_auto_archive_duration(auto_archive_duration)?;
            body.insert(
                "auto_archive_duration".to_string(),
                Value::Number(auto_archive_duration.into()),
            );
        }
        if let Some(rate_limit_per_user) = rate_limit_per_user {
            validate_rate_limit_per_user(rate_limit_per_user, "rate_limit_per_user")?;
            body.insert(
                "rate_limit_per_user".to_string(),
                Value::Number(rate_limit_per_user.into()),
            );
        }

        structured(
            self.bot_http()
                .create_forum_post(forum_channel_id, &Value::Object(body), None)
                .await
                .map_err(|error| {
                    ErrorData::internal_error(format!("Failed to create forum post: {error}"), None)
                })?,
        )
    }

    #[tool(description = "Edit the applied tags of a forum post/thread.")]
    async fn edit_forum_post_tags(
        &self,
        Parameters(EditForumPostTagsParams {
            post_channel_id,
            applied_tag_ids,
        }): Parameters<EditForumPostTagsParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let post_channel_id = ChannelId::new(parse_snowflake("post_channel_id", &post_channel_id)?);
        let body = json!({
            "applied_tags": parse_id_list("applied_tag_ids[]", applied_tag_ids)?,
        });

        structured(
            self.bot_http()
                .edit_thread(post_channel_id, &body, None)
                .await
                .map_err(|error| {
                    ErrorData::internal_error(
                        format!("Failed to edit forum post tags: {error}"),
                        None,
                    )
                })?,
        )
    }

    #[tool(description = "List active threads in a guild (includes active forum posts).")]
    async fn get_guild_active_threads(
        &self,
        Parameters(GuildIdParams { guild_id }): Parameters<GuildIdParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let guild_id = GuildId::new(parse_snowflake("guild_id", &guild_id)?);

        structured(
            self.bot_http()
                .get_guild_active_threads(guild_id)
                .await
                .map_err(|error| {
                    ErrorData::internal_error(
                        format!("Failed to fetch active threads: {error}"),
                        None,
                    )
                })?,
        )
    }

    #[tool(description = "List archived public threads from a channel (including forum posts).")]
    async fn get_channel_archived_public_threads(
        &self,
        Parameters(ArchivedThreadsParams {
            channel_id,
            before,
            limit,
        }): Parameters<ArchivedThreadsParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let channel_id = ChannelId::new(parse_snowflake("channel_id", &channel_id)?);

        if let Some(limit) = limit
            && !(1..=100).contains(&limit)
        {
            return Err(ErrorData::invalid_params(
                "Parameter 'limit' must be between 1 and 100.",
                None,
            ));
        }

        structured(
            self.bot_http()
                .get_channel_archived_public_threads(channel_id, before, limit)
                .await
                .map_err(|error| {
                    ErrorData::internal_error(
                        format!("Failed to fetch archived public threads: {error}"),
                        None,
                    )
                })?,
        )
    }

    #[tool(description = "List archived private threads from a channel.")]
    async fn get_channel_archived_private_threads(
        &self,
        Parameters(ArchivedThreadsParams {
            channel_id,
            before,
            limit,
        }): Parameters<ArchivedThreadsParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let channel_id = ChannelId::new(parse_snowflake("channel_id", &channel_id)?);

        if let Some(limit) = limit
            && !(1..=100).contains(&limit)
        {
            return Err(ErrorData::invalid_params(
                "Parameter 'limit' must be between 1 and 100.",
                None,
            ));
        }

        structured(
            self.bot_http()
                .get_channel_archived_private_threads(channel_id, before, limit)
                .await
                .map_err(|error| {
                    ErrorData::internal_error(
                        format!("Failed to fetch archived private threads: {error}"),
                        None,
                    )
                })?,
        )
    }

    #[tool(description = "List joined archived private threads from a channel.")]
    async fn get_channel_joined_archived_private_threads(
        &self,
        Parameters(ArchivedThreadsParams {
            channel_id,
            before,
            limit,
        }): Parameters<ArchivedThreadsParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let channel_id = ChannelId::new(parse_snowflake("channel_id", &channel_id)?);

        if let Some(limit) = limit
            && !(1..=100).contains(&limit)
        {
            return Err(ErrorData::invalid_params(
                "Parameter 'limit' must be between 1 and 100.",
                None,
            ));
        }

        structured(
            self.bot_http()
                .get_channel_joined_archived_private_threads(channel_id, before, limit)
                .await
                .map_err(|error| {
                    ErrorData::internal_error(
                        format!("Failed to fetch joined archived private threads: {error}"),
                        None,
                    )
                })?,
        )
    }

    #[tool(description = "List members currently in a thread channel.")]
    async fn get_channel_thread_members(
        &self,
        Parameters(ChannelIdParams { channel_id }): Parameters<ChannelIdParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let channel_id = ChannelId::new(parse_snowflake("channel_id", &channel_id)?);

        structured(
            self.bot_http()
                .get_channel_thread_members(channel_id)
                .await
                .map_err(|error| {
                    ErrorData::internal_error(
                        format!("Failed to fetch thread members: {error}"),
                        None,
                    )
                })?,
        )
    }

    #[tool(description = "Join a thread channel.")]
    async fn join_thread_channel(
        &self,
        Parameters(ChannelIdParams { channel_id }): Parameters<ChannelIdParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let channel_id = ChannelId::new(parse_snowflake("channel_id", &channel_id)?);

        self.bot_http()
            .join_thread_channel(channel_id)
            .await
            .map_err(|error| {
                ErrorData::internal_error(format!("Failed to join thread channel: {error}"), None)
            })?;

        structured(json!({
            "updated": true,
            "action": "join_thread",
            "channel_id": channel_id,
        }))
    }

    #[tool(description = "Leave a thread channel.")]
    async fn leave_thread_channel(
        &self,
        Parameters(ChannelIdParams { channel_id }): Parameters<ChannelIdParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let channel_id = ChannelId::new(parse_snowflake("channel_id", &channel_id)?);

        self.bot_http()
            .leave_thread_channel(channel_id)
            .await
            .map_err(|error| {
                ErrorData::internal_error(format!("Failed to leave thread channel: {error}"), None)
            })?;

        structured(json!({
            "updated": true,
            "action": "leave_thread",
            "channel_id": channel_id,
        }))
    }

    #[tool(description = "List invites for a channel.")]
    async fn get_channel_invites(
        &self,
        Parameters(ChannelIdParams { channel_id }): Parameters<ChannelIdParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let channel_id = ChannelId::new(parse_snowflake("channel_id", &channel_id)?);

        structured(
            self.bot_http()
                .get_channel_invites(channel_id)
                .await
                .map_err(|error| {
                    ErrorData::internal_error(
                        format!("Failed to fetch channel invites: {error}"),
                        None,
                    )
                })?,
        )
    }

    #[tool(description = "Create an invite for a channel.")]
    async fn create_channel_invite(
        &self,
        Parameters(CreateChannelInviteParams {
            channel_id,
            max_age,
            max_uses,
            temporary,
            unique,
            target_type,
            target_user_id,
            target_application_id,
        }): Parameters<CreateChannelInviteParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let channel_id = ChannelId::new(parse_snowflake("channel_id", &channel_id)?);
        let mut body = Map::new();

        if let Some(max_age) = max_age {
            if max_age > 604_800 {
                return Err(ErrorData::invalid_params(
                    "Parameter 'max_age' must be between 0 and 604800.",
                    None,
                ));
            }
            body.insert("max_age".to_string(), Value::Number(max_age.into()));
        }
        if let Some(max_uses) = max_uses {
            if max_uses > 100 {
                return Err(ErrorData::invalid_params(
                    "Parameter 'max_uses' must be between 0 and 100.",
                    None,
                ));
            }
            body.insert("max_uses".to_string(), Value::Number(max_uses.into()));
        }
        if let Some(temporary) = temporary {
            body.insert("temporary".to_string(), Value::Bool(temporary));
        }
        if let Some(unique) = unique {
            body.insert("unique".to_string(), Value::Bool(unique));
        }
        if target_user_id.is_some() && target_application_id.is_some() {
            return Err(ErrorData::invalid_params(
                "Provide only one of 'target_user_id' or 'target_application_id'.",
                None,
            ));
        }

        if target_user_id.is_some() || target_application_id.is_some() {
            let target_type = target_type.ok_or_else(|| {
                ErrorData::invalid_params(
                    "Parameter 'target_type' is required when target_user_id or target_application_id is set.",
                    None,
                )
            })?;
            body.insert("target_type".to_string(), Value::Number(target_type.into()));

            match target_type {
                1 => {
                    let target_user_id = target_user_id.ok_or_else(|| {
                        ErrorData::invalid_params(
                            "Parameter 'target_user_id' is required when target_type is 1.",
                            None,
                        )
                    })?;
                    body.insert(
                        "target_user_id".to_string(),
                        Value::Number(parse_snowflake("target_user_id", &target_user_id)?.into()),
                    );
                }
                2 => {
                    let target_application_id = target_application_id.ok_or_else(|| {
                        ErrorData::invalid_params(
                            "Parameter 'target_application_id' is required when target_type is 2.",
                            None,
                        )
                    })?;
                    body.insert(
                        "target_application_id".to_string(),
                        Value::Number(
                            parse_snowflake("target_application_id", &target_application_id)?
                                .into(),
                        ),
                    );
                }
                _ => {
                    return Err(ErrorData::invalid_params(
                        "Parameter 'target_type' must be 1 or 2.",
                        None,
                    ));
                }
            }
        } else if target_type.is_some() {
            return Err(ErrorData::invalid_params(
                "Provide target_user_id or target_application_id when using 'target_type'.",
                None,
            ));
        }

        structured(
            self.bot_http()
                .create_invite(channel_id, &Value::Object(body), None)
                .await
                .map_err(|error| {
                    ErrorData::internal_error(
                        format!("Failed to create channel invite: {error}"),
                        None,
                    )
                })?,
        )
    }

    #[tool(description = "Delete an invite by code.")]
    async fn delete_invite(
        &self,
        Parameters(InviteCodeParam { code }): Parameters<InviteCodeParam>,
    ) -> Result<CallToolResult, ErrorData> {
        structured(
            self.bot_http()
                .delete_invite(&code, None)
                .await
                .map_err(|error| {
                    ErrorData::internal_error(format!("Failed to delete invite: {error}"), None)
                })?,
        )
    }
}
