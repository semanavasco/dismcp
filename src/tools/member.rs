//! Tool definitions for guild member-related Discord operations (kick, ban, timeout, etc.).

use crate::{
    server::{Server, structured},
    tools::{GuildUserParams, parse_snowflake},
};
use rmcp::{
    handler::server::wrapper::Parameters,
    model::{CallToolResult, ErrorData},
    tool, tool_router,
};
use schemars::JsonSchema;
use serde::Deserialize;
use serde_json::{Value, json};
use serenity::model::id::{GuildId, UserId};

#[derive(Debug, Deserialize, JsonSchema)]
struct BanMemberParams {
    #[schemars(description = "Guild ID (snowflake, as string).")]
    guild_id: String,
    #[schemars(description = "User ID (snowflake, as string).")]
    user_id: String,
    #[schemars(
        description = "Delete this many days of recent messages (0-7). Defaults to 0.",
        range(min = 0, max = 7)
    )]
    delete_message_days: Option<u64>,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct TimeoutMemberParams {
    #[schemars(description = "Guild ID (snowflake, as string).")]
    guild_id: String,
    #[schemars(description = "User ID (snowflake, as string).")]
    user_id: String,
    #[schemars(
        description = "Timeout-until timestamp in ISO-8601 format (e.g. 2026-07-15T12:00:00Z)."
    )]
    until: String,
}

#[tool_router(router = member_router, vis = "pub(crate)")]
impl Server {
    #[tool(description = "Get a guild member by guild and user IDs.")]
    async fn get_member(
        &self,
        Parameters(GuildUserParams { guild_id, user_id }): Parameters<GuildUserParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let guild_id = GuildId::new(parse_snowflake("guild_id", &guild_id)?);
        let user_id = UserId::new(parse_snowflake("user_id", &user_id)?);

        structured(
            self.bot_http()
                .get_member(guild_id, user_id)
                .await
                .map_err(|error| {
                    ErrorData::internal_error(format!("Failed to fetch member: {error}"), None)
                })?,
        )
    }

    #[tool(description = "Kick a member from a guild.")]
    async fn kick_member(
        &self,
        Parameters(GuildUserParams { guild_id, user_id }): Parameters<GuildUserParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let guild_id = GuildId::new(parse_snowflake("guild_id", &guild_id)?);
        let user_id = UserId::new(parse_snowflake("user_id", &user_id)?);

        self.bot_http()
            .kick_member(guild_id, user_id, None)
            .await
            .map_err(|error| {
                ErrorData::internal_error(format!("Failed to kick member: {error}"), None)
            })?;

        structured(json!({
            "updated": true,
            "action": "kick_member",
            "guild_id": guild_id,
            "user_id": user_id,
        }))
    }

    #[tool(description = "Ban a user from a guild.")]
    async fn ban_member(
        &self,
        Parameters(BanMemberParams {
            guild_id,
            user_id,
            delete_message_days,
        }): Parameters<BanMemberParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let guild_id = GuildId::new(parse_snowflake("guild_id", &guild_id)?);
        let user_id = UserId::new(parse_snowflake("user_id", &user_id)?);
        let delete_message_days = delete_message_days.unwrap_or(0);

        if delete_message_days > 7 {
            return Err(ErrorData::invalid_params(
                "Parameter 'delete_message_days' must be between 0 and 7.",
                None,
            ));
        }

        self.bot_http()
            .ban_user(guild_id, user_id, delete_message_days as u8, None)
            .await
            .map_err(|error| {
                ErrorData::internal_error(format!("Failed to ban member: {error}"), None)
            })?;

        structured(json!({
            "updated": true,
            "action": "ban_member",
            "guild_id": guild_id,
            "user_id": user_id,
            "delete_message_days": delete_message_days,
        }))
    }

    #[tool(description = "Unban a user from a guild.")]
    async fn unban_member(
        &self,
        Parameters(GuildUserParams { guild_id, user_id }): Parameters<GuildUserParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let guild_id = GuildId::new(parse_snowflake("guild_id", &guild_id)?);
        let user_id = UserId::new(parse_snowflake("user_id", &user_id)?);

        self.bot_http()
            .remove_ban(guild_id, user_id, None)
            .await
            .map_err(|error| {
                ErrorData::internal_error(format!("Failed to unban member: {error}"), None)
            })?;

        structured(json!({
            "updated": true,
            "action": "unban_member",
            "guild_id": guild_id,
            "user_id": user_id,
        }))
    }

    #[tool(description = "Set a member timeout until the specified ISO-8601 timestamp.")]
    async fn timeout_member(
        &self,
        Parameters(TimeoutMemberParams {
            guild_id,
            user_id,
            until,
        }): Parameters<TimeoutMemberParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let guild_id = GuildId::new(parse_snowflake("guild_id", &guild_id)?);
        let user_id = UserId::new(parse_snowflake("user_id", &user_id)?);
        let body = json!({
            "communication_disabled_until": until,
        });

        structured(
            self.bot_http()
                .edit_member(guild_id, user_id, &body, None)
                .await
                .map_err(|error| {
                    ErrorData::internal_error(format!("Failed to timeout member: {error}"), None)
                })?,
        )
    }

    #[tool(description = "Remove a member timeout.")]
    async fn clear_member_timeout(
        &self,
        Parameters(GuildUserParams { guild_id, user_id }): Parameters<GuildUserParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let guild_id = GuildId::new(parse_snowflake("guild_id", &guild_id)?);
        let user_id = UserId::new(parse_snowflake("user_id", &user_id)?);
        let body = json!({
            "communication_disabled_until": Value::Null,
        });

        structured(
            self.bot_http()
                .edit_member(guild_id, user_id, &body, None)
                .await
                .map_err(|error| {
                    ErrorData::internal_error(
                        format!("Failed to clear member timeout: {error}"),
                        None,
                    )
                })?,
        )
    }
}
