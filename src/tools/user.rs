use rmcp::{
    handler::server::wrapper::Parameters,
    model::{CallToolResult, ErrorData},
    tool, tool_router,
};
use schemars::JsonSchema;
use serde::Deserialize;
use serenity::model::id::UserId;

use crate::{
    server::{Server, structured},
    tools::parse_snowflake,
};

#[derive(Debug, Deserialize, JsonSchema)]
struct GetUserParams {
    #[schemars(description = "User ID (snowflake, as string).")]
    user_id: String,
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
        Parameters(GetUserParams { user_id }): Parameters<GetUserParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let user_id = UserId::new(parse_snowflake("user_id", &user_id)?);

        structured(self.bot_http().get_user(user_id).await.map_err(|error| {
            ErrorData::internal_error(format!("Failed to fetch user: {error}"), None)
        })?)
    }
}
