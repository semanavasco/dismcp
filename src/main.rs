//! The main entry point for the `dismcp` server.
//!
//! This module sets up the configuration, initializes the Discord API client
//! via `serenity`, and starts the MCP server using `rmcp`.

use std::sync::Arc;

use rmcp::transport::streamable_http_server::{
    StreamableHttpServerConfig, StreamableHttpService, session::local::LocalSessionManager,
};
use serenity::http::Http;
use tracing::info;

use config::AppConfig;
use server::Server;

mod cli;
mod config;
mod server;
mod tools;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    cli::handle_args();

    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .init();

    let config = AppConfig::from_env();
    server::init_config(config.omit_nulls);

    let discord_http = Arc::new(Http::new(&config.discord_token));
    let current_application = discord_http.get_current_application_info().await?;
    discord_http.set_application_id(current_application.id);

    info!(
        application_id = %current_application.id,
        "Initialized Discord application context"
    );

    match config.mcp_transport {
        config::TransportType::Http(bind_address) => {
            let service: StreamableHttpService<Server, LocalSessionManager> =
                StreamableHttpService::new(
                    {
                        let discord_http = discord_http.clone();
                        move || {
                            Ok(Server::new(
                                discord_http.clone(),
                                config.enabled_tools.clone(),
                            ))
                        }
                    },
                    Default::default(),
                    StreamableHttpServerConfig::default()
                        .with_stateful_mode(false)
                        .with_json_response(true),
                );

            let router = axum::Router::new().route_service("/", service);
            let listener = tokio::net::TcpListener::bind(&bind_address).await?;

            info!(address = %bind_address, "Discord MCP server listening (HTTP)");
            info!("MCP endpoint: http://{}/", bind_address);

            axum::serve(listener, router)
                .with_graceful_shutdown(async {
                    let _ = tokio::signal::ctrl_c().await;
                    info!("Shutdown signal received");
                })
                .await?;
        }
        config::TransportType::Stdio => {
            use rmcp::ServiceExt;

            info!("Discord MCP server listening (STDIO)");

            let server = Server::new(discord_http, config.enabled_tools);
            let (stdin, stdout) = rmcp::transport::io::stdio();

            let running_service = server.serve((stdin, stdout)).await?;
            let cancel_token = running_service.cancellation_token();

            tokio::spawn(async move {
                let _ = tokio::signal::ctrl_c().await;
                info!("Shutdown signal received");
                cancel_token.cancel();
            });

            running_service.waiting().await?;
        }
    }

    Ok(())
}
