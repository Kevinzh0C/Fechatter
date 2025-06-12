//! Message Service Factory with proper NATS configuration

use super::{DualStreamDispatcher, DualStreamMessageService, MessageConfig};
use crate::domains::messaging::messaging_domain::{MessageDomainService, MessageDomainServiceImpl};
use crate::domains::messaging::repository::MessageRepository;
use crate::services::infrastructure::flows::notifications::{
  NotificationServiceTrait, create_notification_flow_service_with_nats, create_notification_service,
};
use crate::{AppState, error::AppError};
use std::sync::Arc;

/// Create message service with proper NATS connection
pub async fn create_message_service_with_nats(
  state: &AppState,
) -> Result<DualStreamMessageService, AppError> {
  // 1. Create domain service
  let pool = state.pool().clone();
  let repository = Arc::new(MessageRepository::new(pool));
  let config = MessageConfig::default();
  let domain_impl = Arc::new(MessageDomainServiceImpl::new(repository, config));
  let domain_service: Arc<dyn MessageDomainService> = domain_impl;

  // 2. Get or create NATS client
  let nats_client = if let Some(event_publisher) = state.event_publisher_dyn() {
    // Try to get NATS client from event publisher's transport
    if let Some(nats_transport) = event_publisher
      .transport()
      .as_any()
      .downcast_ref::<crate::services::infrastructure::event::NatsTransport>(
    ) {
      nats_transport.client().clone()
    } else {
      // Create new NATS client from config
      create_nats_client_from_config(&state.config()).await?
    }
  } else {
    // Create new NATS client from config
    create_nats_client_from_config(&state.config()).await?
  };

  let dispatcher = Arc::new(DualStreamDispatcher::new(nats_client.clone()));

  // 3. Create notification service
  let flow_service = create_notification_flow_service_with_nats(nats_client);
  let notification_service = create_notification_service(flow_service);

  // 4. Create event publisher adapter
  let event_publisher = Arc::new(super::AppStateEventPublisher::new(
    state.event_publisher_dyn().cloned(),
  ));

  // 5. Create the service with all dependencies
  Ok(DualStreamMessageService::new(
    domain_service,
    dispatcher,
    notification_service,
    event_publisher,
  ))
}

/// Create NATS client from config
async fn create_nats_client_from_config(
  config: &crate::config::AppConfig,
) -> Result<async_nats::Client, AppError> {
  let nats_url = config
    .messaging
    .as_ref()
    .and_then(|m| m.nats.as_ref())
    .map(|n| n.url.as_str())
    .unwrap_or("nats://localhost:4222");

  async_nats::connect(nats_url)
    .await
    .map_err(|e| AppError::Internal(format!("Failed to connect to NATS: {}", e)))
}
