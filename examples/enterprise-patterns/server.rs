// Static scan fixture.
use rmcp::{tool, tool_handler, ServerHandler};
struct Demo;
impl Demo {
    #[tool(description="Ping")]
    fn ping(&self) -> String { "pong".into() }
}
#[tool_handler]
impl ServerHandler for Demo {}
