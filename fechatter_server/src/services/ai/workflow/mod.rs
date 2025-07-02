//! AI Workflow Orchestration Engine
//!
//! This module provides multi-step AI processing workflows that can chain
//! different AI services together to accomplish complex tasks.

use crate::error::AppError;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::time::{timeout, Duration};

/// Workflow execution context
#[derive(Debug, Clone)]
pub struct WorkflowContext {
    pub chat_id: Option<i64>,
    pub user_id: Option<i64>,
    pub workspace_id: Option<i64>,
    pub variables: HashMap<String, serde_json::Value>,
}

impl WorkflowContext {
    pub fn new() -> Self {
        Self {
            chat_id: None,
            user_id: None,
            workspace_id: None,
            variables: HashMap::new(),
        }
    }

    pub fn with_chat(mut self, chat_id: i64) -> Self {
        self.chat_id = Some(chat_id);
        self
    }

    pub fn with_user(mut self, user_id: i64) -> Self {
        self.user_id = Some(user_id);
        self
    }

    pub fn set_variable(&mut self, key: &str, value: serde_json::Value) {
        self.variables.insert(key.to_string(), value);
    }

    pub fn get_variable(&self, key: &str) -> Option<&serde_json::Value> {
        self.variables.get(key)
    }
}

/// Workflow step execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepResult {
    pub success: bool,
    pub output: serde_json::Value,
    pub error: Option<String>,
    pub execution_time_ms: u64,
}

/// Workflow step trait
#[async_trait]
pub trait WorkflowStep: Send + Sync {
    async fn execute(&self, context: &mut WorkflowContext) -> Result<StepResult, AppError>;
    fn name(&self) -> &str;
    fn description(&self) -> &str;
}

/// Workflow definition
pub struct Workflow {
    pub name: String,
    pub description: String,
    pub steps: Vec<Box<dyn WorkflowStep>>,
    pub timeout_seconds: u64,
}

impl std::fmt::Debug for Workflow {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Workflow")
            .field("name", &self.name)
            .field("description", &self.description)
            .field("steps_count", &self.steps.len())
            .field("timeout_seconds", &self.timeout_seconds)
            .finish()
    }
}

/// Workflow execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowResult {
    pub workflow_name: String,
    pub success: bool,
    pub steps_executed: usize,
    pub step_results: Vec<StepResult>,
    pub total_execution_time_ms: u64,
    pub final_output: Option<serde_json::Value>,
    pub error: Option<String>,
}

/// Workflow orchestration engine
pub struct WorkflowEngine;

impl WorkflowEngine {
    pub fn new() -> Self {
        Self
    }

    /// Execute a workflow with the given context
    pub async fn execute_workflow(
        &self,
        workflow: &Workflow,
        mut context: WorkflowContext,
    ) -> Result<WorkflowResult, AppError> {
        let start_time = std::time::Instant::now();
        let mut step_results = Vec::new();
        let mut steps_executed = 0;

        tracing::info!("🔄 Starting workflow: {}", workflow.name);

        // Execute workflow with timeout
        let execution_result = timeout(
            Duration::from_secs(workflow.timeout_seconds),
            self.execute_steps(
                &workflow.steps,
                &mut context,
                &mut step_results,
                &mut steps_executed,
            ),
        )
        .await;

        let total_execution_time = start_time.elapsed().as_millis() as u64;

        match execution_result {
            Ok(result) => match result {
                Ok(final_output) => {
                    tracing::info!("Workflow completed: {}", workflow.name);
                    Ok(WorkflowResult {
                        workflow_name: workflow.name.clone(),
                        success: true,
                        steps_executed,
                        step_results,
                        total_execution_time_ms: total_execution_time,
                        final_output: Some(final_output),
                        error: None,
                    })
                }
                Err(error) => {
                    tracing::error!("ERROR: Workflow failed: {} - {}", workflow.name, error);
                    Ok(WorkflowResult {
                        workflow_name: workflow.name.clone(),
                        success: false,
                        steps_executed,
                        step_results,
                        total_execution_time_ms: total_execution_time,
                        final_output: None,
                        error: Some(error.to_string()),
                    })
                }
            },
            Err(_timeout_error) => {
                tracing::error!("⏰ Workflow timeout: {}", workflow.name);
                Ok(WorkflowResult {
                    workflow_name: workflow.name.clone(),
                    success: false,
                    steps_executed,
                    step_results,
                    total_execution_time_ms: total_execution_time,
                    final_output: None,
                    error: Some("Workflow execution timeout".to_string()),
                })
            }
        }
    }

    /// Execute all workflow steps sequentially
    async fn execute_steps(
        &self,
        steps: &[Box<dyn WorkflowStep>],
        context: &mut WorkflowContext,
        step_results: &mut Vec<StepResult>,
        steps_executed: &mut usize,
    ) -> Result<serde_json::Value, AppError> {
        let mut final_output = serde_json::Value::Null;

        for (index, step) in steps.iter().enumerate() {
            tracing::debug!("🔄 Executing step {}: {}", index + 1, step.name());

            let step_start = std::time::Instant::now();
            let result = step.execute(context).await;
            let execution_time = step_start.elapsed().as_millis() as u64;

            match result {
                Ok(mut step_result) => {
                    step_result.execution_time_ms = execution_time;
                    tracing::debug!("Step completed: {}", step.name());

                    // Update final output with this step's output
                    final_output = step_result.output.clone();

                    step_results.push(step_result);
                    *steps_executed += 1;
                }
                Err(error) => {
                    let step_result = StepResult {
                        success: false,
                        output: serde_json::Value::Null,
                        error: Some(error.to_string()),
                        execution_time_ms: execution_time,
                    };

                    tracing::error!("ERROR: Step failed: {} - {}", step.name(), error);
                    step_results.push(step_result);
                    *steps_executed += 1;

                    return Err(error);
                }
            }
        }

        Ok(final_output)
    }
}

/// Builder for creating workflows
pub struct WorkflowBuilder {
    name: String,
    description: String,
    steps: Vec<Box<dyn WorkflowStep>>,
    timeout_seconds: u64,
}

impl WorkflowBuilder {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: String::new(),
            steps: Vec::new(),
            timeout_seconds: 300, // 5 minutes default
        }
    }

    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = description.into();
        self
    }

    pub fn timeout_seconds(mut self, seconds: u64) -> Self {
        self.timeout_seconds = seconds;
        self
    }

    pub fn add_step(mut self, step: Box<dyn WorkflowStep>) -> Self {
        self.steps.push(step);
        self
    }

    pub fn build(self) -> Workflow {
        Workflow {
            name: self.name,
            description: self.description,
            steps: self.steps,
            timeout_seconds: self.timeout_seconds,
        }
    }
}
