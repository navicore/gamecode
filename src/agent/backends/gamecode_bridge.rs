use async_trait::async_trait;
use gamecode_backend::{LLMBackend, ChatRequest, Message as BackendMessage, Tool as BackendTool, BackendError, RetryConfig, MessageRole};
use gamecode_bedrock::BedrockBackend;
use gamecode_tools::jsonrpc::Dispatcher;
use serde_json::Value;
use std::collections::HashMap;
use tracing::{error, trace};
use uuid::Uuid;

use super::{Backend, BackendCore, BackendResponse, ToolUse};

/// Bridge that adapts our modular gamecode architecture to the existing desktop UI interface
pub struct GamecodeBridge {
    /// The modular gamecode backend
    backend: BedrockBackend,
    
    /// Gamecode tools dispatcher
    tool_dispatcher: Dispatcher,
    
    /// Tool schema registry (same instance as dispatcher)
    tool_schema_registry: gamecode_tools::schema::ToolSchemaRegistry,
    
    /// Current session ID
    session_id: Uuid,
    
    /// Retry configuration
    retry_config: RetryConfig,
}

impl GamecodeBridge {
    pub async fn new(region: &str, profile: Option<String>) -> Result<Self, BackendError> {
        let backend = BedrockBackend::new().await.map_err(|e| BackendError::NetworkError { message: e.to_string() })?;
        let (tool_dispatcher, tool_schema_registry) = gamecode_tools::create_bedrock_dispatcher_with_schemas();
        let session_id = Uuid::new_v4();
        let retry_config = RetryConfig::default();
        
        Ok(Self {
            backend,
            tool_dispatcher,
            tool_schema_registry,
            session_id,
            retry_config,
        })
    }
    
    /// Convert desktop UI message format to backend message format
    fn convert_to_backend_message(role: &str, content: &str) -> BackendMessage {
        let message_role = match role {
            "user" => MessageRole::User,
            "assistant" => MessageRole::Assistant,
            "system" => MessageRole::System,
            _ => MessageRole::User, // Default fallback
        };
        BackendMessage::text(message_role, content)
    }
    
    /// Convert backend tool calls to UI format
    fn convert_tool_calls_to_ui(&self, tool_calls: &[gamecode_backend::ToolCall]) -> Vec<ToolUse> {
        tool_calls.iter().map(|tc| {
            // Convert JSON Value to HashMap<String, Value> for UI compatibility
            let mut args_map = HashMap::new();
            if let Some(obj) = tc.input.as_object() {
                for (k, v) in obj {
                    args_map.insert(k.clone(), v.clone());
                }
            }
            
            ToolUse {
                name: tc.name.clone(),
                args: args_map,
                id: Some(tc.id.clone()),
            }
        }).collect()
    }
    
    /// Execute a tool using the gamecode-tools JSONRPC dispatcher
    pub async fn execute_tool(&self, tool_name: &str, arguments: &Value) -> Result<String, String> {
        trace!("Executing tool: {} with args: {}", tool_name, arguments);
        
        // Create JSONRPC request
        let jsonrpc_request = serde_json::json!({
            "jsonrpc": "2.0",
            "method": tool_name,
            "params": arguments,
            "id": 1
        });
        
        let request_str = serde_json::to_string(&jsonrpc_request)
            .map_err(|e| format!("Failed to serialize JSONRPC request: {}", e))?;
        
        // Execute via dispatcher
        match self.tool_dispatcher.dispatch(&request_str).await {
            Ok(response) => {
                trace!("Tool {} executed successfully", tool_name);
                Ok(response)
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
        let messages = vec![Self::convert_to_backend_message("user", prompt)];
        
        // Get available tools from our stored schema registry
        let tool_specs = self.tool_schema_registry.to_bedrock_specs();
        
        let tools: Vec<BackendTool> = tool_specs.into_iter()
            .map(|spec| BackendTool {
                name: spec.name,
                description: spec.description,
                input_schema: spec.input_schema.json, // Extract the JSON Value from BedrockInputSchema
            })
            .collect();
        
        let request = ChatRequest {
            messages,
            model: None, // Let backend choose the model
            tools: Some(tools),
            inference_config: None, // Use backend defaults
            session_id: Some(self.session_id),
            status_callback: None, // Status handled elsewhere
        };
        
        match self.backend.chat_with_retry(request, self.retry_config.clone()).await {
            Ok(response) => {
                // Convert tool calls from backend format to UI format
                let tool_calls = self.convert_tool_calls_to_ui(&response.tool_calls);
                
                // Extract text content from message
                let content = response.message.content.iter()
                    .filter_map(|block| match block {
                        gamecode_backend::ContentBlock::Text(text) => Some(text.clone()),
                        _ => None,
                    })
                    .collect::<Vec<_>>()
                    .join("");
                
                let backend_response = BackendResponse {
                    content,
                    model: response.model,
                    tokens_used: response.usage.map(|u| u.total_tokens as usize),
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

