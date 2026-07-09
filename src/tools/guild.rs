use rmcp::{
    handler::server::wrapper::Parameters,
    model::{CallToolResult, ErrorData},
    tool, tool_router,
};
use schemars::JsonSchema;
use serde::Deserialize;
use serenity::{http::GuildPagination, model::id::GuildId};

use crate::server::{Server, structured};

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
                let guild_id = before.parse::<u64>().map_err(|_| {
                    ErrorData::invalid_params(
                        "Parameter 'before' must be a valid Discord snowflake string.",
                        None,
                    )
                })?;
                Some(GuildPagination::Before(GuildId::new(guild_id)))
            }
            (None, Some(after)) => {
                let guild_id = after.parse::<u64>().map_err(|_| {
                    ErrorData::invalid_params(
                        "Parameter 'after' must be a valid Discord snowflake string.",
                        None,
                    )
                })?;
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
}
