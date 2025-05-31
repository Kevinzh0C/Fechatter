use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::error::AppError;
// Import AIService from fechatter_core where it's actually defined
use fechatter_core::contracts::infrastructure::AIService;

/// Cohere client for semantic search and text analysis
///
/// Cohere specializes in:
/// - High-quality text embeddings for semantic search
/// - Reranking search results for better relevance
/// - Text classification and clustering
/// - Language detection
pub struct CohereClient {
  client: Client,
  api_key: String,
  base_url: String,
}

#[derive(Debug, Serialize)]
struct EmbedRequest {
  texts: Vec<String>,
  model: String,
  input_type: String,
}

#[derive(Debug, Deserialize)]
struct EmbedResponse {
  embeddings: Vec<Vec<f32>>,
}

#[derive(Debug, Serialize)]
struct RerankRequest {
  query: String,
  documents: Vec<String>,
  model: String,
  top_n: Option<usize>,
}

#[derive(Debug, Deserialize)]
struct RerankResponse {
  results: Vec<RerankResult>,
}

#[derive(Debug, Deserialize)]
struct RerankResult {
  index: usize,
  relevance_score: f32,
}

#[derive(Debug, Serialize)]
struct ClassifyRequest {
  inputs: Vec<String>,
  examples: Vec<Example>,
}

#[derive(Debug, Serialize)]
struct Example {
  text: String,
  label: String,
}

#[derive(Debug, Deserialize)]
struct ClassifyResponse {
  classifications: Vec<Classification>,
}

#[derive(Debug, Deserialize)]
struct Classification {
  input: String,
  prediction: String,
  confidence: f32,
}

impl CohereClient {
  pub fn new(api_key: String) -> Self {
    Self {
      client: Client::new(),
      api_key,
      base_url: "https://api.cohere.ai/v1".to_string(),
    }
  }

  /// Generate embeddings for semantic search
  /// Used to convert text into vectors for similarity matching
  pub async fn embed_for_search(&self, texts: Vec<String>) -> Result<Vec<Vec<f32>>, AppError> {
    let request = EmbedRequest {
      texts,
      model: "embed-english-v3.0".to_string(),
      input_type: "search_document".to_string(),
    };

    let response = self
      .client
      .post(format!("{}/embed", self.base_url))
      .header("Authorization", format!("Bearer {}", self.api_key))
      .json(&request)
      .send()
      .await
      .map_err(|e| AppError::ExternalService(format!("Cohere API error: {}", e)))?;

    if !response.status().is_success() {
      let error_text = response.text().await.unwrap_or_default();
      return Err(AppError::ExternalService(format!(
        "Cohere API error: {}",
        error_text
      )));
    }

    let embed_response: EmbedResponse = response
      .json()
      .await
      .map_err(|e| AppError::ExternalService(format!("Failed to parse response: {}", e)))?;

    Ok(embed_response.embeddings)
  }

  /// Rerank search results based on relevance to query
  /// Improves search quality by reordering results
  pub async fn rerank_results(
    &self,
    query: String,
    documents: Vec<String>,
    top_n: Option<usize>,
  ) -> Result<Vec<(usize, f32)>, AppError> {
    let request = RerankRequest {
      query,
      documents,
      model: "rerank-english-v2.0".to_string(),
      top_n,
    };

    let response = self
      .client
      .post(format!("{}/rerank", self.base_url))
      .header("Authorization", format!("Bearer {}", self.api_key))
      .json(&request)
      .send()
      .await
      .map_err(|e| AppError::ExternalService(format!("Cohere API error: {}", e)))?;

    if !response.status().is_success() {
      let error_text = response.text().await.unwrap_or_default();
      return Err(AppError::ExternalService(format!(
        "Cohere API error: {}",
        error_text
      )));
    }

    let rerank_response: RerankResponse = response
      .json()
      .await
      .map_err(|e| AppError::ExternalService(format!("Failed to parse response: {}", e)))?;

    Ok(
      rerank_response
        .results
        .into_iter()
        .map(|r| (r.index, r.relevance_score))
        .collect(),
    )
  }

  /// Classify text into categories
  /// Useful for content moderation, spam detection, sentiment analysis
  pub async fn classify_text(
    &self,
    texts: Vec<String>,
    examples: Vec<(String, String)>,
  ) -> Result<Vec<(String, String, f32)>, AppError> {
    let request = ClassifyRequest {
      inputs: texts,
      examples: examples
        .into_iter()
        .map(|(text, label)| Example { text, label })
        .collect(),
    };

    let response = self
      .client
      .post(format!("{}/classify", self.base_url))
      .header("Authorization", format!("Bearer {}", self.api_key))
      .json(&request)
      .send()
      .await
      .map_err(|e| AppError::ExternalService(format!("Cohere API error: {}", e)))?;

    if !response.status().is_success() {
      let error_text = response.text().await.unwrap_or_default();
      return Err(AppError::ExternalService(format!(
        "Cohere API error: {}",
        error_text
      )));
    }

    let classify_response: ClassifyResponse = response
      .json()
      .await
      .map_err(|e| AppError::ExternalService(format!("Failed to parse response: {}", e)))?;

    Ok(
      classify_response
        .classifications
        .into_iter()
        .map(|c| (c.input, c.prediction, c.confidence))
        .collect(),
    )
  }
}

// Implement AIService trait with Cohere-specific features
#[async_trait]
impl AIService for CohereClient {
  async fn chat_completion(
    &self,
    _messages: Vec<fechatter_core::contracts::infrastructure::ChatMessage>,
  ) -> Result<String, fechatter_core::error::CoreError> {
    // Cohere doesn't have a chat completion API like OpenAI
    // Return a simple response for now
    Err(fechatter_core::error::CoreError::Internal(
      "Use OpenAI for chat completion".to_string(),
    ))
  }

  async fn generate_summary(
    &self,
    content: &str,
  ) -> Result<String, fechatter_core::error::CoreError> {
    // Cohere's summarization is best done via their generate endpoint
    // For now, we'll return a simple truncation
    // In production, you'd use Cohere's generate API with a summarization prompt
    Ok(content.chars().take(200).collect::<String>() + "...")
  }

  async fn analyze_sentiment(
    &self,
    content: &str,
  ) -> Result<fechatter_core::contracts::infrastructure::Sentiment, fechatter_core::error::CoreError>
  {
    // Use Cohere's classification for sentiment analysis
    let examples = vec![
      ("I love this!".to_string(), "positive".to_string()),
      ("This is terrible".to_string(), "negative".to_string()),
      ("It's okay".to_string(), "neutral".to_string()),
      ("Amazing work!".to_string(), "positive".to_string()),
    ];

    let results = self
      .classify_text(vec![content.to_string()], examples)
      .await
      .map_err(|e| fechatter_core::error::CoreError::Internal(e.to_string()))?;

    let (label, confidence) = results
      .first()
      .map(|(_, label, confidence)| (label.clone(), *confidence))
      .unwrap_or_else(|| ("neutral".to_string(), 0.5));

    Ok(fechatter_core::contracts::infrastructure::Sentiment {
      label,
      score: confidence,
    })
  }

  async fn suggest_replies(
    &self,
    _context: &str,
  ) -> Result<Vec<String>, fechatter_core::error::CoreError> {
    // Cohere is better suited for search/classification than generation
    // Return empty for now, use OpenAI for this feature
    Ok(vec![])
  }
}

// Additional methods specific to CohereClient (not part of AIService trait)
impl CohereClient {
  /// Check for content moderation using classification
  pub async fn moderate_content(&self, content: &str) -> Result<bool, AppError> {
    // Use Cohere's classification for content moderation
    let examples = vec![
      (
        "I hate you and want to hurt you".to_string(),
        "toxic".to_string(),
      ),
      ("This is spam, click here!".to_string(), "toxic".to_string()),
      ("Hello, how are you today?".to_string(), "safe".to_string()),
      ("Thanks for your help!".to_string(), "safe".to_string()),
    ];

    let results = self
      .classify_text(vec![content.to_string()], examples)
      .await?;

    // Content is safe if classified as "safe" with high confidence
    Ok(
      results
        .first()
        .map(|(_, label, confidence)| label == "safe" && *confidence > 0.7)
        .unwrap_or(true),
    )
  }
}
