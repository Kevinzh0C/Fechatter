//! Simple AI workflow - delegate complexity to LLM
use crate::error::AppError;
use std::sync::Arc;

/// Simple workflow executor that chains AI operations
pub struct SimpleWorkflow {
    openai_client: Arc<crate::services::ai::openai::OpenAIClient>,
}

impl SimpleWorkflow {
    pub fn new(openai_client: Arc<crate::services::ai::openai::OpenAIClient>) -> Self {
        Self { openai_client }
    }

    /// Complete chat analysis workflow
    pub async fn analyze_chat_complete(
        &self,
        chat_id: i64,
        messages: Vec<(i64, i64, String, chrono::DateTime<chrono::Utc>)>,
    ) -> Result<ChatAnalysisResult, AppError> {
        tracing::info!("🔄 Starting complete chat analysis for chat {}", chat_id);

        // Step 1: Generate summary (let LLM do it)
        let summary = self.generate_chat_summary(&messages).await?;

        // Step 2: Extract topics (let LLM do it)
        let topics = self.extract_topics_simple(&messages).await?;

        // Step 3: Analyze sentiment (let LLM do it)
        let sentiment = self.analyze_overall_sentiment(&messages).await?;

        // Step 4: Create timeline (simple version)
        let timeline = self.create_simple_timeline(&messages, &topics).await?;

        tracing::info!("Chat analysis completed for chat {}", chat_id);

        Ok(ChatAnalysisResult {
            chat_id,
            summary,
            topics,
            sentiment,
            timeline,
            message_count: messages.len(),
            analyzed_at: chrono::Utc::now(),
        })
    }

    /// Generate chat summary using LLM
    async fn generate_chat_summary(
        &self,
        messages: &[(i64, i64, String, chrono::DateTime<chrono::Utc>)],
    ) -> Result<String, AppError> {
        let messages_text = messages
            .iter()
            .take(20) // Limit for efficiency
            .map(|(_, _, content, _)| content.as_str())
            .collect::<Vec<_>>()
            .join("\n");

        let prompt = format!(
            "Summarize this chat conversation in 2-3 sentences. Focus on main topics and key decisions:\n\n{}",
            messages_text
        );

        fechatter_core::contracts::infrastructure::AIService::generate_summary(
            &*self.openai_client,
            &prompt,
        )
        .await
        .map_err(|e| AppError::AnyError(anyhow::anyhow!("Summary generation failed: {}", e)))
    }

    /// Extract topics using simple LLM call
    async fn extract_topics_simple(
        &self,
        messages: &[(i64, i64, String, chrono::DateTime<chrono::Utc>)],
    ) -> Result<Vec<String>, AppError> {
        let messages_text = messages
            .iter()
            .take(30)
            .map(|(_, _, content, _)| content.as_str())
            .collect::<Vec<_>>()
            .join("\n");

        let prompt = format!(
            "List the main topics discussed in this conversation. Return only a comma-separated list of topics:\n\n{}",
            messages_text
        );

        let response = fechatter_core::contracts::infrastructure::AIService::chat_completion(
            &*self.openai_client,
            vec![fechatter_core::contracts::infrastructure::ChatMessage {
                role: "user".to_string(),
                content: prompt,
            }],
        )
        .await
        .map_err(|e| AppError::AnyError(anyhow::anyhow!("Topic extraction failed: {}", e)))?;

        // Parse comma-separated topics
        let topics: Vec<String> = response
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .take(5) // Limit topics
            .collect();

        Ok(topics)
    }

    /// Analyze overall sentiment
    async fn analyze_overall_sentiment(
        &self,
        messages: &[(i64, i64, String, chrono::DateTime<chrono::Utc>)],
    ) -> Result<SentimentSummary, AppError> {
        let messages_text = messages
            .iter()
            .take(15)
            .map(|(_, _, content, _)| content.as_str())
            .collect::<Vec<_>>()
            .join("\n");

        let sentiment = fechatter_core::contracts::infrastructure::AIService::analyze_sentiment(
            &*self.openai_client,
            &messages_text,
        )
        .await
        .map_err(|e| AppError::AnyError(anyhow::anyhow!("Sentiment analysis failed: {}", e)))?;

        Ok(SentimentSummary {
            overall_label: sentiment.label,
            overall_score: sentiment.score,
            message_count: messages.len(),
        })
    }

    /// Create simple timeline
    async fn create_simple_timeline(
        &self,
        messages: &[(i64, i64, String, chrono::DateTime<chrono::Utc>)],
        topics: &[String],
    ) -> Result<Vec<TimelineEvent>, AppError> {
        let mut events = Vec::new();

        // Create events for each topic
        for topic in topics {
            let related_messages: Vec<_> = messages
                .iter()
                .filter(|(_, _, content, _)| content.to_lowercase().contains(&topic.to_lowercase()))
                .collect();

            if !related_messages.is_empty() {
                let first_time = related_messages.first().unwrap().3;
                let last_time = related_messages.last().unwrap().3;

                events.push(TimelineEvent {
                    title: format!("Discussion: {}", topic),
                    description: format!("{} messages about {}", related_messages.len(), topic),
                    start_time: first_time,
                    end_time: last_time,
                    event_type: "topic".to_string(),
                });
            }
        }

        // Sort by time
        events.sort_by(|a, b| a.start_time.cmp(&b.start_time));

        Ok(events)
    }
}

/// Chat analysis result
#[derive(Debug, serde::Serialize)]
pub struct ChatAnalysisResult {
    pub chat_id: i64,
    pub summary: String,
    pub topics: Vec<String>,
    pub sentiment: SentimentSummary,
    pub timeline: Vec<TimelineEvent>,
    pub message_count: usize,
    pub analyzed_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, serde::Serialize)]
pub struct SentimentSummary {
    pub overall_label: String,
    pub overall_score: f32,
    pub message_count: usize,
}

#[derive(Debug, serde::Serialize)]
pub struct TimelineEvent {
    pub title: String,
    pub description: String,
    pub start_time: chrono::DateTime<chrono::Utc>,
    pub end_time: chrono::DateTime<chrono::Utc>,
    pub event_type: String,
}
