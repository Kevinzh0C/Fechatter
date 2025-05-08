pub mod chat;
pub mod error;
pub mod http;
pub mod jwt;
pub mod macros;
pub mod middlewares;
pub mod models;
pub mod services;

// Re-export common traits for easier import
pub use error::{CoreError, ErrorMapper};
pub use middlewares::{
  ActualAuthServiceProvider, TokenVerifier, WithServiceProvider, WithTokenManager,
};
pub use models::jwt::{TokenService, UserClaims};
pub use models::{AuthUser, CreateUser, SigninUser, UserStatus, chat::*, user::*};
pub use services::AuthContext;
