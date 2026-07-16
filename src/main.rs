use std::sync::Arc;

use rmcp::transport::streamable_http_server::{
    StreamableHttpServerConfig, StreamableHttpService, session::local::LocalSessionManager,
};
use serenity::http::Http;
use tracing::info;

use config::AppConfig;
use server::Server;

mod config;
mod server;
mod tools;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    let config = AppConfig::from_env();

    let discord_http = Arc::new(Http::new(&config.discord_token));
    let current_application = discord_http.get_current_application_info().await?;
    discord_http.set_application_id(current_application.id);

    let service: StreamableHttpService<Server, LocalSessionManager> = StreamableHttpService::new(
        {
            let discord_http = discord_http.clone();
            move || Ok(Server::new(discord_http.clone()))
        },
        Default::default(),
        StreamableHttpServerConfig::default()
            .with_stateful_mode(false)
            .with_json_response(true),
    );

    let router = axum::Router::new().route_service("/", service);
    let listener = tokio::net::TcpListener::bind(&config.bind_address).await?;

    info!(address = %config.bind_address, "Discord MCP server listening");
    info!(
        application_id = %current_application.id,
        "Initialized Discord application context"
    );
    info!("MCP endpoint: http://{}/", config.bind_address);

    axum::serve(listener, router)
        .with_graceful_shutdown(async {
            let _ = tokio::signal::ctrl_c().await;
            info!("Shutdown signal received");
        })
        .await?;

    Ok(())
}
