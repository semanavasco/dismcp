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
    /// Composes and registers enabled tool routers (channel, guild, message, etc.).
    pub(crate) fn new(bot_http: Arc<Http>, enabled_tools: String) -> Self {
        let tool_router: ToolRouter<Server> = if enabled_tools.trim() == "all" {
            Self::application_router()
                + Self::channel_router()
                + Self::emoji_router()
                + Self::guild_router()
                + Self::member_router()
                + Self::message_router()
                + Self::role_router()
                + Self::user_router()
                + Self::webhook_router()
        } else {
            let mut tool_router = ToolRouter::new();

            let routers = enabled_tools
                .split(',')
                .map(|tool| tool.trim())
                .filter(|tool| !tool.is_empty())
                .collect::<Vec<_>>();

            for tool in routers {
                match tool {
                    "application" => tool_router += Self::application_router(),
                    "channel" => tool_router += Self::channel_router(),
                    "emoji" => tool_router += Self::emoji_router(),
                    "guild" => tool_router += Self::guild_router(),
                    "member" => tool_router += Self::member_router(),
                    "message" => tool_router += Self::message_router(),
                    "role" => tool_router += Self::role_router(),
                    "user" => tool_router += Self::user_router(),
                    "webhook" => tool_router += Self::webhook_router(),
                    _ => {
                        tracing::error!(
                            "Unknown tool '{tool}' specified in MCP_ENABLED_TOOLS. Ignoring."
                        );
                    }
                }
            }

            tool_router
        };

        Self {
            bot_http,
            tool_router,
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
