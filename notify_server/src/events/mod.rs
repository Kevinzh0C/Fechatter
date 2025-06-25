pub mod nats;
pub mod processor;
pub mod types;

pub use processor::{EventProcessor, handle_system_event};
