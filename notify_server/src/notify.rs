use std::{collections::HashSet, sync::Arc};

use crate::AppState;
use anyhow::Result;
use chrono::{DateTime, Utc};
use fechatter_core::{Chat, Message};
use futures::StreamExt;
use serde::{Deserialize, Serialize};
use sqlx::postgres::PgListener;
use tracing::{info, warn};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ChatMemberEventData {
  pub chat_id: i64,
  pub user_id: i64,
  pub joined_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "event")]
pub enum NotifyEvent {
  NewChat(Chat),
  UserJoinedChat(ChatMemberEventData),
  UserLeftChat(ChatMemberEventData),
  NewMessage(Message),
  DuplicateMessageAttempted(DuplicateMessagePayload),
}

#[derive(Debug)]
struct Notification {
  pub user_ids: HashSet<i64>,
  pub event: Arc<NotifyEvent>,
}

// pg_notify('chat_updated', json_build_object('op', TG_OP, 'old', OLD, 'new', NEW)::text);
#[derive(Debug, Serialize, Deserialize)]
struct ChatUpdated {
  op: String,
  old: Option<ChatMemberEventData>,
  new: Option<ChatMemberEventData>,
}

// pg_notify('message_created', row_to_json(NEW)::text);
#[derive(Debug, Serialize, Deserialize)]
struct MessageCreated {
  message: MessagePayload,
  chat_members: Vec<i64>,
}

// Custom struct to match the exact format from PostgreSQL
#[derive(Debug, Serialize, Deserialize)]
struct MessagePayload {
  id: i64,
  chat_id: i64,
  sender_id: i64,
  content: String,
  files: Vec<String>,
  created_at: String, // String to handle non-standard timestamp format
  idempotency_key: String,
}

impl From<MessagePayload> for Message {
  fn from(payload: MessagePayload) -> Self {
    Message {
      id: payload.id,
      chat_id: payload.chat_id,
      sender_id: payload.sender_id,
      content: payload.content,
      files: Some(payload.files),
      created_at: chrono::DateTime::parse_from_str(
        &format!("{}Z", payload.created_at),
        "%Y-%m-%dT%H:%M:%S.%fZ",
      )
      .unwrap_or_else(|_| chrono::Utc::now().into())
      .with_timezone(&chrono::Utc),
      idempotency_key: Some(
        uuid::Uuid::parse_str(&payload.idempotency_key).unwrap_or_else(|_| uuid::Uuid::nil()),
      ),
    }
  }
}

// Payload for duplicate_message_attempted event
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DuplicateMessagePayload {
  idempotency_key: Uuid,
  chat_id: i64,
  sender_id: i64,
}

pub async fn set_up_pg_listener(state: AppState) -> Result<()> {
  let mut listener = PgListener::connect(&state.config.server.db_url).await?;

  listener.listen("chat_updated").await?;
  listener.listen("message_created").await?;
  listener.listen("duplicate_message_attempted").await?;

  let mut stream = listener.into_stream();

  tokio::spawn(async move {
    while let Some(Ok(notification)) = stream.next().await {
      info!("event: {:?}", notification);
      match Notification::load(&notification.channel(), notification.payload()) {
        Ok(loaded_notification) => {
          let users = &state.users;
          if loaded_notification.user_ids.is_empty() {
            info!(
              "No users to notify for event: {:?}",
              loaded_notification.event
            );
          } else {
            for user_id in loaded_notification.user_ids {
              if let Some(tx) = users.get(&user_id) {
                info!(
                  "sending event {:?} to user: {}",
                  loaded_notification.event, user_id
                );
                if let Err(e) = tx.send(loaded_notification.event.clone()) {
                  warn!("Failed to send notification to user {}: {}", user_id, e);
                }
              }
            }
          }
        }
        Err(e) => {
          warn!("Failed to load notification: {}", e);
        }
      }
    }

    Ok::<_, anyhow::Error>(())
  });

  Ok(())
}

impl Notification {
  pub fn load(r#type: &str, payload: &str) -> Result<Self> {
    match r#type {
      "chat_updated" => {
        let payload_data: ChatUpdated = serde_json::from_str(payload)?;
        info!("chat_updated: {:?}", payload_data);

        let mut user_ids = HashSet::new();
        let event = match payload_data.op.as_str() {
          "INSERT" => {
            let new_member_data = payload_data.new.ok_or_else(|| {
              anyhow::anyhow!("Missing new member data for INSERT on chat_members")
            })?;
            user_ids.insert(new_member_data.user_id);
            NotifyEvent::UserJoinedChat(new_member_data)
          }
          "DELETE" => {
            let old_member_data = payload_data.old.ok_or_else(|| {
              anyhow::anyhow!("Missing old member data for DELETE on chat_members")
            })?;
            user_ids.insert(old_member_data.user_id);
            NotifyEvent::UserLeftChat(old_member_data)
          }
          "UPDATE" => {
            warn!(
              "Unhandled UPDATE operation for chat_updated on chat_members: {:?}",
              payload_data
            );
            return Err(anyhow::anyhow!(
              "Unhandled UPDATE operation for chat_members in chat_updated"
            ));
          }
          _ => return Err(anyhow::anyhow!("Invalid operation")),
        };
        Ok(Self {
          user_ids,
          event: Arc::new(event),
        })
      }
      "message_created" => {
        let payload_data: MessageCreated = serde_json::from_str(payload)?;
        let user_ids = payload_data.chat_members.iter().cloned().collect();
        Ok(Self {
          user_ids,
          event: Arc::new(NotifyEvent::NewMessage(payload_data.message.into())),
        })
      }
      "duplicate_message_attempted" => {
        let payload_data: DuplicateMessagePayload = serde_json::from_str(payload)?;
        info!("duplicate_message_attempted: {:?}", payload_data);
        let mut user_ids = HashSet::new();
        user_ids.insert(payload_data.sender_id);
        Ok(Self {
          user_ids,
          event: Arc::new(NotifyEvent::DuplicateMessageAttempted(payload_data)),
        })
      }
      _ => Err(anyhow::anyhow!("Invalid notification type: {}", r#type)),
    }
  }
}

fn get_affected_chat_user_ids(old: Option<&Chat>, new: Option<&Chat>) -> HashSet<i64> {
  match (old, new) {
    (Some(old), Some(new)) => {
      let old_user_ids: HashSet<_> = old.chat_members.iter().cloned().collect();
      let new_user_ids: HashSet<_> = new.chat_members.iter().cloned().collect();
      if old_user_ids == new_user_ids {
        HashSet::new()
      } else {
        old_user_ids.union(&new_user_ids).copied().collect()
      }
    }
    (Some(old), None) => old.chat_members.iter().cloned().collect(),
    (None, Some(new)) => new.chat_members.iter().cloned().collect(),
    (None, None) => HashSet::new(),
  }
}
