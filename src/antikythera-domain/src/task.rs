use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RetryCondition {
    #[default]
    Always,
    OnTransient,
    Never,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ErrorKind {
    Transient,
    Permanent,
    Cancelled,
    DeadlineExceeded,
    BudgetExhausted,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RoutingDecision {
    pub router_name: String,
    pub selected_agent_id: String,
    pub candidates_considered: usize,
    #[serde(default)]
    pub reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentTask {
    pub task_id: String,
    #[serde(default)]
    pub agent_id: Option<String>,
    pub input: String,
    #[serde(default)]
    pub session_id: Option<String>,
    #[serde(default)]
    pub max_steps: Option<usize>,
    #[serde(default)]
    pub timeout_ms: Option<u64>,
    #[serde(default)]
    pub deadline_unix_ms: Option<i64>,
    #[serde(default)]
    pub retry_policy: Option<TaskRetryPolicy>,
    #[serde(default)]
    pub budget_steps: Option<usize>,
    #[serde(default)]
    pub correlation_id: Option<String>,
    #[serde(default)]
    pub metadata: HashMap<String, Value>,
}

impl AgentTask {
    pub fn new(input: impl Into<String>) -> Self {
        Self {
            task_id: Uuid::new_v4().to_string(),
            agent_id: None,
            input: input.into(),
            session_id: None,
            max_steps: None,
            timeout_ms: None,
            deadline_unix_ms: None,
            retry_policy: None,
            budget_steps: None,
            correlation_id: None,
            metadata: HashMap::new(),
        }
    }

    pub fn for_agent(mut self, agent_id: impl Into<String>) -> Self {
        self.agent_id = Some(agent_id.into());
        self
    }

    pub fn with_session(mut self, session_id: impl Into<String>) -> Self {
        self.session_id = Some(session_id.into());
        self
    }

    pub fn with_max_steps(mut self, max_steps: usize) -> Self {
        self.max_steps = Some(max_steps);
        self
    }

    pub fn with_timeout_ms(mut self, timeout_ms: u64) -> Self {
        self.timeout_ms = Some(timeout_ms);
        self
    }

    pub fn with_retry_policy(mut self, retry_policy: TaskRetryPolicy) -> Self {
        self.retry_policy = Some(retry_policy);
        self
    }

    pub fn with_budget_steps(mut self, budget_steps: usize) -> Self {
        self.budget_steps = Some(budget_steps);
        self
    }

    pub fn with_correlation_id(mut self, correlation_id: impl Into<String>) -> Self {
        self.correlation_id = Some(correlation_id.into());
        self
    }

    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Serialize) -> Self {
        self.metadata.insert(
            key.into(),
            serde_json::to_value(value).unwrap_or(Value::Null),
        );
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TaskRetryPolicy {
    #[serde(default)]
    pub max_retries: u8,
    #[serde(default)]
    pub backoff_ms: u64,
    #[serde(default)]
    pub condition: RetryCondition,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TaskExecutionMetadata {
    #[serde(default)]
    pub attempt_count: u8,
    #[serde(default)]
    pub duration_ms: u64,
    #[serde(default)]
    pub timed_out: bool,
    #[serde(default)]
    pub deadline_exceeded: bool,
    #[serde(default)]
    pub cancelled: bool,
    #[serde(default)]
    pub retry_applied: bool,
    #[serde(default)]
    pub routed_by: Option<String>,
    #[serde(default)]
    pub execution_mode: Option<String>,
    #[serde(default)]
    pub correlation_id: Option<String>,
    #[serde(default)]
    pub error_kind: Option<ErrorKind>,
    #[serde(default)]
    pub routing_decision: Option<RoutingDecision>,
    #[serde(default)]
    pub concurrency_wait_ms: u64,
    #[serde(default)]
    pub budget_exhausted: bool,
    #[serde(default)]
    pub guardrail_name: Option<String>,
    #[serde(default)]
    pub guardrail_stage: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskResult {
    pub task_id: String,
    pub agent_id: String,
    pub output: Value,
    pub success: bool,
    #[serde(default)]
    pub error: Option<String>,
    #[serde(default)]
    pub error_kind: Option<ErrorKind>,
    pub steps_used: usize,
    pub session_id: String,
    #[serde(default)]
    pub metadata: TaskExecutionMetadata,
}

impl TaskResult {
    pub fn success(
        task_id: String,
        agent_id: String,
        output: Value,
        steps_used: usize,
        session_id: String,
    ) -> Self {
        Self {
            task_id,
            agent_id,
            output,
            success: true,
            error: None,
            error_kind: None,
            steps_used,
            session_id,
            metadata: TaskExecutionMetadata::default(),
        }
    }

    pub fn failure_with_kind(
        task_id: String,
        agent_id: String,
        error: String,
        kind: ErrorKind,
    ) -> Self {
        Self {
            task_id,
            agent_id,
            output: Value::Null,
            success: false,
            error: Some(error),
            error_kind: Some(kind),
            steps_used: 0,
            session_id: String::new(),
            metadata: TaskExecutionMetadata::default(),
        }
    }

    pub fn failure(task_id: String, agent_id: String, error: String) -> Self {
        Self::failure_with_kind(task_id, agent_id, error, ErrorKind::Permanent)
    }

    pub fn with_metadata(mut self, metadata: TaskExecutionMetadata) -> Self {
        self.metadata = metadata;
        self
    }

    pub fn is_transient(&self) -> bool {
        matches!(self.error_kind, Some(ErrorKind::Transient))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineResult {
    pub task_results: Vec<TaskResult>,
    pub final_output: Value,
    pub total_steps: usize,
    pub success: bool,
    #[serde(default)]
    pub error: Option<String>,
}

impl PipelineResult {
    pub fn from_results(results: Vec<TaskResult>) -> Self {
        let total_steps = results.iter().map(|r| r.steps_used).sum();
        let success = results.iter().all(|r| r.success);
        let final_output = results
            .last()
            .map(|r| r.output.clone())
            .unwrap_or(Value::Null);
        let error = if !success {
            results
                .iter()
                .find(|r| !r.success)
                .and_then(|r| r.error.clone())
        } else {
            None
        };
        Self {
            task_results: results,
            final_output,
            total_steps,
            success,
            error,
        }
    }
}
