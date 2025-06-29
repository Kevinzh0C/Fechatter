use anyhow::Result;
use async_nats;
use tracing::{error, info, warn};

/// NATS client manager
pub struct NatsClient {
  client: async_nats::Client,
}

impl NatsClient {
  /// Create a NATS connection with retry logic
  pub async fn connect_with_retry(url: &str) -> Result<Self> {
    let mut retries = 0;
    const MAX_RETRIES: u32 = 5;
    const RETRY_DELAY: std::time::Duration = std::time::Duration::from_secs(2);

    // Set NATS connection options for better stability
    let connect_options = async_nats::ConnectOptions::new()
      .connection_timeout(std::time::Duration::from_secs(10))
      .ping_interval(std::time::Duration::from_secs(30))
      .max_reconnects(Some(10))
      .reconnect_delay_callback(|attempts: usize| {
        std::time::Duration::from_secs(std::cmp::min(2u64.saturating_pow(attempts as u32), 30))
      });

    loop {
      info!("🔗 Attempting to connect to NATS: {}", url);

      // Note: async_nats::ConnectOptions does not implement Clone, so recreate each time
      let connect_options = async_nats::ConnectOptions::new()
        .connection_timeout(std::time::Duration::from_secs(10))
        .ping_interval(std::time::Duration::from_secs(30))
        .max_reconnects(Some(10))
        .reconnect_delay_callback(|attempts: usize| {
          std::time::Duration::from_secs(std::cmp::min(2u64.saturating_pow(attempts as u32), 30))
        });

      match connect_options.connect(url).await {
        Ok(client) => {
          info!("Successfully connected to NATS: {}", url);
          return Ok(Self { client });
        }
        Err(e) => {
          retries += 1;
          if retries >= MAX_RETRIES {
            error!(
              "ERROR: Failed to connect to NATS after {} retries: {}",
              MAX_RETRIES, e
            );
            return Err(e.into());
          }
          warn!(
            "WARNING: Failed to connect to NATS (attempt {}/{}): {}. Retrying in {:?}",
            retries, MAX_RETRIES, e, RETRY_DELAY
          );
          tokio::time::sleep(RETRY_DELAY).await;
        }
      }
    }
  }

  /// Get the underlying NATS client
  pub fn client(&self) -> &async_nats::Client {
    &self.client
  }

  /// Simple subscribe
  pub async fn subscribe(&self, subject: &str) -> Result<async_nats::Subscriber> {
    info!("SUBSCRIPTION: Subscribing to subject: {}", subject);
    let subscriber = self.client.subscribe(subject.to_string()).await?;
    Ok(subscriber)
  }

  /// Publish a message
  pub async fn publish(&self, subject: &str, payload: &[u8]) -> Result<()> {
    self
      .client
      .publish(subject.to_string(), payload.to_vec().into())
      .await?;
    Ok(())
  }

  /// Publish with reply (request)
  pub async fn request(&self, subject: &str, payload: &[u8]) -> Result<async_nats::Message> {
    let response = self
      .client
      .request(subject.to_string(), payload.to_vec().into())
      .await?;
    Ok(response)
  }

  /// Check connection status
  pub fn is_connected(&self) -> bool {
    // The NATS client manages connection state internally; this is a placeholder.
    // Actual connection checking can be done via ping if needed.
    true
  }

  /// Gracefully close the connection
  pub async fn close(self) -> Result<()> {
    // async_nats client will close the connection automatically on drop.
    // This provides an explicit close method.
    drop(self.client);
    info!("📴 NATS connection closed");
    Ok(())
  }
}
