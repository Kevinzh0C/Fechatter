// Contract module defining all service interfaces
pub mod repositories;
pub mod services;

// Infrastructure service interface contracts
pub mod infrastructure;

// Event interface contracts
pub mod events;

// Re-export interface contracts
pub use events::*;
pub use infrastructure::*;
pub use repositories::*;
pub use services::*;

// Explicitly export AuthContext to resolve visibility issues
pub use services::AuthContext;
