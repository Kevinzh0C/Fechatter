use analytics_server::pb::*;
use anyhow::Result;
use prost::Message;

#[tokio::main]
async fn main() -> Result<()> {
  // Create event context
  let mut context = EventContext {
    client_id: "fechatter_client_123".to_string(),
    session_id: "session_456".to_string(),
    user_id: "user_456".to_string(),
    app_version: "1.0.0".to_string(),
    client_ts: chrono::Utc::now().timestamp_millis(),
    server_ts: 0, // will be set by server
    user_agent: "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36".to_string(),
    ip: "".to_string(), // will be set by server
    system: Some(SystemInfo {
      os: "macos".to_string(),
      arch: "x64".to_string(),
      locale: "en-US".to_string(),
      timezone: "America/New_York".to_string(),
      browser: "Chrome".to_string(),
      browser_version: "120.0.0".to_string(),
    }),
    geo: None, // will be set by server
  };

  // Example 1: User login event
  let login_event = AnalyticsEvent {
    context: Some(context.clone()),
    event_type: Some(analytics_event::EventType::UserLogin(UserLoginEvent {
      email: "user@example.com".to_string(),
      login_method: "password".to_string(),
    })),
  };
  send_event(&login_event).await?;

  // Example 2: Message sent event
  let message_event = AnalyticsEvent {
    context: Some(context.clone()),
    event_type: Some(analytics_event::EventType::MessageSent(MessageSentEvent {
      chat_id: "chat_789".to_string(),
      r#type: "text".to_string(),
      size: 256,
      total_files: 0,
      has_mentions: true,
      has_links: false,
    })),
  };
  send_event(&message_event).await?;

  // Example 3: File uploaded event
  let file_event = AnalyticsEvent {
    context: Some(context.clone()),
    event_type: Some(analytics_event::EventType::FileUploaded(
      FileUploadedEvent {
        file_type: "image/png".to_string(),
        file_size: 1024 * 1024, // 1MB
        upload_method: "drag_drop".to_string(),
        upload_duration_ms: 1500,
      },
    )),
  };
  send_event(&file_event).await?;

  // Example 4: Search performed event
  let search_event = AnalyticsEvent {
    context: Some(context.clone()),
    event_type: Some(analytics_event::EventType::SearchPerformed(
      SearchPerformedEvent {
        search_type: "messages".to_string(),
        query_length: "10".to_string(),
        results_count: 25,
        search_duration_ms: 150,
        has_filters: true,
      },
    )),
  };
  send_event(&search_event).await?;

  // Example 5: Error occurred event
  let error_event = AnalyticsEvent {
    context: Some(context.clone()),
    event_type: Some(analytics_event::EventType::ErrorOccurred(
      ErrorOccurredEvent {
        error_type: "NetworkError".to_string(),
        error_code: "ERR_CONNECTION_REFUSED".to_string(),
        error_message: "Failed to connect to server".to_string(),
        stack_trace: "".to_string(),
        context: "Sending message in chat".to_string(),
      },
    )),
  };
  send_event(&error_event).await?;

  // Example 6: Navigation event
  let nav_event = AnalyticsEvent {
    context: Some(context.clone()),
    event_type: Some(analytics_event::EventType::Navigation(NavigationEvent {
      from: "/chats".to_string(),
      to: "/chats/123".to_string(),
      duration_ms: 50,
    })),
  };
  send_event(&nav_event).await?;

  Ok(())
}

async fn send_event(event: &AnalyticsEvent) -> Result<()> {
  let client = reqwest::Client::new();
  let data = Message::encode_to_vec(event);

  let res = client
    .post("http://127.0.0.1:6690/api/event")
    .header("content-type", "application/protobuf")
    .body(data)
    .send()
    .await?;

  println!(
    "Sent {:?} event, server returned: {:?}",
    event
      .event_type
      .as_ref()
      .map(|e| match e {
        analytics_event::EventType::UserLogin(_) => "UserLogin",
        analytics_event::EventType::MessageSent(_) => "MessageSent",
        analytics_event::EventType::FileUploaded(_) => "FileUploaded",
        analytics_event::EventType::SearchPerformed(_) => "SearchPerformed",
        analytics_event::EventType::ErrorOccurred(_) => "ErrorOccurred",
        analytics_event::EventType::Navigation(_) => "Navigation",
        _ => "Other",
      })
      .unwrap_or("Unknown"),
    res.status()
  );

  if !res.status().is_success() {
    let body = res.text().await?;
    println!("Error response: {}", body);
  }

  Ok(())
}
