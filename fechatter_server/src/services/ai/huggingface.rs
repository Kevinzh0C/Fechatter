use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::error::AppError;
// Import AIService from fechatter_core where it's actually defined
use fechatter_core::contracts::infrastructure::AIService;

/// Hugging Face client for specialized AI models
///
/// Hugging Face is used for:
/// - Sentiment analysis (understanding message tone)
/// - Language detection (auto-detecting chat language)
/// - Named Entity Recognition (extracting important info)
/// - Text classification (categorizing messages)
/// - Translation (cross-language communication)
pub struct HuggingFaceClient {
    client: Client,
    api_key: String,
    base_url: String,
}

#[derive(Debug, Serialize)]
struct InferenceRequest {
    inputs: String,
}

#[derive(Debug, Serialize)]
struct BatchInferenceRequest {
    inputs: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct SentimentResponse {
    label: String,
    score: f32,
}

#[derive(Debug, Deserialize)]
struct LanguageDetectionResponse {
    label: String,
    score: f32,
}

#[derive(Debug, Deserialize)]
struct NERResponse {
    entity_group: String,
    score: f32,
    word: String,
    start: usize,
    end: usize,
}

#[derive(Debug, Deserialize)]
struct TranslationResponse {
    translation_text: String,
}

impl HuggingFaceClient {
    pub fn new(api_key: String) -> Self {
        Self {
            client: Client::new(),
            api_key,
            base_url: "https://api-inference.huggingface.co/models".to_string(),
        }
    }

    /// Analyze sentiment of messages
    /// Returns: positive, negative, neutral with confidence scores
    pub async fn analyze_sentiment(&self, text: &str) -> Result<(String, f32), AppError> {
        let model = "distilbert-base-uncased-finetuned-sst-2-english";
        let request = InferenceRequest {
            inputs: text.to_string(),
        };

        let response = self
            .client
            .post(format!("{}/{}", self.base_url, model))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&request)
            .send()
            .await
            .map_err(|e| AppError::ExternalServiceError(format!("HuggingFace API error: {}", e)))?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(AppError::ExternalServiceError(format!(
                "HuggingFace API error: {}",
                error_text
            )));
        }

        let sentiments: Vec<Vec<SentimentResponse>> = response.json().await.map_err(|e| {
            AppError::ExternalServiceError(format!("Failed to parse response: {}", e))
        })?;

        let best_match = sentiments
            .first()
            .and_then(|s| {
                s.iter()
                    .max_by(|a, b| a.score.partial_cmp(&b.score).unwrap())
            })
            .ok_or_else(|| AppError::ExternalServiceError("No sentiment detected".to_string()))?;

        Ok((best_match.label.clone(), best_match.score))
    }

    /// Detect language of text
    /// Useful for multi-language chat support
    pub async fn detect_language(&self, text: &str) -> Result<(String, f32), AppError> {
        let model = "papluca/xlm-roberta-base-language-detection";
        let request = InferenceRequest {
            inputs: text.to_string(),
        };

        let response = self
            .client
            .post(format!("{}/{}", self.base_url, model))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&request)
            .send()
            .await
            .map_err(|e| AppError::ExternalServiceError(format!("HuggingFace API error: {}", e)))?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(AppError::ExternalServiceError(format!(
                "HuggingFace API error: {}",
                error_text
            )));
        }

        let languages: Vec<Vec<LanguageDetectionResponse>> =
            response.json().await.map_err(|e| {
                AppError::ExternalServiceError(format!("Failed to parse response: {}", e))
            })?;

        let best_match = languages
            .first()
            .and_then(|l| {
                l.iter()
                    .max_by(|a, b| a.score.partial_cmp(&b.score).unwrap())
            })
            .ok_or_else(|| AppError::ExternalServiceError("No language detected".to_string()))?;

        Ok((best_match.label.clone(), best_match.score))
    }

    /// Extract named entities from text
    /// Finds: people, organizations, locations, dates, etc.
    pub async fn extract_entities(
        &self,
        text: &str,
    ) -> Result<Vec<(String, String, f32)>, AppError> {
        let model = "dslim/bert-base-NER";
        let request = InferenceRequest {
            inputs: text.to_string(),
        };

        let response = self
            .client
            .post(format!("{}/{}", self.base_url, model))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&request)
            .send()
            .await
            .map_err(|e| AppError::ExternalServiceError(format!("HuggingFace API error: {}", e)))?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(AppError::ExternalServiceError(format!(
                "HuggingFace API error: {}",
                error_text
            )));
        }

        let entities: Vec<NERResponse> = response.json().await.map_err(|e| {
            AppError::ExternalServiceError(format!("Failed to parse response: {}", e))
        })?;

        Ok(entities
            .into_iter()
            .map(|e| (e.word, e.entity_group, e.score))
            .collect())
    }

    /// Translate text between languages
    pub async fn translate(
        &self,
        text: &str,
        source_lang: &str,
        target_lang: &str,
    ) -> Result<String, AppError> {
        let model = format!("Helsinki-NLP/opus-mt-{}-{}", source_lang, target_lang);
        let request = InferenceRequest {
            inputs: text.to_string(),
        };

        let response = self
            .client
            .post(format!("{}/{}", self.base_url, model))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&request)
            .send()
            .await
            .map_err(|e| AppError::ExternalServiceError(format!("HuggingFace API error: {}", e)))?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(AppError::ExternalServiceError(format!(
                "HuggingFace API error: {}",
                error_text
            )));
        }

        let translations: Vec<TranslationResponse> = response.json().await.map_err(|e| {
            AppError::ExternalServiceError(format!("Failed to parse response: {}", e))
        })?;

        translations
            .first()
            .map(|t| t.translation_text.clone())
            .ok_or_else(|| AppError::ExternalServiceError("No translation available".to_string()))
    }

    /// Check for toxic content using specialized model
    pub async fn check_toxicity(&self, text: &str) -> Result<f32, AppError> {
        let model = "unitary/toxic-bert";
        let request = InferenceRequest {
            inputs: text.to_string(),
        };

        let response = self
            .client
            .post(format!("{}/{}", self.base_url, model))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&request)
            .send()
            .await
            .map_err(|e| AppError::ExternalServiceError(format!("HuggingFace API error: {}", e)))?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(AppError::ExternalServiceError(format!(
                "HuggingFace API error: {}",
                error_text
            )));
        }

        let results: Vec<Vec<SentimentResponse>> = response.json().await.map_err(|e| {
            AppError::ExternalServiceError(format!("Failed to parse response: {}", e))
        })?;

        // Find the "TOXIC" label score
        Ok(results
            .first()
            .and_then(|r| r.iter().find(|s| s.label == "TOXIC"))
            .map(|s| s.score)
            .unwrap_or(0.0))
    }
}

// Implement AIService trait with HuggingFace-specific features
#[async_trait]
impl AIService for HuggingFaceClient {
    async fn chat_completion(
        &self,
        _messages: Vec<fechatter_core::contracts::infrastructure::ChatMessage>,
    ) -> Result<String, fechatter_core::error::CoreError> {
        // HuggingFace has generation models but they're slower
        // For chat completion, use OpenAI instead
        Err(fechatter_core::error::CoreError::Internal(
            "Use OpenAI for chat completion".to_string(),
        ))
    }

    async fn generate_summary(
        &self,
        _content: &str,
    ) -> Result<String, fechatter_core::error::CoreError> {
        // HuggingFace has summarization models but they're slower
        // For chat summarization, use OpenAI instead
        Err(fechatter_core::error::CoreError::Internal(
            "Use OpenAI for summarization".to_string(),
        ))
    }

    async fn analyze_sentiment(
        &self,
        content: &str,
    ) -> Result<
        fechatter_core::contracts::infrastructure::Sentiment,
        fechatter_core::error::CoreError,
    > {
        // Use HuggingFace's sentiment analysis
        let (label, score) = self
            .analyze_sentiment_internal(content)
            .await
            .map_err(|e| fechatter_core::error::CoreError::Internal(e.to_string()))?;

        Ok(fechatter_core::contracts::infrastructure::Sentiment { label, score })
    }

    async fn suggest_replies(
        &self,
        _context: &str,
    ) -> Result<Vec<String>, fechatter_core::error::CoreError> {
        // HuggingFace generation models are not ideal for chat replies
        // Use OpenAI for this feature
        Err(fechatter_core::error::CoreError::Internal(
            "Use OpenAI for reply suggestions".to_string(),
        ))
    }
}

// Additional methods specific to HuggingFaceClient (not part of AIService trait)
impl HuggingFaceClient {
    /// Check for content moderation using toxicity detection
    pub async fn moderate_content(&self, content: &str) -> Result<bool, AppError> {
        // Use HuggingFace's toxicity detection
        let toxicity_score = self.check_toxicity(content).await?;

        // Content is safe if toxicity score is below threshold
        Ok(toxicity_score < 0.7)
    }

    /// Internal sentiment analysis method (renamed to avoid conflict)
    async fn analyze_sentiment_internal(&self, text: &str) -> Result<(String, f32), AppError> {
        let model = "distilbert-base-uncased-finetuned-sst-2-english";
        let request = InferenceRequest {
            inputs: text.to_string(),
        };

        let response = self
            .client
            .post(format!("{}/{}", self.base_url, model))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&request)
            .send()
            .await
            .map_err(|e| AppError::ExternalServiceError(format!("HuggingFace API error: {}", e)))?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(AppError::ExternalServiceError(format!(
                "HuggingFace API error: {}",
                error_text
            )));
        }

        let sentiments: Vec<Vec<SentimentResponse>> = response.json().await.map_err(|e| {
            AppError::ExternalServiceError(format!("Failed to parse response: {}", e))
        })?;

        let best_match = sentiments
            .first()
            .and_then(|s| {
                s.iter()
                    .max_by(|a, b| a.score.partial_cmp(&b.score).unwrap())
            })
            .ok_or_else(|| AppError::ExternalServiceError("No sentiment detected".to_string()))?;

        Ok((best_match.label.clone(), best_match.score))
    }
}
