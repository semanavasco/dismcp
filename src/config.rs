const DEFAULT_BIND_ADDRESS: &str = "127.0.0.1:3000";

pub(crate) struct AppConfig {
    pub(crate) discord_token: String,
    pub(crate) bind_address: String,
}

impl AppConfig {
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
