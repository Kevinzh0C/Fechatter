use crate::dtos::models::{
  requests::auth::{ChangePasswordRequest, LoginRequest, RefreshTokenRequest, RegisterRequest},
  responses::auth::{
    ChangePasswordResponse, LoginResponse, RefreshTokenResponse, RegisterResponse,
  },
};
use crate::{AppState, error::ErrorOutput};

use axum::{Router, response::Json, routing::get, response::Html};
use fechatter_core::models::message::SearchResult;
use fechatter_core::{
  AuthUser, Chat, ChatType, ChatUser, CreateChat, CreateMessage, CreateUser, ListMessages, Message,
  SearchMessages, SearchableMessage, SigninUser, User, Workspace,
};

use utoipa::{
  Modify, OpenApi,
  openapi::security::{ApiKey, ApiKeyValue, HttpAuthScheme, HttpBuilder, SecurityScheme},
};

pub(crate) trait OpenApiRouter {
  fn openapi(self) -> Self;
}

/// API Documentation
#[derive(OpenApi)]
#[openapi(
        // TODO: Re-enable paths when handlers have proper utoipa annotations
        components(
            schemas(
                // Error schemas
                ErrorOutput,
                
                // Auth schemas
                SigninUser,
                AuthUser,
                User,
                CreateUser,
                
                // Auth DTOs
                LoginRequest,
                RegisterRequest,
                RefreshTokenRequest,
                ChangePasswordRequest,
                LoginResponse,
                RegisterResponse,
                RefreshTokenResponse,
                ChangePasswordResponse,
                
                // Core chat schemas
                Chat,
                CreateChat,
                ChatUser,
                ChatType,
                
                // Message schemas
                Message,
                CreateMessage,
                ListMessages,
                
                // Workspace schema
                Workspace,
                
                // Search schemas - streamlined to core 3 schemas
                SearchMessages,
                SearchResult,
                SearchableMessage,
            )
        ),
        modifiers(&SecurityAddon),
        tags(
            (name = "auth", description = "Authentication operations"),
            (name = "chats", description = "Chat management operations"),
            (name = "chat members", description = "Chat member management operations"),
            (name = "messages", description = "Message operations"),
            (name = "files", description = "File operations"),
            (name = "search", description = "Search operations"),
            (name = "workspace", description = "Workspace operations")
        )
    )]
pub(crate) struct ApiDoc;

struct SecurityAddon;

impl Modify for SecurityAddon {
  fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
    if let Some(components) = openapi.components.as_mut() {
      components.add_security_scheme(
        "access_token",
        SecurityScheme::Http(HttpBuilder::new().scheme(HttpAuthScheme::Bearer).build()),
      );
      components.add_security_scheme(
        "refresh_token",
        SecurityScheme::ApiKey(ApiKey::Header(ApiKeyValue::new("X-Refresh-Token"))),
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

/// Serve basic Swagger UI
async fn swagger_ui() -> Html<&'static str> {
  Html(r#"
<!DOCTYPE html>
<html>
<head>
    <title>Fechatter API Documentation</title>
    <link rel="stylesheet" type="text/css" href="https://unpkg.com/swagger-ui-dist@5.1.0/swagger-ui.css" />
</head>
<body>
    <div id="swagger-ui"></div>
    <script src="https://unpkg.com/swagger-ui-dist@5.1.0/swagger-ui-bundle.js"></script>
    <script>
    SwaggerUIBundle({
        url: '/api-docs/openapi.json',
        dom_id: '#swagger-ui',
        presets: [
            SwaggerUIBundle.presets.apis,
            SwaggerUIBundle.presets.standalone
        ]
    });
    </script>
</body>
</html>
"#)
}

/// Serve basic Redoc UI
async fn redoc_ui() -> Html<&'static str> {
  Html(r#"
<!DOCTYPE html>
<html>
<head>
    <title>Fechatter API Documentation - Redoc</title>
    <meta charset="utf-8"/>
    <meta name="viewport" content="width=device-width, initial-scale=1">
</head>
<body>
    <redoc spec-url="/api-docs/openapi.json"></redoc>
    <script src="https://cdn.jsdelivr.net/npm/redoc@2.1.3/bundles/redoc.standalone.js"></script>
</body>
</html>
"#)
}

/// Serve basic RapiDoc UI
async fn rapidoc_ui() -> Html<String> {
  let html = format!(r#"
<!DOCTYPE html>
<html>
<head>
    <title>Fechatter API Documentation - RapiDoc</title>
    <meta charset="utf-8">
    <script type="module" src="https://unpkg.com/rapidoc@9.3.4/dist/rapidoc-min.js"></script>
</head>
<body>
    <rapi-doc 
        spec-url="/api-docs/openapi.json"
        theme="dark"
        render-style="read"
        nav-bg-color="{}"
        primary-color="{}"
    ></rapi-doc>
</body>
</html>
"#, "#1a1a1a", "#ff6b35");
  Html(html)
}

impl OpenApiRouter for Router<AppState> {
  fn openapi(self) -> Self {
    self
      .route("/api-docs/openapi.json", get(openapi_json))
      .route("/docs", get(swagger_redirect))
      .route("/swagger-ui", get(swagger_ui))
      .route("/swagger-ui/", get(swagger_ui))
      .route("/redoc", get(redoc_ui))
      .route("/rapidoc", get(rapidoc_ui))
  }
}
