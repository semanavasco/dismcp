//! Tool definitions for user-related Discord operations.

use rmcp::{
    handler::server::wrapper::Parameters,
    model::{CallToolResult, ErrorData},
    tool, tool_router,
};
use schemars::JsonSchema;
use serde::Deserialize;
use serenity::model::id::{GuildId, UserId};

use crate::{
    server::{Server, structured},
    tools::{UserIdParams, matches_query, normalize_query, parse_snowflake},
};

#[derive(Debug, Deserialize, JsonSchema)]
struct SearchGuildMembersParams {
    #[schemars(description = "Guild ID (snowflake, as string).")]
    guild_id: String,
    #[schemars(description = "Search query for username/global name/nickname.")]
    query: String,
    #[schemars(
        description = "Maximum number of members to fetch from Discord (1-1000). Defaults to 1000.",
        range(min = 1, max = 1000)
    )]
    fetch_limit: Option<u64>,
    #[schemars(description = "Exact match in username/global name/nickname. Defaults to false.")]
    exact: Option<bool>,
}

#[tool_router(router = user_router, vis = "pub(crate)")]
impl Server {
    #[tool(description = "Get information about the currently authenticated user.")]
    async fn get_current_user(&self) -> Result<CallToolResult, ErrorData> {
        structured(self.bot_http().get_current_user().await.map_err(|error| {
            ErrorData::internal_error(format!("Failed to fetch user: {error}"), None)
        })?)
    }

    #[tool(description = "Get information about a user by ID.")]
    async fn get_user(
        &self,
        Parameters(UserIdParams { user_id }): Parameters<UserIdParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let user_id = UserId::new(parse_snowflake("user_id", &user_id)?);

        structured(self.bot_http().get_user(user_id).await.map_err(|error| {
            ErrorData::internal_error(format!("Failed to fetch user: {error}"), None)
        })?)
    }

    #[tool(description = "Search guild members by username, global name, or nickname.")]
    async fn search_guild_members(
        &self,
        Parameters(SearchGuildMembersParams {
            guild_id,
            query,
            fetch_limit,
            exact,
        }): Parameters<SearchGuildMembersParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let guild_id = GuildId::new(parse_snowflake("guild_id", &guild_id)?);
        let exact = exact.unwrap_or(false);
        let query = normalize_query(&query);
        let fetch_limit = fetch_limit.unwrap_or(1000);

        if query.is_empty() {
            return Err(ErrorData::invalid_params(
                "Parameter 'query' cannot be empty.",
                None,
            ));
        }

        if !(1..=1000).contains(&fetch_limit) {
            return Err(ErrorData::invalid_params(
                "Parameter 'fetch_limit' must be between 1 and 1000.",
                None,
            ));
        }

        let page_limit = fetch_limit;
        let mut members = Vec::new();
        let mut after = None;

        loop {
            let page = self
                .bot_http()
                .get_guild_members(guild_id, Some(page_limit), after)
                .await
                .map_err(|error| {
                    ErrorData::internal_error(
                        format!("Failed to fetch guild members: {error}"),
                        None,
                    )
                })?;

            if page.is_empty() {
                break;
            }

            after = page.last().map(|member| member.user.id.get());
            members.extend(page);

            if members.len() >= fetch_limit as usize || members.len() % page_limit as usize != 0 {
                break;
            }
        }

        members.truncate(fetch_limit as usize);

        let filtered: Vec<_> = members
            .into_iter()
            .filter(|member| {
                let username = normalize_query(&member.user.name);
                let global_name = member
                    .user
                    .global_name
                    .as_deref()
                    .map(normalize_query)
                    .unwrap_or_default();
                let nickname = member
                    .nick
                    .as_deref()
                    .map(normalize_query)
                    .unwrap_or_default();

                matches_query(&username, &query, exact)
                    || matches_query(&global_name, &query, exact)
                    || matches_query(&nickname, &query, exact)
            })
            .collect();

        structured(filtered)
    }
}
