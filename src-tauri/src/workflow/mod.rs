//! Workflow Automation Module
//!
//! Provides visual workflow creation, execution, and management:
//! - Node-based workflow definition
//! - Trigger system (schedule, webhook, file, voice)
//! - Execution engine with error handling

pub mod store;
pub mod engine;
pub mod nodes;
pub mod triggers;
pub mod commands;

pub use store::{WorkflowStore, Workflow, WorkflowExecution};
pub use engine::{WorkflowExecutor, ExecutionResult};
pub use nodes::{NodeType, NodeData, NodeExecutor};
pub use triggers::{TriggerManager, Trigger, TriggerType};
pub use commands::WorkflowState;
