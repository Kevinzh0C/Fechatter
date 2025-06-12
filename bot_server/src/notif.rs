use std::collections::HashSet;

use crate::{UnifiedBotAnalyticsPublisher, AppConfig};
use fechatter_core::{Message, UserId};
use futures::StreamExt;
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, postgres::PgPoolOptions};
use std::sync::Arc;
use swiftide::{
  integrations,
  query::{self, answers, query_transformers, response_transformers},
  traits::{EmbeddingModel, SimplePrompt},
};
use swiftide_pgvector::PgVectorBuilder;
use tracing::{debug, error, info, warn};

#[allow(dead_code)]
#[derive(Debug)]
struct BotNotification {
  bot_id: UserId,
  event: Message,
}

/// NATS event payload structure (from fechatter_server)
#[derive(Debug, Serialize, Deserialize)]
struct MessageCreatedEvent {
  pub msg: Message,
  pub members: HashSet<UserId>,
}

/// Setup NATS subscriber for bot event processing
pub async fn setup_nats_subscriber(config: &AppConfig) -> anyhow::Result<()> {
  if !config.messaging.enabled {
    warn!("⚠️ NATS messaging is disabled - bot_server will not process events");
    return Ok(());
  }

  info!("🚀 Setting up NATS subscriber for bot event processing...");

  // Connect to NATS
  let nats_client = async_nats::ConnectOptions::new()
    .connection_timeout(std::time::Duration::from_secs(10))
    .ping_interval(std::time::Duration::from_secs(60))
    .max_reconnects(Some(5))
    .reconnect_delay_callback(|attempts: usize| {
      std::time::Duration::from_secs(std::cmp::min(2u64.pow(attempts as u32), 10))
    })
    .connect(&config.messaging.nats.url)
    .await?;

  info!("✅ Connected to NATS: {}", config.messaging.nats.url);

  // Setup unified NATS + Protobuf analytics publisher
  let analytics_publisher = if config.analytics.enabled {
    Some(Arc::new(UnifiedBotAnalyticsPublisher::new(nats_client.clone())))
  } else {
    info!("📊 Analytics disabled in configuration");
    None
  };

  // Connect to database
  let pool = PgPoolOptions::new().connect(&config.server.db_url).await?;

  // Get bot user IDs from database
  let bots = get_bots(&pool).await?;
  info!("🤖 Found {} bots in database", bots.len());

  // Setup AI client
  let ai_client = integrations::openai::OpenAI::builder()
    .default_embed_model(&config.bot.openai.embed_model)
    .default_prompt_model(&config.bot.openai.model)
    .build()?;

  info!(
    "🧠 OpenAI client initialized with model: {}",
    config.bot.openai.model
  );

  // Subscribe to configured subjects
  for subject in &config.messaging.nats.subscription_subjects {
    let subscriber = nats_client.subscribe(subject.clone()).await?;
    let subject_str = subject.clone();
    let pool_clone = pool.clone();
    let bots_clone = bots.clone();
    let ai_client_clone = ai_client.clone();
    let config_clone = config.clone();
    let analytics_clone = analytics_publisher.clone();

    // Spawn a handler for each subscription
    tokio::spawn(async move {
      info!("📡 Bot NATS subscriber started: {}", subject_str);
      let mut subscriber = subscriber;

      while let Some(msg) = subscriber.next().await {
        let subject = msg.subject.as_str();
        let payload = msg.payload.as_ref();
        let payload_size = payload.len();

        // Upgrade to INFO level with detailed logging
        info!("📨 [BOT] Received NATS event from subject: {} (size: {} bytes)", subject, payload_size);

        // Process the event
        if let Err(e) = process_nats_event(
          &pool_clone,
          &bots_clone,
          &ai_client_clone,
          &config_clone,
          analytics_clone.as_ref(),
          subject,
          payload,
        )
        .await
        {
          error!("❌ [BOT] Failed to process event from {}: {}", subject, e);
          
          // Track error in analytics using unified publisher
          if let Some(analytics) = &analytics_clone {
            let _ = analytics
              .track_bot_error(
                "unknown_bot".to_string(),
                "unknown_chat".to_string(),
                "NATS_EVENT_PROCESSING".to_string(),
                format!("Failed to process NATS event {}: {}", subject, e),
              )
              .await;
          }
        } else {
          info!("✅ [BOT] Successfully processed event from: {}", subject);
        }
      }

      warn!("⚠️ Bot NATS subscriber ended: {}", subject_str);
    });
  }

  info!("✅ Bot NATS event processor setup complete");
  Ok(())
}

/// Process NATS events for bot functionality
pub async fn process_nats_event(
  pool: &PgPool,
  bots: &HashSet<UserId>,
  ai_client: &integrations::openai::OpenAI,
  config: &AppConfig,
  analytics_publisher: Option<&Arc<UnifiedBotAnalyticsPublisher>>,
  subject: &str,
  payload: &[u8],
) -> anyhow::Result<()> {
  info!("🤖 [BOT] Processing event from subject: {}", subject);

  // Parse message created event
  if subject.contains("message.created") || subject.contains("messages.created") {
    info!("💬 [BOT] Parsing message created event from: {}", subject);
    
    let event = match serde_json::from_slice::<MessageCreatedEvent>(payload) {
      Ok(event) => {
        info!("✅ [BOT] Successfully parsed message event: chat_id={}, members_count={}", 
              event.msg.chat_id.0, event.members.len());
        event
      }
      Err(e) => {
        error!("❌ [BOT] Failed to parse message event from {}: {}", subject, e);
        return Err(e.into());
      }
    };

    // Check if any bots should respond to this message
    let responding_bots: Vec<UserId> = bots
      .iter()
      .filter(|&bot_id| event.members.contains(bot_id))
      .copied()
      .collect();

    if responding_bots.is_empty() {
      debug!("🤖 [BOT] No bots are members of chat {}, skipping", event.msg.chat_id.0);
      return Ok(());
    }

    info!("🤖 [BOT] Found {} responding bots for chat {}: {:?}", 
          responding_bots.len(), 
          event.msg.chat_id.0,
          responding_bots.iter().map(|b| b.0).collect::<Vec<_>>());

    // Process each bot response
    for bot_id in responding_bots {
      let bot_notification = BotNotification {
        bot_id,
        event: event.msg.clone(),
      };

      info!("🤖 [BOT] Bot {} processing message in chat {}", 
            bot_id.0, event.msg.chat_id.0);

      if let Err(e) = bot_notification.process(
        pool,
        ai_client.clone(),
        ai_client.clone(),
        config,
        analytics_publisher,
      )
      .await
      {
        error!("❌ [BOT] Bot {} failed to process notification: {}", bot_id.0, e);
        
        // Track bot error in analytics
        if let Some(analytics) = analytics_publisher {
          let _ = analytics
            .track_bot_error(
              bot_id.0.to_string(),
              event.msg.chat_id.0.to_string(),
              "MESSAGE_PROCESSING".to_string(),
              format!("Bot processing failed: {}", e),
            )
            .await;
        }
      } else {
        info!("✅ [BOT] Bot {} successfully processed message in chat {}", 
              bot_id.0, event.msg.chat_id.0);
        
        // Track successful bot response in analytics
        if let Some(analytics) = analytics_publisher {
          let _ = analytics
            .track_bot_response(
              bot_id.0.to_string(),
              event.msg.chat_id.0.to_string(),
              "MESSAGE_RESPONSE".to_string(),
              0, // response_time_ms: u64
              0, // tokens_used: u32 
              true, // success: bool
              None, // error_message: Option<String>
            )
            .await;
        }
      }
    }
    
    info!("🎯 [BOT] Completed processing message event from: {}", subject);
  } else {
    debug!("🤖 [BOT] Unhandled subject pattern: {}", subject);
  }

  Ok(())
}

/// Handle message created events for bot processing
async fn process_message_created_event(
  pool: &PgPool,
  bots: &HashSet<UserId>,
  ai_client: &integrations::openai::OpenAI,
  config: &AppConfig,
  analytics_publisher: Option<&Arc<UnifiedBotAnalyticsPublisher>>,
  payload: &[u8],
) -> anyhow::Result<()> {
  // Parse the event payload
  let event: MessageCreatedEvent = serde_json::from_slice(payload)?;
  let message = event.msg;
  let mut members = event.members;

  info!("💬 Bot processing message in chat {}", message.chat_id);

  // Remove the sender from members
  members.remove(&message.sender_id);

  // Only process if it's a direct message with exactly one other participant
  if members.len() == 1 {
    let other_user = members.iter().next().unwrap();

    // Check if the other participant is a bot
    if bots.contains(other_user) {
      let notification = BotNotification {
        bot_id: *other_user,
        event: message,
      };

      info!(
        "🤖 Bot {} will respond to message: {}",
        notification.bot_id.0, notification.event.id
      );

      // Process the notification with AI
      notification
        .process(pool, ai_client.clone(), ai_client.clone(), config, analytics_publisher)
        .await?;
    } else {
      debug!("👤 Other participant is not a bot, skipping");
    }
  } else {
    debug!(
      "👥 Not a direct message (has {} participants), bots only respond to direct messages",
      members.len()
    );
  }

  Ok(())
}

impl BotNotification {
  /// Process bot notification with AI response generation
  async fn process(
    self,
    pool: &PgPool,
    client: impl SimplePrompt + Clone + 'static,
    embed: impl EmbeddingModel + Clone + 'static,
    config: &AppConfig,
    analytics_publisher: Option<&Arc<UnifiedBotAnalyticsPublisher>>,
  ) -> anyhow::Result<()> {
    info!(
      "🧠 Processing bot notification for message: {}",
      self.event.id
    );

    let start_time = std::time::Instant::now();

    // Add response delay to seem more natural
    if config.bot.response_delay_ms > 0 {
      tokio::time::sleep(std::time::Duration::from_millis(
        config.bot.response_delay_ms,
      ))
      .await;
    }

    // Setup vector store for RAG
    let store = PgVectorBuilder::default()
      .pool(pool.clone())
      .vector_size(config.bot.vector.size as i32)
      .build()?;

    // Create query pipeline with RAG
    let pipeline = query::Pipeline::default()
      .then_transform_query(query_transformers::GenerateSubquestions::from_client(
        client.clone(),
      ))
      .then_transform_query(query_transformers::Embed::from_client(embed.clone()))
      .then_retrieve(store)
      .then_transform_response(response_transformers::Summary::from_client(client.clone()))
      .then_answer(answers::Simple::from_client(client.clone()));

    info!("🔍 Querying AI with: {}", self.event.content);

    // Generate AI response
    let result = pipeline.query(&self.event.content).await?;
    let mut answer = result.answer().to_string();

    // Truncate response if too long
    if answer.len() > config.bot.max_response_length {
      answer.truncate(config.bot.max_response_length - 3);
      answer.push_str("...");
      info!(
        "✂️ Truncated bot response to {} characters",
        config.bot.max_response_length
      );
    }

    info!("💬 Bot response generated, saving to database...");

    // Insert bot response into database
    let message_id: (i64,) = sqlx::query_as(
      r#"
      INSERT INTO messages (chat_id, sender_id, content, created_at)
      VALUES ($1, $2, $3, NOW())
      RETURNING id
      "#,
    )
    .bind(self.event.chat_id)
    .bind(self.bot_id.0 as i64)
    .bind(&answer)
    .fetch_one(pool)
    .await?;

    info!(
      "✅ Bot {} responded to message {} with new message {}",
      self.bot_id.0, self.event.id, message_id.0
    );

    // Track analytics event using unified publisher
    if let Some(analytics_publisher) = analytics_publisher {
      let response_time = start_time.elapsed().as_millis() as u64;
      let tokens_used = (answer.len() / 4) as u32; // Rough estimate: 4 chars per token

      if let Err(e) = analytics_publisher
        .track_bot_response(
          self.bot_id.0.to_string(),
          self.event.chat_id.to_string(),
          "ai_chat".to_string(),
          response_time,
          tokens_used,
          true,
          None,
        )
        .await
      {
        warn!("Failed to track bot response analytics: {}", e);
      }
    }

    Ok(())
  }
}

/// Get all bot user IDs from the database
pub async fn get_bots(pool: &PgPool) -> anyhow::Result<HashSet<UserId>> {
  let bots: Vec<(i64,)> = sqlx::query_as(r#"SELECT id FROM users WHERE is_bot = TRUE"#)
    .fetch_all(pool)
    .await?;

  let bot_set: HashSet<UserId> = bots.into_iter().map(|b| UserId::from(b.0)).collect();

  debug!("🔍 Retrieved {} bots from database", bot_set.len());

  Ok(bot_set)
}
