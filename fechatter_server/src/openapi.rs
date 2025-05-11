use crate::{AppState, error::ErrorOutput, handlers::*};

use axum::Router;
use fechatter_core::{
  AuthUser, Chat, ChatType, ChatUser, CreateChat, CreateMessage, CreateUser, ListMessages, Message,
  SigninUser, User, Workspace,
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
        paths(            // Auth endpoints
            auth::signup_handler,
            auth::signin_handler,
            auth::refresh_token_handler,
            auth::logout_handler,
            auth::logout_all_handler,
            
            // Chat endpoints
            chat::list_chats_handler,
            chat::create_chat_handler,
            chat::update_chat_handler,
            chat::delete_chat_handler,
            
            // Chat members endpoints
            chat_member::list_chat_members_handler,
            chat_member::add_chat_members_batch_handler,
            chat_member::remove_chat_member_handler,
            chat_member::transfer_chat_ownership_handler,
            
            // Messages endpoints
            messages::send_message_handler,
            messages::list_messages_handler,
            
            // Workspace endpoints
            workspace::list_all_workspace_users_handler,
            workspace::get_workspace_by_id
        ),
        components(
            schemas(
                AuthUser,
                Chat,
                ChatType,
                ChatUser,
                CreateChat,
                CreateMessage,
                CreateUser,
                ErrorOutput,
                ListMessages,
                Message,
                SigninUser,
                User,
                Workspace,
                auth::AuthResponse
            )
        ),
        modifiers(&SecurityAddon),
        tags(
            (name = "auth", description = "Authentication operations"),
            (name = "chats", description = "Chat management operations"),
            (name = "chat members", description = "Chat member management operations"),
            (name = "messages", description = "Message operations"),
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
    self
      .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
      .merge(Redoc::with_url("/redoc", ApiDoc::openapi()))
      .merge(RapiDoc::new("/api-docs/openapi.json").path("/rapidoc"))
  }
}
