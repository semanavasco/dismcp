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
    tools::parse_snowflake,
};

#[derive(Debug, Deserialize, JsonSchema)]
struct GetMemberParams {
    #[schemars(description = "Guild ID (snowflake, as string).")]
    guild_id: String,
    #[schemars(description = "User ID (snowflake, as string).")]
    user_id: String,
}

#[tool_router(router = member_router, vis = "pub(crate)")]
impl Server {
    #[tool(description = "Get a guild member by guild and user IDs.")]
    async fn get_member(
        &self,
        Parameters(GetMemberParams { guild_id, user_id }): Parameters<GetMemberParams>,
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
}
