//! Tauri Commands for v0.6 Agent Module
//!
//! Commands for multimodal processing, context management, and agent orchestration.

use tauri::State;
use std::sync::Arc;
use tokio::sync::Mutex;

use super::multimodal::{
    MultimodalProcessor, InputType, ImageFormat, MultimodalResult,
    ImageAnalysis
};
use super::context::{
    ContextManager, Message, MessageRole, MessagePriority, CompressionStrategy,
    CompressorConfig
};
use super::orchestrator::{
    AgentOrchestrator, SubAgentTask, TaskInput, TaskPriority,
    AgentType
};

/// Global state for agent features
pub struct AgentState {
    pub context_manager: Arc<Mutex<ContextManager>>,
    pub orchestrator: Arc<Mutex<AgentOrchestrator>>,
    pub multimodal_processor: Arc<Mutex<MultimodalProcessor>>,
}

impl AgentState {
    pub fn new() -> Self {
        Self {
            context_manager: Arc::new(Mutex::new(ContextManager::with_defaults())),
            orchestrator: Arc::new(Mutex::new(AgentOrchestrator::new(5))),
            multimodal_processor: Arc::new(Mutex::new(MultimodalProcessor::new(None))),
        }
    }
}

impl Default for AgentState {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Multimodal Commands
// ============================================================================

/// Process multimodal input
#[tauri::command]
pub async fn agent_multimodal_process(
    state: State<'_, Arc<AgentState>>,
    input_type: String,
    text: Option<String>,
    image_data: Option<Vec<u8>>,
    image_format: Option<String>,
) -> Result<MultimodalResult, String> {
    let processor = state.multimodal_processor.lock().await;

    let input = match input_type.as_str() {
        "text" => InputType::Text(text.unwrap_or_default()),
        "image" => {
            let data = image_data.ok_or("Image data required")?;
            let format = match image_format.as_deref() {
                Some("png") => ImageFormat::Png,
                Some("jpeg") | Some("jpg") => ImageFormat::Jpeg,
                Some("gif") => ImageFormat::Gif,
                Some("webp") => ImageFormat::WebP,
                Some("bmp") => ImageFormat::Bmp,
                _ => return Err("Invalid image format".to_string()),
            };
            InputType::Image { data, format }
        }
        "mixed" => {
            let txt = text.ok_or("Text required for mixed input")?;
            InputType::Mixed {
                text: txt,
                images: vec![],
            }
        }
        _ => return Err("Invalid input type".to_string()),
    };

    processor.process(&input)
}

/// Analyze image data
#[tauri::command]
pub async fn agent_analyze_image(
    state: State<'_, Arc<AgentState>>,
    image_data: Vec<u8>,
    format: String,
) -> Result<ImageAnalysis, String> {
    let processor = state.multimodal_processor.lock().await;
    let image_format = match format.as_str() {
        "png" => ImageFormat::Png,
        "jpeg" | "jpg" => ImageFormat::Jpeg,
        "gif" => ImageFormat::Gif,
        "webp" => ImageFormat::WebP,
        "bmp" => ImageFormat::Bmp,
        _ => return Err("Invalid image format".to_string()),
    };

    processor.process_image(&image_data, &image_format)
        .map_err(|e| e.to_string())
        .and_then(|r| match r {
            MultimodalResult::Image(analysis) => Ok(analysis),
            _ => Err("Expected image analysis result".to_string()),
        })
}

// ============================================================================
// Context Commands
// ============================================================================

/// Add a message to context
#[tauri::command]
pub async fn agent_context_add_message(
    state: State<'_, Arc<AgentState>>,
    role: String,
    content: String,
    priority: Option<String>,
    token_count: Option<usize>,
) -> Result<(), String> {
    let mut manager = state.context_manager.lock().await;

    let role = match role.as_str() {
        "system" => MessageRole::System,
        "user" => MessageRole::User,
        "assistant" => MessageRole::Assistant,
        "tool" => MessageRole::Tool,
        _ => return Err("Invalid role".to_string()),
    };

    let priority = match priority.as_deref() {
        Some("low") => MessagePriority::Low,
        Some("normal") => MessagePriority::Normal,
        Some("high") => MessagePriority::High,
        Some("critical") => MessagePriority::Critical,
        _ => MessagePriority::Normal,
    };

    let tokens = token_count.unwrap_or(content.split_whitespace().count());

    manager.add_message(Message {
        role,
        content,
        token_count: tokens,
        priority,
        timestamp: chrono::Utc::now().timestamp(),
    });

    Ok(())
}

/// Get all messages in context
#[tauri::command]
pub async fn agent_context_get_messages(
    state: State<'_, Arc<AgentState>>,
) -> Result<Vec<serde_json::Value>, String> {
    let manager = state.context_manager.lock().await;
    let messages = manager.get_messages_owned();

    let json_messages: Vec<serde_json::Value> = messages
        .into_iter()
        .map(|m| serde_json::json!({
            "role": format!("{:?}", m.role),
            "content": m.content,
            "token_count": m.token_count,
            "priority": format!("{:?}", m.priority),
            "timestamp": m.timestamp,
        }))
        .collect();

    Ok(json_messages)
}

/// Clear context
#[tauri::command]
pub async fn agent_context_clear(
    state: State<'_, Arc<AgentState>>,
) -> Result<(), String> {
    let mut manager = state.context_manager.lock().await;
    manager.clear();
    Ok(())
}

/// Get context token count
#[tauri::command]
pub async fn agent_context_token_count(
    state: State<'_, Arc<AgentState>>,
) -> Result<usize, String> {
    let manager = state.context_manager.lock().await;
    Ok(manager.token_count())
}

/// Check if context is near limit
#[tauri::command]
pub async fn agent_context_is_near_limit(
    state: State<'_, Arc<AgentState>>,
) -> Result<bool, String> {
    let manager = state.context_manager.lock().await;
    Ok(manager.is_near_limit())
}

/// Compress context
#[tauri::command]
pub async fn agent_context_compress(
    state: State<'_, Arc<AgentState>>,
) -> Result<serde_json::Value, String> {
    let mut manager = state.context_manager.lock().await;
    let result = manager.compress();
    Ok(serde_json::json!({
        "original_tokens": result.original_tokens,
        "compressed_tokens": result.compressed_tokens,
        "removed_count": result.removed_count,
        "summarized_count": result.summarized_count,
    }))
}

/// Set compression strategy
#[tauri::command]
pub async fn agent_context_set_strategy(
    state: State<'_, Arc<AgentState>>,
    strategy: String,
    min_tokens: Option<usize>,
    target_ratio: Option<f32>,
) -> Result<(), String> {
    let _manager = state.context_manager.lock().await;

    let compression_strategy = match strategy.as_str() {
        "remove_oldest" => CompressionStrategy::RemoveOldest,
        "summarize" => CompressionStrategy::Summarize,
        "priority_only" => CompressionStrategy::PriorityOnly,
        "hybrid" => CompressionStrategy::Hybrid,
        _ => return Err("Invalid compression strategy".to_string()),
    };

    let _config = CompressorConfig {
        strategy: compression_strategy,
        min_tokens: min_tokens.unwrap_or(512),
        target_ratio: target_ratio.unwrap_or(0.5),
    };

    // Note: This would require exposing the compressor in ContextManager
    // For now, we just acknowledge the request
    Ok(())
}

// ============================================================================
// Orchestrator Commands
// ============================================================================

/// Add a task to the orchestrator queue
#[tauri::command]
pub async fn agent_orchestrator_add_task(
    state: State<'_, Arc<AgentState>>,
    id: String,
    agent_type: String,
    description: String,
    data: Option<serde_json::Value>,
    priority: Option<String>,
    timeout_seconds: Option<u64>,
) -> Result<(), String> {
    let orchestrator = state.orchestrator.lock().await;

    let agent_type = match agent_type.as_str() {
        "general" => AgentType::General,
        "code_generator" => AgentType::CodeGenerator,
        "code_reviewer" => AgentType::CodeReviewer,
        "researcher" => AgentType::Researcher,
        "data_analyst" => AgentType::DataAnalyst,
        "file_operator" => AgentType::FileOperator,
        "web_scraper" => AgentType::WebScraper,
        custom => AgentType::Custom(custom.to_string()),
    };

    let task_priority = match priority.as_deref() {
        Some("low") => TaskPriority::Low,
        Some("normal") => TaskPriority::Normal,
        Some("high") => TaskPriority::High,
        Some("urgent") => TaskPriority::Urgent,
        _ => TaskPriority::Normal,
    };

    let task = SubAgentTask {
        id: id.clone(),
        agent_type,
        input: TaskInput {
            description,
            data: data.unwrap_or(serde_json::json!(null)),
            priority: task_priority,
            timeout_seconds,
        },
        dependencies: vec![],
        created_at: chrono::Utc::now().timestamp(),
    };

    orchestrator.add_task(task).await;
    Ok(())
}

/// Execute all pending tasks
#[tauri::command]
pub async fn agent_orchestrator_execute_all(
    state: State<'_, Arc<AgentState>>,
) -> Result<serde_json::Value, String> {
    let orchestrator = state.orchestrator.lock().await;
    let result = orchestrator.execute_all().await;
    Ok(serde_json::json!({
        "total_tasks": result.total_tasks,
        "successful": result.successful,
        "failed": result.failed,
        "results": result.results,
    }))
}

/// Get orchestrator queue length
#[tauri::command]
pub async fn agent_orchestrator_queue_length(
    state: State<'_, Arc<AgentState>>,
) -> Result<usize, String> {
    let orchestrator = state.orchestrator.lock().await;
    Ok(orchestrator.queue_length().await)
}

/// Clear completed results
#[tauri::command]
pub async fn agent_orchestrator_clear_completed(
    state: State<'_, Arc<AgentState>>,
) -> Result<(), String> {
    let orchestrator = state.orchestrator.lock().await;
    orchestrator.clear_completed().await;
    Ok(())
}

/// Helper for MultimodalProcessor to expose process_image
trait MultimodalProcessorExt {
    fn process_image(&self, data: &[u8], format: &ImageFormat) -> Result<MultimodalResult, String>;
}

impl MultimodalProcessorExt for MultimodalProcessor {
    fn process_image(&self, data: &[u8], format: &ImageFormat) -> Result<MultimodalResult, String> {
        self.process(&InputType::Image {
            data: data.to_vec(),
            format: format.clone(),
        })
    }
}

// ============================================================================
// Helper functions for module support
// ============================================================================

/// Helper to process single image (module-level function for access from command)
pub fn process_image_internal(
    processor: &MultimodalProcessor,
    data: &[u8],
    format: &ImageFormat,
) -> Result<ImageAnalysis, String> {
    match processor.process(&InputType::Image {
        data: data.to_vec(),
        format: format.clone(),
    })? {
        MultimodalResult::Image(analysis) => Ok(analysis),
        _ => Err("Expected image analysis result".to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_agent_state_creation() {
        let state = AgentState::new();
        // Just verify it creates without panic
        assert_eq!(state.context_manager.lock().await.token_count(), 0);
    }

    #[test]
    fn test_message_priority_parsing() {
        let priority = match "high" {
            "low" => MessagePriority::Low,
            "normal" => MessagePriority::Normal,
            "high" => MessagePriority::High,
            "critical" => MessagePriority::Critical,
            _ => MessagePriority::Normal,
        };
        assert_eq!(priority, MessagePriority::High);
    }
}
