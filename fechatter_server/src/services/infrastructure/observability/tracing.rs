use crate::AppError;
use tracing::info;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::{
    filter::EnvFilter, fmt, layer::SubscriberExt, registry::Registry, util::SubscriberInitExt,
};

pub struct TracingGuard {
    _file_guard: Option<WorkerGuard>,
}

/// Initialize tracing without OpenTelemetry for now (to avoid compilation issues)
pub async fn init_tracing(
    config: &crate::config::ObservabilityConfig,
) -> Result<TracingGuard, AppError> {
    // Set up file logging if enabled
    let file_guard = if config.log_to_file {
        let file_appender =
            tracing_appender::rolling::hourly(&config.log_file_path, "fechatter.log");
        let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);

        // Use simple global filter - guaranteed to work
        let env_filter = if config.log_level == "debug" {
            EnvFilter::new("debug")
        } else {
            EnvFilter::new(&config.log_level)
        };

        // Initialize tracing subscriber with multiple layers
        let registry = Registry::default()
            .with(env_filter)
            .with(
                fmt::layer()
                    .with_target(true)
                    .with_thread_ids(true)
                    .with_level(true)
                    .with_file(true)
                    .with_line_number(true),
            )
            .with(
                fmt::layer()
                    .with_writer(non_blocking)
                    .with_target(true)
                    .with_ansi(false), // Plain text format for file output
            );

        registry
            .try_init()
            .map_err(|e| AppError::Internal(format!("Failed to init tracing subscriber: {}", e)))?;

        Some(guard)
    } else {
        // Console logging with simple global filter - guaranteed to work
        let env_filter = if config.log_level == "debug" {
            EnvFilter::new("debug")
        } else {
            EnvFilter::new(&config.log_level)
        };

        let registry = Registry::default().with(env_filter).with(
            fmt::layer()
                .with_target(true)
                .with_thread_ids(true)
                .with_level(true)
                .with_file(true)
                .with_line_number(true),
        );

        registry
            .try_init()
            .map_err(|e| AppError::Internal(format!("Failed to init tracing subscriber: {}", e)))?;

        None
    };

    info!(
        service_name = %config.service_name,
        service_version = %config.service_version,
        environment = %config.environment,
        log_level = %config.log_level,
        "Tracing initialized - Debug logs should now be visible!"
    );

    Ok(TracingGuard {
        _file_guard: file_guard,
    })
}

/// Custom tracing utilities
pub mod utils {
    use tracing::{error, Span};
    use uuid::Uuid;

    /// Add correlation ID to current span
    pub fn add_correlation_id() -> String {
        let correlation_id = Uuid::new_v4().to_string();
        Span::current().record("correlation_id", &correlation_id.as_str());
        correlation_id
    }

    /// Add user context to current span
    pub fn add_user_context(user_id: i64, workspace_id: i64) {
        let span = Span::current();
        span.record("user_id", user_id);
        span.record("workspace_id", workspace_id);
    }

    /// Add request context to current span
    pub fn add_request_context(method: &str, path: &str, user_agent: Option<&str>) {
        let span = Span::current();
        span.record("http.method", method);
        span.record("http.route", path);
        if let Some(ua) = user_agent {
            span.record("http.user_agent", ua);
        }
    }

    /// Add error context to current span
    pub fn add_error_context(error: &dyn std::error::Error) {
        let span = Span::current();
        span.record("error", true);
        span.record("error.message", &error.to_string().as_str());
        error!("Error occurred: {}", error);
    }

    /// Add chat context to current span
    pub fn add_chat_context(chat_id: i64, chat_type: &str) {
        let span = Span::current();
        span.record("chat_id", chat_id);
        span.record("chat_type", chat_type);
    }

    /// Add message context to current span
    pub fn add_message_context(message_id: i64, sender_id: i64, has_files: bool) {
        let span = Span::current();
        span.record("message_id", message_id);
        span.record("sender_id", sender_id);
        span.record("has_files", has_files);
    }
}
