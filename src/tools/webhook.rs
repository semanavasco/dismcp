//! Tool definitions for webhook-related Discord operations.

use rmcp::{
    handler::server::wrapper::Parameters,
    model::{CallToolResult, ErrorData},
    tool, tool_router,
};
use schemars::JsonSchema;
use serde::Deserialize;
use serde_json::{Map, Value};
use serenity::model::id::{ChannelId, GuildId, WebhookId};

use crate::{
    server::{Server, structured},
    tools::{ChannelIdParams, GuildIdParams, parse_snowflake},
};

#[derive(Debug, Deserialize, JsonSchema)]
struct WebhookIdParams {
    #[schemars(description = "Webhook ID (snowflake, as string).")]
    webhook_id: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct CreateWebhookParams {
    #[schemars(description = "Channel ID (snowflake, as string).")]
    channel_id: String,
    #[schemars(description = "Webhook name (1-80 characters).")]
    name: String,
    #[schemars(description = "Webhook avatar (data URI scheme).")]
    avatar_data_uri: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct EditWebhookParams {
    #[schemars(description = "Webhook ID (snowflake, as string).")]
    webhook_id: String,
    #[schemars(description = "Webhook name (1-80 characters).")]
    name: Option<String>,
    #[schemars(description = "Webhook avatar (data URI scheme).")]
    avatar_data_uri: Option<String>,
    #[schemars(description = "Channel ID to move the webhook to (snowflake, as string).")]
    channel_id: Option<String>,
}

#[tool_router(router = webhook_router, vis = "pub(crate)")]
impl Server {
    #[tool(description = "List all webhooks in a specific channel.")]
    async fn get_channel_webhooks(
        &self,
        Parameters(ChannelIdParams { channel_id }): Parameters<ChannelIdParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let channel_id = ChannelId::new(parse_snowflake("channel_id", &channel_id)?);

        structured(
            self.bot_http()
                .get_channel_webhooks(channel_id)
                .await
                .map_err(|error| {
                    ErrorData::internal_error(
                        format!("Failed to fetch channel webhooks: {error}"),
                        None,
                    )
                })?,
        )
    }

    #[tool(description = "List all webhooks in a specific guild.")]
    async fn get_guild_webhooks(
        &self,
        Parameters(GuildIdParams { guild_id }): Parameters<GuildIdParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let guild_id = GuildId::new(parse_snowflake("guild_id", &guild_id)?);

        structured(
            self.bot_http()
                .get_guild_webhooks(guild_id)
                .await
                .map_err(|error| {
                    ErrorData::internal_error(
                        format!("Failed to fetch guild webhooks: {error}"),
                        None,
                    )
                })?,
        )
    }

    #[tool(description = "Get details for a specific webhook by ID.")]
    async fn get_webhook(
        &self,
        Parameters(WebhookIdParams { webhook_id }): Parameters<WebhookIdParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let webhook_id = WebhookId::new(parse_snowflake("webhook_id", &webhook_id)?);

        structured(
            self.bot_http()
                .get_webhook(webhook_id)
                .await
                .map_err(|error| {
                    ErrorData::internal_error(format!("Failed to fetch webhook: {error}"), None)
                })?,
        )
    }

    #[tool(description = "Create a new webhook for a specific channel.")]
    async fn create_webhook(
        &self,
        Parameters(CreateWebhookParams {
            channel_id,
            name,
            avatar_data_uri,
        }): Parameters<CreateWebhookParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let channel_id = ChannelId::new(parse_snowflake("channel_id", &channel_id)?);

        let mut body = Map::new();
        body.insert("name".to_string(), Value::String(name));

        if let Some(avatar) = avatar_data_uri {
            body.insert("avatar".to_string(), Value::String(avatar));
        }

        structured(
            self.bot_http()
                .create_webhook(channel_id, &Value::Object(body), None)
                .await
                .map_err(|error| {
                    ErrorData::internal_error(format!("Failed to create webhook: {error}"), None)
                })?,
        )
    }

    #[tool(description = "Edit an existing webhook by ID.")]
    async fn edit_webhook(
        &self,
        Parameters(EditWebhookParams {
            webhook_id,
            name,
            avatar_data_uri,
            channel_id,
        }): Parameters<EditWebhookParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let webhook_id = WebhookId::new(parse_snowflake("webhook_id", &webhook_id)?);

        if name.is_none() && avatar_data_uri.is_none() && channel_id.is_none() {
            return Err(ErrorData::invalid_params(
                "Provide at least one field to update.",
                None,
            ));
        }

        let mut body = Map::new();

        if let Some(name) = name {
            body.insert("name".to_string(), Value::String(name));
        }
        if let Some(avatar) = avatar_data_uri {
            body.insert("avatar".to_string(), Value::String(avatar));
        }
        if let Some(channel_id) = channel_id {
            body.insert(
                "channel_id".to_string(),
                Value::Number(parse_snowflake("channel_id", &channel_id)?.into()),
            );
        }

        structured(
            self.bot_http()
                .edit_webhook(webhook_id, &Value::Object(body), None)
                .await
                .map_err(|error| {
                    ErrorData::internal_error(format!("Failed to edit webhook: {error}"), None)
                })?,
        )
    }

    #[tool(description = "Delete a webhook by ID.")]
    async fn delete_webhook(
        &self,
        Parameters(WebhookIdParams { webhook_id }): Parameters<WebhookIdParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let webhook_id = WebhookId::new(parse_snowflake("webhook_id", &webhook_id)?);

        self.bot_http()
            .delete_webhook(webhook_id, None)
            .await
            .map_err(|error| {
                ErrorData::internal_error(format!("Failed to delete webhook: {error}"), None)
            })?;

        structured(())
    }
}
