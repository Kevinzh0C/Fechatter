use crate::{AppState, ErrorOutput, handlers::*};
use axum::{Router, response::Json, routing::get};
use utoipa::{
  Modify, OpenApi,
  openapi::security::{HttpAuthScheme, HttpBuilder, SecurityScheme},
};

pub(crate) trait OpenApiRouter {
  fn openapi(self) -> Self;
}

#[derive(OpenApi)]
#[openapi(
    info(
        title = "Analytics Server API",
        version = "1.0.0",
        description = "High-performance analytics data collection service for Fechatter",
        contact(
            name = "Analytics Team",
            email = "analytics@fechatter.com"
        ),
        license(
            name = "MIT",
            url = "https://opensource.org/licenses/MIT"
        )
    ),
    servers(
        (url = "/", description = "Current server")
    ),
    paths(
        create_event_handler,
        health_check_handler,
    ),
    components(
        schemas(
            ErrorOutput
        ),
    ),
    modifiers(&SecurityAddon),
    tags(
        (name = "analytics", description = "Analytics event collection endpoints"),
        (name = "health", description = "Service health and monitoring endpoints"),
    )
)]
pub(crate) struct ApiDoc;

struct SecurityAddon;

impl Modify for SecurityAddon {
  fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
    if let Some(components) = openapi.components.as_mut() {
      components.add_security_scheme(
        "bearer_auth",
        SecurityScheme::Http(
          HttpBuilder::new()
            .scheme(HttpAuthScheme::Bearer)
            .bearer_format("JWT")
            .description(Some("Optional JWT token for authenticated requests"))
            .build(),
        ),
      );
    }
  }
}

/// Serve OpenAPI JSON specification
async fn openapi_json() -> Json<utoipa::openapi::OpenApi> {
  Json(ApiDoc::openapi())
}

/// Redirect to Swagger UI
async fn swagger_redirect() -> axum::response::Redirect {
  axum::response::Redirect::permanent("/swagger-ui/")
}

/// Serve a basic Swagger UI (simplified version)
async fn swagger_ui() -> axum::response::Html<&'static str> {
  axum::response::Html(include_str!("../swagger-ui.html"))
}

impl OpenApiRouter for Router<AppState> {
  fn openapi(self) -> Self {
    self
      .route("/openapi.json", get(openapi_json))
      .route("/docs", get(swagger_redirect))
      .route("/swagger-ui/", get(swagger_ui))
      .route("/api-docs/openapi.json", get(openapi_json))
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_openapi_generation() {
    let openapi = ApiDoc::openapi();
    assert_eq!(openapi.info.title, "Analytics Server API");
    assert_eq!(openapi.info.version, "1.0.0");
    assert!(!openapi.paths.paths.is_empty());
  }

  #[test]
  fn test_security_scheme_added() {
    let openapi = ApiDoc::openapi();
    assert!(openapi.components.is_some());

    if let Some(components) = &openapi.components {
      assert!(components.security_schemes.contains_key("bearer_auth"));
    }
  }
}
