use crate::{AppState, error::ErrorOutput};

use axum::Router;
use fechatter_core::{
  AuthUser, Chat, ChatType, ChatUser, CreateChat, CreateMessage, CreateUser, ListMessages, Message,
  SigninUser, User, Workspace,
  SearchMessages, SearchableMessage,
};
use fechatter_core::models::message::SearchResult;

use utoipa::{
  Modify, OpenApi,
  openapi::security::{ApiKey, ApiKeyValue, HttpAuthScheme, HttpBuilder, SecurityScheme},
};
use utoipa_rapidoc::RapiDoc;
use utoipa_redoc::{Redoc, Servable};
use utoipa_swagger_ui::SwaggerUi;

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
                
                // Search schemas - 精简后只保留核心的3个
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

impl OpenApiRouter for Router<AppState> {
  fn openapi(self) -> Self {
      self.merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
          .merge(Redoc::with_url("/redoc", ApiDoc::openapi()))
          .merge(RapiDoc::new("/api-docs/openapi.json").path("/rapidoc"))
  }
}
