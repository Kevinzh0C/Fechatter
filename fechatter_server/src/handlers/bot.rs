use crate::{
    dtos::bot::{
        DetectLanguageRequest, DetectLanguageResponse, Language, SupportedLanguagesResponse,
        TranslateRequest, TranslateResponse,
    },
    error::AppError,
    AppState,
};
use axum::{extract::Extension, Json};
use chrono;
use fechatter_core::models::AuthUser;
use reqwest;
use serde_json::json;
use sqlx::Row;
use tracing::{debug, error, info};

/// Daily quota limit per user
const DAILY_QUOTA_LIMIT: i32 = 20;

/// External translation service configuration
const TRANSLATION_API_BASE: &str = "http://45.77.178.85:8000";

/// Translate a message using external translation service
pub async fn translate_message_handler(
    Extension(state): Extension<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Json(payload): Json<TranslateRequest>,
) -> Result<Json<TranslateResponse>, AppError> {
    info!(
        "🤖 [BOT] Translation request from user {} for message {} to {}",
        auth_user.id, payload.message_id, payload.target_language
    );

    // Check daily quota - convert UserId to i32
    let user_id = i64::from(auth_user.id) as i32;
    let quota_used = get_user_daily_quota(&state, user_id).await?;
    if quota_used >= DAILY_QUOTA_LIMIT {
        return Err(AppError::BadRequest(format!(
            "Daily translation limit exceeded. You have used {}/{} translations today.",
            quota_used, DAILY_QUOTA_LIMIT
        )));
    }

    // Get message content from database
    let message_content = get_message_content(&state, payload.message_id, user_id).await?;

    if message_content.trim().is_empty() {
        return Err(AppError::BadRequest("Message content is empty".to_string()));
    }

    // Call external translation API
    let translation_result =
        call_external_translation_api(&message_content, &payload.target_language).await?;

    // Increment user quota
    increment_user_quota(&state, user_id).await?;

    // Get updated quota info
    let remaining_quota = DAILY_QUOTA_LIMIT - (quota_used + 1);

    info!(
        "🤖 [BOT] Translation successful for user {}. Remaining quota: {}",
        auth_user.id, remaining_quota
    );

    Ok(Json(TranslateResponse {
        translation: translation_result.translation,
        source_language: translation_result.source_language,
        target_language: payload.target_language,
        confidence: translation_result.confidence.unwrap_or(0.9),
        quota_used: quota_used + 1,
        quota_remaining: remaining_quota,
        quota_limit: DAILY_QUOTA_LIMIT,
    }))
}

/// Get supported languages
pub async fn get_supported_languages_handler(
    Extension(_state): Extension<AppState>,
    Extension(_auth_user): Extension<AuthUser>,
) -> Result<Json<SupportedLanguagesResponse>, AppError> {
    debug!("🤖 [BOT] Fetching supported languages");

    // Return predefined supported languages
    let languages = vec![
        Language {
            code: "en".to_string(),
            name: "English".to_string(),
        },
        Language {
            code: "zh".to_string(),
            name: "Chinese (Simplified)".to_string(),
        },
        Language {
            code: "zh-TW".to_string(),
            name: "Chinese (Traditional)".to_string(),
        },
        Language {
            code: "ja".to_string(),
            name: "Japanese".to_string(),
        },
        Language {
            code: "ko".to_string(),
            name: "Korean".to_string(),
        },
        Language {
            code: "es".to_string(),
            name: "Spanish".to_string(),
        },
        Language {
            code: "fr".to_string(),
            name: "French".to_string(),
        },
        Language {
            code: "de".to_string(),
            name: "German".to_string(),
        },
        Language {
            code: "ru".to_string(),
            name: "Russian".to_string(),
        },
        Language {
            code: "ar".to_string(),
            name: "Arabic".to_string(),
        },
        Language {
            code: "hi".to_string(),
            name: "Hindi".to_string(),
        },
        Language {
            code: "pt".to_string(),
            name: "Portuguese".to_string(),
        },
        Language {
            code: "it".to_string(),
            name: "Italian".to_string(),
        },
        Language {
            code: "nl".to_string(),
            name: "Dutch".to_string(),
        },
        Language {
            code: "sv".to_string(),
            name: "Swedish".to_string(),
        },
        Language {
            code: "no".to_string(),
            name: "Norwegian".to_string(),
        },
        Language {
            code: "da".to_string(),
            name: "Danish".to_string(),
        },
        Language {
            code: "fi".to_string(),
            name: "Finnish".to_string(),
        },
        Language {
            code: "pl".to_string(),
            name: "Polish".to_string(),
        },
        Language {
            code: "tr".to_string(),
            name: "Turkish".to_string(),
        },
    ];

    Ok(Json(SupportedLanguagesResponse { languages }))
}

/// Detect language of given text
pub async fn detect_language_handler(
    Extension(_state): Extension<AppState>,
    Extension(_auth_user): Extension<AuthUser>,
    Json(payload): Json<DetectLanguageRequest>,
) -> Result<Json<DetectLanguageResponse>, AppError> {
    debug!(
        "🤖 [BOT] Language detection request for text: {:.100}...",
        payload.text
    );

    // Call external language detection API
    let detected_language = call_external_language_detection(&payload.text).await?;

    Ok(Json(DetectLanguageResponse {
        language: detected_language.language,
        confidence: detected_language.confidence,
    }))
}

// ============================================================================
// Private Helper Functions
// ============================================================================

/// Internal translation result structure
#[derive(Debug)]
struct TranslationResult {
    translation: String,
    source_language: String,
    confidence: Option<f32>,
}

/// Internal language detection result
#[derive(Debug)]
struct LanguageDetectionResult {
    language: String,
    confidence: f32,
}

/// Get user's daily quota usage from database
async fn get_user_daily_quota(state: &AppState, user_id: i32) -> Result<i32, AppError> {
    let pool = state.pool();

    let today = chrono::Utc::now().date_naive();

    let row = sqlx::query(
        r#"
        SELECT COUNT(*)::int as quota_used
        FROM bot_translation_logs 
        WHERE user_id = $1 AND DATE(created_at) = $2
        "#,
    )
    .bind(user_id as i64)
    .bind(today)
    .fetch_one(pool.as_ref())
    .await
    .map_err(|e| {
        error!("Failed to get user quota: {}", e);
        AppError::Internal("Failed to check quota".to_string())
    })?;

    let quota_used: i32 = row.try_get("quota_used").unwrap_or(0);
    Ok(quota_used)
}

/// Increment user's daily quota usage
async fn increment_user_quota(state: &AppState, user_id: i32) -> Result<(), AppError> {
    let pool = state.pool();

    sqlx::query(
        r#"
        INSERT INTO bot_translation_logs (user_id, created_at)
        VALUES ($1, NOW())
        "#,
    )
    .bind(user_id as i64)
    .execute(pool.as_ref())
    .await
    .map_err(|e| {
        error!("Failed to increment user quota: {}", e);
        AppError::Internal("Failed to record translation usage".to_string())
    })?;

    Ok(())
}

/// Get message content from database with access control
async fn get_message_content(
    state: &AppState,
    message_id: i32,
    user_id: i32,
) -> Result<String, AppError> {
    let pool = state.pool();

    // Check if user has access to this message (via chat membership)
    let row = sqlx::query(
        r#"
        SELECT m.content 
        FROM messages m
        JOIN chat_members cm ON m.chat_id = cm.chat_id
        WHERE m.id = $1 AND cm.user_id = $2
        "#,
    )
    .bind(message_id as i64)
    .bind(user_id as i64)
    .fetch_optional(pool.as_ref())
    .await
    .map_err(|e| {
        error!("Failed to get message content: {}", e);
        AppError::Internal("Failed to access message".to_string())
    })?;

    match row {
        Some(record) => {
            let content: String = record.try_get("content").map_err(|e| {
                error!("Failed to extract content from query result: {}", e);
                AppError::Internal("Failed to extract message content".to_string())
            })?;
            Ok(content)
        }
        None => Err(AppError::NotFound(vec![
            "Message not found or access denied".to_string(),
        ])),
    }
}

/// Call external translation API
async fn call_external_translation_api(
    text: &str,
    target_language: &str,
) -> Result<TranslationResult, AppError> {
    let client = reqwest::Client::new();

    let payload = json!({
        "text": text,
        "target_language": target_language,
        "source_language": "auto"
    });

    debug!(
        "🤖 [BOT] Calling external translation API: {}/translate",
        TRANSLATION_API_BASE
    );

    let response = client
        .post(&format!("{}/translate", TRANSLATION_API_BASE))
        .json(&payload)
        .timeout(std::time::Duration::from_secs(30))
        .send()
        .await
        .map_err(|e| {
            error!("Translation API request failed: {}", e);
            AppError::Internal("Translation service unavailable".to_string())
        })?;

    if !response.status().is_success() {
        error!("Translation API returned error: {}", response.status());
        return Err(AppError::Internal("Translation failed".to_string()));
    }

    let result: serde_json::Value = response.json().await.map_err(|e| {
        error!("Failed to parse translation response: {}", e);
        AppError::Internal("Invalid translation response".to_string())
    })?;

    debug!("🤖 [BOT] Translation API response: {:?}", result);

    Ok(TranslationResult {
        translation: result["translation"]
            .as_str()
            .unwrap_or("Translation failed")
            .to_string(),
        source_language: result["source_language"]
            .as_str()
            .unwrap_or("unknown")
            .to_string(),
        confidence: result["confidence"].as_f64().map(|v| v as f32),
    })
}

/// Call external language detection API
async fn call_external_language_detection(text: &str) -> Result<LanguageDetectionResult, AppError> {
    let client = reqwest::Client::new();

    let payload = json!({
        "text": text
    });

    debug!("🤖 [BOT] Calling external language detection API");

    let response = client
        .post(&format!("{}/detect", TRANSLATION_API_BASE))
        .json(&payload)
        .timeout(std::time::Duration::from_secs(15))
        .send()
        .await
        .map_err(|e| {
            error!("Language detection API request failed: {}", e);
            AppError::Internal("Language detection service unavailable".to_string())
        })?;

    if !response.status().is_success() {
        error!(
            "Language detection API returned error: {}",
            response.status()
        );
        return Err(AppError::Internal("Language detection failed".to_string()));
    }

    let result: serde_json::Value = response.json().await.map_err(|e| {
        error!("Failed to parse language detection response: {}", e);
        AppError::Internal("Invalid language detection response".to_string())
    })?;

    debug!("🤖 [BOT] Language detection API response: {:?}", result);

    Ok(LanguageDetectionResult {
        language: result["language"].as_str().unwrap_or("unknown").to_string(),
        confidence: result["confidence"].as_f64().unwrap_or(0.5) as f32,
    })
}
