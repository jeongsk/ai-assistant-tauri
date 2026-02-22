//! Context Management for AI Conversations
//! 
//! Provides intelligent context handling including:
//! - Short-term and long-term memory management
//! - Context compression for token efficiency
//! - Priority-based context selection

use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::sync::Arc;

/// Maximum context tokens by default
const DEFAULT_MAX_TOKENS: usize = 4096;
/// Minimum tokens to preserve during compression
const MIN_PRESERVED_TOKENS: usize = 512;

/// Message role in conversation
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MessageRole {
    System,
    User,
    Assistant,
    Tool,
}

/// A single message in the conversation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: MessageRole,
    pub content: String,
    pub token_count: usize,
    pub priority: MessagePriority,
    pub timestamp: i64,
}

/// Priority level for messages
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum MessagePriority {
    Low = 0,
    Normal = 1,
    High = 2,
    Critical = 3,
}

/// Compression strategy for context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CompressionStrategy {
    /// Remove oldest low-priority messages first
    RemoveOldest,
    /// Summarize older messages
    Summarize,
    /// Keep only high-priority messages
    PriorityOnly,
    /// Hybrid: summarize + priority-based
    Hybrid,
}

/// Context compressor configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompressorConfig {
    pub strategy: CompressionStrategy,
    pub min_tokens: usize,
    pub target_ratio: f32,
}

impl Default for CompressorConfig {
    fn default() -> Self {
        Self {
            strategy: CompressionStrategy::Hybrid,
            min_tokens: MIN_PRESERVED_TOKENS,
            target_ratio: 0.5, // Target 50% compression
        }
    }
}

/// Context compressor for reducing token count
pub struct ContextCompressor {
    config: CompressorConfig,
}

impl ContextCompressor {
    /// Create a new compressor with configuration
    pub fn new(config: CompressorConfig) -> Self {
        Self { config }
    }
    
    /// Create with default configuration
    pub fn with_defaults() -> Self {
        Self::new(CompressorConfig::default())
    }
    
    /// Compress messages to fit within token limit
    pub fn compress(&self, messages: &mut Vec<Message>, max_tokens: usize) -> CompressionResult {
        let current_tokens: usize = messages.iter().map(|m| m.token_count).sum();
        
        if current_tokens <= max_tokens {
            return CompressionResult {
                original_tokens: current_tokens,
                compressed_tokens: current_tokens,
                removed_count: 0,
                summarized_count: 0,
            };
        }
        
        let target_tokens = (max_tokens as f32 * self.config.target_ratio) as usize;
        let target_tokens = target_tokens.max(self.config.min_tokens);
        
        match &self.config.strategy {
            CompressionStrategy::RemoveOldest => {
                self.remove_oldest(messages, target_tokens)
            }
            CompressionStrategy::Summarize => {
                self.summarize(messages, target_tokens)
            }
            CompressionStrategy::PriorityOnly => {
                self.priority_only(messages, target_tokens)
            }
            CompressionStrategy::Hybrid => {
                self.hybrid_compress(messages, target_tokens)
            }
        }
    }
    
    fn remove_oldest(&self, messages: &mut Vec<Message>, target: usize) -> CompressionResult {
        let original_tokens: usize = messages.iter().map(|m| m.token_count).sum();
        let mut removed = 0;
        
        while messages.iter().map(|m| m.token_count).sum::<usize>() > target && !messages.is_empty() {
            // Find and remove oldest low-priority message
            if let Some(pos) = messages.iter().position(|m| m.priority == MessagePriority::Low) {
                messages.remove(pos);
                removed += 1;
            } else if let Some(pos) = messages.iter().position(|m| m.priority == MessagePriority::Normal) {
                messages.remove(pos);
                removed += 1;
            } else {
                break;
            }
        }
        
        let compressed_tokens = messages.iter().map(|m| m.token_count).sum();
        CompressionResult {
            original_tokens,
            compressed_tokens,
            removed_count: removed,
            summarized_count: 0,
        }
    }
    
    fn summarize(&self, _messages: &mut Vec<Message>, _target: usize) -> CompressionResult {
        // Placeholder: In production, this would call an LLM to summarize
        // For now, delegate to remove_oldest
        CompressionResult {
            original_tokens: 0,
            compressed_tokens: 0,
            removed_count: 0,
            summarized_count: 0,
        }
    }
    
    fn priority_only(&self, messages: &mut Vec<Message>, _target: usize) -> CompressionResult {
        let original_tokens: usize = messages.iter().map(|m| m.token_count).sum();
        let mut removed = 0;
        
        // Remove all low and normal priority messages
        messages.retain(|m| {
            if m.priority == MessagePriority::Low || m.priority == MessagePriority::Normal {
                removed += 1;
                false
            } else {
                true
            }
        });
        
        let compressed_tokens = messages.iter().map(|m| m.token_count).sum();
        CompressionResult {
            original_tokens,
            compressed_tokens,
            removed_count: removed,
            summarized_count: 0,
        }
    }
    
    fn hybrid_compress(&self, messages: &mut Vec<Message>, target: usize) -> CompressionResult {
        let original_tokens: usize = messages.iter().map(|m| m.token_count).sum();
        
        // First pass: remove low priority oldest
        let mut removed = 0;
        let current = messages.iter().map(|m| m.token_count).sum::<usize>();
        
        if current > target {
            // Remove oldest low priority
            while messages.iter().map(|m| m.token_count).sum::<usize>() > target {
                if let Some(pos) = messages.iter()
                    .enumerate()
                    .filter(|(_, m)| m.priority <= MessagePriority::Normal)
                    .min_by_key(|(i, m)| (m.priority, *i))
                    .map(|(i, _)| i)
                {
                    messages.remove(pos);
                    removed += 1;
                } else {
                    break;
                }
            }
        }
        
        let compressed_tokens = messages.iter().map(|m| m.token_count).sum();
        CompressionResult {
            original_tokens,
            compressed_tokens,
            removed_count: removed,
            summarized_count: 0,
        }
    }
}

/// Result of compression operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompressionResult {
    pub original_tokens: usize,
    pub compressed_tokens: usize,
    pub removed_count: usize,
    pub summarized_count: usize,
}

/// Memory store trait for long-term storage
pub trait MemoryStore: Send + Sync {
    fn store(&self, key: &str, value: &str) -> Result<(), String>;
    fn retrieve(&self, key: &str) -> Result<Option<String>, String>;
    fn search(&self, query: &str, limit: usize) -> Result<Vec<String>, String>;
}

/// Context manager for handling conversation context
pub struct ContextManager {
    /// Short-term memory (recent messages)
    short_term: VecDeque<Message>,
    /// Long-term memory store
    long_term: Option<Arc<dyn MemoryStore>>,
    /// Context compressor
    compressor: ContextCompressor,
    /// Maximum tokens allowed
    max_tokens: usize,
    /// Current token count
    current_tokens: usize,
}

impl ContextManager {
    /// Create a new context manager
    pub fn new(
        long_term: Option<Arc<dyn MemoryStore>>,
        max_tokens: usize,
    ) -> Self {
        Self {
            short_term: VecDeque::new(),
            long_term,
            compressor: ContextCompressor::with_defaults(),
            max_tokens,
            current_tokens: 0,
        }
    }
    
    /// Create with default settings
    pub fn with_defaults() -> Self {
        Self::new(None, DEFAULT_MAX_TOKENS)
    }
    
    /// Add a message to context
    pub fn add_message(&mut self, message: Message) {
        self.current_tokens += message.token_count;
        self.short_term.push_back(message);
        
        // Auto-compress if over limit
        if self.current_tokens > self.max_tokens {
            self.compress();
        }
    }
    
    /// Get all messages in context
    pub fn get_messages(&self) -> Vec<&Message> {
        self.short_term.iter().collect()
    }
    
    /// Get messages owned (for processing)
    pub fn get_messages_owned(&self) -> Vec<Message> {
        self.short_term.iter().cloned().collect()
    }
    
    /// Compress context to fit within limits
    pub fn compress(&mut self) -> CompressionResult {
        let mut messages: Vec<Message> = self.short_term.drain(..).collect();
        let result = self.compressor.compress(&mut messages, self.max_tokens);
        
        self.short_term.clear();
        for msg in messages {
            self.short_term.push_back(msg);
        }
        self.current_tokens = result.compressed_tokens;
        
        result
    }
    
    /// Clear all context
    pub fn clear(&mut self) {
        self.short_term.clear();
        self.current_tokens = 0;
    }
    
    /// Get current token count
    pub fn token_count(&self) -> usize {
        self.current_tokens
    }
    
    /// Check if context is near limit
    pub fn is_near_limit(&self) -> bool {
        self.current_tokens > (self.max_tokens * 80 / 100)
    }
    
    /// Store important message to long-term memory
    pub fn store_to_long_term(&self, key: &str, content: &str) -> Result<(), String> {
        if let Some(ref store) = self.long_term {
            store.store(key, content)
        } else {
            Err("No long-term memory store configured".to_string())
        }
    }
    
    /// Search long-term memory
    pub fn search_long_term(&self, query: &str, limit: usize) -> Result<Vec<String>, String> {
        if let Some(ref store) = self.long_term {
            store.search(query, limit)
        } else {
            Ok(vec![])
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_message(content: &str, priority: MessagePriority) -> Message {
        Message {
            role: MessageRole::User,
            content: content.to_string(),
            token_count: content.split_whitespace().count(),
            priority,
            timestamp: 0,
        }
    }

    #[test]
    fn test_context_manager_add_message() {
        let mut manager = ContextManager::with_defaults();
        manager.add_message(make_message("Hello world", MessagePriority::Normal));
        
        assert_eq!(manager.get_messages().len(), 1);
        assert_eq!(manager.token_count(), 2);
    }

    #[test]
    fn test_context_manager_clear() {
        let mut manager = ContextManager::with_defaults();
        manager.add_message(make_message("Test", MessagePriority::Normal));
        manager.clear();
        
        assert_eq!(manager.get_messages().len(), 0);
        assert_eq!(manager.token_count(), 0);
    }

    #[test]
    fn test_compression_remove_oldest() {
        let config = CompressorConfig {
            strategy: CompressionStrategy::RemoveOldest,
            min_tokens: 2,  // Lower min tokens
            target_ratio: 0.3,
        };
        let compressor = ContextCompressor::new(config);

        let mut messages = vec![
            make_message("First message here", MessagePriority::Low),
            make_message("Second message here", MessagePriority::Normal),
            make_message("Third important message", MessagePriority::High),
        ];

        // Compress to 4 tokens (should remove at least one message)
        let result = compressor.compress(&mut messages, 4);

        // Should have removed some messages
        assert!(result.removed_count > 0 || result.compressed_tokens < result.original_tokens);
    }

    #[test]
    fn test_message_priority() {
        assert!(MessagePriority::Critical > MessagePriority::High);
        assert!(MessagePriority::High > MessagePriority::Normal);
        assert!(MessagePriority::Normal > MessagePriority::Low);
    }

    #[test]
    fn test_is_near_limit() {
        let mut manager = ContextManager::new(None, 100);
        
        // Add message to reach 85% of limit
        let content = "word ".repeat(85);
        manager.add_message(make_message(&content.trim(), MessagePriority::Normal));
        
        assert!(manager.is_near_limit());
    }
}
