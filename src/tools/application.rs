use rmcp::{
    model::{CallToolResult, ErrorData},
    tool, tool_router,
};

use crate::server::{Server, structured};

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
}
