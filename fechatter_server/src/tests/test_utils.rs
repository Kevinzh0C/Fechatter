use crate::{AppConfig, AppError, AppState};
use axum::Router;
use sqlx::{Pool, Postgres};
use sqlx_db_tester::TestPg;
use tokio::fs;

/// Test application wrapper
pub struct TestApp {
    pub state: AppState,
    pub app: Router,
}

// Helper to create app from app state
async fn create_app(state: AppState) -> Result<Router, AppError> {
    crate::get_router(state).await
}

#[cfg(any(test, feature = "test-util"))]
impl AppState {
    /// Create a test AppState without NATS
    pub async fn test_new() -> Result<(TestPg, Self), AppError> {
        let config = AppConfig::load().expect("Failed to load config");
        fs::create_dir_all(&config.server.base_dir)
            .await
            .map_err(|e| AppError::IOError(e))?;

        let post = config.server.db_url.rfind('/').expect("invalid db_url");
        let server_url = &config.server.db_url[..post];
        let tdb = TestPg::new(
            server_url.to_string(),
            std::path::Path::new("../migrations"),
        );

        let state = Self::try_new(config).await?;
        Ok((tdb, state))
    }

    /// Create a test AppState with NATS support
    pub async fn test_new_with_nats() -> Result<(TestPg, Self), AppError> {
        let config = AppConfig::load().expect("Failed to load config");
        fs::create_dir_all(&config.server.base_dir)
            .await
            .map_err(|e| AppError::IOError(e))?;

        let post = config.server.db_url.rfind('/').expect("invalid db_url");
        let server_url = &config.server.db_url[..post];
        let tdb = TestPg::new(
            server_url.to_string(),
            std::path::Path::new("../migrations"),
        );

        let state = Self::try_new(config).await?;
        Ok((tdb, state))
    }
}

/// Setup test database for integration tests
pub async fn setup_test_database() -> Pool<Postgres> {
    let config = AppConfig::load().expect("Failed to load config");

    // Use TestPg to create an isolated test database
    let post = config.server.db_url.rfind('/').expect("invalid db_url");
    let server_url = &config.server.db_url[..post];
    let tdb = TestPg::new(
        server_url.to_string(),
        std::path::Path::new("../migrations"),
    );

    // Get connection pool to the test database
    sqlx::PgPool::connect(&tdb.url())
        .await
        .expect("Failed to connect to test database")
}

pub async fn setup_test_app() -> TestApp {
    let config = AppConfig::load().expect("Failed to load config");
    let pool = setup_test_database().await;

    // Create app state with the test database
    let state = AppState::try_new(config)
        .await
        .expect("Failed to create app state");

    let app = create_app(state.clone())
        .await
        .expect("Failed to create app");

    TestApp { state, app }
}

pub async fn setup_integration_test_app() -> TestApp {
    // Initialize tracing for integration tests
    tracing_subscriber::fmt()
        .with_target(false)
        .compact()
        .try_init()
        .ok(); // Ignore error if already initialized

    let config = AppConfig::load().expect("Failed to load config");

    // Create app state with the test database
    let state = AppState::try_new(config)
        .await
        .expect("Failed to create app state");

    let app = create_app(state.clone())
        .await
        .expect("Failed to create app");

    TestApp { state, app }
}
