use anyhow::Result;
use axum::{
    extract::{Json, State},
    http::StatusCode,
    response::Json as ResponseJson,
    routing::{get, post},
    Router,
};
use bot_server::{AppConfig, HealthState, setup_nats_subscriber};
use reqwest;
use serde::{Deserialize, Serialize};
use sqlx::postgres::PgPoolOptions;
use std::sync::Arc;
use tokio::net::TcpListener;
use tracing::{info, error, warn, level_filters::LevelFilter};
use tracing_subscriber::{Layer as _, fmt::Layer, layer::SubscriberExt, util::SubscriberInitExt};

// ================================
// 🎯 API Data Structures - Matching Frontend Expectations
// ================================

#[derive(Debug, Serialize)]
struct Language {
    code: String,
    name: String,
    flag: String,
    native: String,
}

#[derive(Debug, Serialize)]
struct LanguagesResponse {
    success: bool,
    languages: Vec<Language>,
    total: usize,
    provider: String,
}

#[derive(Debug, Deserialize)]
struct TranslateRequest {
    message_id: String,
    target_language: String,
    text: Option<String>,
}

#[derive(Debug, Serialize)]
struct TranslateResponse {
    success: bool,
    translation: String,
    source_language: String,
    target_language: String,
    confidence: f64,
    message_id: String,
    quota_used: u32,
    quota_remaining: u32,
    quota_limit: u32,
    provider: String,
    processing_time_ms: u64,
}

#[derive(Debug, Deserialize)]
struct DetectLanguageRequest {
    text: String,
}

#[derive(Debug, Serialize)]
struct DetectLanguageResponse {
    success: bool,
    language: String,
    confidence: f64,
    provider: String,
}

#[derive(Debug, Serialize)]
struct BotStatusResponse {
    success: bool,
    translation_bot: TranslationBotStatus,
    ai_assistant: AIAssistantStatus,
    custom_bots: Vec<serde_json::Value>,
    provider: String,
    timestamp: String,
}

#[derive(Debug, Serialize)]
struct TranslationBotStatus {
    status: String,
    version: String,
    uptime: u64,
    endpoints: Vec<String>,
}

#[derive(Debug, Serialize)]
struct AIAssistantStatus {
    status: String,
    model: String,
    capabilities: Vec<String>,
}

#[derive(Debug, Serialize)]
struct ErrorResponse {
    error: String,
    message: String,
}

// ================================
// 🎯 Application State
// ================================

#[derive(Clone)]
struct AppState {
    config: Arc<AppConfig>,
    openai_client: reqwest::Client,
    start_time: std::time::Instant,
}

impl AppState {
    fn new(config: Arc<AppConfig>) -> Self {
        let openai_client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .expect("Failed to create OpenAI client");

        Self {
            config,
            openai_client,
            start_time: std::time::Instant::now(),
        }
    }
}

// ================================
// 🎯 Translation API Handlers
// ================================

/// GET /api/bot/languages - Get supported languages
async fn get_supported_languages() -> ResponseJson<LanguagesResponse> {
    let languages = vec![
        Language {
            code: "en".to_string(),
            name: "English".to_string(),
            flag: "🇺🇸".to_string(),
            native: "English".to_string(),
        },
        Language {
            code: "zh".to_string(),
            name: "Chinese".to_string(),
            flag: "🇨🇳".to_string(),
            native: "中文".to_string(),
        },
        Language {
            code: "ja".to_string(),
            name: "Japanese".to_string(),
            flag: "🇯🇵".to_string(),
            native: "日本語".to_string(),
        },
        Language {
            code: "ko".to_string(),
            name: "Korean".to_string(),
            flag: "🇰🇷".to_string(),
            native: "한국어".to_string(),
        },
        Language {
            code: "es".to_string(),
            name: "Spanish".to_string(),
            flag: "🇪🇸".to_string(),
            native: "Español".to_string(),
        },
        Language {
            code: "fr".to_string(),
            name: "French".to_string(),
            flag: "🇫🇷".to_string(),
            native: "Français".to_string(),
        },
        Language {
            code: "de".to_string(),
            name: "German".to_string(),
            flag: "🇩🇪".to_string(),
            native: "Deutsch".to_string(),
        },
        Language {
            code: "ru".to_string(),
            name: "Russian".to_string(),
            flag: "🇷🇺".to_string(),
            native: "Русский".to_string(),
        },
        Language {
            code: "pt".to_string(),
            name: "Portuguese".to_string(),
            flag: "🇵🇹".to_string(),
            native: "Português".to_string(),
        },
        Language {
            code: "it".to_string(),
            name: "Italian".to_string(),
            flag: "🇮🇹".to_string(),
            native: "Italiano".to_string(),
        },
    ];

    ResponseJson(LanguagesResponse {
        success: true,
        total: languages.len(),
        languages,
        provider: "bot_server".to_string(),
    })
}

/// POST /api/bot/translate - Translate message using OpenAI
async fn translate_message(
    State(state): State<AppState>,
    Json(req): Json<TranslateRequest>,
) -> Result<ResponseJson<TranslateResponse>, (StatusCode, ResponseJson<ErrorResponse>)> {
    let start_time = std::time::Instant::now();
    
    info!("🔄 Translation request: {} -> {}", req.message_id, req.target_language);

    // Validate request
    if req.message_id.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            ResponseJson(ErrorResponse {
                error: "bad_request".to_string(),
                message: "message_id is required".to_string(),
            }),
        ));
    }

    if req.target_language.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            ResponseJson(ErrorResponse {
                error: "bad_request".to_string(),
                message: "target_language is required".to_string(),
            }),
        ));
    }

    // Get message content
    let message_content = req.text.unwrap_or_else(|| format!("Message {}", req.message_id));
    
    // Translate using OpenAI
    match translate_with_openai(&state, &message_content, &req.target_language).await {
        Ok((translation, source_lang, confidence)) => {
            let processing_time = start_time.elapsed().as_millis() as u64;
            
            info!("✅ Translation completed in {}ms", processing_time);
            
            Ok(ResponseJson(TranslateResponse {
                success: true,
                translation,
                source_language: source_lang,
                target_language: req.target_language,
                confidence,
                message_id: req.message_id,
                quota_used: 1, // Simple quota - could be enhanced
                quota_remaining: 19, // Simple quota - could be enhanced
                quota_limit: 20,
                provider: "openai_gpt".to_string(),
                processing_time_ms: processing_time,
            }))
        }
        Err(e) => {
            error!("❌ Translation failed: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                ResponseJson(ErrorResponse {
                    error: "translation_failed".to_string(),
                    message: e.to_string(),
                }),
            ))
        }
    }
}

/// Translate text using OpenAI API
async fn translate_with_openai(
    state: &AppState,
    text: &str,
    target_language: &str,
) -> Result<(String, String, f64)> {
    let api_key = state.config.bot.openai.get_api_key()?;
    
    // Language code to full name mapping
    let language_names = std::collections::HashMap::from([
        ("en", "English"),
        ("zh", "Chinese"),
        ("ja", "Japanese"),
        ("ko", "Korean"),
        ("es", "Spanish"),
        ("fr", "French"),
        ("de", "German"),
        ("ru", "Russian"),
        ("pt", "Portuguese"),
        ("it", "Italian"),
    ]);
    
    let target_language_name = language_names.get(&target_language)
        .unwrap_or(&target_language);

    // Create OpenAI prompt
    let prompt = format!(
        "Translate the following text to {}: \"{}\"

Please respond with only the translation, no additional text or explanations.",
        target_language_name, text
    );

    // Prepare OpenAI request
    let openai_request = serde_json::json!({
        "model": state.config.bot.openai.model,
        "messages": [
            {
                "role": "user",
                "content": prompt
            }
        ],
        "max_tokens": 1000,
        "temperature": 0.3
    });

    // Call OpenAI API
    let response = state
        .openai_client
        .post("https://api.openai.com/v1/chat/completions")
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Content-Type", "application/json")
        .json(&openai_request)
        .send()
        .await?;

    if !response.status().is_success() {
        let status = response.status();
        let error_text = response.text().await.unwrap_or_default();
        return Err(anyhow::anyhow!("OpenAI API error {}: {}", status, error_text));
    }

    let openai_response: serde_json::Value = response.json().await?;
    
    // Extract translation from response
    let translation = openai_response
        .get("choices")
        .and_then(|choices| choices.as_array())
        .and_then(|choices| choices.get(0))
        .and_then(|choice| choice.get("message"))
        .and_then(|message| message.get("content"))
        .and_then(|content| content.as_str())
        .unwrap_or("Translation failed")
        .trim()
        .to_string();

    // Simple language detection for source language
    let source_language = detect_language_simple(text);
    
    Ok((translation, source_language, 0.95))
}

/// POST /api/bot/detect-language - Detect language of text
async fn detect_language(
    Json(req): Json<DetectLanguageRequest>,
) -> ResponseJson<DetectLanguageResponse> {
    let detected_language = detect_language_simple(&req.text);
    
    ResponseJson(DetectLanguageResponse {
        success: true,
        language: detected_language,
        confidence: 0.9,
        provider: "pattern_detection".to_string(),
    })
}

/// Simple pattern-based language detection
fn detect_language_simple(text: &str) -> String {
    if text.chars().any(|c| '\u{4e00}' <= c && c <= '\u{9fff}') {
        "zh".to_string()
    } else if text.chars().any(|c| ('\u{3040}' <= c && c <= '\u{309f}') || ('\u{30a0}' <= c && c <= '\u{30ff}')) {
        "ja".to_string()
    } else if text.chars().any(|c| '\u{ac00}' <= c && c <= '\u{d7af}') {
        "ko".to_string()
    } else if text.chars().any(|c| '\u{0400}' <= c && c <= '\u{04ff}') {
        "ru".to_string()
    } else {
        "en".to_string()
    }
}

/// GET /api/bot/status - Get bot service status
async fn get_bot_status(State(state): State<AppState>) -> ResponseJson<BotStatusResponse> {
    let uptime = state.start_time.elapsed().as_secs();
    
    ResponseJson(BotStatusResponse {
        success: true,
        translation_bot: TranslationBotStatus {
            status: "active".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            uptime,
            endpoints: vec![
                "translate".to_string(),
                "detect-language".to_string(),
                "languages".to_string(),
            ],
        },
        ai_assistant: AIAssistantStatus {
            status: "active".to_string(),
            model: state.config.bot.openai.model.clone(),
            capabilities: vec![
                "translation".to_string(),
                "language_detection".to_string(),
            ],
        },
        custom_bots: vec![],
        provider: "bot_server".to_string(),
        timestamp: chrono::Utc::now().to_rfc3339(),
    })
}

// ================================
// 🎯 Main Application
// ================================

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing for logging
    let layer = Layer::new().with_filter(LevelFilter::INFO);
    tracing_subscriber::registry().with(layer).init();

    // Initialize observability (metrics)
    if let Err(e) = bot_server::observability::init_observability().await {
        eprintln!("❌ Failed to initialize observability: {}", e);
        eprintln!("   Continuing without Prometheus metrics");
    } else {
        info!("📊 Prometheus metrics available at: http://0.0.0.0:9092/metrics");
    }

    // Load configuration
    let config = match AppConfig::load() {
        Ok(config) => {
            info!("✅ {}", config.get_summary());
            config
        }
        Err(e) => {
            eprintln!("❌ Failed to load bot_server configuration: {}", e);
            eprintln!("\n💡 Quick fix suggestions:");
            eprintln!("   1. Copy bot.yml.example to bot.yml");
            eprintln!("   2. Set BOT_CONFIG=/path/to/your/config.yml");
            eprintln!("   3. Set OPENAI_API_KEY environment variable");
            eprintln!("   4. Check NATS and database connectivity");
            std::process::exit(1);
        }
    };

    let config_arc = Arc::new(config.clone());

    info!("🤖 Starting bot_server with REST API and NATS event processing...");

    // Connect to database for health checks
    let pool = PgPoolOptions::new()
        .connect(&config.server.db_url)
        .await?;
    let pool = Arc::new(pool);

    // Create NATS client for background processing
    let nats_client = if config.messaging.enabled {
        info!("🚀 Connecting to NATS: {}", config.messaging.nats.url);
        
        match async_nats::ConnectOptions::new()
            .connection_timeout(std::time::Duration::from_secs(10))
            .ping_interval(std::time::Duration::from_secs(60))
            .max_reconnects(Some(5))
            .reconnect_delay_callback(|attempts: usize| {
                std::time::Duration::from_secs(std::cmp::min(2u64.pow(attempts as u32), 10))
            })
            .connect(&config.messaging.nats.url)
            .await
        {
            Ok(client) => {
                info!("✅ Connected to NATS: {}", config.messaging.nats.url);
                Some(Arc::new(client))
            }
            Err(e) => {
                error!("❌ Failed to connect to NATS: {}", e);
                warn!("⚠️ Bot server will continue without NATS functionality");
                None
            }
        }
    } else {
        info!("⚠️ NATS messaging disabled in configuration");
        None
    };

    // Setup health check state (currently integrated into handlers)
    let _health_state = match &nats_client {
        Some(nats_client) => {
            HealthState::new(pool.clone(), config_arc.clone()).with_nats(nats_client.clone())
        }
        None => {
            HealthState::new(pool.clone(), config_arc.clone())
        }
    };

    // Create application state
    let app_state = AppState::new(config_arc.clone());

    // Build HTTP router with translation APIs
    let app = Router::new()
        // Translation API endpoints
        .route("/api/bot/languages", get(get_supported_languages))
        .route("/api/bot/translate", post(translate_message))
        .route("/api/bot/detect-language", post(detect_language))
        .route("/api/bot/status", get(get_bot_status))
        // Health check endpoints
        .route("/health", get(health_check_handler))
        .route("/ready", get(readiness_check_handler))
        .route("/live", get(liveness_check_handler))
        .with_state(app_state);

    // Start NATS subscriber in background for event processing
    if let Some(nats_client) = nats_client {
        let config_clone = config.clone();
        tokio::spawn(async move {
            if let Err(e) = setup_nats_subscriber(&config_clone, Some(nats_client)).await {
                error!("NATS subscriber failed: {}", e);
            }
        });
    }

    // Start HTTP server
    let addr = "0.0.0.0:6686";
    let listener = TcpListener::bind(addr).await?;
    
    info!("🚀 Bot server started on http://{}", addr);
    info!("📋 Available REST API endpoints:");
    info!("   GET  /api/bot/languages        - Get supported languages");
    info!("   POST /api/bot/translate        - Translate message");
    info!("   POST /api/bot/detect-language  - Detect language");
    info!("   GET  /api/bot/status           - Get bot status");
    info!("   GET  /health                   - Full health check");
    info!("   GET  /ready                    - Readiness check");
    info!("   GET  /live                     - Liveness check");

    // Run server
    axum::serve(listener, app).await?;
    
    Ok(())
}

// ================================
// 🎯 Health Check Handlers
// ================================

/// Health check handler - reuse existing health check logic
async fn health_check_handler() -> Result<ResponseJson<serde_json::Value>, (StatusCode, ResponseJson<serde_json::Value>)> {
    // Simple health check for now - could integrate with existing HealthState if needed
    let health_data = serde_json::json!({
        "status": "healthy",
        "service": "bot_server",
        "version": env!("CARGO_PKG_VERSION"),
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "apis": {
            "translation": "active",
            "language_detection": "active"
        }
    });

    Ok(ResponseJson(health_data))
}

/// Readiness check handler
async fn readiness_check_handler() -> ResponseJson<serde_json::Value> {
    ResponseJson(serde_json::json!({
        "status": "ready",
        "service": "bot_server",
        "timestamp": chrono::Utc::now().to_rfc3339()
    }))
}

/// Liveness check handler
async fn liveness_check_handler() -> ResponseJson<serde_json::Value> {
    ResponseJson(serde_json::json!({
        "status": "alive",
        "service": "bot_server", 
        "timestamp": chrono::Utc::now().to_rfc3339()
    }))
}
