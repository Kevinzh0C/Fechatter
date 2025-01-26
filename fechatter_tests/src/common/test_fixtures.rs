use chrono::Utc;
use fechatter_core::{ChatType, CreateChat, CreateMessage, CreateUser};
use uuid::Uuid;

/// 测试数据生成器
pub struct TestFixtures;

impl TestFixtures {
  /// 生成唯一的测试ID
  pub fn unique_id() -> String {
    Uuid::new_v4().to_string().replace("-", "")[..8].to_string()
  }

  /// 生成测试用户数据
  pub fn create_user(prefix: &str) -> CreateUser {
    let id = Self::unique_id();
    CreateUser {
      email: format!("{}{}@test.com", prefix, id),
      fullname: format!("{} User {}", prefix, id),
      password: "password123".to_string(),
      workspace: format!("WS{}{}", prefix, id),
    }
  }

  /// 批量生成测试用户数据
  pub fn create_users(prefix: &str, count: usize) -> Vec<CreateUser> {
    (0..count)
      .map(|i| {
        let id = Self::unique_id();
        CreateUser {
          email: format!("{}{}_{}@test.com", prefix, i, id),
          fullname: format!("{} User {} {}", prefix, i, id),
          password: "password123".to_string(),
          workspace: format!("WS{}{}", prefix, id),
        }
      })
      .collect()
  }

  /// 创建聊天数据
  pub fn create_chat(name: &str) -> CreateChat {
    CreateChat {
      name: format!("{} {}", name, Self::unique_id()),
      chat_type: ChatType::Group,
      members: None,
      description: Some(format!("Test chat created at {}", chrono::Utc::now())),
    }
  }

  /// 创建带成员的聊天数据
  pub fn create_chat_with_members(
    name: &str,
    member_ids: Vec<fechatter_core::UserId>,
  ) -> CreateChat {
    CreateChat {
      name: format!("{} {}", name, Self::unique_id()),
      chat_type: ChatType::Group,
      members: if member_ids.is_empty() {
        None
      } else {
        Some(member_ids)
      },
      description: Some(format!(
        "Test chat with members created at {}",
        chrono::Utc::now()
      )),
    }
  }

  /// 生成测试消息数据
  pub fn create_message(content: &str) -> CreateMessage {
    CreateMessage {
      content: content.to_string(),
      files: vec![],
      idempotency_key: Uuid::now_v7(),
    }
  }

  /// 生成带文件的测试消息数据
  pub fn create_message_with_files(content: &str, files: Vec<String>) -> CreateMessage {
    CreateMessage {
      content: content.to_string(),
      files,
      idempotency_key: Uuid::now_v7(),
    }
  }

  /// 生成测试文件内容
  pub fn create_file_content(size: usize) -> Vec<u8> {
    (0..size).map(|i| (i % 256) as u8).collect()
  }

  /// 生成搜索查询数据
  pub fn create_search_query(
    query: &str,
    workspace_id: i64,
    limit: i64,
  ) -> fechatter_core::SearchMessages {
    fechatter_core::SearchMessages {
      query: query.to_string(),
      workspace_id: fechatter_core::WorkspaceId(workspace_id),
      chat_id: None,
      offset: 0,
      limit,
    }
  }
}

/// 测试数据断言工具
pub struct TestAssertions;

impl TestAssertions {
  /// 断言用户数据有效
  pub fn assert_user_valid(user: &fechatter_core::models::User) {
    assert!(
      user.id > fechatter_core::UserId(0),
      "User ID should be positive"
    );
    assert!(!user.email.is_empty(), "User email should not be empty");
    assert!(
      !user.fullname.is_empty(),
      "User fullname should not be empty"
    );
    assert!(
      user.workspace_id > fechatter_core::WorkspaceId(0),
      "Workspace ID should be positive"
    );
  }

  /// 断言聊天数据有效
  pub fn assert_chat_valid(chat: &fechatter_core::Chat) {
    assert!(
      chat.id > fechatter_core::ChatId(0),
      "Chat ID should be positive"
    );
    assert!(!chat.name.is_empty(), "Chat name should not be empty");
    assert!(
      chat.created_by > fechatter_core::UserId(0),
      "Creator ID should be positive"
    );
  }

  /// 断言消息数据有效
  pub fn assert_message_valid(message: &fechatter_core::Message) {
    assert!(
      message.id > fechatter_core::MessageId(0),
      "Message ID should be positive"
    );
    assert!(
      !message.content.is_empty() || message.files.is_some(),
      "Message should have content or files"
    );
    assert!(
      message.sender_id > fechatter_core::UserId(0),
      "Sender ID should be positive"
    );
    assert!(
      message.chat_id > fechatter_core::ChatId(0),
      "Chat ID should be positive"
    );
  }

  /// 断言搜索结果有效
  pub fn assert_search_result_valid(result: &fechatter_core::SearchResult) {
    assert!(result.query_time_ms > 0, "Query time should be positive");
    for message in &result.messages {
      assert!(
        message.id > fechatter_core::MessageId(0),
        "Message ID should be positive"
      );
      assert!(
        message.chat_id > fechatter_core::ChatId(0),
        "Chat ID should be positive"
      );
    }
  }
}
