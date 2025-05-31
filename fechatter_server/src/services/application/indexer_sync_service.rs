use anyhow::Result;
use async_nats::jetstream;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;
use tokio::time::interval;
use tokio_stream::StreamExt;
use tracing::{error, info};

// 添加必要的导入
use crate::services::infrastructure::search::SearchService;
use crate::AppError;
use fechatter_core::models::Message;
use fechatter_core::models::SearchableMessage;

/// 索引同步工作器
///
/// 负责从 NATS 消息队列中消费消息索引事件，
/// 批量处理并同步到 Meilisearch 搜索引擎。
pub struct IndexerSyncWorker {
  nats_client: async_nats::Client,
  search_service: Arc<SearchService>,
  batch_size: usize,
  batch_timeout: Duration,
}

impl IndexerSyncWorker {
  pub fn new(nats_client: async_nats::Client, search_service: Arc<SearchService>) -> Self {
    Self {
      nats_client,
      search_service,
      batch_size: 50,                        // 批量处理 50 个消息
      batch_timeout: Duration::from_secs(5), // 5秒超时
    }
  }

  /// 从配置创建索引同步工作器
  pub fn from_config(
    nats_client: async_nats::Client,
    search_service: Arc<SearchService>,
    config: &crate::config::AsyncIndexingConfig,
  ) -> Self {
    Self {
      nats_client,
      search_service,
      batch_size: config.batch_size,
      batch_timeout: Duration::from_millis(config.batch_timeout_ms),
    }
  }

  /// 启动索引工作器
  pub async fn start(&self) -> Result<()> {
    let jetstream = jetstream::new(self.nats_client.clone());
    let stream = self.ensure_index_stream(&jetstream).await?;

    let consumer = self.ensure_consumer(&stream).await?;
    let mut messages = consumer.messages().await?;
    let mut batch: Vec<IndexTask> = Vec::with_capacity(self.batch_size);
    let mut batch_timer = interval(self.batch_timeout);

    info!(
      "🚀 Indexer sync worker started | batch_size: {} | timeout: {:?}",
      self.batch_size, self.batch_timeout
    );

    loop {
      tokio::select! {
          // 接收新消息
          message_result = messages.next() => {
              if let Some(Ok(msg)) = message_result {
                  if let Ok(task) = self.parse_index_task(&msg).await {
                      batch.push(task);

                      // 批次满了就处理
                      if batch.len() >= self.batch_size {
                          self.process_batch(&mut batch).await;
                      }
                  }
              }
          }

          // 定时处理未满的批次
          _ = batch_timer.tick() => {
              if !batch.is_empty() {
                  self.process_batch(&mut batch).await;
              }
          }
      }
    }
  }

  /// 启动后台工作器
  pub fn spawn(self) -> tokio::task::JoinHandle<Result<()>> {
    tokio::spawn(async move { self.start().await })
  }

  async fn ensure_index_stream(
    &self,
    js: &jetstream::Context,
  ) -> Result<jetstream::stream::Stream> {
    let stream_config = jetstream::stream::Config {
      name: "fechatter_search_index".to_string(),
      subjects: vec!["fechatter.search.index.*".to_string()],
      storage: jetstream::stream::StorageType::File,
      max_bytes: 100 * 1024 * 1024,            // 100MB
      max_age: Duration::from_secs(24 * 3600), // 24小时
      ..Default::default()
    };

    match js.get_stream("fechatter_search_index").await {
      Ok(stream) => Ok(stream),
      Err(_) => js.create_stream(stream_config).await.map_err(Into::into),
    }
  }

  async fn parse_index_task(&self, msg: &jetstream::message::Message) -> Result<IndexTask> {
    let payload: MessageIndexEvent = serde_json::from_slice(&msg.payload)?;
    Ok(IndexTask {
      message: payload.message,
      chat_info: payload.chat_info,
      nats_message: msg.clone(),
    })
  }

  /// 批量处理索引任务
  async fn process_batch(&self, batch: &mut Vec<IndexTask>) {
    if batch.is_empty() {
      return;
    }

    let batch_size = batch.len();
    let start_time = std::time::Instant::now();

    // 准备批量索引数据
    let searchable_messages: Vec<SearchableMessage> = batch
      .iter()
      .map(|task| self.convert_to_searchable_message(task))
      .collect();

    // 批量索引到 Meilisearch
    if let Err(e) = self
      .search_service
      .index_messages(&searchable_messages)
      .await
    {
      error!("Failed to index {} messages: {}", batch_size, e);
    } else {
      info!(
        "Indexed {} messages in {:?}",
        batch_size,
        start_time.elapsed()
      );
    }

    // 确认所有消息
    for task in batch.iter() {
      let _ = task.nats_message.ack().await;
    }

    batch.clear();
  }

  fn convert_to_searchable_message(&self, task: &IndexTask) -> SearchableMessage {
    SearchableMessage {
      id: task.message.id,
      chat_id: task.message.chat_id,
      sender_id: task.message.sender_id,
      sender_name: task.chat_info.sender_name.clone(),
      content: task.message.content.clone(),
      files: task.message.files.clone(),
      created_at: task.message.created_at,
      relevance_score: None,
    }
  }

  async fn ensure_consumer(
    &self,
    stream: &jetstream::stream::Stream,
  ) -> Result<jetstream::consumer::Consumer<jetstream::consumer::pull::Config>> {
    let consumer_config = jetstream::consumer::pull::Config {
      name: Some("search_indexer".to_string()),
      ack_policy: jetstream::consumer::AckPolicy::Explicit,
      max_deliver: 3,
      ack_wait: Duration::from_secs(30),
      max_batch: self.batch_size as i64,
      ..Default::default()
    };

    match stream.get_consumer("search_indexer").await {
      Ok(consumer) => Ok(consumer),
      Err(_) => stream
        .create_consumer(consumer_config)
        .await
        .map_err(Into::into),
    }
  }
}

/// 索引任务
struct IndexTask {
  message: Message,
  chat_info: ChatInfo,
  nats_message: jetstream::message::Message,
}

/// 消息索引事件
#[derive(Serialize, Deserialize, Debug)]
pub struct MessageIndexEvent {
  pub message: Message,
  pub chat_info: ChatInfo,
}

/// 聊天信息
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ChatInfo {
  pub chat_name: String,
  pub sender_name: String,
  pub chat_type: String,
  pub workspace_id: i64,
}

// 类型别名，用于保持应用层服务命名一致性
pub type IndexerSyncService = IndexerSyncWorker;
