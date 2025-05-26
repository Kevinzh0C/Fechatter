use super::test_fixtures::TestFixtures;
use anyhow::Result;
use chrono::Utc;
use fechatter_core::TokenService;
use fechatter_core::{Chat, ChatType, CreateChat, CreateMessage, CreateUser, Message, User};
use fechatter_server::AppState;
use sqlx_db_tester::TestPg;

/// 测试上下文，封装常用的测试操作
pub struct TestContext {
  pub app_state: AppState,
  pub users: Vec<User>,
  pub chats: Vec<Chat>,
  pub messages: Vec<Message>,
}

impl TestContext {
  /// 创建新的测试上下文
  pub async fn new() -> Result<Self> {
    let (_tdb, app_state) = AppState::test_new().await?;
    Ok(Self {
      app_state,
      users: Vec::new(),
      chats: Vec::new(),
      messages: Vec::new(),
    })
  }

  /// 创建单个测试用户
  pub async fn create_user(&mut self, prefix: &str) -> Result<&User> {
    let user_data = TestFixtures::create_user(prefix);
    let user = self.app_state.create_user(&user_data, None).await?;
    self.users.push(user);
    Ok(self.users.last().unwrap())
  }

  /// 批量创建测试用户
  pub async fn create_users(&mut self, prefix: &str, count: usize) -> Result<&[User]> {
    let start_idx = self.users.len();
    let users_data = TestFixtures::create_users(prefix, count);

    for user_data in users_data {
      let user = self.app_state.create_user(&user_data, None).await?;
      self.users.push(user);
    }

    Ok(&self.users[start_idx..])
  }

  /// 创建聊天
  pub async fn create_chat(
    &mut self,
    prefix: &str,
    creator_idx: usize,
    member_indices: Vec<usize>,
    chat_type: ChatType,
  ) -> Result<&Chat> {
    let creator = &self.users[creator_idx];
    let member_ids: Vec<i64> = member_indices
      .iter()
      .map(|&idx| self.users[idx].id.into())
      .collect();

    let chat = self
      .app_state
      .create_new_chat(
        creator.id.into(),
        &format!("{} {}", prefix, TestFixtures::unique_id()),
        chat_type,
        Some(member_ids),
        Some("Test chat description"),
        creator.workspace_id.into(),
      )
      .await?;

    self.chats.push(chat);
    Ok(self.chats.last().unwrap())
  }

  /// 创建消息
  pub async fn create_message(
    &mut self,
    content: &str,
    chat_idx: usize,
    sender_idx: usize,
  ) -> Result<Message> {
    let chat = &self.chats[chat_idx];
    let sender = &self.users[sender_idx];

    let message_data = TestFixtures::create_message(content);
    let message = self
      .app_state
      .create_message(message_data, chat.id.into(), sender.id.into())
      .await?;

    self.messages.push(message.clone());
    Ok(message)
  }

  /// 创建带文件的消息
  pub async fn create_message_with_files(
    &mut self,
    content: &str,
    files: Vec<String>,
    chat_idx: usize,
    sender_idx: usize,
  ) -> Result<Message> {
    let chat = &self.chats[chat_idx];
    let sender = &self.users[sender_idx];

    let message_data = TestFixtures::create_message_with_files(content, files);
    let message = self
      .app_state
      .create_message(message_data, chat.id.into(), sender.id.into())
      .await?;

    self.messages.push(message.clone());
    Ok(message)
  }

  /// 获取用户的访问令牌
  pub async fn get_user_token(&self, user_idx: usize) -> Result<String> {
    let user = &self.users[user_idx];
    let claims = fechatter_core::models::jwt::UserClaims {
      id: user.id,
      email: user.email.clone(),
      fullname: user.fullname.clone(),
      workspace_id: user.workspace_id,
      status: user.status,
      created_at: user.created_at,
    };

    let token_service = self.app_state.token_manager();
    let tokens = token_service
      .generate_auth_tokens(&claims, None, None)
      .await?;

    Ok(tokens.access_token)
  }

  /// 搜索消息
  pub async fn search_messages(
    &self,
    query: &str,
    user_index: usize,
    chat_index: Option<usize>,
  ) -> Result<fechatter_core::SearchResult> {
    let user = self
      .users
      .get(user_index)
      .ok_or_else(|| anyhow::anyhow!("User not found at index {}", user_index))?;

    let chat_id = if let Some(idx) = chat_index {
      Some(
        self
          .chats
          .get(idx)
          .ok_or_else(|| anyhow::anyhow!("Chat not found at index {}", idx))?
          .id,
      )
    } else {
      None
    };

    let search_query = fechatter_core::SearchMessages {
      query: query.to_string(),
      workspace_id: user.workspace_id,
      chat_id: chat_id.map(Into::into),
      offset: 0,
      limit: 20,
    };

    let search_service = self
      .app_state
      .search_service()
      .ok_or_else(|| anyhow::anyhow!("Search service not available"))?;

    search_service
      .search_messages(&search_query)
      .await
      .map_err(|e| anyhow::anyhow!("Search failed: {}", e))
  }
}
