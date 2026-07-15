use rmcp::model::ErrorData;

mod guild;
mod user;

pub(crate) fn parse_snowflake(param_name: &'static str, value: &str) -> Result<u64, ErrorData> {
    value.parse::<u64>().map_err(|_| {
        ErrorData::invalid_params(
            format!("Parameter '{param_name}' must be a valid Discord snowflake string."),
            None,
        )
    })
}
