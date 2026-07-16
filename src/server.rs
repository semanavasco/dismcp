//! MCP server implementation and state management.
//!
//! Defines the core `Server` state, which holds the Discord HTTP client and routes
//! incoming tool calls to their respective module handlers. Also provides helpers
//! for structuring MCP tool responses.

use std::sync::Arc;

use rmcp::{
    ServerHandler,
    handler::server::router::tool::ToolRouter,
    model::{CallToolResult, ErrorData, ServerCapabilities, ServerInfo},
    tool_handler,
};
use serde::Serialize;
use serenity::http::Http;

/// The main MCP server state.
///
/// Holds the active Discord HTTP client and the compiled router containing all tool definitions.
#[derive(Debug, Clone)]
pub(crate) struct Server {
    /// A thread-safe reference to the serenity HTTP client.
    bot_http: Arc<Http>,
    /// The compiled tool router containing all registered tool handlers.
    tool_router: ToolRouter<Self>,
}

impl Server {
    /// Constructs a new `Server` instance with the given Discord HTTP client.
    ///
    /// Composes and registers all tool routers (channel, guild, message, etc.).
    ///
    /// Add your own tool routers here as needed.
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

    /// Returns a reference to the Discord HTTP client.
    pub(crate) fn bot_http(&self) -> &Http {
        self.bot_http.as_ref()
    }
}

/// A helper function to serialize a value into a structured `CallToolResult`.
///
/// Converts the given serializable value into `serde_json::Value` and wraps it in an MCP `CallToolResult`.
///
/// Use it when you want to return structured JSON data from a tool handler.
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
