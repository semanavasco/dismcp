use rmcp::{
    handler::server::wrapper::Parameters,
    model::{CallToolResult, ErrorData},
    tool, tool_router,
};
use schemars::JsonSchema;
use serde::Deserialize;
use serenity::{http::GuildPagination, model::id::GuildId};

use crate::{
    server::{Server, structured},
    tools::parse_snowflake,
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
struct GuildIdParam {
    #[schemars(description = "Guild ID (snowflake, as string).")]
    guild_id: String,
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
        Parameters(GuildIdParam { guild_id }): Parameters<GuildIdParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let guild_id = GuildId::new(parse_snowflake("guild_id", &guild_id)?);

        structured(self.bot_http().get_guild(guild_id).await.map_err(|error| {
            ErrorData::internal_error(format!("Failed to fetch guild: {error}"), None)
        })?)
    }

    #[tool(description = "List channels in a specific guild.")]
    async fn get_guild_channels(
        &self,
        Parameters(GuildIdParam { guild_id }): Parameters<GuildIdParam>,
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
}
