//! Pre-built workflow steps for common AI operations

use super::{WorkflowStep, WorkflowContext, StepResult};
use crate::{
    error::AppError,
    services::ai::{
        openai::OpenAIClient,
        agents::{search_agent::SearchAgent, summary_agent::SummaryAgent, timeline_agent::TimelineIndexAgent},
        hybrid_search::HybridSearchService,
    },
};
use async_trait::async_trait;
use serde_json::json;
use std::sync::Arc;

/// Step to analyze sentiment of text
pub struct SentimentAnalysisStep {
    openai_client: Arc<OpenAIClient>,
    input_variable: String,
    output_variable: String,
}

impl SentimentAnalysisStep {
    pub fn new(
        openai_client: Arc<OpenAIClient>,
        input_variable: impl Into<String>,
        output_variable: impl Into<String>,
    ) -> Self {
        Self {
            openai_client,
            input_variable: input_variable.into(),
            output_variable: output_variable.into(),
        }
    }
}

#[async_trait]
impl WorkflowStep for SentimentAnalysisStep {
    async fn execute(&self, context: &mut WorkflowContext) -> Result<StepResult, AppError> {
        let text = context
            .get_variable(&self.input_variable)
            .and_then(|v| v.as_str())
            .ok_or_else(|| AppError::AnyError(anyhow::anyhow!("Input variable '{}' not found or not a string", self.input_variable)))?;
        
        let sentiment = fechatter_core::contracts::infrastructure::AIService::analyze_sentiment(
            &*self.openai_client, text
        ).await
        .map_err(|e| AppError::AnyError(anyhow::anyhow!("Sentiment analysis failed: {}", e)))?;
        
        let output = json!({
            "label": sentiment.label,
            "score": sentiment.score
        });
        
        context.set_variable(&self.output_variable, output.clone());
        
        Ok(StepResult {
            success: true,
            output,
            error: None,
            execution_time_ms: 0, // Will be set by engine
        })
    }
    
    fn name(&self) -> &str {
        "sentiment_analysis"
    }
    
    fn description(&self) -> &str {
        "Analyze sentiment of input text"
    }
}

/// Step to search messages using hybrid search
pub struct MessageSearchStep {
    search_agent: Arc<SearchAgent>,
    query_variable: String,
    output_variable: String,
    limit: usize,
}

impl MessageSearchStep {
    pub fn new(
        search_agent: Arc<SearchAgent>,
        query_variable: impl Into<String>,
        output_variable: impl Into<String>,
        limit: usize,
    ) -> Self {
        Self {
            search_agent,
            query_variable: query_variable.into(),
            output_variable: output_variable.into(),
            limit,
        }
    }
}

#[async_trait]
impl WorkflowStep for MessageSearchStep {
    async fn execute(&self, context: &mut WorkflowContext) -> Result<StepResult, AppError> {
        let query = context
            .get_variable(&self.query_variable)
            .and_then(|v| v.as_str())
            .ok_or_else(|| AppError::AnyError(anyhow::anyhow!("Query variable '{}' not found or not a string", self.query_variable)))?;
        
        let chat_id = context.chat_id.ok_or_else(|| {
            AppError::AnyError(anyhow::anyhow!("Chat ID required for message search"))
        })?;
        
        let search_results = self.search_agent
            .search_messages(query, chat_id, self.limit)
            .await?;
        
        let output = json!({
            "results": search_results.into_iter().map(|result| json!({
                "message_id": result.message_id,
                "content": result.content,
                "sender_id": result.sender_id,
                "timestamp": result.timestamp,
                "similarity_score": result.similarity_score
            })).collect::<Vec<_>>()
        });
        
        context.set_variable(&self.output_variable, output.clone());
        
        Ok(StepResult {
            success: true,
            output,
            error: None,
            execution_time_ms: 0,
        })
    }
    
    fn name(&self) -> &str {
        "message_search"
    }
    
    fn description(&self) -> &str {
        "Search messages using hybrid semantic search"
    }
}

/// Step to generate summary of text or messages
pub struct SummaryGenerationStep {
    openai_client: Arc<OpenAIClient>,
    input_variable: String,
    output_variable: String,
    summary_type: SummaryType,
}

#[derive(Debug, Clone)]
pub enum SummaryType {
    General,
    Conversation,
    ActionItems,
    KeyPoints,
}

impl SummaryGenerationStep {
    pub fn new(
        openai_client: Arc<OpenAIClient>,
        input_variable: impl Into<String>,
        output_variable: impl Into<String>,
        summary_type: SummaryType,
    ) -> Self {
        Self {
            openai_client,
            input_variable: input_variable.into(),
            output_variable: output_variable.into(),
            summary_type,
        }
    }
}

#[async_trait]
impl WorkflowStep for SummaryGenerationStep {
    async fn execute(&self, context: &mut WorkflowContext) -> Result<StepResult, AppError> {
        let input_text = match context.get_variable(&self.input_variable) {
            Some(serde_json::Value::String(text)) => text.clone(),
            Some(serde_json::Value::Array(messages)) => {
                // Convert message array to text
                messages.iter()
                    .filter_map(|msg| msg.as_object())
                    .map(|obj| format!("{}: {}", 
                        obj.get("sender").and_then(|s| s.as_str()).unwrap_or("Unknown"),
                        obj.get("content").and_then(|c| c.as_str()).unwrap_or("")
                    ))
                    .collect::<Vec<_>>()
                    .join("\n")
            },
            _ => return Err(AppError::AnyError(anyhow::anyhow!(
                "Input variable '{}' not found or invalid format", self.input_variable
            ))),
        };
        
        let prompt = match self.summary_type {
            SummaryType::General => format!("Please provide a concise summary of the following text:\n\n{}", input_text),
            SummaryType::Conversation => format!("Please summarize this conversation, highlighting the main topics discussed:\n\n{}", input_text),
            SummaryType::ActionItems => format!("Extract action items and decisions from this conversation:\n\n{}", input_text),
            SummaryType::KeyPoints => format!("List the key points from this text:\n\n{}", input_text),
        };
        
        let summary = self.openai_client.generate_summary(&prompt).await?;
        
        let output = json!({
            "summary": summary,
            "summary_type": format!("{:?}", self.summary_type),
            "input_length": input_text.len()
        });
        
        context.set_variable(&self.output_variable, output.clone());
        
        Ok(StepResult {
            success: true,
            output,
            error: None,
            execution_time_ms: 0,
        })
    }
    
    fn name(&self) -> &str {
        "summary_generation"
    }
    
    fn description(&self) -> &str {
        "Generate summary of input text or messages"
    }
}

/// Step to extract and cluster topics
pub struct TopicExtractionStep {
    summary_agent: Arc<SummaryAgent>,
    input_variable: String,
    output_variable: String,
    max_topics: usize,
}

impl TopicExtractionStep {
    pub fn new(
        summary_agent: Arc<SummaryAgent>,
        input_variable: impl Into<String>,
        output_variable: impl Into<String>,
        max_topics: usize,
    ) -> Self {
        Self {
            summary_agent,
            input_variable: input_variable.into(),
            output_variable: output_variable.into(),
            max_topics,
        }
    }
}

#[async_trait]
impl WorkflowStep for TopicExtractionStep {
    async fn execute(&self, context: &mut WorkflowContext) -> Result<StepResult, AppError> {
        let messages = context
            .get_variable(&self.input_variable)
            .and_then(|v| v.as_array())
            .ok_or_else(|| AppError::AnyError(anyhow::anyhow!("Input variable '{}' must be an array of messages", self.input_variable)))?;
        
        // Convert JSON messages to the format expected by SummaryAgent
        let message_data: Vec<(i64, String)> = messages
            .iter()
            .enumerate()
            .filter_map(|(i, msg)| {
                msg.as_object().and_then(|obj| {
                    let content = obj.get("content")?.as_str()?;
                    Some((i as i64, content.to_string()))
                })
            })
            .collect();
        
        let topics = self.summary_agent
            .extract_topics(&message_data, self.max_topics)
            .await?;
        
        let output = json!({
            "topics": topics.into_iter().map(|topic| json!({
                "name": topic.name,
                "keywords": topic.keywords,
                "message_count": topic.message_ids.len(),
                "importance_score": topic.importance_score
            })).collect::<Vec<_>>()
        });
        
        context.set_variable(&self.output_variable, output.clone());
        
        Ok(StepResult {
            success: true,
            output,
            error: None,
            execution_time_ms: 0,
        })
    }
    
    fn name(&self) -> &str {
        "topic_extraction"
    }
    
    fn description(&self) -> &str {
        "Extract and cluster topics from messages"
    }
}

/// Step to generate embeddings for text
pub struct EmbeddingGenerationStep {
    openai_client: Arc<OpenAIClient>,
    input_variable: String,
    output_variable: String,
}

impl EmbeddingGenerationStep {
    pub fn new(
        openai_client: Arc<OpenAIClient>,
        input_variable: impl Into<String>,
        output_variable: impl Into<String>,
    ) -> Self {
        Self {
            openai_client,
            input_variable: input_variable.into(),
            output_variable: output_variable.into(),
        }
    }
}

#[async_trait]
impl WorkflowStep for EmbeddingGenerationStep {
    async fn execute(&self, context: &mut WorkflowContext) -> Result<StepResult, AppError> {
        let text = context
            .get_variable(&self.input_variable)
            .and_then(|v| v.as_str())
            .ok_or_else(|| AppError::AnyError(anyhow::anyhow!("Input variable '{}' not found or not a string", self.input_variable)))?;
        
        let embedding = self.openai_client.generate_embedding(text).await?;
        
        let output = json!({
            "embedding": embedding,
            "dimensions": embedding.len(),
            "text_length": text.len()
        });
        
        context.set_variable(&self.output_variable, output.clone());
        
        Ok(StepResult {
            success: true,
            output,
            error: None,
            execution_time_ms: 0,
        })
    }
    
    fn name(&self) -> &str {
        "embedding_generation"
    }
    
    fn description(&self) -> &str {
        "Generate vector embedding for input text"
    }
}