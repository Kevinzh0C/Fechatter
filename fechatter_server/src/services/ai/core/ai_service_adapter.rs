use ai_sdk::{AiAdapter, AiService, Message as AiMessage, OpenaiAdapter, Role as AiRole};
use anyhow;
use async_trait::async_trait;

use crate::{error::AppError, services::infrastructure::third_party_manager::OpenAIConfig};
use fechatter_core::contracts::infrastructure::{AIService, ChatMessage, Sentiment};

/// Adapter that wraps ai_sdk to implement fechatter's AIService trait
pub struct AiServiceAdapter {
    adapter: AiAdapter,
}

impl AiServiceAdapter {
    /// Create from OpenAI configuration
    pub fn from_openai_config(config: OpenAIConfig) -> Result<Self, AppError> {
        config.validate()?;

        let openai_adapter =
            OpenaiAdapter::new(config.api_key.clone(), config.default_model.clone());

        Ok(Self {
            adapter: openai_adapter.into(),
        })
    }

    /// Create from environment variables
    pub fn from_env() -> Result<Self, AppError> {
        let config = OpenAIConfig::from_env()?;
        Self::from_openai_config(config)
    }

    /// Convert fechatter ChatMessage to ai_sdk Message
    fn convert_chat_message(message: ChatMessage) -> AiMessage {
        let role = match message.role.as_str() {
            "user" => AiRole::User,
            "assistant" => AiRole::Assistant,
            "system" => AiRole::System,
            _ => AiRole::User, // Default fallback
        };

        AiMessage::new(role, message.content)
    }

    /// Convert vector of ChatMessage to ai_sdk Messages
    fn convert_chat_messages(messages: Vec<ChatMessage>) -> Vec<AiMessage> {
        messages
            .into_iter()
            .map(Self::convert_chat_message)
            .collect()
    }
}

#[async_trait]
impl AIService for AiServiceAdapter {
    async fn chat_completion(
        &self,
        messages: Vec<ChatMessage>,
    ) -> Result<String, fechatter_core::error::CoreError> {
        let ai_messages = Self::convert_chat_messages(messages);

        self.adapter
            .complete(&ai_messages)
            .await
            .map_err(|e| fechatter_core::error::CoreError::Internal(e.to_string()))
    }

    async fn generate_summary(
        &self,
        text: &str,
    ) -> Result<String, fechatter_core::error::CoreError> {
        self.adapter
            .generate_summary(text)
            .await
            .map_err(|e| fechatter_core::error::CoreError::Internal(e.to_string()))
    }

    async fn analyze_sentiment(
        &self,
        text: &str,
    ) -> Result<Sentiment, fechatter_core::error::CoreError> {
        let messages = vec![
            AiMessage::system("You are a sentiment analysis assistant. Analyze the sentiment and return a JSON response with 'score' (-1.0 to 1.0) and 'label' (positive/neutral/negative)."),
            AiMessage::user(format!("Analyze the sentiment of: {}", text)),
        ];

        let response = self
            .adapter
            .complete(&messages)
            .await
            .map_err(|e| fechatter_core::error::CoreError::Internal(e.to_string()))?;

        // Parse the response as JSON
        let sentiment_data: serde_json::Value =
            serde_json::from_str(&response).unwrap_or_else(|_| {
                serde_json::json!({
                    "score": 0.0,
                    "label": "neutral"
                })
            });

        Ok(Sentiment {
            score: sentiment_data["score"].as_f64().unwrap_or(0.0) as f32,
            label: sentiment_data["label"]
                .as_str()
                .unwrap_or("neutral")
                .to_string(),
        })
    }

    async fn suggest_replies(
        &self,
        context: &str,
    ) -> Result<Vec<String>, fechatter_core::error::CoreError> {
        self.adapter
            .suggest_replies(context)
            .await
            .map_err(|e| fechatter_core::error::CoreError::Internal(e.to_string()))
    }
}

/// Extended AI service with additional utility methods
impl AiServiceAdapter {
    /// Generate embeddings for texts
    pub async fn embed_texts(&self, texts: Vec<String>) -> Result<Vec<Vec<f32>>, AppError> {
        self.adapter
            .embed_texts(texts)
            .await
            .map_err(|e| AppError::AnyError(anyhow::anyhow!("Embedding generation failed: {}", e)))
    }

    /// Generate single embedding
    pub async fn generate_embedding(&self, text: &str) -> Result<Vec<f32>, AppError> {
        self.adapter
            .generate_embedding(text)
            .await
            .map_err(|e| AppError::AnyError(anyhow::anyhow!("Embedding generation failed: {}", e)))
    }

    /// Moderate content
    pub async fn moderate_content(&self, content: &str) -> Result<bool, AppError> {
        self.adapter
            .moderate_content(content)
            .await
            .map_err(|e| AppError::AnyError(anyhow::anyhow!("Content moderation failed: {}", e)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fechatter_core::contracts::infrastructure::ChatMessage;

    #[test]
    fn test_convert_chat_message() {
        let chat_message = ChatMessage {
            role: "user".to_string(),
            content: "Hello world".to_string(),
        };

        let ai_message = AiServiceAdapter::convert_chat_message(chat_message);
        assert_eq!(ai_message.content, "Hello world");
        // Role comparison would need access to AiRole internals
    }

    #[test]
    fn test_convert_chat_messages() {
        let chat_messages = vec![
            ChatMessage {
                role: "system".to_string(),
                content: "You are helpful".to_string(),
            },
            ChatMessage {
                role: "user".to_string(),
                content: "Hello".to_string(),
            },
        ];

        let ai_messages = AiServiceAdapter::convert_chat_messages(chat_messages);
        assert_eq!(ai_messages.len(), 2);
        assert_eq!(ai_messages[0].content, "You are helpful");
        assert_eq!(ai_messages[1].content, "Hello");
    }
}
