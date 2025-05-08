use std::{collections::HashSet, sync::Arc};

use crate::AppState;
use anyhow::Result;
use fechatter_core::{Chat, Message};
use futures::StreamExt;
use serde::{Deserialize, Serialize};
use sqlx::postgres::PgListener;
use tracing::{info, warn};

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "event")]
pub enum NotifyEvent {
  NewChat(Chat),
  AddToChat(Chat),
  RemoveFromChat(Chat),
  NewMessage(Message),
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
  old: Option<Chat>,
  new: Option<Chat>,
}

// pg_notify('message_created', row_to_json(NEW)::text);
#[derive(Debug, Serialize, Deserialize)]
struct MessageCreated {
  message: Message,
  members: Vec<i64>,
}

pub async fn set_up_pg_listener(state: AppState) -> Result<()> {
  let mut listener = PgListener::connect(&state.config.server.db_url).await?;

  listener.listen("chat_updated").await?;
  listener.listen("message_created").await?;

  let mut stream = listener.into_stream();

  tokio::spawn(async move {
    while let Some(Ok(notification)) = stream.next().await {
      info!("event: {:?}", notification);
      let notification: Notification =
        Notification::load(&notification.channel(), notification.payload())?;
      let users = &state.users;
      for user_id in notification.user_ids {
        if let Some(tx) = users.get(&user_id) {
          info!("sending to user: {}", user_id);
          if let Err(e) = tx.send(notification.event.clone()) {
            warn!("Failed to send notification to user {}: {}", user_id, e);
          }
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
        let payload: ChatUpdated = serde_json::from_str(payload)?;
        info!("chat_updated: {:?}", payload);

        let user_ids = get_affected_chat_user_ids(payload.old.as_ref(), payload.new.as_ref());
        let event = match payload.op.as_str() {
          "INSERT" => NotifyEvent::NewChat(payload.new.expect("new should exist")),
          "UPDATE" => NotifyEvent::AddToChat(payload.new.expect("new should exist")),
          "DELETE" => NotifyEvent::RemoveFromChat(payload.old.expect("old should exist")),
          _ => return Err(anyhow::anyhow!("Invalid operation")),
        };
        Ok(Self {
          user_ids,
          event: Arc::new(event),
        })
      }
      "message_created" => {
        let payload: MessageCreated = serde_json::from_str(payload)?;
        let user_ids = payload.members.iter().map(|v| *v as i64).collect();
        Ok(Self {
          user_ids,
          event: Arc::new(NotifyEvent::NewMessage(payload.message)),
        })
      }
      _ => Err(anyhow::anyhow!("Invalid notification type")),
    }
  }
}

fn get_affected_chat_user_ids(old: Option<&Chat>, new: Option<&Chat>) -> HashSet<i64> {
  match (old, new) {
    (Some(old), Some(new)) => {
      // diff old/new members, if identical, no need to noify, otherwise notify the union of both
      let old_user_ids: HashSet<_> = old.chat_members.iter().map(|v| *v as i64).collect();
      let new_user_ids: HashSet<_> = new.chat_members.iter().map(|v| *v as i64).collect();
      if old_user_ids == new_user_ids {
        HashSet::new()
      } else {
        old_user_ids.union(&new_user_ids).copied().collect()
      }
    }
    (Some(old), None) => old.chat_members.iter().map(|v| *v as i64).collect(),
    (None, Some(new)) => new.chat_members.iter().map(|v| *v as i64).collect(),
    (None, None) => HashSet::new(),
  }
}
