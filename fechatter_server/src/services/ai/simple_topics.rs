//! Simple topic extraction using LLM - let AI do all the work
use crate::error::AppError;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopicCluster {
    pub name: String,
    pub keywords: Vec<String>,
    pub message_ids: Vec<i64>,
    pub importance_score: f32,
}

/// Simple topic extractor that delegates everything to LLM
pub struct SimpleTopicExtractor {
    openai_client: std::sync::Arc<crate::services::ai::openai::OpenAIClient>,
}

impl SimpleTopicExtractor {
    pub fn new(openai_client: std::sync::Arc<crate::services::ai::openai::OpenAIClient>) -> Self {
        Self { openai_client }
    }
    
    /// Extract topics - let LLM do all the heavy lifting
    pub async fn extract_topics(
        &self,
        messages: &[(i64, String)],
        max_topics: usize,
    ) -> Result<Vec<TopicCluster>, AppError> {
        if messages.is_empty() {
            return Ok(Vec::new());
        }

        // Prepare message content for LLM
        let messages_text = messages.iter()
            .take(50) // Limit for API efficiency
            .map(|(id, content)| format!("{}: {}", id, content))
            .collect::<Vec<_>>()
            .join("\n");
        
        let prompt = format!(
            "Analyze these chat messages and extract {} main topics. Return ONLY a JSON array with this exact format:
[
  {{\"name\": \"Topic Name\", \"keywords\": [\"word1\", \"word2\", \"word3\"], \"message_ids\": [1,2,3]}},
  {{\"name\": \"Another Topic\", \"keywords\": [\"word4\", \"word5\"], \"message_ids\": [4,5]}}
]

Messages:
{}

JSON:", max_topics, messages_text
        );
        
        // Let OpenAI analyze topics
        let response = fechatter_core::contracts::infrastructure::AIService::chat_completion(
            &*self.openai_client,
            vec![fechatter_core::contracts::infrastructure::ChatMessage {
                role: "user".to_string(),
                content: prompt,
            }]
        ).await.map_err(|e| AppError::AnyError(anyhow::anyhow!("Topic extraction failed: {}", e)))?;
        
        // Try to parse JSON response
        match serde_json::from_str::<Vec<serde_json::Value>>(&response) {
            Ok(topics_json) => {
                let mut topics = Vec::new();
                for (i, topic_json) in topics_json.iter().enumerate() {
                    if let (Some(name), Some(keywords_json), Some(message_ids_json)) = (
                        topic_json.get("name").and_then(|v| v.as_str()),
                        topic_json.get("keywords").and_then(|v| v.as_array()),
                        topic_json.get("message_ids").and_then(|v| v.as_array())
                    ) {
                        let keywords: Vec<String> = keywords_json
                            .iter()
                            .filter_map(|v| v.as_str().map(|s| s.to_string()))
                            .collect();
                            
                        let message_ids: Vec<i64> = message_ids_json
                            .iter()
                            .filter_map(|v| v.as_i64())
                            .collect();
                        
                        topics.push(TopicCluster {
                            name: name.to_string(),
                            keywords,
                            message_ids,
                            importance_score: 1.0 - (i as f32 * 0.1), // Decreasing importance
                        });
                    }
                }
                Ok(topics)
            },
            Err(_) => {
                // Fallback: simple keyword-based clustering
                tracing::warn!("LLM topic extraction failed, using fallback");
                self.fallback_topic_extraction(messages, max_topics).await
            }
        }
    }
    
    /// Simple fallback topic extraction
    async fn fallback_topic_extraction(
        &self,
        messages: &[(i64, String)],
        max_topics: usize,
    ) -> Result<Vec<TopicCluster>, AppError> {
        let mut word_counts = std::collections::HashMap::new();
        
        // Count words
        for (msg_id, content) in messages {
            let words: Vec<&str> = content
                .split_whitespace()
                .filter(|w| w.len() > 3)
                .take(10) // Limit words per message
                .collect();
            
            for word in words {
                let word_lower = word.to_lowercase();
                let entry = word_counts.entry(word_lower).or_insert((0, Vec::new()));
                entry.0 += 1;
                if !entry.1.contains(msg_id) {
                    entry.1.push(*msg_id);
                }
            }
        }
        
        // Create topics from most frequent words
        let mut word_vec: Vec<(String, (usize, Vec<i64>))> = word_counts.into_iter().collect();
        word_vec.sort_by(|a, b| b.1.0.cmp(&a.1.0));
        
        let topics: Vec<TopicCluster> = word_vec
            .into_iter()
            .take(max_topics)
            .enumerate()
            .map(|(i, (word, (count, msg_ids)))| TopicCluster {
                name: format!("Topic: {}", word),
                keywords: vec![word],
                message_ids: msg_ids,
                importance_score: (count as f32 / messages.len() as f32).min(1.0) - (i as f32 * 0.1),
            })
            .collect();
        
        Ok(topics)
    }
}