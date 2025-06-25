use crate::{AiAdapter, AiService, Message};
use anyhow::anyhow;
use reqwest::Client;
use serde::{Deserialize, Serialize};

pub struct OpenaiAdapter {
  host: String,
  api_key: String,
  model: String,
  client: Client,
}

#[derive(Serialize)]
pub struct OpenAIChatCompletionRequest {
  pub model: String,
  pub messages: Vec<OpenAIMessage>,
}

#[derive(Serialize, Deserialize)]
pub struct OpenAIMessage {
  pub role: String,
  pub content: String,
}
#[derive(Deserialize)]
pub struct OpenAIChatCompletionResponse {
  pub id: String,
  pub object: String,
  pub created: u64,
  pub model: String,
  pub system_fingerprint: String,
  pub choices: Vec<OpenAIChoice>,
  pub usage: OpenAIUsage,
}

#[derive(Deserialize)]
pub struct OpenAIChoice {
  pub index: u32,
  pub message: OpenAIMessage,
  pub logprobs: Option<i64>,
  pub finish_reason: String,
}

#[derive(Deserialize)]
pub struct OpenAIUsage {
  pub prompt_tokens: u32,
  pub completion_tokens: u32,
  pub total_tokens: u32,
  pub completion_tokens_details: Option<OpenAICompletionTokensDetails>,
}

#[derive(Deserialize)]
pub struct OpenAICompletionTokensDetails {
  pub reasoning_tokens: u32,
}

#[derive(Serialize)]
pub struct EmbeddingRequest {
  pub model: String,
  pub input: Vec<String>,
}

#[derive(Deserialize)]
pub struct EmbeddingResponse {
  pub data: Vec<EmbeddingData>,
}

#[derive(Deserialize)]
pub struct EmbeddingData {
  pub embedding: Vec<f32>,
}

#[derive(Serialize)]
pub struct ModerationRequest {
  pub input: String,
}

#[derive(Deserialize)]
pub struct ModerationResponse {
  pub results: Vec<ModerationResult>,
}

#[derive(Deserialize)]
pub struct ModerationResult {
  pub flagged: bool,
}

impl OpenaiAdapter {
  pub fn new(api_key: impl Into<String>, model: impl Into<String>) -> Self {
    let client = Client::new();
    Self {
      host: "https://api.openai.com/v1".to_string(),
      api_key: api_key.into(),
      model: model.into(),
      client,
    }
  }
}

impl AiService for OpenaiAdapter {
  async fn complete(&self, messages: &[Message]) -> anyhow::Result<String> {
    let request = OpenAIChatCompletionRequest {
      model: self.model.clone(),
      messages: messages.iter().map(|m| m.into()).collect(),
    };

    let url = format!("{}/chat/completions", self.host);
    let response = self
      .client
      .post(url)
      .json(&request)
      .header("Authorization", format!("Bearer {}", self.api_key))
      .send()
      .await?;
    let text = response.text().await?;
    println!("OpenAI API Response: {}", text);
    
    // Check if response contains an error
    if text.contains("error") {
        let error: serde_json::Value = serde_json::from_str(&text)?;
        if let Some(err_obj) = error.get("error") {
            if let Some(message) = err_obj.get("message") {
                return Err(anyhow!("OpenAI API Error: {}", message));
            }
        }
        return Err(anyhow!("Unknown OpenAI API Error: {}", text));
    }
    
    let mut data: OpenAIChatCompletionResponse = serde_json::from_str(&text)?;
    let content = data
      .choices
      .pop()
      .ok_or(anyhow!("No response"))?
      .message
      .content;
    Ok(content)
  }
  
  async fn embed_texts(&self, texts: Vec<String>) -> anyhow::Result<Vec<Vec<f32>>> {
    let request = EmbeddingRequest {
      model: "text-embedding-3-small".to_string(),
      input: texts,
    };

    let url = format!("{}/embeddings", self.host);
    let response = self
      .client
      .post(url)
      .header("Authorization", format!("Bearer {}", self.api_key))
      .json(&request)
      .send()
      .await?;

    if !response.status().is_success() {
      let error_text = response.text().await.unwrap_or_default();
      return Err(anyhow!("OpenAI API error: {}", error_text));
    }

    let embedding_response: EmbeddingResponse = response.json().await?;
    Ok(
      embedding_response
        .data
        .into_iter()
        .map(|d| d.embedding)
        .collect(),
    )
  }
  
  async fn moderate_content(&self, content: &str) -> anyhow::Result<bool> {
    let request = ModerationRequest {
      input: content.to_string(),
    };

    let url = format!("{}/moderations", self.host);
    let response = self
      .client
      .post(url)
      .header("Authorization", format!("Bearer {}", self.api_key))
      .json(&request)
      .send()
      .await?;

    if !response.status().is_success() {
      let error_text = response.text().await.unwrap_or_default();
      return Err(anyhow!("OpenAI API error: {}", error_text));
    }

    let moderation: ModerationResponse = response.json().await?;
    Ok(
      moderation
        .results
        .first()
        .map(|r| !r.flagged)
        .unwrap_or(true),
    )
  }
}

impl From<OpenaiAdapter> for AiAdapter {
  fn from(adapter: OpenaiAdapter) -> Self {
    AiAdapter::Openai(adapter)
  }
}

impl From<Message> for OpenAIMessage {
  fn from(message: Message) -> Self {
    OpenAIMessage {
      role: message.role.to_string(),
      content: message.content,
    }
  }
}

impl From<&Message> for OpenAIMessage {
  fn from(message: &Message) -> Self {
    OpenAIMessage {
      role: message.role.to_string(),
      content: message.content.clone(),
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::Role;
  use std::env;

  #[ignore]
  #[tokio::test]
  async fn openai_complete_should_work() {
    let api_key = env::var("OPENAI_API_KEY").unwrap();
    let adapter = OpenaiAdapter::new(api_key, "gpt-4o");
    let messages = vec![Message {
      role: Role::User,
      content: "Hello".to_string(),
    }];
    let response = adapter.complete(&messages).await.unwrap();
    assert!(!response.is_empty());
  }
}
