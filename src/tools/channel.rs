use rmcp::{
    handler::server::wrapper::Parameters,
    model::{CallToolResult, ErrorData},
    tool, tool_router,
};
use schemars::JsonSchema;
use serde::Deserialize;
use serenity::model::id::ChannelId;

use crate::{
    server::{Server, structured},
    tools::parse_snowflake,
};

#[derive(Debug, Deserialize, JsonSchema)]
struct GetChannelParams {
    #[schemars(description = "Channel ID (snowflake, as string).")]
    channel_id: String,
}

#[tool_router(router = channel_router, vis = "pub(crate)")]
impl Server {
    #[tool(description = "Get details for a specific channel.")]
    async fn get_channel(
        &self,
        Parameters(GetChannelParams { channel_id }): Parameters<GetChannelParams>,
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
}
