use crate::{AppConfig, AppError, AppState};
use sqlx_db_tester::TestPg;
use tokio::fs;

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
