//! Sub-agent Orchestration System
//!
//! Provides task distribution, parallel execution, and result aggregation
//! for coordinating multiple specialized agents.

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

/// Agent type for specialization
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum AgentType {
    /// General purpose agent
    General,
    /// Code generation specialist
    CodeGenerator,
    /// Code reviewer
    CodeReviewer,
    /// Research specialist
    Researcher,
    /// Data analyst
    DataAnalyst,
    /// File operations
    FileOperator,
    /// Web scraper
    WebScraper,
    /// Custom agent with ID
    Custom(String),
}

/// Task input for sub-agents
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskInput {
    /// Task description
    pub description: String,
    /// Input data (JSON)
    pub data: serde_json::Value,
    /// Priority level
    pub priority: TaskPriority,
    /// Timeout in seconds
    pub timeout_seconds: Option<u64>,
}

/// Task priority
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum TaskPriority {
    Low = 0,
    Normal = 1,
    High = 2,
    Urgent = 3,
}

/// Sub-agent task definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubAgentTask {
    /// Unique task ID
    pub id: String,
    /// Agent type to handle this task
    pub agent_type: AgentType,
    /// Task input
    pub input: TaskInput,
    /// IDs of tasks that must complete first
    pub dependencies: Vec<String>,
    /// Creation timestamp
    pub created_at: i64,
}

/// Task execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskResult {
    /// Task ID
    pub task_id: String,
    /// Success status
    pub success: bool,
    /// Result data (JSON)
    pub data: serde_json::Value,
    /// Error message if failed
    pub error: Option<String>,
    /// Execution time in milliseconds
    pub execution_time_ms: u64,
}

/// Sub-agent trait for specialization (synchronous for dyn compatibility)
pub trait SubAgent: Send + Sync {
    /// Get agent type
    fn agent_type(&self) -> AgentType;

    /// Execute a task (synchronous for dyn compatibility)
    fn execute(&self, task: &SubAgentTask) -> TaskResult;

    /// Check if agent can handle this task
    fn can_handle(&self, task: &SubAgentTask) -> bool {
        task.agent_type == self.agent_type()
    }
}

/// Result aggregator for combining sub-agent results
pub struct ResultAggregator {
    results: HashMap<String, TaskResult>,
    expected_count: usize,
}

impl ResultAggregator {
    /// Create a new aggregator
    pub fn new(expected_count: usize) -> Self {
        Self {
            results: HashMap::new(),
            expected_count,
        }
    }

    /// Add a result
    pub fn add_result(&mut self, result: TaskResult) {
        self.results.insert(result.task_id.clone(), result);
    }

    /// Check if all results are collected
    pub fn is_complete(&self) -> bool {
        self.results.len() >= self.expected_count
    }

    /// Get all results
    pub fn get_results(&self) -> &HashMap<String, TaskResult> {
        &self.results
    }

    /// Aggregate results into single output
    pub fn aggregate(&self) -> AggregatedResult {
        let success_count = self.results.values().filter(|r| r.success).count();
        let failure_count = self.results.len() - success_count;

        let combined_data: serde_json::Value = self.results
            .iter()
            .map(|(id, r)| (id.clone(), r.data.clone()))
            .collect();

        AggregatedResult {
            total_tasks: self.expected_count,
            successful: success_count,
            failed: failure_count,
            results: self.results.clone(),
            combined_data,
        }
    }
}

/// Aggregated result from multiple sub-agents
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregatedResult {
    pub total_tasks: usize,
    pub successful: usize,
    pub failed: usize,
    pub results: HashMap<String, TaskResult>,
    pub combined_data: serde_json::Value,
}

/// Priority queue for tasks
struct TaskQueue {
    queue: VecDeque<SubAgentTask>,
}

impl TaskQueue {
    fn new() -> Self {
        Self {
            queue: VecDeque::new(),
        }
    }

    fn push(&mut self, task: SubAgentTask) {
        // Insert in priority order
        let pos = self.queue.iter().position(|t| {
            task.input.priority > t.input.priority
        }).unwrap_or(self.queue.len());

        self.queue.insert(pos, task);
    }

    fn pop(&mut self) -> Option<SubAgentTask> {
        self.queue.pop_front()
    }

    fn is_empty(&self) -> bool {
        self.queue.is_empty()
    }

    fn len(&self) -> usize {
        self.queue.len()
    }
}

/// Agent orchestrator for coordinating sub-agents
pub struct AgentOrchestrator {
    /// Sub-agents by type
    sub_agents: HashMap<AgentType, Arc<dyn SubAgent>>,
    /// Task queue
    task_queue: Arc<RwLock<TaskQueue>>,
    /// Completed results
    completed: Arc<RwLock<HashMap<String, TaskResult>>>,
    /// Maximum concurrent tasks (reserved for future use)
    #[allow(dead_code)]
    max_concurrent: usize,
}

impl AgentOrchestrator {
    /// Create a new orchestrator
    pub fn new(max_concurrent: usize) -> Self {
        Self {
            sub_agents: HashMap::new(),
            task_queue: Arc::new(RwLock::new(TaskQueue::new())),
            completed: Arc::new(RwLock::new(HashMap::new())),
            max_concurrent,
        }
    }

    /// Register a sub-agent
    pub fn register_agent(&mut self, agent: Arc<dyn SubAgent>) {
        let agent_type = agent.agent_type();
        self.sub_agents.insert(agent_type, agent);
    }

    /// Add a task to the queue
    pub async fn add_task(&self, task: SubAgentTask) {
        let mut queue = self.task_queue.write().await;
        queue.push(task);
    }

    /// Add multiple tasks
    pub async fn add_tasks(&self, tasks: Vec<SubAgentTask>) {
        let mut queue = self.task_queue.write().await;
        for task in tasks {
            queue.push(task);
        }
    }

    /// Get queue length
    pub async fn queue_length(&self) -> usize {
        self.task_queue.read().await.len()
    }

    /// Execute all pending tasks
    pub async fn execute_all(&self) -> AggregatedResult {
        let queue_len = self.queue_length().await;
        let mut aggregator = ResultAggregator::new(queue_len);

        while !self.task_queue.read().await.is_empty() {
            // Get next task
            let task = {
                let mut queue = self.task_queue.write().await;
                queue.pop()
            };

            if let Some(task) = task {
                // Check dependencies
                let deps_met = self.check_dependencies(&task).await;

                if !deps_met {
                    // Re-queue if dependencies not met
                    let mut queue = self.task_queue.write().await;
                    queue.push(task);
                    tokio::time::sleep(Duration::from_millis(100)).await;
                    continue;
                }

                // Execute task (synchronous call)
                let result = self.execute_task(&task);

                // Store result
                let task_id = task.id.clone();
                self.completed.write().await.insert(task_id.clone(), result.clone());
                aggregator.add_result(result);
            }
        }

        aggregator.aggregate()
    }

    /// Check if all dependencies are met
    async fn check_dependencies(&self, task: &SubAgentTask) -> bool {
        if task.dependencies.is_empty() {
            return true;
        }

        let completed = self.completed.read().await;
        task.dependencies.iter().all(|dep| completed.contains_key(dep))
    }

    /// Execute a single task (synchronous)
    fn execute_task(&self, task: &SubAgentTask) -> TaskResult {
        let start = Instant::now();

        if let Some(agent) = self.sub_agents.get(&task.agent_type) {
            let mut result = agent.execute(task);
            result.execution_time_ms = start.elapsed().as_millis() as u64;
            result
        } else {
            TaskResult {
                task_id: task.id.clone(),
                success: false,
                data: serde_json::json!(null),
                error: Some(format!("No agent registered for type {:?}", task.agent_type)),
                execution_time_ms: start.elapsed().as_millis() as u64,
            }
        }
    }

    /// Clear completed results
    pub async fn clear_completed(&self) {
        self.completed.write().await.clear();
    }
}

/// Mock sub-agent for testing
pub struct MockSubAgent {
    agent_type: AgentType,
    should_succeed: bool,
}

impl MockSubAgent {
    pub fn new(agent_type: AgentType, should_succeed: bool) -> Self {
        Self { agent_type, should_succeed }
    }
}

impl SubAgent for MockSubAgent {
    fn agent_type(&self) -> AgentType {
        self.agent_type.clone()
    }

    fn execute(&self, task: &SubAgentTask) -> TaskResult {
        TaskResult {
            task_id: task.id.clone(),
            success: self.should_succeed,
            data: serde_json::json!({ "processed": task.input.description }),
            error: if self.should_succeed { None } else { Some("Mock failure".to_string()) },
            execution_time_ms: 10,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_task_queue_priority() {
        let mut queue = TaskQueue::new();

        queue.push(SubAgentTask {
            id: "1".to_string(),
            agent_type: AgentType::General,
            input: TaskInput {
                description: "Low priority".to_string(),
                data: serde_json::json!(null),
                priority: TaskPriority::Low,
                timeout_seconds: None,
            },
            dependencies: vec![],
            created_at: 0,
        });

        queue.push(SubAgentTask {
            id: "2".to_string(),
            agent_type: AgentType::General,
            input: TaskInput {
                description: "High priority".to_string(),
                data: serde_json::json!(null),
                priority: TaskPriority::High,
                timeout_seconds: None,
            },
            dependencies: vec![],
            created_at: 0,
        });

        let first = queue.pop().unwrap();
        assert_eq!(first.id, "2"); // High priority first
    }

    #[test]
    fn test_result_aggregator() {
        let mut aggregator = ResultAggregator::new(2);

        aggregator.add_result(TaskResult {
            task_id: "1".to_string(),
            success: true,
            data: serde_json::json!("result1"),
            error: None,
            execution_time_ms: 100,
        });

        aggregator.add_result(TaskResult {
            task_id: "2".to_string(),
            success: false,
            data: serde_json::json!(null),
            error: Some("failed".to_string()),
            execution_time_ms: 50,
        });

        assert!(aggregator.is_complete());

        let aggregated = aggregator.aggregate();
        assert_eq!(aggregated.successful, 1);
        assert_eq!(aggregated.failed, 1);
    }

    #[tokio::test]
    async fn test_orchestrator_single_task() {
        let mut orchestrator = AgentOrchestrator::new(2);

        let agent = Arc::new(MockSubAgent::new(AgentType::General, true));
        orchestrator.register_agent(agent);

        let task = SubAgentTask {
            id: "task1".to_string(),
            agent_type: AgentType::General,
            input: TaskInput {
                description: "Test task".to_string(),
                data: serde_json::json!(null),
                priority: TaskPriority::Normal,
                timeout_seconds: None,
            },
            dependencies: vec![],
            created_at: 0,
        };

        orchestrator.add_task(task).await;

        let result = orchestrator.execute_all().await;

        assert_eq!(result.total_tasks, 1);
        assert_eq!(result.successful, 1);
    }

    #[tokio::test]
    async fn test_orchestrator_dependencies() {
        let mut orchestrator = AgentOrchestrator::new(2);

        let agent = Arc::new(MockSubAgent::new(AgentType::General, true));
        orchestrator.register_agent(agent);

        // Add task with dependency
        let dependent_task = SubAgentTask {
            id: "task2".to_string(),
            agent_type: AgentType::General,
            input: TaskInput {
                description: "Dependent task".to_string(),
                data: serde_json::json!(null),
                priority: TaskPriority::Normal,
                timeout_seconds: None,
            },
            dependencies: vec!["task1".to_string()],
            created_at: 0,
        };

        orchestrator.add_task(dependent_task).await;

        // Add the dependency task
        let dependency_task = SubAgentTask {
            id: "task1".to_string(),
            agent_type: AgentType::General,
            input: TaskInput {
                description: "Dependency task".to_string(),
                data: serde_json::json!(null),
                priority: TaskPriority::High,
                timeout_seconds: None,
            },
            dependencies: vec![],
            created_at: 0,
        };

        orchestrator.add_task(dependency_task).await;

        let result = orchestrator.execute_all().await;

        assert_eq!(result.total_tasks, 2);
        assert_eq!(result.successful, 2);
    }
}
