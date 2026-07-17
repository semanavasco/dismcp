//! Application configuration management.
//!
//! Handles loading and validating environment variables required for the server.

const DEFAULT_BIND_ADDRESS: &str = "127.0.0.1:3000";
const DEFAULT_ENABLED_TOOLS: &str = "all";

/// Configuration for the `dismcp` application.
pub(crate) struct AppConfig {
    /// The Discord bot token used to authenticate with the Discord API.
    pub(crate) discord_token: String,
    /// The address the MCP HTTP server will bind to (e.g. `127.0.0.1:3000`).
    pub(crate) bind_address: String,
    /// The tools to enable for the MCP server. Defaults to "all".
    /// Available tools can be specified as a comma-separated list (e.g. "channel,guild,message").
    pub(crate) enabled_tools: String,
}

impl AppConfig {
    /// Loads the configuration from environment variables.
    ///
    /// # Panics
    /// Panics if the `DISCORD_TOKEN` environment variable is not set.
    pub(crate) fn from_env() -> Self {
        let discord_token =
            std::env::var("DISCORD_TOKEN").expect("Expected DISCORD_TOKEN in the environment");
        let bind_address =
            std::env::var("MCP_BIND_ADDRESS").unwrap_or_else(|_| DEFAULT_BIND_ADDRESS.to_string());
        let enabled_tools = std::env::var("MCP_ENABLED_TOOLS")
            .unwrap_or_else(|_| DEFAULT_ENABLED_TOOLS.to_string());

        Self {
            discord_token,
            bind_address,
            enabled_tools,
        }
    }
}
