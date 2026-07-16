use rmcp::{
    handler::server::wrapper::Parameters,
    model::{CallToolResult, ErrorData},
    tool, tool_router,
};
use schemars::JsonSchema;
use serde::Deserialize;
use serde_json::{Map, Value};
use serenity::{
    http::{GuildPagination, UserPagination},
    model::id::{GuildId, ScheduledEventId, UserId},
};

use crate::{
    server::{Server, structured},
    tools::{GuildIdParams, parse_snowflake},
};

#[derive(Debug, Deserialize, JsonSchema)]
struct GetGuildsParams {
    #[schemars(
        description = "Maximum number of guilds to fetch (1-200). Defaults to 200.",
        range(min = 1, max = 200)
    )]
    limit: Option<u64>,
    #[schemars(description = "Fetch guilds before this guild ID (snowflake, as string).")]
    before: Option<String>,
    #[schemars(description = "Fetch guilds after this guild ID (snowflake, as string).")]
    after: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct GetGuildMembersParams {
    #[schemars(description = "Guild ID (snowflake, as string).")]
    guild_id: String,
    #[schemars(
        description = "Maximum number of members to fetch (1-1000). Defaults to 1000.",
        range(min = 1, max = 1000)
    )]
    limit: Option<u64>,
    #[schemars(
        description = "Highest user ID in the previous page (snowflake, as string). Optional."
    )]
    after: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct ScheduledEventIdParams {
    #[schemars(description = "Guild ID (snowflake, as string).")]
    guild_id: String,
    #[schemars(description = "Event ID (snowflake, as string).")]
    event_id: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct CreateScheduledEventParams {
    #[schemars(description = "Guild ID (snowflake, as string).")]
    guild_id: String,
    #[schemars(description = "Event name (1-100 characters).")]
    name: String,
    #[schemars(description = "Event description (1-1000 characters).")]
    description: Option<String>,
    #[schemars(description = "Scheduled start time (ISO8601).")]
    scheduled_start_time: String,
    #[schemars(description = "Scheduled end time (ISO8601).")]
    scheduled_end_time: Option<String>,
    #[schemars(description = "Entity type: 1 stage, 2 voice, 3 external.")]
    entity_type: u8,
    #[schemars(description = "Privacy level: 2 guild_only.")]
    privacy_level: Option<u8>,
    #[schemars(description = "Channel ID for stage/voice events (snowflake, as string).")]
    channel_id: Option<String>,
    #[schemars(description = "Location for external events.")]
    location: Option<String>,
    #[schemars(description = "Event cover image (data URI scheme).")]
    image_data_uri: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct EditScheduledEventParams {
    #[schemars(description = "Guild ID (snowflake, as string).")]
    guild_id: String,
    #[schemars(description = "Event ID (snowflake, as string).")]
    event_id: String,
    #[schemars(description = "Event name (1-100 characters).")]
    name: Option<String>,
    #[schemars(description = "Event description (1-1000 characters).")]
    description: Option<String>,
    #[schemars(description = "Scheduled start time (ISO8601).")]
    scheduled_start_time: Option<String>,
    #[schemars(description = "Scheduled end time (ISO8601).")]
    scheduled_end_time: Option<String>,
    #[schemars(description = "Entity type: 1 stage, 2 voice, 3 external.")]
    entity_type: Option<u8>,
    #[schemars(description = "Status: 1 scheduled, 2 active, 3 completed, 4 canceled.")]
    status: Option<u8>,
    #[schemars(description = "Channel ID for stage/voice events (snowflake, as string).")]
    channel_id: Option<String>,
    #[schemars(description = "Location for external events.")]
    location: Option<String>,
    #[schemars(description = "Event cover image (data URI scheme).")]
    image_data_uri: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct ScheduledEventUserParams {
    #[schemars(description = "Guild ID (snowflake, as string).")]
    guild_id: String,
    #[schemars(description = "Event ID (snowflake, as string).")]
    event_id: String,
    #[schemars(
        description = "Maximum number of users to fetch (1-100). Defaults to 100.",
        range(min = 1, max = 100)
    )]
    limit: Option<u64>,
    #[schemars(description = "Fetch users after this user ID (snowflake, as string).")]
    after: Option<String>,
}

fn build_scheduled_event_payload(
    name: Option<String>,
    description: Option<String>,
    scheduled_start_time: Option<String>,
    scheduled_end_time: Option<String>,
    entity_type: Option<u8>,
    privacy_level: Option<u8>,
    status: Option<u8>,
    channel_id: Option<String>,
    location: Option<String>,
    image_data_uri: Option<String>,
) -> Result<Map<String, Value>, ErrorData> {
    let mut body = Map::new();

    if let Some(name) = name {
        body.insert("name".to_string(), Value::String(name));
    }
    if let Some(description) = description {
        body.insert("description".to_string(), Value::String(description));
    }
    if let Some(time) = scheduled_start_time {
        body.insert("scheduled_start_time".to_string(), Value::String(time));
    }
    if let Some(time) = scheduled_end_time {
        body.insert("scheduled_end_time".to_string(), Value::String(time));
    }
    if let Some(entity_type) = entity_type {
        if !(1..=3).contains(&entity_type) {
            return Err(ErrorData::invalid_params(
                "Parameter 'entity_type' must be 1, 2, or 3.",
                None,
            ));
        }
        body.insert("entity_type".to_string(), Value::Number(entity_type.into()));
    }
    if let Some(privacy_level) = privacy_level {
        if privacy_level != 2 {
            return Err(ErrorData::invalid_params(
                "Parameter 'privacy_level' must be 2.",
                None,
            ));
        }
        body.insert(
            "privacy_level".to_string(),
            Value::Number(privacy_level.into()),
        );
    }
    if let Some(status) = status {
        body.insert("status".to_string(), Value::Number(status.into()));
    }
    if let Some(channel_id) = channel_id {
        body.insert(
            "channel_id".to_string(),
            Value::Number(parse_snowflake("channel_id", &channel_id)?.into()),
        );
    }
    if let Some(location) = location {
        let mut metadata = Map::new();
        metadata.insert("location".to_string(), Value::String(location));
        body.insert("entity_metadata".to_string(), Value::Object(metadata));
    }
    if let Some(image) = image_data_uri {
        body.insert("image".to_string(), Value::String(image));
    }

    Ok(body)
}

#[tool_router(router = guild_router, vis = "pub(crate)")]
impl Server {
    #[tool(description = "List guilds visible to the currently authenticated user.")]
    async fn get_guilds(
        &self,
        Parameters(GetGuildsParams {
            limit,
            before,
            after,
        }): Parameters<GetGuildsParams>,
    ) -> Result<CallToolResult, ErrorData> {
        if before.is_some() && after.is_some() {
            return Err(ErrorData::invalid_params(
                "Use either 'before' or 'after', not both.",
                None,
            ));
        }

        let limit = limit.unwrap_or(200);
        if !(1..=200).contains(&limit) {
            return Err(ErrorData::invalid_params(
                "Parameter 'limit' must be between 1 and 200.",
                None,
            ));
        }

        let target = match (before, after) {
            (Some(before), None) => {
                let guild_id = parse_snowflake("before", &before)?;
                Some(GuildPagination::Before(GuildId::new(guild_id)))
            }
            (None, Some(after)) => {
                let guild_id = parse_snowflake("after", &after)?;
                Some(GuildPagination::After(GuildId::new(guild_id)))
            }
            (None, None) => None,
            (Some(_), Some(_)) => unreachable!(),
        };

        structured(
            self.bot_http()
                .get_guilds(target, Some(limit))
                .await
                .map_err(|error| {
                    ErrorData::internal_error(format!("Failed to fetch guilds: {error}"), None)
                })?,
        )
    }

    #[tool(description = "Get details for a specific guild.")]
    async fn get_guild(
        &self,
        Parameters(GuildIdParams { guild_id }): Parameters<GuildIdParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let guild_id = GuildId::new(parse_snowflake("guild_id", &guild_id)?);

        structured(self.bot_http().get_guild(guild_id).await.map_err(|error| {
            ErrorData::internal_error(format!("Failed to fetch guild: {error}"), None)
        })?)
    }

    #[tool(description = "List members from a specific guild.")]
    async fn get_guild_members(
        &self,
        Parameters(GetGuildMembersParams {
            guild_id,
            limit,
            after,
        }): Parameters<GetGuildMembersParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let guild_id = GuildId::new(parse_snowflake("guild_id", &guild_id)?);
        let limit = limit.unwrap_or(1000);

        if !(1..=1000).contains(&limit) {
            return Err(ErrorData::invalid_params(
                "Parameter 'limit' must be between 1 and 1000.",
                None,
            ));
        }

        let after = after
            .as_deref()
            .map(|value| parse_snowflake("after", value))
            .transpose()?;

        structured(
            self.bot_http()
                .get_guild_members(guild_id, Some(limit), after)
                .await
                .map_err(|error| {
                    ErrorData::internal_error(
                        format!("Failed to fetch guild members: {error}"),
                        None,
                    )
                })?,
        )
    }

    #[tool(description = "List all active invites for a specific guild.")]
    async fn get_guild_invites(
        &self,
        Parameters(GuildIdParams { guild_id }): Parameters<GuildIdParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let guild_id = GuildId::new(parse_snowflake("guild_id", &guild_id)?);

        structured(
            self.bot_http()
                .get_guild_invites(guild_id)
                .await
                .map_err(|error| {
                    ErrorData::internal_error(
                        format!("Failed to fetch guild invites: {error}"),
                        None,
                    )
                })?,
        )
    }

    #[tool(description = "List all scheduled events in a specific guild.")]
    async fn get_scheduled_events(
        &self,
        Parameters(GuildIdParams { guild_id }): Parameters<GuildIdParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let guild_id = GuildId::new(parse_snowflake("guild_id", &guild_id)?);

        structured(
            self.bot_http()
                .get_scheduled_events(guild_id, true)
                .await
                .map_err(|error| {
                    ErrorData::internal_error(
                        format!("Failed to fetch scheduled events: {error}"),
                        None,
                    )
                })?,
        )
    }

    #[tool(description = "Get details for a specific scheduled event.")]
    async fn get_scheduled_event(
        &self,
        Parameters(ScheduledEventIdParams { guild_id, event_id }): Parameters<
            ScheduledEventIdParams,
        >,
    ) -> Result<CallToolResult, ErrorData> {
        let guild_id = GuildId::new(parse_snowflake("guild_id", &guild_id)?);
        let event_id = ScheduledEventId::new(parse_snowflake("event_id", &event_id)?);

        structured(
            self.bot_http()
                .get_scheduled_event(guild_id, event_id, true)
                .await
                .map_err(|error| {
                    ErrorData::internal_error(
                        format!("Failed to fetch scheduled event: {error}"),
                        None,
                    )
                })?,
        )
    }

    #[tool(description = "Create a new scheduled event in a guild.")]
    async fn create_scheduled_event(
        &self,
        Parameters(CreateScheduledEventParams {
            guild_id,
            name,
            description,
            scheduled_start_time,
            scheduled_end_time,
            entity_type,
            privacy_level,
            channel_id,
            location,
            image_data_uri,
        }): Parameters<CreateScheduledEventParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let guild_id = GuildId::new(parse_snowflake("guild_id", &guild_id)?);

        if entity_type == 3 && (location.is_none() || scheduled_end_time.is_none()) {
            return Err(ErrorData::invalid_params(
                "External events (entity_type 3) require 'location' and 'scheduled_end_time'.",
                None,
            ));
        }
        if (entity_type == 1 || entity_type == 2) && channel_id.is_none() {
            return Err(ErrorData::invalid_params(
                "Stage/Voice events (entity_type 1 or 2) require 'channel_id'.",
                None,
            ));
        }

        let body = build_scheduled_event_payload(
            Some(name),
            description,
            Some(scheduled_start_time),
            scheduled_end_time,
            Some(entity_type),
            Some(privacy_level.unwrap_or(2)),
            None,
            channel_id,
            location,
            image_data_uri,
        )?;

        structured(
            self.bot_http()
                .create_scheduled_event(guild_id, &Value::Object(body), None)
                .await
                .map_err(|error| {
                    ErrorData::internal_error(
                        format!("Failed to create scheduled event: {error}"),
                        None,
                    )
                })?,
        )
    }

    #[tool(description = "Edit an existing scheduled event in a guild.")]
    async fn edit_scheduled_event(
        &self,
        Parameters(EditScheduledEventParams {
            guild_id,
            event_id,
            name,
            description,
            scheduled_start_time,
            scheduled_end_time,
            entity_type,
            status,
            channel_id,
            location,
            image_data_uri,
        }): Parameters<EditScheduledEventParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let guild_id = GuildId::new(parse_snowflake("guild_id", &guild_id)?);
        let event_id = ScheduledEventId::new(parse_snowflake("event_id", &event_id)?);

        let body = build_scheduled_event_payload(
            name,
            description,
            scheduled_start_time,
            scheduled_end_time,
            entity_type,
            None,
            status,
            channel_id,
            location,
            image_data_uri,
        )?;

        if body.is_empty() {
            return Err(ErrorData::invalid_params(
                "Provide at least one field to update.",
                None,
            ));
        }

        structured(
            self.bot_http()
                .edit_scheduled_event(guild_id, event_id, &Value::Object(body), None)
                .await
                .map_err(|error| {
                    ErrorData::internal_error(
                        format!("Failed to edit scheduled event: {error}"),
                        None,
                    )
                })?,
        )
    }

    #[tool(description = "Delete a scheduled event from a guild.")]
    async fn delete_scheduled_event(
        &self,
        Parameters(ScheduledEventIdParams { guild_id, event_id }): Parameters<
            ScheduledEventIdParams,
        >,
    ) -> Result<CallToolResult, ErrorData> {
        let guild_id = GuildId::new(parse_snowflake("guild_id", &guild_id)?);
        let event_id = ScheduledEventId::new(parse_snowflake("event_id", &event_id)?);

        self.bot_http()
            .delete_scheduled_event(guild_id, event_id)
            .await
            .map_err(|error| {
                ErrorData::internal_error(
                    format!("Failed to delete scheduled event: {error}"),
                    None,
                )
            })?;

        structured(())
    }

    #[tool(description = "List users subscribed to a scheduled event.")]
    async fn get_scheduled_event_users(
        &self,
        Parameters(ScheduledEventUserParams {
            guild_id,
            event_id,
            limit,
            after,
        }): Parameters<ScheduledEventUserParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let guild_id = GuildId::new(parse_snowflake("guild_id", &guild_id)?);
        let event_id = ScheduledEventId::new(parse_snowflake("event_id", &event_id)?);
        let limit = limit.unwrap_or(100);

        if !(1..=100).contains(&limit) {
            return Err(ErrorData::invalid_params(
                "Parameter 'limit' must be between 1 and 100.",
                None,
            ));
        }

        let target = after
            .as_deref()
            .map(|value| parse_snowflake("after", value))
            .transpose()?
            .map(|id| UserPagination::After(UserId::new(id)));

        structured(
            self.bot_http()
                .get_scheduled_event_users(guild_id, event_id, Some(limit), target, Some(true))
                .await
                .map_err(|error| {
                    ErrorData::internal_error(
                        format!("Failed to fetch scheduled event users: {error}"),
                        None,
                    )
                })?,
        )
    }
}
