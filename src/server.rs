use std::sync::Arc;

use rmcp::{
    ServerHandler,
    handler::server::router::tool::ToolRouter,
    model::{CallToolResult, ErrorData, ServerCapabilities, ServerInfo},
    tool_handler,
};
use serde::Serialize;
use serenity::http::Http;

#[derive(Debug, Clone)]
pub(crate) struct Server {
    bot_http: Arc<Http>,
    tool_router: ToolRouter<Self>,
}

impl Server {
    pub(crate) fn new(bot_http: Arc<Http>) -> Self {
        Self {
            bot_http,
            tool_router: Self::application_router()
                + Self::channel_router()
                + Self::emoji_router()
                + Self::guild_router()
                + Self::message_router()
                + Self::member_router()
                + Self::role_router()
                + Self::user_router()
                + Self::webhook_router(),
        }
    }

    pub(crate) fn bot_http(&self) -> &Http {
        self.bot_http.as_ref()
    }
}

pub(crate) fn structured<T: Serialize>(value: T) -> Result<CallToolResult, ErrorData> {
    let value = serde_json::to_value(value).map_err(|error| {
        ErrorData::internal_error(format!("Failed to serialize result: {error}"), None)
    })?;

    Ok(CallToolResult::structured(value))
}

#[tool_handler(router = self.tool_router)]
impl ServerHandler for Server {
    fn get_info(&self) -> ServerInfo {
        ServerInfo::new(ServerCapabilities::builder().enable_tools().build())
            .with_instructions("Simple Discord MCP server.")
    }
}
