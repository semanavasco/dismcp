//! Tool definitions for role-related Discord operations.

use rmcp::{
    handler::server::wrapper::Parameters,
    model::{CallToolResult, ErrorData},
    tool, tool_router,
};
use schemars::JsonSchema;
use serde::Deserialize;
use serde_json::{Map, Value, json};
use serenity::model::id::{GuildId, RoleId, UserId};

use crate::{
    server::{Server, structured},
    tools::{GuildIdParams, matches_query, normalize_query, parse_snowflake},
};

#[derive(Debug, Deserialize, JsonSchema)]
struct GuildRoleParams {
    #[schemars(description = "Guild ID (snowflake, as string).")]
    guild_id: String,
    #[schemars(description = "Role ID (snowflake, as string).")]
    role_id: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct CreateRoleParams {
    #[schemars(description = "Guild ID (snowflake, as string).")]
    guild_id: String,
    #[schemars(description = "Role name.")]
    name: Option<String>,
    #[schemars(description = "Role color as a decimal integer (0-16777215).")]
    color: Option<u64>,
    #[schemars(description = "Role permissions bitset as a stringified integer.")]
    permissions: Option<String>,
    #[schemars(description = "Whether the role is shown separately in member list.")]
    hoist: Option<bool>,
    #[schemars(description = "Whether the role is mentionable.")]
    mentionable: Option<bool>,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct EditRoleParams {
    #[schemars(description = "Guild ID (snowflake, as string).")]
    guild_id: String,
    #[schemars(description = "Role ID (snowflake, as string).")]
    role_id: String,
    #[schemars(description = "Role name.")]
    name: Option<String>,
    #[schemars(description = "Role color as a decimal integer (0-16777215).")]
    color: Option<u64>,
    #[schemars(description = "Role permissions bitset as a stringified integer.")]
    permissions: Option<String>,
    #[schemars(description = "Whether the role is shown separately in member list.")]
    hoist: Option<bool>,
    #[schemars(description = "Whether the role is mentionable.")]
    mentionable: Option<bool>,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct EditRolePositionParams {
    #[schemars(description = "Guild ID (snowflake, as string).")]
    guild_id: String,
    #[schemars(description = "Role ID (snowflake, as string).")]
    role_id: String,
    #[schemars(description = "Target role position.")]
    position: u64,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct GuildMemberRoleParams {
    #[schemars(description = "Guild ID (snowflake, as string).")]
    guild_id: String,
    #[schemars(description = "User ID (snowflake, as string).")]
    user_id: String,
    #[schemars(description = "Role ID (snowflake, as string).")]
    role_id: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct SearchGuildRolesParams {
    #[schemars(description = "Guild ID (snowflake, as string).")]
    guild_id: String,
    #[schemars(description = "Role name search query.")]
    query: String,
    #[schemars(description = "Exact name match instead of contains. Defaults to false.")]
    exact: Option<bool>,
}

fn build_role_payload(
    name: Option<String>,
    color: Option<u64>,
    permissions: Option<String>,
    hoist: Option<bool>,
    mentionable: Option<bool>,
) -> Result<Value, ErrorData> {
    if matches!(color, Some(value) if value > 0xFF_FF_FF) {
        return Err(ErrorData::invalid_params(
            "Parameter 'color' must be between 0 and 16777215.",
            None,
        ));
    }

    let mut map = Map::new();

    if let Some(name) = name {
        if name.is_empty() || name.len() > 100 {
            return Err(ErrorData::invalid_params(
                "Parameter 'name' must be between 1 and 100 characters.",
                None,
            ));
        }
        map.insert("name".to_string(), Value::String(name));
    }
    if let Some(color) = color {
        map.insert("color".to_string(), Value::Number(color.into()));
    }
    if let Some(permissions) = permissions {
        permissions.parse::<u64>().map_err(|_| {
            ErrorData::invalid_params(
                "Parameter 'permissions' must be a stringified unsigned integer.",
                None,
            )
        })?;
        map.insert("permissions".to_string(), Value::String(permissions));
    }
    if let Some(hoist) = hoist {
        map.insert("hoist".to_string(), Value::Bool(hoist));
    }
    if let Some(mentionable) = mentionable {
        map.insert("mentionable".to_string(), Value::Bool(mentionable));
    }

    Ok(Value::Object(map))
}

#[tool_router(router = role_router, vis = "pub(crate)")]
impl Server {
    #[tool(description = "List roles in a guild.")]
    async fn get_guild_roles(
        &self,
        Parameters(GuildIdParams { guild_id }): Parameters<GuildIdParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let guild_id = GuildId::new(parse_snowflake("guild_id", &guild_id)?);

        structured(
            self.bot_http()
                .get_guild_roles(guild_id)
                .await
                .map_err(|error| {
                    ErrorData::internal_error(format!("Failed to fetch guild roles: {error}"), None)
                })?,
        )
    }

    #[tool(description = "Get details for a role in a guild.")]
    async fn get_guild_role(
        &self,
        Parameters(GuildRoleParams { guild_id, role_id }): Parameters<GuildRoleParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let guild_id = GuildId::new(parse_snowflake("guild_id", &guild_id)?);
        let role_id = RoleId::new(parse_snowflake("role_id", &role_id)?);

        structured(
            self.bot_http()
                .get_guild_role(guild_id, role_id)
                .await
                .map_err(|error| {
                    ErrorData::internal_error(format!("Failed to fetch guild role: {error}"), None)
                })?,
        )
    }

    #[tool(description = "Search guild roles by name.")]
    async fn search_guild_roles(
        &self,
        Parameters(SearchGuildRolesParams {
            guild_id,
            query,
            exact,
        }): Parameters<SearchGuildRolesParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let guild_id = GuildId::new(parse_snowflake("guild_id", &guild_id)?);
        let exact = exact.unwrap_or(false);
        let query = normalize_query(&query);

        if query.is_empty() {
            return Err(ErrorData::invalid_params(
                "Parameter 'query' cannot be empty.",
                None,
            ));
        }

        let roles = self
            .bot_http()
            .get_guild_roles(guild_id)
            .await
            .map_err(|error| {
                ErrorData::internal_error(format!("Failed to fetch guild roles: {error}"), None)
            })?;

        let filtered: Vec<_> = roles
            .into_iter()
            .filter(|role| matches_query(&normalize_query(&role.name), &query, exact))
            .collect();

        structured(filtered)
    }

    #[tool(description = "Create a new role in a guild.")]
    async fn create_guild_role(
        &self,
        Parameters(CreateRoleParams {
            guild_id,
            name,
            color,
            permissions,
            hoist,
            mentionable,
        }): Parameters<CreateRoleParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let guild_id = GuildId::new(parse_snowflake("guild_id", &guild_id)?);
        let payload = build_role_payload(name, color, permissions, hoist, mentionable)?;

        structured(
            self.bot_http()
                .create_role(guild_id, &payload, None)
                .await
                .map_err(|error| {
                    ErrorData::internal_error(format!("Failed to create guild role: {error}"), None)
                })?,
        )
    }

    #[tool(description = "Edit an existing role in a guild.")]
    async fn edit_guild_role(
        &self,
        Parameters(EditRoleParams {
            guild_id,
            role_id,
            name,
            color,
            permissions,
            hoist,
            mentionable,
        }): Parameters<EditRoleParams>,
    ) -> Result<CallToolResult, ErrorData> {
        if name.is_none()
            && color.is_none()
            && permissions.is_none()
            && hoist.is_none()
            && mentionable.is_none()
        {
            return Err(ErrorData::invalid_params(
                "Provide at least one role field to update.",
                None,
            ));
        }

        let guild_id = GuildId::new(parse_snowflake("guild_id", &guild_id)?);
        let role_id = RoleId::new(parse_snowflake("role_id", &role_id)?);
        let payload = build_role_payload(name, color, permissions, hoist, mentionable)?;

        structured(
            self.bot_http()
                .edit_role(guild_id, role_id, &payload, None)
                .await
                .map_err(|error| {
                    ErrorData::internal_error(format!("Failed to edit guild role: {error}"), None)
                })?,
        )
    }

    #[tool(description = "Delete a role from a guild.")]
    async fn delete_guild_role(
        &self,
        Parameters(GuildRoleParams { guild_id, role_id }): Parameters<GuildRoleParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let guild_id = GuildId::new(parse_snowflake("guild_id", &guild_id)?);
        let role_id = RoleId::new(parse_snowflake("role_id", &role_id)?);

        self.bot_http()
            .delete_role(guild_id, role_id, None)
            .await
            .map_err(|error| {
                ErrorData::internal_error(format!("Failed to delete guild role: {error}"), None)
            })?;

        structured(json!({
            "deleted": true,
            "guild_id": guild_id,
            "role_id": role_id,
        }))
    }

    #[tool(description = "Edit a role position in a guild.")]
    async fn edit_guild_role_position(
        &self,
        Parameters(EditRolePositionParams {
            guild_id,
            role_id,
            position,
        }): Parameters<EditRolePositionParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let guild_id = GuildId::new(parse_snowflake("guild_id", &guild_id)?);
        let role_id = RoleId::new(parse_snowflake("role_id", &role_id)?);
        let position: u16 = position.try_into().map_err(|_| {
            ErrorData::invalid_params("Parameter 'position' must be between 0 and 65535.", None)
        })?;

        structured(
            self.bot_http()
                .edit_role_position(guild_id, role_id, position, None)
                .await
                .map_err(|error| {
                    ErrorData::internal_error(
                        format!("Failed to edit guild role position: {error}"),
                        None,
                    )
                })?,
        )
    }

    #[tool(description = "Assign a role to a guild member.")]
    async fn add_member_role(
        &self,
        Parameters(GuildMemberRoleParams {
            guild_id,
            user_id,
            role_id,
        }): Parameters<GuildMemberRoleParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let guild_id = GuildId::new(parse_snowflake("guild_id", &guild_id)?);
        let user_id = UserId::new(parse_snowflake("user_id", &user_id)?);
        let role_id = RoleId::new(parse_snowflake("role_id", &role_id)?);

        self.bot_http()
            .add_member_role(guild_id, user_id, role_id, None)
            .await
            .map_err(|error| {
                ErrorData::internal_error(format!("Failed to add member role: {error}"), None)
            })?;

        structured(json!({
            "updated": true,
            "action": "add",
            "guild_id": guild_id,
            "user_id": user_id,
            "role_id": role_id,
        }))
    }

    #[tool(description = "Remove a role from a guild member.")]
    async fn remove_member_role(
        &self,
        Parameters(GuildMemberRoleParams {
            guild_id,
            user_id,
            role_id,
        }): Parameters<GuildMemberRoleParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let guild_id = GuildId::new(parse_snowflake("guild_id", &guild_id)?);
        let user_id = UserId::new(parse_snowflake("user_id", &user_id)?);
        let role_id = RoleId::new(parse_snowflake("role_id", &role_id)?);

        self.bot_http()
            .remove_member_role(guild_id, user_id, role_id, None)
            .await
            .map_err(|error| {
                ErrorData::internal_error(format!("Failed to remove member role: {error}"), None)
            })?;

        structured(json!({
            "updated": true,
            "action": "remove",
            "guild_id": guild_id,
            "user_id": user_id,
            "role_id": role_id,
        }))
    }
}
