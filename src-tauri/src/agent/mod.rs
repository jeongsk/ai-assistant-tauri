//! AI Agent System Enhancement Module
//!
//! This module provides advanced AI agent capabilities including:
//! - Multimodal input processing (text, image)
//! - Context management and compression
//! - Sub-agent orchestration

pub mod multimodal;
pub mod context;
pub mod orchestrator;
pub mod commands;

pub use multimodal::{MultimodalProcessor, InputType, ImageAnalysis};
pub use context::{ContextManager, ContextCompressor, CompressionStrategy};
pub use orchestrator::{AgentOrchestrator, SubAgentTask, AgentType};
pub use commands::AgentState;
