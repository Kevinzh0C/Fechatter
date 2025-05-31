// Event bus infrastructure
use async_trait::async_trait;
use std::sync::Arc;

/// Domain event trait
pub trait DomainEvent: Send + Sync {
  fn event_type(&self) -> &'static str;
}

/// Event handler trait
#[async_trait]
pub trait EventHandler<E: DomainEvent>: Send + Sync {
  async fn handle(&self, event: &E) -> Result<(), crate::AppError>;
}

/// Event bus for publishing and subscribing to domain events
pub struct EventBus {
  handlers: std::collections::HashMap<&'static str, Vec<Box<dyn std::any::Any + Send + Sync>>>,
}

impl EventBus {
  pub fn new() -> Self {
    Self {
      handlers: std::collections::HashMap::new(),
    }
  }

  pub fn subscribe<E: DomainEvent + 'static>(&mut self, handler: Arc<dyn EventHandler<E>>) {
    let event_type = std::any::type_name::<E>();
    self
      .handlers
      .entry(event_type)
      .or_insert_with(Vec::new)
      .push(Box::new(handler));
  }

  pub async fn publish<E: DomainEvent + 'static>(&self, event: E) -> Result<(), crate::AppError> {
    let event_type = std::any::type_name::<E>();

    if let Some(handlers) = self.handlers.get(event_type) {
      for handler in handlers {
        if let Some(h) = handler.downcast_ref::<Arc<dyn EventHandler<E>>>() {
          h.handle(&event).await?;
        }
      }
    }

    Ok(())
  }
}
