// Conditional compilation - only include legacy backend if AWS features are enabled
#[cfg(feature = "aws-legacy")]
mod bedrock;

mod gamecode_bridge;

// Export types - conditionally from legacy backend or define locally
#[cfg(feature = "aws-legacy")]
pub use bedrock::{BedrockModel, ToolUse};

// Define ToolUse locally for compatibility
#[cfg(not(feature = "aws-legacy"))]
#[derive(Debug, Clone)]
pub struct ToolUse {
    /// Tool name
    pub name: String,
    /// Tool arguments as JSON
    pub args: std::collections::HashMap<String, serde_json::Value>,
    /// Tool call ID (from Claude response)
    pub id: Option<String>,
}

// Always export the new bridge
pub use gamecode_bridge::GamecodeBridge;

// Define BedrockModel locally for compatibility
#[cfg(not(feature = "aws-legacy"))]
#[derive(Clone, Copy, Debug)]
pub enum BedrockModel {
    Claude37Sonnet,
    Claude35Haiku,
}
use tracing::trace;

/// Initialize all available backends
pub fn init() {
    trace!("Initializing agent backends...");
}

/// Trait defining a language model backend core functionality
pub trait BackendCore: Send + Sync {
    /// Get the backend's name
    fn name(&self) -> &'static str;

    /// Get the backend's context window size
    fn context_window(&self) -> usize;
}

/// Trait defining the async operations for the backend
#[async_trait::async_trait]
pub trait Backend: BackendCore {
    /// Generate a response from the given prompt
    async fn generate_response(&self, prompt: &str) -> Result<BackendResponse, String>;
}

/// Structure containing a response from an LLM backend
#[derive(Default, Clone)]
pub struct BackendResponse {
    /// The text content of the response
    pub content: String,

    /// Model used for generation
    pub model: String,

    /// Tokens used in this request and response
    pub tokens_used: Option<usize>,

    /// Tool calls extracted from the response (if any)
    pub tool_calls: Vec<ToolUse>,
}
