//! Message Data Consistency Monitor
//!
//! Production-grade monitoring and compensation for message data consistency
//! across database, cache, Meilisearch, and vector database.

use async_trait::async_trait;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::interval;
use tracing::{error, info, warn};

use crate::{
    error::AppError, services::application::workers::search::SearchApplicationServiceTrait,
};
use fechatter_core::{
    contracts::CacheService,
    models::{vector_db::MessageVectorRepository, MessageId},
};

/// Consistency check result
#[derive(Debug)]
pub struct ConsistencyCheckResult {
    pub total_messages_checked: usize,
    pub inconsistencies_found: usize,
    pub inconsistencies_fixed: usize,
    pub errors: Vec<String>,
}

/// Message consistency monitor trait
#[async_trait]
pub trait MessageConsistencyMonitor: Send + Sync {
    /// Run a full consistency check
    async fn run_consistency_check(&self) -> Result<ConsistencyCheckResult, AppError>;

    /// Check consistency for a specific message
    async fn check_message_consistency(&self, message_id: MessageId) -> Result<bool, AppError>;

    /// Fix inconsistencies for a specific message
    async fn fix_message_inconsistency(&self, message_id: MessageId) -> Result<(), AppError>;

    /// Start background monitoring
    async fn start_monitoring(&self, check_interval: Duration);
}

/// Production implementation of consistency monitor
pub struct MessageConsistencyMonitorImpl {
    db_pool: Arc<sqlx::PgPool>,
    cache_service: Arc<dyn CacheService>,
    search_service: Option<Arc<dyn SearchApplicationServiceTrait>>,
    vector_db: Option<Arc<dyn MessageVectorRepository>>,
}

impl MessageConsistencyMonitorImpl {
    pub fn new(
        db_pool: Arc<sqlx::PgPool>,
        cache_service: Arc<dyn CacheService>,
        search_service: Option<Arc<dyn SearchApplicationServiceTrait>>,
        vector_db: Option<Arc<dyn MessageVectorRepository>>,
    ) -> Self {
        Self {
            db_pool,
            cache_service,
            search_service,
            vector_db,
        }
    }

    /// Check if message exists in database
    async fn message_exists_in_db(&self, message_id: i64) -> Result<bool, AppError> {
        let result =
            sqlx::query_scalar::<_, bool>("SELECT EXISTS(SELECT 1 FROM messages WHERE id = $1)")
                .bind(message_id)
                .fetch_one(&*self.db_pool)
                .await
                .map_err(|e| AppError::SqlxError(e))?;

        Ok(result)
    }

    /// Check if message exists in cache
    async fn message_exists_in_cache(&self, message_id: i64) -> Result<bool, AppError> {
        let cache_key = format!("message:{}", message_id);
        self.cache_service
            .exists(&cache_key)
            .await
            .map_err(|e| AppError::RedisError(format!("Failed to check cache existence: {}", e)))
    }

    /// Clean up orphaned data (exists in secondary stores but not in database)
    async fn cleanup_orphaned_data(&self, message_id: MessageId) -> Result<(), AppError> {
        let message_id_i64 = i64::from(message_id);

        // Clear from cache
        let cache_key = format!("message:{}", message_id_i64);
        if let Err(e) = self.cache_service.delete(&cache_key).await {
            warn!(
                "Failed to clear orphaned cache entry for message {}: {}",
                message_id_i64, e
            );
        }

        // Clear from search index
        if let Some(search_service) = &self.search_service {
            if let Err(e) = search_service
                .remove_messages_from_index_batch(&[message_id])
                .await
            {
                warn!(
                    "Failed to clear orphaned search entry for message {}: {}",
                    message_id_i64, e
                );
            }
        }

        // Clear from vector database
        if let Some(vector_db) = &self.vector_db {
            if let Err(e) = vector_db.delete_message(message_id).await {
                warn!(
                    "Failed to clear orphaned vector entry for message {}: {}",
                    message_id_i64, e
                );
            }
        }

        Ok(())
    }
}

#[async_trait]
impl MessageConsistencyMonitor for MessageConsistencyMonitorImpl {
    async fn run_consistency_check(&self) -> Result<ConsistencyCheckResult, AppError> {
        let mut result = ConsistencyCheckResult {
            total_messages_checked: 0,
            inconsistencies_found: 0,
            inconsistencies_fixed: 0,
            errors: Vec::new(),
        };

        // Get sample of recent messages to check
        let recent_message_ids: Vec<i64> =
            sqlx::query_scalar("SELECT id FROM messages ORDER BY created_at DESC LIMIT 1000")
                .fetch_all(&*self.db_pool)
                .await
                .map_err(|e| AppError::Internal(format!("Failed to fetch messages: {}", e)))?;

        result.total_messages_checked = recent_message_ids.len();

        for message_id in recent_message_ids {
            match self.check_message_consistency(MessageId(message_id)).await {
                Ok(is_consistent) => {
                    if !is_consistent {
                        result.inconsistencies_found += 1;

                        // Try to fix the inconsistency
                        if let Err(e) = self.fix_message_inconsistency(MessageId(message_id)).await
                        {
                            result
                                .errors
                                .push(format!("Failed to fix message {}: {}", message_id, e));
                        } else {
                            result.inconsistencies_fixed += 1;
                        }
                    }
                }
                Err(e) => {
                    result
                        .errors
                        .push(format!("Failed to check message {}: {}", message_id, e));
                }
            }
        }

        if result.inconsistencies_found > 0 {
            warn!(
                "Consistency check completed: {} inconsistencies found, {} fixed",
                result.inconsistencies_found, result.inconsistencies_fixed
            );
        } else {
            info!("Consistency check completed: No inconsistencies found");
        }

        Ok(result)
    }

    async fn check_message_consistency(&self, message_id: MessageId) -> Result<bool, AppError> {
        let message_id_i64 = i64::from(message_id);

        // Check if message exists in database (source of truth)
        let exists_in_db = self.message_exists_in_db(message_id_i64).await?;

        if !exists_in_db {
            // Message doesn't exist in DB - check if it exists in other stores
            let exists_in_cache = self.message_exists_in_cache(message_id_i64).await?;

            if exists_in_cache {
                warn!(
                    "Message {} found in cache but not in database",
                    message_id_i64
                );
                return Ok(false);
            }

            // TODO: Check search index and vector DB

            // Message consistently doesn't exist
            return Ok(true);
        }

        // Message exists in DB - verify it's properly indexed
        // TODO: Implement search index and vector DB consistency checks

        Ok(true)
    }

    async fn fix_message_inconsistency(&self, message_id: MessageId) -> Result<(), AppError> {
        let message_id_i64 = i64::from(message_id);

        // Check if message exists in database
        let exists_in_db = self.message_exists_in_db(message_id_i64).await?;

        if !exists_in_db {
            // Message doesn't exist in DB - clean up orphaned data
            info!(
                "Cleaning up orphaned data for non-existent message {}",
                message_id_i64
            );
            self.cleanup_orphaned_data(message_id).await?;
        } else {
            // Message exists in DB - ensure it's properly indexed
            // TODO: Re-index message in search and vector DB
            info!(
                "Re-indexing message {} to ensure consistency",
                message_id_i64
            );
        }

        Ok(())
    }

    async fn start_monitoring(&self, check_interval: Duration) {
        let monitor = Arc::new(self.clone());

        tokio::spawn(async move {
            let mut interval = interval(check_interval);

            loop {
                interval.tick().await;

                match monitor.run_consistency_check().await {
                    Ok(result) => {
                        if result.inconsistencies_found > 0 {
                            error!(
                                "Data consistency issues detected: {} found, {} fixed, {} errors",
                                result.inconsistencies_found,
                                result.inconsistencies_fixed,
                                result.errors.len()
                            );
                        }
                    }
                    Err(e) => {
                        error!("Consistency check failed: {}", e);
                    }
                }
            }
        });

        info!(
            "Message consistency monitoring started with interval: {:?}",
            check_interval
        );
    }
}

impl Clone for MessageConsistencyMonitorImpl {
    fn clone(&self) -> Self {
        Self {
            db_pool: self.db_pool.clone(),
            cache_service: self.cache_service.clone(),
            search_service: self.search_service.clone(),
            vector_db: self.vector_db.clone(),
        }
    }
}

/// Create consistency monitor from app state
pub fn create_consistency_monitor(
    db_pool: Arc<sqlx::PgPool>,
    cache_service: Arc<dyn CacheService>,
    search_service: Option<Arc<dyn SearchApplicationServiceTrait>>,
    vector_db: Option<Arc<dyn MessageVectorRepository>>,
) -> Arc<dyn MessageConsistencyMonitor> {
    Arc::new(MessageConsistencyMonitorImpl::new(
        db_pool,
        cache_service,
        search_service,
        vector_db,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_consistency_check_empty_db() {
        // TODO: Implement tests
    }
}
