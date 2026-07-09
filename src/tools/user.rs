use rmcp::{
    model::{CallToolResult, ErrorData},
    tool, tool_router,
};

use crate::server::{Server, structured};

#[tool_router(router = user_router, vis = "pub(crate)")]
impl Server {
    #[tool(description = "Get information about the currently authenticated user.")]
    async fn get_current_user(&self) -> Result<CallToolResult, ErrorData> {
        structured(self.bot_http().get_current_user().await.map_err(|error| {
            ErrorData::internal_error(format!("Failed to fetch user: {error}"), None)
        })?)
    }
}
