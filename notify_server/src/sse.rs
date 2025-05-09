use axum::{
  Extension,
  extract::State,
  response::{Sse, sse::Event},
};

use axum_extra::{TypedHeader, headers};
use futures::Stream;
use serde::Deserialize;
use std::{
  convert::Infallible,
  pin::Pin,
  task::{Context, Poll},
  time::Duration,
};
use tokio::sync::{broadcast, mpsc::Sender};
use tokio_stream::{StreamExt, wrappers::BroadcastStream};
use tracing::{debug, info};

use crate::{AppState, NotifyEvent};
use fechatter_core::AuthUser;

const CHANNEL_CAPACITY: usize = 256;

#[derive(Debug, Deserialize)]
pub struct EventQuery {
  token: Option<String>,
}

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

pub async fn sse_handler(
  State(state): State<AppState>,
  Extension(user): Extension<AuthUser>,
  TypedHeader(user_agent): TypedHeader<headers::UserAgent>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
  info!("`{}` connected", user_agent.as_str());

  let user_id = user.id;
  let users = state.users.clone();

  let rx = if let Some(tx) = users.get(&user_id) {
    tx.subscribe()
  } else {
    let (tx, rx) = broadcast::channel(CHANNEL_CAPACITY);
    state.users.insert(user_id, tx);
    rx
  };
  info!("User {} subscribed to channel", user_id);

  let stream = BroadcastStream::new(rx)
    .filter_map(|result| result.ok())
    .map(|v| {
      let event_type = match v.as_ref() {
        NotifyEvent::NewChat(_) => "NewChat",
        NotifyEvent::UserJoinedChat(_) => "UserJoinedChat",
        NotifyEvent::UserLeftChat(_) => "UserLeftChat",
        NotifyEvent::NewMessage(_) => "NewMessage",
        NotifyEvent::DuplicateMessageAttempted(_) => "DuplicateMessageAttempted",
      };
      let v = serde_json::to_string(&v).expect("Failed to serialize event");
      debug!("Sending event {}: {:?}", event_type, v);
      Ok(Event::default().data(v).event(event_type))
    });

  Sse::new(stream).keep_alive(
    axum::response::sse::KeepAlive::new()
      .interval(Duration::from_secs(1))
      .text("keep-alive-text"),
  )
}
