use axum::{
  extract::Query,
  http::Request,
  response::{Sse, sse::Event},
};

use futures::Stream;
use serde::Deserialize;
use std::{
  convert::Infallible,
  pin::Pin,
  task::{Context, Poll},
};
use tokio::sync::mpsc::{self, Sender};
use tokio_stream::wrappers::ReceiverStream;

use crate::app_state::{NotifyState, TokenVerifier};
use fechatter_core::AuthUser;

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
  state: NotifyState,
  req: Request<axum::body::Body>,
) -> Result<Sse<EventStream>, (axum::http::StatusCode, String)> {
  let token = if let Some(auth) = req.headers().get("Authorization") {
    let auth_str = auth.to_str().map_err(|_| {
      (
        axum::http::StatusCode::UNAUTHORIZED,
        "Invalid Authorization header".to_string(),
      )
    })?;

    if auth_str.starts_with("Bearer ") {
      auth_str[7..].to_string()
    } else {
      return Err((
        axum::http::StatusCode::UNAUTHORIZED,
        "Invalid Authorization format".to_string(),
      ));
    }
  } else {
    let query = Query::<EventQuery>::try_from_uri(req.uri()).map_err(|_| {
      (
        axum::http::StatusCode::BAD_REQUEST,
        "Invalid query parameters".to_string(),
      )
    })?;

    query.0.token.ok_or_else(|| {
      (
        axum::http::StatusCode::UNAUTHORIZED,
        "Missing authentication token".to_string(),
      )
    })?
  };

  let user_claims = state.verify_token(&token).map_err(|e| {
    (
      axum::http::StatusCode::UNAUTHORIZED,
      format!("Invalid token: {:?}", e),
    )
  })?;

  let user: AuthUser = user_claims.into();

  let (tx, rx) = mpsc::channel(100);
  let rx_stream = ReceiverStream::new(rx);

  let subscription_id = uuid::Uuid::now_v7().to_string();
  state.add_subscriber(user.id, subscription_id.clone());

  let state_clone = state.clone();
  let user_id = user.id;
  tokio::spawn(async move {
    state_clone.remove_subscriber(user_id, &subscription_id);
  });

  let stream = EventStream {
    _tx: tx,
    rx: Box::pin(rx_stream),
  };

  Ok(Sse::new(stream))
}
