//! Discord MCP tool implementations and helpers.
//!
//! This module contains all the discrete tools exposed by the MCP server, grouped
//! into submodules by category (e.g., channels, guilds, messages). It also provides
//! shared parameter structs and parsing utilities used across multiple tools.

use rmcp::model::ErrorData;
use schemars::JsonSchema;
use serde::Deserialize;

mod application;
mod channel;
mod emoji;
mod guild;
mod member;
mod message;
mod role;
mod user;
mod webhook;

#[derive(Debug, Deserialize, JsonSchema)]
pub(crate) struct GuildIdParams {
    #[schemars(description = "Guild ID (snowflake, as string).")]
    pub(crate) guild_id: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub(crate) struct ChannelIdParams {
    #[schemars(description = "Channel ID (snowflake, as string).")]
    pub(crate) channel_id: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub(crate) struct UserIdParams {
    #[schemars(description = "User ID (snowflake, as string).")]
    pub(crate) user_id: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub(crate) struct GuildUserParams {
    #[schemars(description = "Guild ID (snowflake, as string).")]
    pub(crate) guild_id: String,
    #[schemars(description = "User ID (snowflake, as string).")]
    pub(crate) user_id: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub(crate) struct ChannelMessageIdParams {
    #[schemars(description = "Channel ID (snowflake, as string).")]
    pub(crate) channel_id: String,
    #[schemars(description = "Message ID (snowflake, as string).")]
    pub(crate) message_id: String,
}

/// Parses a string representing a Discord snowflake ID into a `u64`.
///
/// Returns an MCP `ErrorData` result if the string is not a valid `u64`, making it easy
/// to return `invalid_params` directly to the MCP client.
pub(crate) fn parse_snowflake(param_name: &'static str, value: &str) -> Result<u64, ErrorData> {
    value.parse::<u64>().map_err(|_| {
        ErrorData::invalid_params(
            format!("Parameter '{param_name}' must be a valid Discord snowflake string."),
            None,
        )
    })
}

/// Normalizes a search query string by trimming whitespace and converting to lowercase.
pub(crate) fn normalize_query(value: &str) -> String {
    value.trim().to_lowercase()
}

/// Checks if a string value matches a query.
///
/// If `exact` is true, performs an exact equality check. Otherwise, performs a substring match.
pub(crate) fn matches_query(value: &str, query: &str, exact: bool) -> bool {
    if exact {
        value == query
    } else {
        value.contains(query)
    }
}

/// Parses a list of strings representing Discord snowflake IDs into a `Vec<u64>`.
pub(crate) fn parse_id_list(
    param_name: &'static str,
    values: Vec<String>,
) -> Result<Vec<u64>, ErrorData> {
    values
        .into_iter()
        .map(|value| parse_snowflake(param_name, &value))
        .collect()
}
