use axum::response::{Sse, sse::Event};
use axum_extra::{TypedHeader, headers};
use futures::{Stream, stream};
use std::{convert::Infallible, time::Duration};
use tokio_stream::StreamExt;
use tracing::info;

pub(crate) async fn sse_handler(
  TypedHeader(user_agent): TypedHeader<headers::UserAgent>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
  info!("`{}` connected ", user_agent.as_str());

  let stream = stream::repeat_with(|| Event::default().data("hi"))
    .map(Ok)
    .throttle(Duration::from_secs(1));

  Sse::new(stream).keep_alive(
    axum::response::sse::KeepAlive::new()
      .interval(Duration::from_secs(1))
      .text("keep-alive-text"),
  )
}
