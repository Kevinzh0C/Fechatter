pub mod bearer_auth;
pub mod custom_builder;
pub mod mw_traits;
pub mod request_id;
pub mod server_time;
pub mod token_refresh;

use axum::{Router, middleware::from_fn};

use tower::ServiceBuilder;
use tower_http::{
  LatencyUnit,
  compression::CompressionLayer,
  trace::{DefaultMakeSpan, DefaultOnRequest, DefaultOnResponse, TraceLayer},
};
use tracing::Level;

use crate::models::AuthUser;
use crate::models::jwt::AuthServiceTrait;


pub use self::bearer_auth::verify_token_middleware;
pub use self::custom_builder::*;
pub use self::request_id::request_id_middleware;
pub use self::server_time::ServerTimeLayer;

pub const REQUEST_ID_HEADER: &str = "x-request-id";
pub const SERVER_TIME_HEADER: &str = "x-server-time";

/// 一个简单的trait，表示类型可以提供ID
/// 用于需要访问ID的通用代码
pub trait HasIdField {
  fn id(&self) -> i64;
}

/// 为标准AuthUser实现HasIdField
impl HasIdField for AuthUser {
  fn id(&self) -> i64 {
    self.id
  }
}

/// 查找密钥文件并返回文件路径
///
/// 按照优先级顺序查找:
/// 1. 项目根目录/fechatter_core/fixtures
/// 2. 当前目录/fixtures
/// 3. 上级目录/fixtures
///
/// 返回 (encoding_path, decoding_path) 元组
pub fn find_key_files() -> (String, String) {
  let paths = ["fechatter_core/fixtures", "fixtures", "../fixtures"];
  let mut enc_path = String::from("fixtures/encoding.pem");
  let mut dec_path = String::from("fixtures/decoding.pem");

  for base_path in paths {
    let test_enc = format!("{}/encoding.pem", base_path);
    let test_dec = format!("{}/decoding.pem", base_path);

    if std::path::Path::new(&test_enc).exists() && std::path::Path::new(&test_dec).exists() {
      enc_path = test_enc;
      dec_path = test_dec;
      break;
    }
  }

  println!("Using keys from: {}", enc_path);
  (enc_path, dec_path)
}

pub trait TokenVerifier {
  type Error: std::fmt::Debug;
  type Claims;
  fn verify_token(&self, token: &str) -> Result<Self::Claims, Self::Error>;
}

// New trait for the type that actually provides an AuthService instance
pub trait ActualAuthServiceProvider {
  // The associated type AuthService should itself be bound by AuthServiceTrait
  type AuthService: AuthServiceTrait;
  fn create_service(&self) -> Self::AuthService;
}

pub trait WithServiceProvider {
  type ServiceProviderType: ActualAuthServiceProvider; // Bound ServiceProviderType
  fn service_provider(&self) -> &Self::ServiceProviderType;
}

pub trait WithTokenManager {
  type TokenManagerType: TokenVerifier; // Bound TokenManagerType
  fn token_manager(&self) -> &Self::TokenManagerType;
}

/// Apply common middleware to a router
pub trait SetLayer {
  fn set_layer(self) -> Self;
}

impl<S> SetLayer for Router<S>
where
  S: Clone + Send + Sync + 'static,
{
  fn set_layer(self) -> Self {
    self.layer(
      ServiceBuilder::new()
        .layer(
          TraceLayer::new_for_http()
            .make_span_with(DefaultMakeSpan::new().include_headers(true))
            .on_request(DefaultOnRequest::new().level(Level::INFO))
            .on_response(
              DefaultOnResponse::new()
                .level(Level::INFO)
                .latency_unit(LatencyUnit::Micros),
            ),
        )
        .layer(CompressionLayer::new().gzip(true).br(true).deflate(true))
        .layer(from_fn(request_id_middleware))
        .layer(ServerTimeLayer),
    )
  }
}

/// Apply authentication middleware to a router
pub fn with_auth_layer<S, T>(router: Router<S>, _state: T) -> Router<S>
where
  S: Clone + Send + Sync + 'static,
  T: TokenVerifier + Clone + Send + Sync + 'static,
  AuthUser: From<T::Claims>,
{
  router
}
