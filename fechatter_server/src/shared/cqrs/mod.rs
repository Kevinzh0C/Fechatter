// CQRS Infrastructure
use async_trait::async_trait;
use std::sync::Arc;

/// Command trait - represents a write operation
pub trait Command: Send + Sync {
  type Result: Send + Sync;
}

/// Query trait - represents a read operation
pub trait Query: Send + Sync {
  type Result: Send + Sync;
}

/// Command handler trait
#[async_trait]
pub trait CommandHandler<C: Command>: Send + Sync {
  async fn handle(&self, command: C) -> Result<C::Result, crate::AppError>;
}

/// Query handler trait
#[async_trait]
pub trait QueryHandler<Q: Query>: Send + Sync {
  async fn handle(&self, query: Q) -> Result<Q::Result, crate::AppError>;
}

/// Command bus - dispatches commands to handlers
pub struct CommandBus {
  handlers: std::collections::HashMap<std::any::TypeId, Box<dyn std::any::Any + Send + Sync>>,
}

impl CommandBus {
  pub fn new() -> Self {
    Self {
      handlers: std::collections::HashMap::new(),
    }
  }

  pub fn register<C: Command + 'static>(&mut self, handler: Arc<dyn CommandHandler<C>>) {
    self
      .handlers
      .insert(std::any::TypeId::of::<C>(), Box::new(handler));
  }

  pub async fn dispatch<C: Command + 'static>(
    &self,
    command: C,
  ) -> Result<C::Result, crate::AppError> {
    let type_id = std::any::TypeId::of::<C>();
    let handler = self
      .handlers
      .get(&type_id)
      .ok_or_else(|| crate::AppError::Internal("No handler registered for command".to_string()))?;

    let handler = handler
      .downcast_ref::<Arc<dyn CommandHandler<C>>>()
      .ok_or_else(|| crate::AppError::Internal("Handler type mismatch".to_string()))?;

    handler.handle(command).await
  }
}

/// Query bus - dispatches queries to handlers
pub struct QueryBus {
  handlers: std::collections::HashMap<std::any::TypeId, Box<dyn std::any::Any + Send + Sync>>,
}

impl QueryBus {
  pub fn new() -> Self {
    Self {
      handlers: std::collections::HashMap::new(),
    }
  }

  pub fn register<Q: Query + 'static>(&mut self, handler: Arc<dyn QueryHandler<Q>>) {
    self
      .handlers
      .insert(std::any::TypeId::of::<Q>(), Box::new(handler));
  }

  pub async fn dispatch<Q: Query + 'static>(&self, query: Q) -> Result<Q::Result, crate::AppError> {
    let type_id = std::any::TypeId::of::<Q>();
    let handler = self
      .handlers
      .get(&type_id)
      .ok_or_else(|| crate::AppError::Internal("No handler registered for query".to_string()))?;

    let handler = handler
      .downcast_ref::<Arc<dyn QueryHandler<Q>>>()
      .ok_or_else(|| crate::AppError::Internal("Handler type mismatch".to_string()))?;

    handler.handle(query).await
  }
}
