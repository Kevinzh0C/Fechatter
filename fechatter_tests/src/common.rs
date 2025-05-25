//! Common test utilities and setup

use anyhow::Result;
use fechatter_core::*;
use fechatter_server::AppState;
use md5;
use sqlx_db_tester::TestPg;
use std::sync::Once;
use tokio_stream::StreamExt;
use tracing::info;

static INIT: Once = Once::new();

/// Initialize test logging
pub fn init_test_logging() {
  INIT.call_once(|| {
    tracing_subscriber::fmt()
      .with_env_filter("debug")
      .with_test_writer()
      .try_init()
      .ok();
  });
}

/// Test environment configuration
#[derive(Debug, Clone)]
pub struct TestConfig {
  pub db_url: String,
  pub nats_url: String,
  pub server_port: u16,
  pub notify_port: u16,
}

impl Default for TestConfig {
  fn default() -> Self {
    Self {
      db_url: std::env::var("TEST_DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/postgres".to_string()),
      nats_url: std::env::var("TEST_NATS_URL")
        .unwrap_or_else(|_| "nats://localhost:4222".to_string()),
      server_port: 0, // Random port
      notify_port: 0, // Random port
    }
  }
}

/// Test user data
#[derive(Debug, Clone)]
pub struct TestUser {
  pub id: i64,
  pub email: String,
  pub fullname: String,
  pub workspace_id: i64,
  pub workspace_name: String,
  pub access_token: String,
  pub refresh_token: String,
}

/// Test environment
pub struct TestEnvironment {
  pub config: TestConfig,
  pub db: TestPg,
  pub fechatter_state: AppState,
  pub nats_client: Option<async_nats::Client>,
  pub users: Vec<TestUser>,
}

impl TestEnvironment {
  /// Create test environment
  pub async fn new() -> Result<Self> {
    init_test_logging();

    let config = TestConfig::default();

    // Use AppState::test_new() method which creates and manages its own test database
    let (db, fechatter_state) = fechatter_server::AppState::test_new().await?;

    // Try connecting to NATS (optional)
    let nats_client = match async_nats::connect(&config.nats_url).await {
      Ok(client) => {
        info!("Connected to NATS server");
        Some(client)
      }
      Err(e) => {
        info!("Failed to connect to NATS: {}, continuing without NATS", e);
        None
      }
    };

    Ok(Self {
      config,
      db,
      fechatter_state,
      nats_client,
      users: Vec::new(),
    })
  }

  /// Create test environment with NATS event publishing support
  pub async fn new_with_nats() -> Result<Self> {
    init_test_logging();

    let config = TestConfig::default();

    // Use initialization method that supports NATS
    let (db, fechatter_state) = fechatter_server::AppState::test_new_with_nats().await?;

    // Try connecting to NATS (optional)
    let nats_client = match async_nats::connect(&config.nats_url).await {
      Ok(client) => {
        info!("Connected to NATS server for test environment");
        Some(client)
      }
      Err(e) => {
        info!("Failed to connect to NATS: {}, continuing without NATS", e);
        None
      }
    };

    Ok(Self {
      config,
      db,
      fechatter_state,
      nats_client,
      users: Vec::new(),
    })
  }

  /// Cleanup test environment explicitly
  pub async fn cleanup(&mut self) -> Result<()> {
    // Close NATS connection if it exists
    if let Some(client) = &self.nats_client {
      // Close all subscriptions and drain the client
      if let Err(e) = client.flush().await {
        tracing::warn!("Failed to flush NATS client: {}", e);
      }
    }
    self.nats_client = None;

    // Close database connection pool
    self.fechatter_state.pool().close().await;

    info!("✅ Test environment cleaned up");
    Ok(())
  }

  /// Create test user
  pub async fn create_test_user(&mut self, suffix: &str) -> Result<&TestUser> {
    // Generate unique timestamp and process-based identifier to avoid conflicts
    let timestamp = std::time::SystemTime::now()
      .duration_since(std::time::UNIX_EPOCH)
      .unwrap()
      .as_nanos();
    let process_id = std::process::id();
    let thread_id = format!("{:?}", std::thread::current().id()).len() as u64;
    let unique_id = format!("{}{}{}", timestamp, process_id, thread_id);

    let fullname = format!("TestUser{}", suffix);
    let email = format!("testuser{}{}@example.com", suffix, unique_id);

    // Ensure workspace name doesn't exceed 32 character limit
    let workspace = if suffix.len() > 15 {
      // If suffix is too long, use hash to shorten it
      let hash = format!("{:x}", md5::compute(suffix.as_bytes()));
      format!("TestWS{}", &hash[0..20]) // "TestWS" (6) + 20 char hash = 26 chars, safely under 32
    } else {
      // Use first 8 chars of unique_id to ensure total length stays under 32
      let short_unique = &unique_id[0..8.min(unique_id.len())];
      let available_chars = 32 - 6 - short_unique.len(); // 32 - "TestWS" - unique_id_part
      let safe_suffix = if suffix.len() > available_chars {
        &suffix[0..available_chars]
      } else {
        suffix
      };
      format!("TestWS{}{}", safe_suffix, short_unique)
    };

    // Create user
    let user_payload = CreateUser::new(&fullname, &email, &workspace, "password123");
    let created_user = self
      .fechatter_state
      .create_user(&user_payload, None)
      .await?;

    // Login to get tokens
    let signin_payload = SigninUser {
      email: email.clone(),
      password: "password123".to_string(),
    };

    let tokens = self.fechatter_state.signin(&signin_payload, None).await?;
    let tokens = tokens.ok_or_else(|| anyhow::anyhow!("Failed to get tokens"))?;

    let test_user = TestUser {
      id: created_user.id,
      email,
      fullname,
      workspace_id: created_user.workspace_id,
      workspace_name: workspace,
      access_token: tokens.access_token,
      refresh_token: tokens.refresh_token.token, // Fix: get actual token string
    };

    self.users.push(test_user);
    Ok(self.users.last().unwrap())
  }

  /// Create multiple test users
  pub async fn create_test_users(&mut self, count: usize) -> Result<&[TestUser]> {
    // Generate a unique base identifier for this batch of users
    let batch_timestamp = std::time::SystemTime::now()
      .duration_since(std::time::UNIX_EPOCH)
      .unwrap()
      .as_nanos();
    let batch_process_id = std::process::id();
    let batch_id = format!("{}{}", batch_timestamp, batch_process_id);

    for i in 0..count {
      // Use batch_id combined with index to ensure uniqueness across different test runs
      let unique_suffix = format!("{}_{}", batch_id, i);
      self.create_test_user(&unique_suffix).await?;
    }
    Ok(&self.users)
  }

  /// Check if NATS is available
  pub fn is_nats_available(&self) -> bool {
    self.nats_client.is_some()
  }

  /// Get NATS client
  pub fn nats_client(&self) -> Option<&async_nats::Client> {
    self.nats_client.as_ref()
  }
}

/// HTTP client helper
pub struct HttpClient {
  client: reqwest::Client,
  base_url: String,
}

impl HttpClient {
  pub fn new(base_url: String) -> Self {
    let client = reqwest::Client::builder()
      .cookie_store(true)
      .build()
      .expect("Failed to create HTTP client");

    Self { client, base_url }
  }

  /// Send authenticated request
  pub async fn authenticated_request(
    &self,
    method: reqwest::Method,
    path: &str,
    token: &str,
    body: Option<serde_json::Value>,
  ) -> Result<reqwest::Response> {
    let url = format!("{}{}", self.base_url, path);

    let mut request = self
      .client
      .request(method, &url)
      .header("Authorization", format!("Bearer {}", token));

    if let Some(body) = body {
      request = request.json(&body);
    }

    let response = request.send().await?;
    Ok(response)
  }

  /// GET request
  pub async fn get(&self, path: &str, token: &str) -> Result<reqwest::Response> {
    self
      .authenticated_request(reqwest::Method::GET, path, token, None)
      .await
  }

  /// POST request
  pub async fn post(
    &self,
    path: &str,
    token: &str,
    body: serde_json::Value,
  ) -> Result<reqwest::Response> {
    self
      .authenticated_request(reqwest::Method::POST, path, token, Some(body))
      .await
  }

  /// PUT request
  pub async fn put(
    &self,
    path: &str,
    token: &str,
    body: serde_json::Value,
  ) -> Result<reqwest::Response> {
    self
      .authenticated_request(reqwest::Method::PUT, path, token, Some(body))
      .await
  }

  /// DELETE request
  pub async fn delete(&self, path: &str, token: &str) -> Result<reqwest::Response> {
    self
      .authenticated_request(reqwest::Method::DELETE, path, token, None)
      .await
  }
}

/// NATS test utilities
pub struct NatsTestUtils {
  client: async_nats::Client,
}

impl NatsTestUtils {
  pub fn new(client: async_nats::Client) -> Self {
    Self { client }
  }

  /// Subscribe to topic and wait for message
  pub async fn wait_for_message(
    &self,
    subject: String,
    timeout_ms: u64,
  ) -> Result<async_nats::Message> {
    let mut subscriber = self.client.subscribe(subject).await?;

    tokio::select! {
        msg = subscriber.next() => {
            msg.ok_or_else(|| anyhow::anyhow!("No message received"))
        }
        _ = tokio::time::sleep(tokio::time::Duration::from_millis(timeout_ms)) => {
            Err(anyhow::anyhow!("Timeout waiting for message"))
        }
    }
  }

  /// Publish message
  pub async fn publish(&self, subject: String, payload: Vec<u8>) -> Result<()> {
    self.client.publish(subject, payload.into()).await?;
    Ok(())
  }

  /// Check JetStream status
  pub async fn check_jetstream(&self) -> Result<()> {
    let jetstream = async_nats::jetstream::new(self.client.clone());
    let _account_info = jetstream.query_account().await?;
    Ok(())
  }
}

/// Assertion macro
#[macro_export]
macro_rules! assert_json_eq {
  ($left:expr, $right:expr) => {
    assert_eq!(
      serde_json::to_value($left).unwrap(),
      serde_json::to_value($right).unwrap()
    );
  };
}

/// Wait for condition macro
#[macro_export]
macro_rules! wait_for {
  ($condition:expr, $timeout_ms:expr) => {{
    let start = std::time::Instant::now();
    let timeout = std::time::Duration::from_millis($timeout_ms);

    loop {
      if $condition {
        break Ok(());
      }

      if start.elapsed() > timeout {
        break Err(anyhow::anyhow!("Timeout waiting for condition"));
      }

      tokio::time::sleep(std::time::Duration::from_millis(10)).await;
    }
  }};
}
