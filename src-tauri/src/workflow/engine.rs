//! Workflow Execution Engine
//! 
//! Executes workflows node by node with error handling.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use super::store::Workflow;
use super::nodes::{NodeExecutor, NodeContext, NodeResult};

/// Workflow executor
pub struct WorkflowExecutor {
    node_executors: HashMap<String, Box<dyn NodeExecutor>>,
}

impl WorkflowExecutor {
    /// Create a new workflow executor
    pub fn new() -> Self {
        Self {
            node_executors: HashMap::new(),
        }
    }
    
    /// Register a node executor
    pub fn register_executor(&mut self, node_type: &str, executor: Box<dyn NodeExecutor>) {
        self.node_executors.insert(node_type.to_string(), executor);
    }
    
    /// Execute a workflow
    pub fn execute(&self, workflow: &Workflow, input: serde_json::Value) -> ExecutionResult {
        let definition = &workflow.definition;
        let mut context = NodeContext {
            workflow_id: workflow.id.clone(),
            variables: HashMap::new(),
            input,
            results: HashMap::new(),
        };
        
        // Start from entry point
        let mut current_node_id = definition.entry_point.clone();
        let mut executed_nodes = Vec::new();
        
        while !current_node_id.is_empty() {
            // Get node
            let node = match definition.nodes.get(&current_node_id) {
                Some(n) => n,
                None => {
                    return ExecutionResult {
                        success: false,
                        output: serde_json::json!(null),
                        executed_nodes,
                        error: Some(format!("Node not found: {}", current_node_id)),
                    };
                }
            };
            
            // Get executor
            let executor = match self.node_executors.get(&node.node_type) {
                Some(e) => e,
                None => {
                    return ExecutionResult {
                        success: false,
                        output: serde_json::json!(null),
                        executed_nodes,
                        error: Some(format!("No executor for node type: {}", node.node_type)),
                    };
                }
            };
            
            // Execute node
            let result = executor.execute(node, &context);
            
            match result {
                NodeResult::Success { output, next_node } => {
                    context.results.insert(current_node_id.clone(), output.clone());
                    executed_nodes.push(current_node_id.clone());
                    
                    match next_node {
                        Some(next) => current_node_id = next,
                        None => break, // End of workflow
                    }
                }
                NodeResult::Failure { error } => {
                    return ExecutionResult {
                        success: false,
                        output: serde_json::json!(null),
                        executed_nodes,
                        error: Some(error),
                    };
                }
            }
        }
        
        // Get final output from last node result
        let final_output = executed_nodes.last()
            .and_then(|id| context.results.get(id))
            .cloned()
            .unwrap_or(serde_json::json!(null));
        
        ExecutionResult {
            success: true,
            output: final_output,
            executed_nodes,
            error: None,
        }
    }
}

impl Default for WorkflowExecutor {
    fn default() -> Self {
        Self::new()
    }
}

/// Result of workflow execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionResult {
    pub success: bool,
    pub output: serde_json::Value,
    pub executed_nodes: Vec<String>,
    pub error: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_executor_creation() {
        let executor = WorkflowExecutor::new();
        assert!(executor.node_executors.is_empty());
    }
}
