use anyhow;
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

use crate::{error::AppError, services::infrastructure::third_party_manager::OpenAIConfig};
use fechatter_core::contracts::infrastructure::{AIService, ChatMessage, Sentiment};

#[derive(Debug, Serialize)]
struct CompletionRequest {
  model: String,
  messages: Vec<Message>,
  temperature: f32,
  max_tokens: u32,
}

#[derive(Debug, Serialize, Deserialize)]
struct Message {
  role: String,
  content: String,
}

#[derive(Debug, Deserialize)]
struct CompletionResponse {
  choices: Vec<Choice>,
}

#[derive(Debug, Deserialize)]
struct Choice {
  message: Message,
}

#[derive(Debug, Serialize)]
struct EmbeddingRequest {
  model: String,
  input: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct EmbeddingResponse {
  data: Vec<EmbeddingData>,
}

#[derive(Debug, Deserialize)]
struct EmbeddingData {
  embedding: Vec<f32>,
}

#[derive(Debug)]
pub struct OpenAIClient {
  client: Client,
  config: OpenAIConfig,
}

impl OpenAIClient {
  /// Get the base URL for API calls
  fn base_url(&self) -> &str {
    self
      .config
      .base_url
      .as_deref()
      .unwrap_or("https://api.openai.com/v1")
  }

  /// Create a new OpenAI client with configuration
  pub fn new(config: OpenAIConfig) -> Result<Self, AppError> {
    config.validate()?;

    let client = Client::builder()
      .timeout(Duration::from_secs(config.timeout_seconds))
      .build()
      .map_err(|e| AppError::AnyError(anyhow::anyhow!("Failed to create HTTP client: {}", e)))?;

    Ok(Self { client, config })
  }

  /// Create a new OpenAI client from environment variables
  pub fn from_env() -> Result<Self, AppError> {
    let config = OpenAIConfig::from_env()?;
    Self::new(config)
  }

  /// Create a new OpenAI client with just API key (uses defaults for other settings)
  pub fn with_api_key(api_key: String) -> Result<Self, AppError> {
    let config = OpenAIConfig::new(api_key);
    Self::new(config)
  }

  /// Get the current configuration
  pub fn config(&self) -> &OpenAIConfig {
    &self.config
  }

  pub async fn embed_texts(&self, texts: Vec<String>) -> Result<Vec<Vec<f32>>, AppError> {
    let request = EmbeddingRequest {
      model: "text-embedding-3-small".to_string(),
      input: texts,
    };

    let response = self
      .client
      .post(format!("{}/embeddings", self.base_url()))
      .header("Authorization", format!("Bearer {}", self.config.api_key))
      .json(&request)
      .send()
      .await
      .map_err(|e| AppError::AnyError(anyhow::anyhow!("OpenAI API error: {}", e)))?;

    if !response.status().is_success() {
      let error_text = response.text().await.unwrap_or_default();
      return Err(AppError::AnyError(anyhow::anyhow!(
        "OpenAI API error: {}",
        error_text
      )));
    }

    let embedding_response: EmbeddingResponse = response
      .json()
      .await
      .map_err(|e| AppError::AnyError(anyhow::anyhow!("Failed to parse response: {}", e)))?;

    Ok(
      embedding_response
        .data
        .into_iter()
        .map(|d| d.embedding)
        .collect(),
    )
  }

  /// Generate a summary for given text
  pub async fn generate_summary(&self, text: &str) -> Result<String, AppError> {
    let request = CompletionRequest {
      model: self.config.default_model.clone(),
      messages: vec![
        Message {
          role: "system".to_string(),
          content: "You are a helpful assistant that creates concise summaries.".to_string(),
        },
        Message {
          role: "user".to_string(),
          content: format!("Please summarize the following text:\n\n{}", text),
        },
      ],
      temperature: self.config.temperature,
      max_tokens: std::cmp::min(self.config.max_tokens, 150), // Limit for summaries
    };

    let response = self
      .client
      .post(format!("{}/chat/completions", self.base_url()))
      .header("Authorization", format!("Bearer {}", self.config.api_key))
      .header("Content-Type", "application/json")
      .json(&request)
      .send()
      .await
      .map_err(|e| AppError::AnyError(anyhow::anyhow!("OpenAI request failed: {}", e)))?;

    if !response.status().is_success() {
      let error_text = response
        .text()
        .await
        .unwrap_or_else(|_| "Unknown error".to_string());
      return Err(AppError::AnyError(anyhow::anyhow!(
        "OpenAI API error: {}",
        error_text
      )));
    }

    let completion: CompletionResponse = response
      .json()
      .await
      .map_err(|e| AppError::AnyError(anyhow::anyhow!("Failed to parse OpenAI response: {}", e)))?;

    completion
      .choices
      .first()
      .map(|choice| choice.message.content.trim().to_string())
      .ok_or_else(|| AppError::AnyError(anyhow::anyhow!("Empty response from OpenAI")))
  }

  /// Generate embedding for text using OpenAI's embedding model
  pub async fn generate_embedding(&self, text: &str) -> Result<Vec<f32>, AppError> {
    let embeddings = self.embed_texts(vec![text.to_string()]).await?;
    embeddings
      .into_iter()
      .next()
      .ok_or_else(|| AppError::AnyError(anyhow::anyhow!("Failed to generate embedding")))
  }

  /// Generate a suggested reply based on context
  pub async fn suggest_reply(&self, context: &str) -> Result<Vec<String>, AppError> {
    let request = CompletionRequest {
      model: self.config.default_model.clone(),
      messages: vec![
        Message {
          role: "system".to_string(),
          content: "You are a helpful assistant that suggests appropriate replies to messages. Provide 3 different reply options, each on a new line.".to_string(),
        },
        Message {
          role: "user".to_string(),
          content: format!("Based on this conversation context, suggest 3 possible replies:\n\n{}", context),
        },
      ],
      temperature: (self.config.temperature + 0.1).min(2.0), // Slightly higher for creativity
      max_tokens: std::cmp::min(self.config.max_tokens, 200), // Limit for suggestions
    };

    let response = self
      .client
      .post(format!("{}/chat/completions", self.base_url()))
      .header("Authorization", format!("Bearer {}", self.config.api_key))
      .json(&request)
      .send()
      .await
      .map_err(|e| AppError::AnyError(anyhow::anyhow!("OpenAI API error: {}", e)))?;

    if !response.status().is_success() {
      let error_text = response.text().await.unwrap_or_default();
      return Err(AppError::AnyError(anyhow::anyhow!(
        "OpenAI API error: {}",
        error_text
      )));
    }

    let completion: CompletionResponse = response
      .json()
      .await
      .map_err(|e| AppError::AnyError(anyhow::anyhow!("Failed to parse response: {}", e)))?;

    let suggestions = completion
      .choices
      .first()
      .map(|c| c.message.content.clone())
      .ok_or_else(|| AppError::AnyError(anyhow::anyhow!("No response from OpenAI")))?;

    Ok(
      suggestions
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(|s| s.to_string())
        .collect(),
    )
  }

  async fn moderate_content(&self, content: &str) -> Result<bool, AppError> {
    #[derive(Debug, Serialize)]
    struct ModerationRequest {
      input: String,
    }

    #[derive(Debug, Deserialize)]
    struct ModerationResponse {
      results: Vec<ModerationResult>,
    }

    #[derive(Debug, Deserialize)]
    struct ModerationResult {
      flagged: bool,
    }

    let request = ModerationRequest {
      input: content.to_string(),
    };

    let response = self
      .client
      .post(format!("{}/moderations", self.base_url()))
      .header("Authorization", format!("Bearer {}", self.config.api_key))
      .json(&request)
      .send()
      .await
      .map_err(|e| AppError::AnyError(anyhow::anyhow!("OpenAI API error: {}", e)))?;

    if !response.status().is_success() {
      let error_text = response.text().await.unwrap_or_default();
      return Err(AppError::AnyError(anyhow::anyhow!(
        "OpenAI API error: {}",
        error_text
      )));
    }

    let moderation: ModerationResponse = response
      .json()
      .await
      .map_err(|e| AppError::AnyError(anyhow::anyhow!("Failed to parse response: {}", e)))?;

    Ok(
      moderation
        .results
        .first()
        .map(|r| !r.flagged)
        .unwrap_or(true),
    )
  }
}

#[async_trait]
impl AIService for OpenAIClient {
  async fn chat_completion(
    &self,
    messages: Vec<ChatMessage>,
  ) -> Result<String, fechatter_core::error::CoreError> {
    let openai_messages: Vec<Message> = messages
      .into_iter()
      .map(|m| Message {
        role: m.role,
        content: m.content,
      })
      .collect();

    let request = CompletionRequest {
      model: self.config.default_model.clone(),
      messages: openai_messages,
      temperature: self.config.temperature,
      max_tokens: self.config.max_tokens,
    };

    let response = self
      .client
      .post(format!("{}/chat/completions", self.base_url()))
      .header("Authorization", format!("Bearer {}", self.config.api_key))
      .json(&request)
      .send()
      .await
      .map_err(|e| fechatter_core::error::CoreError::Internal(e.to_string()))?;

    if !response.status().is_success() {
      let error_text = response.text().await.unwrap_or_default();
      return Err(fechatter_core::error::CoreError::Internal(format!(
        "OpenAI API error: {}",
        error_text
      )));
    }

    let completion: CompletionResponse = response
      .json()
      .await
      .map_err(|e| fechatter_core::error::CoreError::Internal(e.to_string()))?;

    completion
      .choices
      .first()
      .map(|c| c.message.content.clone())
      .ok_or_else(|| {
        fechatter_core::error::CoreError::Internal("No response from OpenAI".to_string())
      })
  }

  async fn generate_summary(&self, text: &str) -> Result<String, fechatter_core::error::CoreError> {
    OpenAIClient::generate_summary(self, text)
      .await
      .map_err(|e| fechatter_core::error::CoreError::Internal(e.to_string()))
  }

  async fn analyze_sentiment(
    &self,
    text: &str,
  ) -> Result<Sentiment, fechatter_core::error::CoreError> {
    let messages = vec![
      ChatMessage {
        role: "system".to_string(),
        content: "You are a sentiment analysis assistant. Analyze the sentiment and return a JSON response with 'score' (-1.0 to 1.0) and 'label' (positive/neutral/negative).".to_string(),
      },
      ChatMessage {
        role: "user".to_string(),
        content: format!("Analyze the sentiment of: {}", text),
      },
    ];

    let response = self.chat_completion(messages).await?;

    // Parse the response as JSON
    let sentiment_data: serde_json::Value = serde_json::from_str(&response).unwrap_or_else(|_| {
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
    self
      .suggest_reply(context)
      .await
      .map_err(|e| fechatter_core::error::CoreError::Internal(e.to_string()))
  }
}
