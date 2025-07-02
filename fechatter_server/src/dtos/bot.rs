use serde::{Deserialize, Serialize};

/// Request structure for translating a message
#[derive(Debug, Deserialize)]
pub struct TranslateRequest {
    /// ID of the message to translate
    pub message_id: i32,
    /// Target language code (e.g., "en", "zh", "ja")
    pub target_language: String,
}

/// Response structure for translation
#[derive(Debug, Serialize)]
pub struct TranslateResponse {
    /// Translated text
    pub translation: String,
    /// Detected or specified source language
    pub source_language: String,
    /// Target language
    pub target_language: String,
    /// Translation confidence score (0.0 - 1.0)
    pub confidence: f32,
    /// Number of translations used today
    pub quota_used: i32,
    /// Number of translations remaining today
    pub quota_remaining: i32,
    /// Daily translation limit
    pub quota_limit: i32,
}

/// Request structure for language detection
#[derive(Debug, Deserialize)]
pub struct DetectLanguageRequest {
    /// Text to detect language for
    pub text: String,
}

/// Response structure for language detection
#[derive(Debug, Serialize)]
pub struct DetectLanguageResponse {
    /// Detected language code
    pub language: String,
    /// Detection confidence score (0.0 - 1.0)
    pub confidence: f32,
}

/// Language information
#[derive(Debug, Serialize, Clone)]
pub struct Language {
    /// Language code (ISO 639-1)
    pub code: String,
    /// Human-readable language name
    pub name: String,
}

/// Response structure for supported languages list
#[derive(Debug, Serialize)]
pub struct SupportedLanguagesResponse {
    /// List of supported languages
    pub languages: Vec<Language>,
}
