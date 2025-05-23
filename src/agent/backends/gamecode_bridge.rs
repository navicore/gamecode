use async_trait::async_trait;
use gamecode_backend::{LLMBackend, ChatRequest, ChatResponse, Message as BackendMessage, Tool as BackendTool, BackendError, StatusCallback};
use gamecode_bedrock::BedrockBackend;
use gamecode_tools::{ToolRegistry as GamecodeToolRegistry, ToolResult as GamecodeToolResult};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{error, info, trace, warn};
use uuid::Uuid;

use super::{Backend, BackendCore, BackendResponse, ToolUse};

/// Bridge that adapts our modular gamecode architecture to the existing desktop UI interface
pub struct GamecodeBridge {
    /// The modular gamecode backend
    backend: BedrockBackend,
    
    /// Gamecode tools registry
    tools: GamecodeToolRegistry,
    
    /// Current session ID
    session_id: Uuid,
    
    /// Status callback for UI feedback
    status_callback: Option<StatusCallback>,
}

impl GamecodeBridge {
    pub fn new(region: &str, profile: Option<String>) -> Result<Self, BackendError> {
        let backend = BedrockBackend::new(region.to_string(), profile)?;
        let tools = GamecodeToolRegistry::new();
        let session_id = Uuid::new_v4();
        
        Ok(Self {
            backend,
            tools,
            session_id,
            status_callback: None,
        })
    }
    
    pub fn with_status_callback(mut self, callback: StatusCallback) -> Self {
        self.status_callback = Some(callback);
        self
    }
    
    /// Convert desktop UI message format to backend message format
    fn convert_to_backend_message(role: &str, content: &str) -> BackendMessage {
        BackendMessage {
            role: role.to_string(),
            content: content.to_string(),
        }
    }
    
    /// Parse tool calls from the backend response content
    fn parse_tool_calls_from_response(&self, content: &str) -> Vec<ToolUse> {
        let mut tool_calls = Vec::new();
        
        // Try to parse tool calls from the response content
        // This depends on the format returned by the backend
        if let Ok(parsed) = serde_json::from_str::<Value>(content) {
            if let Some(tools) = parsed.get("tool_calls").and_then(|t| t.as_array()) {
                for tool in tools {
                    if let (Some(name), Some(args)) = (
                        tool.get("name").and_then(|n| n.as_str()),
                        tool.get("arguments")
                    ) {
                        // Convert to HashMap format expected by ToolUse
                        let mut args_map = HashMap::new();
                        if let Some(obj) = args.as_object() {
                            for (k, v) in obj {
                                args_map.insert(k.clone(), v.to_string());
                            }
                        }
                        
                        tool_calls.push(ToolUse {
                            name: name.to_string(),
                            args: args_map,
                            id: tool.get("id").and_then(|id| id.as_str()).map(|s| s.to_string()),
                        });
                    }
                }
            }
        }
        
        tool_calls
    }
    
    /// Execute a tool using the gamecode-tools registry
    pub async fn execute_tool(&self, tool_name: &str, arguments: &Value) -> Result<String, String> {
        trace!("Executing tool: {} with args: {}", tool_name, arguments);
        
        match self.tools.execute_tool(tool_name, arguments.clone()).await {
            Ok(GamecodeToolResult::Success(result)) => {
                info!("Tool {} executed successfully", tool_name);
                Ok(result)
            }
            Ok(GamecodeToolResult::Error(err)) => {
                warn!("Tool {} failed: {}", tool_name, err);
                Err(err)
            }
            Err(e) => {
                error!("Tool execution error: {}", e);
                Err(format!("Tool execution error: {}", e))
            }
        }
    }
}

#[async_trait]
impl Backend for GamecodeBridge {
    async fn generate_response(&self, prompt: &str) -> Result<BackendResponse, String> {
        trace!("Generating response for prompt: {} chars", prompt.len());
        
        // Parse the context to extract messages - assume it's formatted properly
        let messages = vec![BackendMessage {
            role: "user".to_string(),
            content: prompt.to_string(),
        }];
        
        // Get available tools from gamecode-tools
        let tools: Vec<BackendTool> = self.tools.get_all_tool_schemas().into_iter()
            .map(|schema| BackendTool {
                name: schema.name,
                description: schema.description,
                parameters: schema.parameters,
            })
            .collect();
        
        let request = ChatRequest {
            messages,
            tools: Some(tools),
            session_id: Some(self.session_id.to_string()),
        };
        
        match self.backend.chat_with_retry(request, self.status_callback.as_ref()).await {
            Ok(response) => {
                // Parse tool calls from response
                let tool_calls = self.parse_tool_calls_from_response(&response.content);
                
                let backend_response = BackendResponse {
                    content: response.content,
                    model: "claude-3-7-sonnet".to_string(),
                    tokens_used: None,
                    tool_calls,
                };
                
                trace!("Generated response: {} chars, {} tool calls", 
                    backend_response.content.len(), 
                    backend_response.tool_calls.len());
                
                Ok(backend_response)
            }
            Err(e) => {
                error!("Backend error: {}", e);
                Err(format!("Backend error: {}", e))
            }
        }
    }
}

impl BackendCore for GamecodeBridge {
    fn name(&self) -> &'static str {
        "GameCode Modular Backend"
    }
    
    fn context_window(&self) -> usize {
        200000 // Claude 3.7 context length
    }
}

