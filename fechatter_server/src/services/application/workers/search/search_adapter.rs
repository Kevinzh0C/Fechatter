//! Search Service Adapter
//!
//! Adapts InfraSearchService to the interface expected by handlers

use async_trait::async_trait;
use std::sync::Arc;
use tracing::{info, warn};

use crate::{error::AppError, services::infrastructure::search::InfraSearchService};
use fechatter_core::{models::SearchMessages, ChatId, MessageId, UserId, WorkspaceId};

use super::{MessageSearchResults, SearchApplicationServiceTrait, SearchPage, SearchableMessage};

/// Adapter that wraps InfraSearchService to provide the expected interface
pub struct SearchServiceAdapter {
    inner: Arc<InfraSearchService>,
}

impl SearchServiceAdapter {
    pub fn new(search_service: Arc<InfraSearchService>) -> Self {
        Self {
            inner: search_service,
        }
    }
}

#[async_trait]
impl SearchApplicationServiceTrait for SearchServiceAdapter {
    async fn search_messages_in_chat(
        &self,
        chat_id: ChatId,
        query: &str,
        user_id: UserId,
        limit: u32,
        offset: u32,
    ) -> Result<MessageSearchResults, AppError> {
        info!(
            "Searching messages in chat {} for user {} with query: {}",
            chat_id, user_id, query
        );

        let start = std::time::Instant::now();

        // Create search request
        let request = SearchMessages {
            query: query.to_string(),
            workspace_id: WorkspaceId::new(1), // TODO: Get from user context
            chat_id: Some(chat_id),
            limit: limit as i64,
            offset: offset as i64,
        };

        // Execute search
        let results = self.inner.search_messages(&request).await?;
        let elapsed_ms = start.elapsed().as_millis() as u64;

        // Convert to expected format (using the correct SearchableMessage from service.rs)
        let hits: Vec<SearchableMessage> = results
            .messages
            .into_iter()
            .map(|msg| SearchableMessage {
                id: msg.id.into(),
                chat_id: msg.chat_id.into(),
                sender_id: msg.sender_id.into(),
                sender_name: if msg.sender_name.is_empty() {
                    "Unknown".to_string()
                } else {
                    msg.sender_name
                },
                content: msg.content,
                files: msg.files,
                created_at: msg.created_at,
                workspace_id: 1,          // TODO: Get from context
                chat_name: String::new(), // TODO: Get from context
                chat_type: String::new(), // TODO: Get from context
            })
            .collect();

        Ok(MessageSearchResults {
            hits,
            total: results.total_hits as u64,
            took_ms: elapsed_ms,
            query: query.to_string(),
            page: SearchPage {
                offset,
                limit,
                has_more: (offset as u64 + limit as u64) < results.total_hits as u64,
            },
        })
    }

    async fn global_search_messages(
        &self,
        query: &str,
        user_id: UserId,
        workspace_id: WorkspaceId,
        limit: u32,
        offset: u32,
    ) -> Result<MessageSearchResults, AppError> {
        info!(
            "Global search in workspace {} for user {} with query: {}",
            workspace_id, user_id, query
        );

        let start = std::time::Instant::now();

        // Create search request
        let request = SearchMessages {
            query: query.to_string(),
            workspace_id,
            chat_id: None,
            limit: limit as i64,
            offset: offset as i64,
        };

        // Execute search
        let results = self.inner.search_messages(&request).await?;
        let elapsed_ms = start.elapsed().as_millis() as u64;

        // Convert to expected format
        let hits: Vec<SearchableMessage> = results
            .messages
            .into_iter()
            .map(|msg| SearchableMessage {
                id: msg.id.into(),
                chat_id: msg.chat_id.into(),
                sender_id: msg.sender_id.into(),
                sender_name: if msg.sender_name.is_empty() {
                    "Unknown".to_string()
                } else {
                    msg.sender_name
                },
                content: msg.content,
                files: msg.files,
                created_at: msg.created_at,
                workspace_id: i64::from(workspace_id),
                chat_name: String::new(), // TODO: Get from context
                chat_type: String::new(), // TODO: Get from context
            })
            .collect();

        Ok(MessageSearchResults {
            hits,
            total: results.total_hits as u64,
            took_ms: elapsed_ms,
            query: query.to_string(),
            page: SearchPage {
                offset,
                limit,
                has_more: (offset as u64 + limit as u64) < results.total_hits as u64,
            },
        })
    }

    async fn get_search_suggestions(
        &self,
        partial_query: &str,
        _limit: u32,
    ) -> Result<Vec<String>, AppError> {
        warn!(
            "Search suggestions not implemented with query: {}",
            partial_query
        );

        // For now, return empty suggestions
        Ok(vec![])
    }

    async fn index_messages_batch(&self, _messages: &[SearchableMessage]) -> Result<(), AppError> {
        warn!("index_messages_batch not implemented in adapter");
        Ok(())
    }

    async fn remove_messages_from_index_batch(
        &self,
        _message_ids: &[MessageId],
    ) -> Result<(), AppError> {
        warn!("remove_messages_from_index_batch not implemented in adapter");
        Ok(())
    }

    async fn update_messages_in_index_batch(
        &self,
        _messages: &[SearchableMessage],
    ) -> Result<(), AppError> {
        warn!("update_messages_in_index_batch not implemented in adapter");
        Ok(())
    }

    async fn reindex_chat_messages(&self, chat_id: ChatId) -> Result<u64, AppError> {
        warn!(
            "reindex_chat_messages not implemented in adapter for chat {}",
            chat_id
        );
        Ok(0)
    }
}
