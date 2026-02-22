//! Integration tests for v0.6 Agent Module
//!
//! Tests for multimodal processing, context management, and agent orchestration.

#[cfg(test)]
mod tests {
    use ai_assistant_tauri_lib::agent::context::ContextManager;
    use ai_assistant_tauri_lib::agent::context::Message;
    use ai_assistant_tauri_lib::agent::context::MessageRole;
    use ai_assistant_tauri_lib::agent::context::MessagePriority;
    use ai_assistant_tauri_lib::agent::multimodal::ImageFormat;
    use ai_assistant_tauri_lib::agent::multimodal::InputType;
    use ai_assistant_tauri_lib::agent::multimodal::MultimodalProcessor;
    use ai_assistant_tauri_lib::agent::orchestrator::AgentOrchestrator;
    use ai_assistant_tauri_lib::agent::orchestrator::AgentType;
    use ai_assistant_tauri_lib::agent::orchestrator::TaskPriority;
    use ai_assistant_tauri_lib::agent::context::CompressionStrategy;

    #[test]
    fn test_image_format_display() {
        // Test ImageFormat Display implementation from multimodal module
        assert_eq!(format!("{}", ImageFormat::Png), "png");
        assert_eq!(format!("{}", ImageFormat::Jpeg), "jpeg");
        assert_eq!(format!("{}", ImageFormat::WebP), "webp");
    }

    #[test]
    fn test_message_priority_ord() {
        // Test MessagePriority ordering from context module
        assert!(MessagePriority::Critical > MessagePriority::High);
        assert!(MessagePriority::High > MessagePriority::Normal);
        assert!(MessagePriority::Normal > MessagePriority::Low);
    }

    #[test]
    fn test_agent_type_equality() {
        // Test AgentType from orchestrator module
        assert_eq!(AgentType::General, AgentType::General);
        assert_ne!(AgentType::General, AgentType::CodeGenerator);
    }

    #[test]
    fn test_compression_strategy_variants() {
        // Test all CompressionStrategy variants
        let _ = CompressionStrategy::RemoveOldest;
        let _ = CompressionStrategy::Summarize;
        let _ = CompressionStrategy::PriorityOnly;
        let _ = CompressionStrategy::Hybrid;
    }

    #[test]
    fn test_task_priority_ord() {
        // Test TaskPriority ordering from orchestrator module
        assert!(TaskPriority::Urgent > TaskPriority::High);
        assert!(TaskPriority::High > TaskPriority::Normal);
        assert!(TaskPriority::Normal > TaskPriority::Low);
    }

    #[tokio::test]
    async fn test_context_manager_with_defaults() {
        let manager = ContextManager::with_defaults();
        assert_eq!(manager.get_messages().len(), 0);
        assert_eq!(manager.token_count(), 0);
        assert!(!manager.is_near_limit());
    }

    #[tokio::test]
    async fn test_orchestrator_creation() {
        let orchestrator = AgentOrchestrator::new(3);
        assert_eq!(orchestrator.queue_length().await, 0);
    }

    #[test]
    fn test_multimodal_processor_creation() {
        let processor = MultimodalProcessor::new(None);
        // Just verify it creates without panic
        let input = InputType::Text("test".to_string());
        assert!(processor.process(&input).is_ok());
    }

    #[tokio::test]
    async fn test_context_manager_compress() {
        let mut manager = ContextManager::with_defaults();

        // Add some messages
        for i in 0..10 {
            manager.add_message(Message {
                role: MessageRole::User,
                content: format!("Message {}", i),
                token_count: 10,
                priority: MessagePriority::Normal,
                timestamp: i as i64,
            });
        }

        // Compress
        let result = manager.compress();
        assert!(result.compressed_tokens <= result.original_tokens);
    }
}
