use crate::{AppState, error::ErrorOutput};

use axum::Router;
use fechatter_core::{
  AuthUser, Chat, ChatType, ChatUser, CreateChat, CreateMessage, CreateUser, ListMessages, Message,
  SigninUser, User, Workspace,
  SearchMessages, SearchResult, SearchableMessage,
};

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
        paths(
            // Health endpoints
            crate::handlers::health_check,
            crate::handlers::simple_health_check,
            
            // Auth endpoints
            crate::handlers::signup_handler,
            crate::handlers::signin_handler,
            crate::handlers::refresh_token_handler,
            crate::handlers::logout_handler,
            crate::handlers::logout_all_handler,
            
            // Chat endpoints
            crate::handlers::list_chats_handler,
            crate::handlers::create_chat_handler, 
            crate::handlers::update_chat_handler,
            crate::handlers::delete_chat_handler,
            
            // Chat members endpoints
            crate::handlers::list_chat_members_handler,
            crate::handlers::add_chat_members_batch_handler,
            crate::handlers::remove_chat_member_handler,
            crate::handlers::transfer_chat_ownership_handler,
            
            // Messages endpoints
            crate::handlers::send_message_handler,
            crate::handlers::list_messages_handler,
            crate::handlers::search_messages,
            
            // File endpoints
            crate::handlers::file_handler,
            crate::handlers::upload_handler,
            crate::handlers::fix_file_storage_handler,
            
            // Workspace endpoints
            crate::handlers::list_all_workspace_users_handler,
            crate::handlers::get_workspace_by_id
        ),
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
