//! Workflow Node Types and Executors
//! 
//! Defines node types and their execution logic.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use super::store::WorkflowNode;

/// Node types supported by the workflow engine
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum NodeType {
    /// Trigger node - starts workflow
    Trigger,
    /// Action node - performs an operation
    Action,
    /// Condition node - branches based on condition
    Condition,
    /// Loop node - iterates over data
    Loop,
    /// Agent node - calls AI agent
    Agent,
}

/// Node execution context
#[derive(Debug, Clone)]
pub struct NodeContext {
    pub workflow_id: String,
    pub variables: HashMap<String, serde_json::Value>,
    pub input: serde_json::Value,
    pub results: HashMap<String, serde_json::Value>,
}

/// Result of node execution
#[derive(Debug, Clone)]
pub enum NodeResult {
    /// Node executed successfully
    Success {
        output: serde_json::Value,
        next_node: Option<String>,
    },
    /// Node execution failed
    Failure {
        error: String,
    },
}

/// Node executor trait
pub trait NodeExecutor: Send + Sync {
    /// Execute a node
    fn execute(&self, node: &WorkflowNode, context: &NodeContext) -> NodeResult;
}

/// Data associated with a node
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeData {
    /// Node type specific configuration
    pub config: serde_json::Value,
    /// Display label
    pub label: Option<String>,
    /// Description
    pub description: Option<String>,
}

// Built-in node executors

/// Trigger node executor
pub struct TriggerExecutor;

impl NodeExecutor for TriggerExecutor {
    fn execute(&self, node: &WorkflowNode, context: &NodeContext) -> NodeResult {
        // Get next node from config
        let next = node.data.get("next").and_then(|v| v.as_str()).map(String::from);
        
        NodeResult::Success {
            output: context.input.clone(),
            next_node: next,
        }
    }
}

/// Action node executor
pub struct ActionExecutor;

impl NodeExecutor for ActionExecutor {
    fn execute(&self, node: &WorkflowNode, context: &NodeContext) -> NodeResult {
        let action_type = node.data.get("action_type")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");
        
        let next = node.data.get("next").and_then(|v| v.as_str()).map(String::from);
        
        // Placeholder: In real implementation, would execute the actual action
        let output = serde_json::json!({
            "action": action_type,
            "status": "completed",
            "input": context.input
        });
        
        NodeResult::Success {
            output,
            next_node: next,
        }
    }
}

/// Condition node executor
pub struct ConditionExecutor;

impl NodeExecutor for ConditionExecutor {
    fn execute(&self, node: &WorkflowNode, context: &NodeContext) -> NodeResult {
        // Get condition expression
        let condition = node.data.get("condition")
            .and_then(|v| v.as_str())
            .unwrap_or("true");
        
        // Get branches
        let true_next = node.data.get("true_next").and_then(|v| v.as_str()).map(String::from);
        let false_next = node.data.get("false_next").and_then(|v| v.as_str()).map(String::from);
        
        // Simple condition evaluation (placeholder)
        let result = evaluate_condition(condition, &context.variables);
        
        let next = if result { true_next } else { false_next };
        
        NodeResult::Success {
            output: serde_json::json!({ "condition_result": result }),
            next_node: next,
        }
    }
}

/// Loop node executor
pub struct LoopExecutor;

impl NodeExecutor for LoopExecutor {
    fn execute(&self, node: &WorkflowNode, context: &NodeContext) -> NodeResult {
        // Get loop configuration
        let items = node.data.get("items")
            .and_then(|v| v.as_array())
            .map(|a| a.clone())
            .unwrap_or_default();
        
        let body_node = node.data.get("body").and_then(|v| v.as_str()).map(String::from);
        let next = node.data.get("next").and_then(|v| v.as_str()).map(String::from);
        
        // Placeholder: In real implementation, would iterate through items
        let output = serde_json::json!({
            "loop_count": items.len(),
            "items": items
        });
        
        NodeResult::Success {
            output,
            next_node: next.or(body_node),
        }
    }
}

/// Agent node executor
pub struct AgentExecutor;

impl NodeExecutor for AgentExecutor {
    fn execute(&self, node: &WorkflowNode, context: &NodeContext) -> NodeResult {
        let prompt = node.data.get("prompt")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        
        let next = node.data.get("next").and_then(|v| v.as_str()).map(String::from);
        
        // Placeholder: In real implementation, would call the AI agent
        let output = serde_json::json!({
            "response": format!("Agent response to: {}", prompt),
            "input": context.input
        });
        
        NodeResult::Success {
            output,
            next_node: next,
        }
    }
}

/// Evaluate a simple condition expression
fn evaluate_condition(condition: &str, _variables: &HashMap<String, serde_json::Value>) -> bool {
    // Placeholder: simple true/false evaluation
    match condition {
        "true" => true,
        "false" => false,
        _ => true, // Default to true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_test_node(node_type: &str, data: serde_json::Value) -> WorkflowNode {
        WorkflowNode {
            id: "test-node".to_string(),
            node_type: node_type.to_string(),
            position: super::super::store::NodePosition { x: 0.0, y: 0.0 },
            data,
            label: None,
        }
    }

    fn make_test_context() -> NodeContext {
        NodeContext {
            workflow_id: "test-workflow".to_string(),
            variables: HashMap::new(),
            input: serde_json::json!({ "test": "input" }),
            results: HashMap::new(),
        }
    }

    #[test]
    fn test_trigger_executor() {
        let executor = TriggerExecutor;
        let node = make_test_node("trigger", serde_json::json!({ "next": "next-node" }));
        let context = make_test_context();
        
        let result = executor.execute(&node, &context);
        
        match result {
            NodeResult::Success { next_node, .. } => {
                assert_eq!(next_node, Some("next-node".to_string()));
            }
            NodeResult::Failure { .. } => panic!("Expected success"),
        }
    }

    #[test]
    fn test_action_executor() {
        let executor = ActionExecutor;
        let node = make_test_node("action", serde_json::json!({ "action_type": "http_request" }));
        let context = make_test_context();
        
        let result = executor.execute(&node, &context);
        
        match result {
            NodeResult::Success { output, .. } => {
                assert_eq!(output["action"], "http_request");
            }
            NodeResult::Failure { .. } => panic!("Expected success"),
        }
    }

    #[test]
    fn test_condition_executor() {
        let executor = ConditionExecutor;
        let node = make_test_node("condition", serde_json::json!({
            "condition": "true",
            "true_next": "true-branch",
            "false_next": "false-branch"
        }));
        let context = make_test_context();
        
        let result = executor.execute(&node, &context);
        
        match result {
            NodeResult::Success { next_node, .. } => {
                assert_eq!(next_node, Some("true-branch".to_string()));
            }
            NodeResult::Failure { .. } => panic!("Expected success"),
        }
    }
}
