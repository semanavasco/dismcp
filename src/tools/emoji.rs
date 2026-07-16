//! Tool definitions for emoji-related Discord operations.

use rmcp::{
    handler::server::wrapper::Parameters,
    model::{CallToolResult, ErrorData},
    tool, tool_router,
};
use schemars::JsonSchema;
use serde::Deserialize;
use serde_json::{Value, json};
use serenity::model::id::{EmojiId, GuildId, RoleId};

use crate::{
    server::{Server, structured},
    tools::{GuildIdParams, parse_snowflake},
};

#[derive(Debug, Deserialize, JsonSchema)]
struct GuildEmojiParams {
    #[schemars(description = "Guild ID (snowflake, as string).")]
    guild_id: String,
    #[schemars(description = "Emoji ID (snowflake, as string).")]
    emoji_id: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct ApplicationEmojiParams {
    #[schemars(description = "Application emoji ID (snowflake, as string).")]
    emoji_id: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct CreateGuildEmojiParams {
    #[schemars(description = "Guild ID (snowflake, as string).")]
    guild_id: String,
    #[schemars(description = "Emoji name (2-32 characters).")]
    name: String,
    #[schemars(description = "Emoji image as a data URI (e.g. data:image/png;base64,....).")]
    image_data_uri: String,
    #[schemars(
        description = "Optional role IDs allowed to use this emoji (snowflakes, as strings)."
    )]
    role_ids: Option<Vec<String>>,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct EditGuildEmojiParams {
    #[schemars(description = "Guild ID (snowflake, as string).")]
    guild_id: String,
    #[schemars(description = "Emoji ID (snowflake, as string).")]
    emoji_id: String,
    #[schemars(description = "Emoji name (2-32 characters).")]
    name: String,
    #[schemars(
        description = "Optional role IDs allowed to use this emoji (snowflakes, as strings)."
    )]
    role_ids: Option<Vec<String>>,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct CreateApplicationEmojiParams {
    #[schemars(description = "Emoji name (2-32 characters).")]
    name: String,
    #[schemars(description = "Emoji image as a data URI (e.g. data:image/png;base64,....).")]
    image_data_uri: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct EditApplicationEmojiParams {
    #[schemars(description = "Application emoji ID (snowflake, as string).")]
    emoji_id: String,
    #[schemars(description = "Emoji name (2-32 characters).")]
    name: String,
}

fn validate_emoji_name(name: &str) -> Result<(), ErrorData> {
    if !(2..=32).contains(&name.len()) {
        return Err(ErrorData::invalid_params(
            "Parameter 'name' must be between 2 and 32 characters.",
            None,
        ));
    }
    Ok(())
}

fn validate_image_data_uri(value: &str) -> Result<(), ErrorData> {
    if !value.starts_with("data:image/") || !value.contains(";base64,") {
        return Err(ErrorData::invalid_params(
            "Parameter 'image_data_uri' must be a base64 image data URI.",
            None,
        ));
    }
    Ok(())
}

fn parse_role_ids(role_ids: Option<Vec<String>>) -> Result<Option<Vec<RoleId>>, ErrorData> {
    role_ids
        .map(|role_ids| {
            role_ids
                .into_iter()
                .map(|role_id| parse_snowflake("role_ids[]", &role_id).map(RoleId::new))
                .collect::<Result<Vec<_>, _>>()
        })
        .transpose()
}

#[tool_router(router = emoji_router, vis = "pub(crate)")]
impl Server {
    #[tool(description = "List emojis in a guild.")]
    async fn get_guild_emojis(
        &self,
        Parameters(GuildIdParams { guild_id }): Parameters<GuildIdParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let guild_id = GuildId::new(parse_snowflake("guild_id", &guild_id)?);

        structured(
            self.bot_http()
                .get_emojis(guild_id)
                .await
                .map_err(|error| {
                    ErrorData::internal_error(
                        format!("Failed to fetch guild emojis: {error}"),
                        None,
                    )
                })?,
        )
    }

    #[tool(description = "Get details for a guild emoji.")]
    async fn get_guild_emoji(
        &self,
        Parameters(GuildEmojiParams { guild_id, emoji_id }): Parameters<GuildEmojiParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let guild_id = GuildId::new(parse_snowflake("guild_id", &guild_id)?);
        let emoji_id = EmojiId::new(parse_snowflake("emoji_id", &emoji_id)?);

        structured(
            self.bot_http()
                .get_emoji(guild_id, emoji_id)
                .await
                .map_err(|error| {
                    ErrorData::internal_error(format!("Failed to fetch guild emoji: {error}"), None)
                })?,
        )
    }

    #[tool(description = "Create a guild emoji from a data URI image.")]
    async fn create_guild_emoji(
        &self,
        Parameters(CreateGuildEmojiParams {
            guild_id,
            name,
            image_data_uri,
            role_ids,
        }): Parameters<CreateGuildEmojiParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let guild_id = GuildId::new(parse_snowflake("guild_id", &guild_id)?);
        validate_emoji_name(&name)?;
        validate_image_data_uri(&image_data_uri)?;
        let roles = parse_role_ids(role_ids)?;

        let mut body = json!({
            "name": name,
            "image": image_data_uri,
        });

        if let Some(roles) = roles {
            body["roles"] = serde_json::to_value(roles).map_err(|error| {
                ErrorData::internal_error(format!("Failed to serialize emoji roles: {error}"), None)
            })?;
        }

        structured(
            self.bot_http()
                .create_emoji(guild_id, &body, None)
                .await
                .map_err(|error| {
                    ErrorData::internal_error(
                        format!("Failed to create guild emoji: {error}"),
                        None,
                    )
                })?,
        )
    }

    #[tool(description = "Edit a guild emoji.")]
    async fn edit_guild_emoji(
        &self,
        Parameters(EditGuildEmojiParams {
            guild_id,
            emoji_id,
            name,
            role_ids,
        }): Parameters<EditGuildEmojiParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let guild_id = GuildId::new(parse_snowflake("guild_id", &guild_id)?);
        let emoji_id = EmojiId::new(parse_snowflake("emoji_id", &emoji_id)?);
        validate_emoji_name(&name)?;
        let roles = parse_role_ids(role_ids)?;

        let mut body = json!({
            "name": name,
        });

        if let Some(roles) = roles {
            body["roles"] = serde_json::to_value(roles).map_err(|error| {
                ErrorData::internal_error(format!("Failed to serialize emoji roles: {error}"), None)
            })?;
        }

        structured(
            self.bot_http()
                .edit_emoji(guild_id, emoji_id, &body, None)
                .await
                .map_err(|error| {
                    ErrorData::internal_error(format!("Failed to edit guild emoji: {error}"), None)
                })?,
        )
    }

    #[tool(description = "Delete a guild emoji.")]
    async fn delete_guild_emoji(
        &self,
        Parameters(GuildEmojiParams { guild_id, emoji_id }): Parameters<GuildEmojiParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let guild_id = GuildId::new(parse_snowflake("guild_id", &guild_id)?);
        let emoji_id = EmojiId::new(parse_snowflake("emoji_id", &emoji_id)?);

        self.bot_http()
            .delete_emoji(guild_id, emoji_id, None)
            .await
            .map_err(|error| {
                ErrorData::internal_error(format!("Failed to delete guild emoji: {error}"), None)
            })?;

        structured(json!({
            "deleted": true,
            "guild_id": guild_id,
            "emoji_id": emoji_id,
        }))
    }

    #[tool(description = "List application emojis.")]
    async fn get_application_emojis(&self) -> Result<CallToolResult, ErrorData> {
        structured(
            self.bot_http()
                .get_application_emojis()
                .await
                .map_err(|error| {
                    ErrorData::internal_error(
                        format!("Failed to fetch application emojis: {error}"),
                        None,
                    )
                })?,
        )
    }

    #[tool(description = "Get details for an application emoji.")]
    async fn get_application_emoji(
        &self,
        Parameters(ApplicationEmojiParams { emoji_id }): Parameters<ApplicationEmojiParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let emoji_id = EmojiId::new(parse_snowflake("emoji_id", &emoji_id)?);

        structured(
            self.bot_http()
                .get_application_emoji(emoji_id)
                .await
                .map_err(|error| {
                    ErrorData::internal_error(
                        format!("Failed to fetch application emoji: {error}"),
                        None,
                    )
                })?,
        )
    }

    #[tool(description = "Create an application emoji from a data URI image.")]
    async fn create_application_emoji(
        &self,
        Parameters(CreateApplicationEmojiParams {
            name,
            image_data_uri,
        }): Parameters<CreateApplicationEmojiParams>,
    ) -> Result<CallToolResult, ErrorData> {
        validate_emoji_name(&name)?;
        validate_image_data_uri(&image_data_uri)?;
        let body = json!({
            "name": name,
            "image": image_data_uri,
        });

        structured(
            self.bot_http()
                .create_application_emoji(&body)
                .await
                .map_err(|error| {
                    ErrorData::internal_error(
                        format!("Failed to create application emoji: {error}"),
                        None,
                    )
                })?,
        )
    }

    #[tool(description = "Edit an application emoji.")]
    async fn edit_application_emoji(
        &self,
        Parameters(EditApplicationEmojiParams { emoji_id, name }): Parameters<
            EditApplicationEmojiParams,
        >,
    ) -> Result<CallToolResult, ErrorData> {
        let emoji_id = EmojiId::new(parse_snowflake("emoji_id", &emoji_id)?);
        validate_emoji_name(&name)?;
        let body: Value = json!({
            "name": name,
        });

        structured(
            self.bot_http()
                .edit_application_emoji(emoji_id, &body)
                .await
                .map_err(|error| {
                    ErrorData::internal_error(
                        format!("Failed to edit application emoji: {error}"),
                        None,
                    )
                })?,
        )
    }

    #[tool(description = "Delete an application emoji.")]
    async fn delete_application_emoji(
        &self,
        Parameters(ApplicationEmojiParams { emoji_id }): Parameters<ApplicationEmojiParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let emoji_id = EmojiId::new(parse_snowflake("emoji_id", &emoji_id)?);

        self.bot_http()
            .delete_application_emoji(emoji_id)
            .await
            .map_err(|error| {
                ErrorData::internal_error(
                    format!("Failed to delete application emoji: {error}"),
                    None,
                )
            })?;

        structured(json!({
            "deleted": true,
            "emoji_id": emoji_id,
            "scope": "application",
        }))
    }
}
