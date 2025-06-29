use axum::{
  Extension,
  extract::State,
  response::{Sse, sse::Event},
};

use axum_extra::{TypedHeader, headers};
use futures::{Stream, StreamExt};
use std::{
  convert::Infallible,
  pin::Pin,
  task::{Context, Poll},
  time::{Duration, Instant},
};
use tokio::sync::{broadcast, mpsc::Sender};
use tokio_stream::wrappers::BroadcastStream;
use tracing::{debug, info, warn};
use serde_json::json;
use chrono::Utc;
use std::sync::Arc;

use crate::{events::types::NotifyEvent, state::AppState};
use fechatter_core::{AuthUser, UserId};

const CHANNEL_CAPACITY: usize = 256;

pub struct EventStream {
  _tx: Sender<Result<Event, Infallible>>,
  rx: Pin<Box<dyn Stream<Item = Result<Event, Infallible>> + Send>>,
}

impl Stream for EventStream {
  type Item = Result<Event, Infallible>;

  fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
    self.rx.as_mut().poll_next(cx)
  }
}

/// SSE connection handler - handles user's Server-Sent Events connection
#[allow(dead_code)]
pub async fn sse_handler(
  State(state): State<AppState>,
  Extension(user): Extension<AuthUser>,
  user_agent: Option<TypedHeader<headers::UserAgent>>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
  let user_agent_str = user_agent
    .map(|TypedHeader(ua)| ua.as_str().to_string())
    .unwrap_or_else(|| "Unknown".to_string());

  info!(
    "👤 User {} (`{}`) connected to SSE",
    user.id, user_agent_str
  );

  let user_id = UserId(user.id.into());
  let connection_id = uuid::Uuid::new_v4().to_string();
  let connection_start = Instant::now();

  // 1. Create the user's SSE connection
  let (tx, rx) = broadcast::channel(CHANNEL_CAPACITY);
  state.user_connections.insert(user_id, tx.clone());

  // 2. Register the user to all their chats (critical fix)
  let chat_count = if let Err(e) = state.register_user_to_chats(user_id).await {
    warn!("❌ Failed to register user {} to chats: {}", user_id.0, e);
    0
  } else {
    // Get the number of chats the user is registered to
    state.get_user_chat_count(user_id).await.unwrap_or(0)
  };

  // 🔧 CRITICAL FIX 1: Send immediate SSE connection confirmation event
  let welcome_notification = json!({
    "type": "connection_confirmed",
    "user_id": user_id.0,
    "connection_id": connection_id,
    "connected_chats": chat_count,
    "timestamp": Utc::now(),
    "server_time": Utc::now().timestamp(),
    "message": "SSE connection established successfully"
  });

  // Send welcome event immediately
  if let Err(e) = tx.send(Arc::new(NotifyEvent::Generic(welcome_notification))) {
    warn!("Failed to send welcome notification to user {}: {}", user_id.0, e);
  } else {
    info!("✅ [SSE] Sent connection confirmation to user {}", user_id.0);
  }

  // 3. Send analytics event for user connection
  state.analytics.user_connected(
    user_id,
    connection_id.clone(),
    Some(user_agent_str.clone()),
  );

  // 🔧 CRITICAL FIX 2: Start heartbeat mechanism for this user
  let heartbeat_tx = tx.clone();
  let heartbeat_user_id = user_id;
  tokio::spawn(async move {
    let mut interval = tokio::time::interval(Duration::from_secs(30));
    let mut heartbeat_count = 1;
    
    loop {
      interval.tick().await;
      
      let heartbeat_event = json!({
        "type": "heartbeat",
        "user_id": heartbeat_user_id.0,
        "heartbeat_id": heartbeat_count,
        "timestamp": Utc::now(),
        "server_time": Utc::now().timestamp()
      });
      
      if heartbeat_tx.send(Arc::new(NotifyEvent::Generic(heartbeat_event))).is_err() {
        info!("💓 [SSE] Heartbeat stopped for user {} (connection closed)", heartbeat_user_id.0);
        break;
      }
      
      debug!("💓 [SSE] Sent heartbeat #{} to user {}", heartbeat_count, heartbeat_user_id.0);
      heartbeat_count += 1;
    }
  });

  info!(
    "✅ User {} successfully connected to SSE and registered to {} chats",
    user_id.0, chat_count
  );

  // 4. Create the SSE stream, including cleanup logic on disconnect
  let state_for_cleanup = state.clone();
  let cleanup_connection_id = connection_id.clone();
  let stream = BroadcastStream::new(rx)
    .filter_map(|result| async move { result.ok() })
    .map(move |v| {
      let event_type = match v.as_ref() {
        NotifyEvent::NewChat(_) => "NewChat",
        NotifyEvent::UserJoinedChat(_) => "UserJoinedChat",
        NotifyEvent::UserLeftChat(_) => "UserLeftChat",
        NotifyEvent::NewMessage(_) => "NewMessage",
        NotifyEvent::DuplicateMessageAttempted(_) => "DuplicateMessageAttempted",
        NotifyEvent::MessageRead(_) => "MessageRead",
        NotifyEvent::MessageUnread(_) => "MessageUnread",
        NotifyEvent::TypingStatus(_) => "TypingStatus",
        NotifyEvent::UserPresence(_) => "UserPresence",
        NotifyEvent::Generic(_) => "Generic",
      };

      // Track analytics for notification delivery
      let notification_start = Instant::now();
      let state_analytics = state.clone();
      let analytics_user_id = user_id;
      let analytics_event_type = event_type.to_string();
      
      tokio::spawn(async move {
        let delivery_duration = notification_start.elapsed().as_millis() as u64;
        state_analytics.analytics.notification_received(
          analytics_user_id,
          analytics_event_type,
          "sse".to_string(),
          true,
          Some(delivery_duration),
        );
      });

      let v = serde_json::to_string(&v).expect("Failed to serialize event");
      debug!(
        "📤 [SSE] Sending event {} to user {}: {}",
        event_type, user_id.0, 
        if v.len() > 100 { format!("{}...", &v[..100]) } else { v.clone() }
      );
      Ok(Event::default().data(v).event(event_type))
    })
    .inspect(move |_| {
      // Cleanup user connection when the stream ends (triggered when user disconnects SSE)
      let state_cleanup = state_for_cleanup.clone();
      let cleanup_user_id = user_id;
      let final_connection_id = cleanup_connection_id.clone();
      let final_connection_start = connection_start;
      
      tokio::spawn(async move {
        // Calculate connection duration
        let connection_duration = final_connection_start.elapsed().as_millis() as u64;
        
        info!("🔌 [SSE] User {} disconnected after {}ms", cleanup_user_id.0, connection_duration);
        
        // Send analytics event for user disconnection
        state_cleanup.analytics.user_disconnected(
          cleanup_user_id,
          final_connection_id,
          connection_duration,
        );
        
        // Clean up user from chats
        state_cleanup
          .unregister_user_from_chats(cleanup_user_id)
          .await;
      });
    });

  // Enhanced keep-alive with more frequent pings
  Sse::new(stream).keep_alive(
    axum::response::sse::KeepAlive::new()
      .interval(Duration::from_secs(25))
      .text("ping"),
  )
}
