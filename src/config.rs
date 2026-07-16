//! Application configuration management.
//!
//! Handles loading and validating environment variables required for the server.

const DEFAULT_BIND_ADDRESS: &str = "127.0.0.1:3000";

/// Configuration for the `dismcp` application.
pub(crate) struct AppConfig {
    /// The Discord bot token used to authenticate with the Discord API.
    pub(crate) discord_token: String,
    /// The address the MCP HTTP server will bind to (e.g. `127.0.0.1:3000`).
    pub(crate) bind_address: String,
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

        Self {
            discord_token,
            bind_address,
        }
    }
}
