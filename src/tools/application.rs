//! Tool definitions for application-related Discord operations.

use base64::Engine;
use rmcp::{
    handler::server::wrapper::Parameters,
    model::{CallToolResult, ErrorData},
    tool, tool_router,
};
use schemars::JsonSchema;
use serde::Deserialize;
use serenity::{
    builder::{CreateAttachment, EditProfile},
    model::user::CurrentUser,
};

use crate::server::{Server, structured};

#[derive(Debug, Deserialize, JsonSchema)]
struct EditBotProfileParams {
    #[schemars(description = "New bot username.")]
    username: Option<String>,
    #[schemars(
        description = "Avatar image as a data URI (e.g. data:image/png;base64,<BASE64_DATA>)."
    )]
    avatar_data_uri: Option<String>,
    #[schemars(
        description = "Banner image as a data URI (e.g. data:image/png;base64,<BASE64_DATA>)."
    )]
    banner_data_uri: Option<String>,
    #[schemars(description = "Delete the current avatar. Defaults to false.")]
    delete_avatar: Option<bool>,
    #[schemars(description = "Delete the current banner. Defaults to false.")]
    delete_banner: Option<bool>,
}

fn parse_image_data_uri(
    field_name: &'static str,
    value: &str,
) -> Result<CreateAttachment, ErrorData> {
    let (mime, encoded) = value.split_once(";base64,").ok_or_else(|| {
        ErrorData::invalid_params(
            format!("Parameter '{field_name}' must be a base64 data URI."),
            None,
        )
    })?;

    let mime = mime.strip_prefix("data:").ok_or_else(|| {
        ErrorData::invalid_params(
            format!("Parameter '{field_name}' must start with 'data:image/'."),
            None,
        )
    })?;

    if !mime.starts_with("image/") {
        return Err(ErrorData::invalid_params(
            format!("Parameter '{field_name}' must be an image data URI."),
            None,
        ));
    }

    let extension = mime.rsplit('/').next().unwrap_or("png");
    let filename = format!("discord-{}.{extension}", field_name.replace('_', "-"));
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(encoded)
        .map_err(|_| {
            ErrorData::invalid_params(
                format!("Parameter '{field_name}' contains invalid base64 data."),
                None,
            )
        })?;

    Ok(CreateAttachment::bytes(bytes, filename))
}

#[tool_router(router = application_router, vis = "pub(crate)")]
impl Server {
    #[tool(description = "Get information about the current Discord application.")]
    async fn get_current_application(&self) -> Result<CallToolResult, ErrorData> {
        structured(
            self.bot_http()
                .get_current_application_info()
                .await
                .map_err(|error| {
                    ErrorData::internal_error(
                        format!("Failed to fetch current application info: {error}"),
                        None,
                    )
                })?,
        )
    }

    #[tool(description = "Edit the current bot profile (username, avatar, banner).")]
    async fn edit_bot_profile(
        &self,
        Parameters(EditBotProfileParams {
            username,
            avatar_data_uri,
            banner_data_uri,
            delete_avatar,
            delete_banner,
        }): Parameters<EditBotProfileParams>,
    ) -> Result<CallToolResult, ErrorData> {
        if username.is_none()
            && avatar_data_uri.is_none()
            && banner_data_uri.is_none()
            && !delete_avatar.unwrap_or(false)
            && !delete_banner.unwrap_or(false)
        {
            return Err(ErrorData::invalid_params(
                "Provide at least one profile field to update.",
                None,
            ));
        }

        let mut profile = EditProfile::new();

        if let Some(username) = username {
            profile = profile.username(username);
        }

        if delete_avatar.unwrap_or(false) {
            profile = profile.delete_avatar();
        } else if let Some(avatar_data_uri) = avatar_data_uri {
            let attachment = parse_image_data_uri("avatar_data_uri", &avatar_data_uri)?;
            profile = profile.avatar(&attachment);
        }

        if delete_banner.unwrap_or(false) {
            profile = profile.delete_banner();
        } else if let Some(banner_data_uri) = banner_data_uri {
            let attachment = parse_image_data_uri("banner_data_uri", &banner_data_uri)?;
            profile = profile.banner(&attachment);
        }

        let current_user: CurrentUser =
            self.bot_http()
                .edit_profile(&profile)
                .await
                .map_err(|error| {
                    ErrorData::internal_error(format!("Failed to edit bot profile: {error}"), None)
                })?;

        structured(current_user)
    }
}
