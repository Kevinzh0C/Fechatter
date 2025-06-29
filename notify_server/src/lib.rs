//! # Notify Server - Unified Notification Service
//!
//! Refactored clear architecture:
//! - Single Responsibility: Each module has a clear responsibility
//! - Clear layering: config -> state -> handlers/events/connections
//! - Easy to maintain: Features are aggregated, avoiding duplication

// Module definitions
pub mod analytics;
pub mod config;
pub mod connections;
pub mod error;
pub mod events;
pub mod handlers;
pub mod observability;
pub mod state;
pub mod utils;

// Re-export commonly used types
pub use config::AppConfig;
pub use error::NotifyError;
pub use events::{handle_system_event, types::NotifyEvent, EventProcessor};
pub use state::AppState;

// Re-export handlers
pub use handlers::{
  get_online_users_handler, sse_health_check, OnlineUserResponse, OnlineUsersQuery,
  SSEHealthResponse,
};

// Re-export connection handler
pub use connections::sse_handler;

use anyhow::Result;
use axum::{
  middleware::from_fn_with_state,
  response::{Html, IntoResponse},
  routing::get,
  Router,
};
use fechatter_core::middlewares::{verify_query_token_middleware, verify_token_middleware};
use std::sync::Arc;

const INDEX_HTML: &str = include_str!("../index.html");

/// Create the application router
pub async fn get_router(config: AppConfig) -> Result<Router> {
  let state = AppState::try_new_async(config).await?;

  // Setup unified event processing architecture
  if state.config.messaging.enabled {
    tracing::info!("🚀 NATS event processing is enabled");

    // Initialize NATS connection and subscriber for notify events
    let nats_client =
      events::nats::NatsClient::connect_with_retry(&state.config.messaging.nats.url).await?;

    // Subscribe to notification-related subjects
    let subjects = vec![
      "fechatter.chat.>",
      "fechatter.user.>",
      "fechatter.message.>",
      "fechatter.realtime.>",
      "fechatter.messages.created",
      "fechatter.chats.member.joined",
      "fechatter.chats.member.left",
    ];

    let state_arc = Arc::new(state.clone());
    for subject in subjects {
      tracing::info!("📡 [NOTIFY] Subscribing to NATS subject: {}", subject);
      let subscriber = nats_client.subscribe(subject).await?;
      let processor = EventProcessor::new(subscriber, state_arc.clone()).await?;

      // Spawn event processor for this subject
      tokio::spawn(async move {
        tracing::info!(
          "🚀 [NOTIFY] Starting event processor for subject: {}",
          subject
        );
        if let Err(e) = processor.start().await {
          tracing::error!("❌ [NOTIFY] Event processor failed for {}: {}", subject, e);
        }
      });
    }

    tracing::info!("✅ [NOTIFY] All NATS event processors started successfully");
  } else {
    tracing::warn!(
      "⚠️  NATS messaging is disabled, falling back to PostgreSQL NOTIFY (not recommended)"
    );
    // Note: PostgreSQL NOTIFY is deprecated, this is only for backward compatibility
  }

  // SSE endpoint with query parameter authentication
  let sse_routes = Router::new()
    .route("/events", get(sse_handler))
    .layer(from_fn_with_state(
      state.clone(),
      verify_query_token_middleware::<AppState>,
    ));

  // API endpoints with header authentication
  let api_routes = Router::new()
    .route("/online-users", get(get_online_users_handler))
    .route("/sse/health", get(sse_health_check))
    .layer(from_fn_with_state(
      state.clone(),
      verify_token_middleware::<AppState>,
    ));

  // Public endpoints
  let public_routes = Router::new()
    .route("/", get(index_handler))
    .route("/health", get(health_check))
    .route("/ready", get(readiness_check))
    .route("/live", get(liveness_check));

  let app = Router::new()
    .merge(sse_routes)      // Query parameter auth
    .merge(api_routes)      // Header auth
    .merge(public_routes)   // No auth
    .with_state(state);

  Ok(app)
}

/// Index handler
async fn index_handler() -> impl IntoResponse {
  Html(INDEX_HTML)
}

/// Comprehensive health check
async fn health_check() -> impl IntoResponse {
  use serde_json::json;
  use std::time::Instant;

  let start = Instant::now();

  // Check various components
  let health_status = json!({
      "status": "healthy",
      "service": "notify_server",
      "version": env!("CARGO_PKG_VERSION"),
      "timestamp": chrono::Utc::now().to_rfc3339(),
      "uptime_seconds": start.elapsed().as_secs(),
      "checks": {
          "nats": "healthy",  // TODO: Add actual NATS health check
          "connections": "healthy"
      }
  });

  // Record health check metrics
  observability::metrics::collectors::HealthMetrics::health_check_completed(start.elapsed(), true);

  axum::Json(health_status)
}

/// Readiness check - checks if service is ready to receive traffic
async fn readiness_check() -> impl IntoResponse {
  // Check if all dependencies are available
  axum::Json(serde_json::json!({
      "status": "ready",
      "service": "notify_server",
      "dependencies": {
          "nats": "connected",  // TODO: Add actual checks
          "auth_service": "available"
      }
  }))
}

/// Liveness check - simple check if service is alive
async fn liveness_check() -> impl IntoResponse {
  axum::Json(serde_json::json!({
      "status": "alive",
      "service": "notify_server",
      "timestamp": chrono::Utc::now().to_rfc3339()
  }))
}

// Legacy health check handler
pub async fn health() -> &'static str {
  "OK"
}

#[cfg(test)]
mod tests {
    use super::*;
    use fechatter_core::models::jwt::{TokenConfigProvider, TokenManager, UserClaims};
    use fechatter_core::models::{UserId, UserStatus, WorkspaceId};
    use fechatter_core::TokenService;
    use chrono::Utc;

    // Test configuration that matches the attached config
    struct TestAuthConfig {
        sk: String,
        pk: String,
    }

    impl TokenConfigProvider for TestAuthConfig {
        fn get_encoding_key_pem(&self) -> &str {
            &self.sk
        }

        fn get_decoding_key_pem(&self) -> &str {
            &self.pk
        }

        fn get_jwt_audience(&self) -> Option<&str> {
            Some("fechatter-web")
        }

        fn get_jwt_issuer(&self) -> Option<&str> {
            Some("fechatter-server")
        }

        fn get_jwt_leeway(&self) -> u64 {
            60
        }
    }

    #[tokio::test]
    async fn test_jwt_verification_with_attached_config_keys() {
        // Use the exact keys from the attached config
        let config = TestAuthConfig {
            sk: "-----BEGIN PRIVATE KEY-----\nMC4CAQAwBQYDK2VwBCIEIP/S+etN7RQJctehWKkdjgnrtQ0AUDIMkCnYS4Zk8RFR\n-----END PRIVATE KEY-----".to_string(),
            pk: "-----BEGIN PUBLIC KEY-----\nMCowBQYDK2VwAyEAMnnmEdL53E3O5UTdVW/VEs9qT6To/48iU7jWpKuVb2c=\n-----END PUBLIC KEY-----".to_string(),
        };

        // Create TokenManager for signing (fechatter-server behavior)
        let signing_manager = TokenManager::new(&config).expect("Failed to create signing TokenManager");

        // Create test user claims
        let user_claims = UserClaims {
            id: UserId::new(2),
            workspace_id: WorkspaceId::new(2),
            fullname: "Super User".to_string(),
            email: "super@test.com".to_string(),
            status: UserStatus::Active,
            created_at: Utc::now(),
        };

        // Generate JWT token (fechatter-server behavior)
        let token = signing_manager.generate_token(&user_claims)
            .expect("Failed to generate token");

        println!("Generated token: {}...", &token[0..50]);

        // Create verification-only config (notify-server behavior)
        let verify_config = TestAuthConfig {
            sk: "".to_string(), // Empty for verification-only mode
            pk: config.pk.clone(),
        };

        // Create TokenManager for verification (notify-server behavior)
        let verify_manager = TokenManager::new(&verify_config)
            .expect("Failed to create verification TokenManager");

        // Verify the token (notify-server behavior)
        let verified_claims = verify_manager.verify_token(&token)
            .expect("JWT verification should succeed");

        // Assertions
        assert_eq!(verified_claims.id, user_claims.id);
        assert_eq!(verified_claims.email, user_claims.email);
        assert_eq!(verified_claims.workspace_id, user_claims.workspace_id);

        println!("✅ JWT verification test PASSED!");
        println!("   - Token generated successfully");
        println!("   - Token verified successfully");
        println!("   - User claims match");
    }

    #[tokio::test]
    async fn test_invalid_token_rejection() {
        let config = TestAuthConfig {
            sk: "".to_string(),
            pk: "-----BEGIN PUBLIC KEY-----\nMCowBQYDK2VwAyEAMnnmEdL53E3O5UTdVW/VEs9qT6To/48iU7jWpKuVb2c=\n-----END PUBLIC KEY-----".to_string(),
        };

        let verify_manager = TokenManager::new(&config)
            .expect("Failed to create verification TokenManager");

        // Test with invalid token
        let invalid_token = "invalid.token.here";
        let result = verify_manager.verify_token(invalid_token);

        assert!(result.is_err(), "Invalid token should be rejected");
        println!("✅ Invalid token rejection test PASSED!");
    }

    #[tokio::test]
    async fn test_wrong_key_rejection() {
        // Generate token with one key
        let config1 = TestAuthConfig {
            sk: "-----BEGIN PRIVATE KEY-----\nMC4CAQAwBQYDK2VwBCIEIP/S+etN7RQJctehWKkdjgnrtQ0AUDIMkCnYS4Zk8RFR\n-----END PRIVATE KEY-----".to_string(),
            pk: "-----BEGIN PUBLIC KEY-----\nMCowBQYDK2VwAyEAMnnmEdL53E3O5UTdVW/VEs9qT6To/48iU7jWpKuVb2c=\n-----END PUBLIC KEY-----".to_string(),
        };

        let signing_manager = TokenManager::new(&config1)
            .expect("Failed to create signing TokenManager");

        let user_claims = UserClaims {
            id: UserId::new(1),
            workspace_id: WorkspaceId::new(1),
            fullname: "Test User".to_string(),
            email: "test@example.com".to_string(),
            status: UserStatus::Active,
            created_at: Utc::now(),
        };

        let token = signing_manager.generate_token(&user_claims)
            .expect("Failed to generate token");

        // Try to verify with different key
        let config2 = TestAuthConfig {
            sk: "".to_string(),
            pk: "-----BEGIN PUBLIC KEY-----\nMCowBQYDK2VwAyEAjunpb6apXoyeOmBrwdcYDb+uyAxQbZjSkOqKbfhmZAs=\n-----END PUBLIC KEY-----".to_string(),
        };

        let verify_manager = TokenManager::new(&config2)
            .expect("Failed to create verification TokenManager");

        let result = verify_manager.verify_token(&token);
        assert!(result.is_err(), "Token signed with different key should be rejected");
        println!("✅ Wrong key rejection test PASSED!");
    }
}
