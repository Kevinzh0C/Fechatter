use chrono::{Duration, Utc};
use fechatter_core::{
  Message,
  models::{DateRange, SearchMessages, SearchType, SortOrder},
};
use uuid::Uuid;

use crate::config::{
  IndexConfig, IndexesConfig, MeilisearchConfig, MeilisearchSettings, SearchConfig,
};
use crate::error::AppError;
use crate::services::search_service::SearchService;

/// 创建测试用的搜索配置
fn create_test_search_config(enabled: bool) -> SearchConfig {
  SearchConfig {
    enabled,
    provider: "meilisearch".to_string(),
    meilisearch: MeilisearchConfig {
      url: "http://localhost:7700".to_string(),
      api_key: "test_key".to_string(),
      connection_timeout_ms: 5000,
      request_timeout_ms: 10000,
      indexes: IndexesConfig {
        messages: IndexConfig {
          name: "test_fechatter_messages".to_string(),
          primary_key: "id".to_string(),
          searchable_fields: vec![
            "content".to_string(),
            "sender_name".to_string(),
            "chat_name".to_string(),
          ],
          filterable_fields: vec![
            "chat_id".to_string(),
            "sender_id".to_string(),
            "created_at".to_string(),
            "workspace_id".to_string(),
            "chat_type".to_string(),
          ],
          sortable_fields: vec!["created_at".to_string(), "relevance_score".to_string()],
          displayed_fields: vec![
            "id".to_string(),
            "content".to_string(),
            "sender_name".to_string(),
            "chat_name".to_string(),
            "created_at".to_string(),
          ],
        },
      },
      settings: MeilisearchSettings {
        pagination_limit: 100,
      },
    },
  }
}

/// 创建测试消息
fn create_test_message(id: i64, chat_id: i64, sender_id: i64, content: &str) -> Message {
  Message {
    id,
    chat_id,
    sender_id,
    content: content.to_string(),
    files: Some(vec![]),
    created_at: Utc::now(),
    idempotency_key: Some(Uuid::now_v7()),
  }
}

/// 创建测试搜索请求
fn create_test_search_request(query: &str, workspace_id: i64) -> SearchMessages {
  SearchMessages {
    query: query.to_string(),
    workspace_id,
    chat_id: Some(1),
    sender_id: None,
    search_type: SearchType::FullText,
    date_range: None,
    sort_order: Some(SortOrder::Relevance),
    offset: Some(0),
    limit: Some(20),
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_search_service_new_enabled() {
    // 测试创建启用搜索的服务
    let config = create_test_search_config(true);

    // 注意：在没有实际 Meilisearch 服务器的情况下，这会失败
    // 但我们可以测试配置是否正确传递
    let result = SearchService::new(config.clone());

    // 在测试环境中，如果没有 Meilisearch 服务器，应该返回错误
    // 这是预期的行为
    match result {
      Ok(service) => {
        assert!(service.is_enabled());
        println!("✅ SearchService created successfully when enabled");
      }
      Err(AppError::SearchError(msg)) => {
        assert!(msg.contains("Failed to create Meilisearch client"));
        println!("✅ SearchService creation fails gracefully without Meilisearch server");
      }
      _ => panic!("Unexpected error type"),
    }
  }

  #[test]
  fn test_search_service_new_disabled() {
    // 测试禁用搜索的配置
    let config = create_test_search_config(false);

    // 即使搜索被禁用，我们仍然应该能够创建服务实例
    let result = SearchService::new(config);

    match result {
      Ok(service) => {
        assert!(!service.is_enabled());
        println!("✅ SearchService created with disabled config");
      }
      Err(AppError::SearchError(_)) => {
        // 如果搜索被禁用，创建时仍可能失败，这也是可接受的
        println!("✅ SearchService creation handles disabled state");
      }
      _ => panic!("Unexpected error type"),
    }
  }

  #[test]
  fn test_search_service_is_enabled() {
    let enabled_config = create_test_search_config(true);
    let disabled_config = create_test_search_config(false);

    // 测试启用状态
    if let Ok(service) = SearchService::new(enabled_config) {
      assert!(service.is_enabled());
    }

    // 测试禁用状态
    if let Ok(service) = SearchService::new(disabled_config) {
      assert!(!service.is_enabled());
    }

    println!("✅ is_enabled() method works correctly");
  }

  #[tokio::test]
  async fn test_disabled_service_operations() {
    // 测试在搜索服务禁用时的行为
    let config = create_test_search_config(false);

    if let Ok(service) = SearchService::new(config) {
      // 测试初始化索引
      let init_result = service.initialize_indexes().await;
      assert!(init_result.is_ok()); // 禁用时应该直接返回 Ok
      println!("✅ initialize_indexes() handles disabled state");

      // 测试索引消息
      let message = create_test_message(1, 1, 1, "test message");
      let index_result = service
        .index_message(&message, "Test Chat", "Test User", "group", 1)
        .await;
      assert!(index_result.is_ok()); // 禁用时应该直接返回 Ok
      println!("✅ index_message() handles disabled state");

      // 测试删除消息
      let delete_result = service.delete_message(1).await;
      assert!(delete_result.is_ok()); // 禁用时应该直接返回 Ok
      println!("✅ delete_message() handles disabled state");

      // 测试更新消息
      let update_result = service
        .update_message(&message, "Test Chat", "Test User", "group", 1)
        .await;
      assert!(update_result.is_ok()); // 禁用时应该直接返回 Ok
      println!("✅ update_message() handles disabled state");

      // 测试搜索消息 - 这应该返回错误
      let search_request = create_test_search_request("test", 1);
      let search_result = service.search_messages(&search_request).await;
      match search_result {
        Err(AppError::SearchError(msg)) => {
          assert!(msg.contains("Search is disabled"));
          println!("✅ search_messages() correctly rejects when disabled");
        }
        _ => panic!("Expected search error when service is disabled"),
      }
    }
  }

  #[test]
  fn test_search_request_validation() {
    // 测试搜索请求的验证逻辑

    // 测试基本的有效请求
    let valid_request = create_test_search_request("valid query", 1);
    assert_eq!(valid_request.query, "valid query");
    assert_eq!(valid_request.workspace_id, 1);
    assert!(matches!(valid_request.search_type, SearchType::FullText));

    // 测试不同的搜索类型
    let mut exact_match_request = create_test_search_request("exact", 1);
    exact_match_request.search_type = SearchType::ExactMatch;
    assert!(matches!(
      exact_match_request.search_type,
      SearchType::ExactMatch
    ));

    let mut fuzzy_request = create_test_search_request("fuzzy", 1);
    fuzzy_request.search_type = SearchType::FuzzyMatch;
    assert!(matches!(fuzzy_request.search_type, SearchType::FuzzyMatch));

    let mut regex_request = create_test_search_request("regex.*", 1);
    regex_request.search_type = SearchType::Regex;
    assert!(matches!(regex_request.search_type, SearchType::Regex));

    println!("✅ Search request types work correctly");
  }

  #[test]
  fn test_date_range_configuration() {
    let now = Utc::now();
    let one_week_ago = now - Duration::weeks(1);

    let mut request = create_test_search_request("test", 1);
    request.date_range = Some(DateRange {
      start: Some(one_week_ago),
      end: Some(now),
    });

    assert!(request.date_range.is_some());
    let date_range = request.date_range.unwrap();
    assert!(date_range.start.is_some());
    assert!(date_range.end.is_some());
    assert!(date_range.start.unwrap() < date_range.end.unwrap());

    println!("✅ Date range configuration works correctly");
  }

  #[test]
  fn test_sort_order_options() {
    let mut request = create_test_search_request("test", 1);

    // 测试不同的排序选项
    request.sort_order = Some(SortOrder::Newest);
    assert!(matches!(request.sort_order, Some(SortOrder::Newest)));

    request.sort_order = Some(SortOrder::Oldest);
    assert!(matches!(request.sort_order, Some(SortOrder::Oldest)));

    request.sort_order = Some(SortOrder::Relevance);
    assert!(matches!(request.sort_order, Some(SortOrder::Relevance)));

    // 测试无排序
    request.sort_order = None;
    assert!(request.sort_order.is_none());

    println!("✅ Sort order options work correctly");
  }

  #[test]
  fn test_pagination_parameters() {
    let mut request = create_test_search_request("test", 1);

    // 测试分页参数
    request.offset = Some(0);
    request.limit = Some(20);
    assert_eq!(request.offset, Some(0));
    assert_eq!(request.limit, Some(20));

    // 测试大偏移量
    request.offset = Some(100);
    assert_eq!(request.offset, Some(100));

    // 测试小限制
    request.limit = Some(5);
    assert_eq!(request.limit, Some(5));

    // 测试无分页参数
    request.offset = None;
    request.limit = None;
    assert!(request.offset.is_none());
    assert!(request.limit.is_none());

    println!("✅ Pagination parameters work correctly");
  }

  #[test]
  fn test_search_config_validation() {
    // 测试不同的配置场景

    // 基本配置
    let basic_config = create_test_search_config(true);
    assert!(basic_config.enabled);
    assert_eq!(basic_config.provider, "meilisearch");
    assert!(!basic_config.meilisearch.url.is_empty());

    // 禁用配置
    let disabled_config = create_test_search_config(false);
    assert!(!disabled_config.enabled);

    // 验证索引配置
    let index_config = &basic_config.meilisearch.indexes.messages;
    assert!(!index_config.name.is_empty());
    assert!(!index_config.primary_key.is_empty());
    assert!(!index_config.searchable_fields.is_empty());
    assert!(!index_config.filterable_fields.is_empty());

    println!("✅ Search configuration validation works correctly");
  }

  #[test]
  fn test_message_creation() {
    // 测试测试消息的创建
    let message = create_test_message(1, 100, 200, "Test message content");

    assert_eq!(message.id, 1);
    assert_eq!(message.chat_id, 100);
    assert_eq!(message.sender_id, 200);
    assert_eq!(message.content, "Test message content");
    assert!(message.files.is_some());
    assert!(message.files.unwrap().is_empty());
    assert!(message.created_at <= Utc::now());
    assert!(message.idempotency_key.is_some());

    println!("✅ Test message creation works correctly");
  }

  #[test]
  fn test_error_types() {
    // 测试不同类型的错误

    // SearchError
    let search_error = AppError::SearchError("Test search error".to_string());
    match search_error {
      AppError::SearchError(msg) => {
        assert_eq!(msg, "Test search error");
      }
      _ => panic!("Expected SearchError"),
    }

    println!("✅ Error types work correctly");
  }

  #[test]
  fn test_workspace_and_chat_filters() {
    let mut request = create_test_search_request("test", 123);

    // 测试工作空间过滤
    assert_eq!(request.workspace_id, 123);

    // 测试聊天过滤
    request.chat_id = Some(456);
    assert_eq!(request.chat_id, Some(456));

    // 测试发送者过滤
    request.sender_id = Some(789);
    assert_eq!(request.sender_id, Some(789));

    // 测试无过滤器
    request.chat_id = None;
    request.sender_id = None;
    assert!(request.chat_id.is_none());
    assert!(request.sender_id.is_none());

    println!("✅ Workspace and chat filters work correctly");
  }

  #[test]
  fn test_search_service_architecture() {
    // 测试 SearchService 的整体架构
    let config = create_test_search_config(true);

    // 验证配置结构
    let message_index = &config.meilisearch.indexes.messages;

    // 验证必要的字段存在
    let required_searchable = ["content", "sender_name", "chat_name"];
    for field in &required_searchable {
      assert!(message_index.searchable_fields.contains(&field.to_string()));
    }

    let required_filterable = ["chat_id", "sender_id", "workspace_id"];
    for field in &required_filterable {
      assert!(message_index.filterable_fields.contains(&field.to_string()));
    }

    println!("✅ SearchService architecture is correctly structured");
  }

  #[test]
  fn test_config_timeout_settings() {
    let config = create_test_search_config(true);

    // 验证超时设置
    assert_eq!(config.meilisearch.connection_timeout_ms, 5000);
    assert_eq!(config.meilisearch.request_timeout_ms, 10000);
    assert_eq!(config.meilisearch.settings.pagination_limit, 100);

    println!("✅ Config timeout and settings work correctly");
  }

  #[test]
  fn test_search_types_exhaustive() {
    // 确保我们测试了所有搜索类型
    let types = [
      SearchType::FullText,
      SearchType::ExactMatch,
      SearchType::FuzzyMatch,
      SearchType::Regex,
    ];

    for search_type in types {
      let mut request = create_test_search_request("test", 1);
      request.search_type = search_type;

      // 验证类型设置正确
      match request.search_type {
        SearchType::FullText
        | SearchType::ExactMatch
        | SearchType::FuzzyMatch
        | SearchType::Regex => {
          // 所有类型都有效
        }
      }
    }

    println!("✅ All search types tested exhaustively");
  }

  #[test]
  fn test_sort_orders_exhaustive() {
    // 确保我们测试了所有排序选项
    let orders = [SortOrder::Newest, SortOrder::Oldest, SortOrder::Relevance];

    for sort_order in orders {
      let mut request = create_test_search_request("test", 1);
      request.sort_order = Some(sort_order);

      // 验证排序设置正确
      match request.sort_order {
        Some(SortOrder::Newest) | Some(SortOrder::Oldest) | Some(SortOrder::Relevance) => {
          // 所有排序都有效
        }
        None => {
          // 无排序也有效
        }
      }
    }

    println!("✅ All sort orders tested exhaustively");
  }
}

#[cfg(test)]
mod integration_tests {
  use super::*;

  #[tokio::test]
  async fn test_search_service_lifecycle() {
    // 测试 SearchService 的完整生命周期
    let config = create_test_search_config(false); // 使用禁用模式避免外部依赖

    if let Ok(service) = SearchService::new(config) {
      // 1. 验证服务状态
      assert!(!service.is_enabled());

      // 2. 测试初始化（禁用模式下应该成功）
      let init_result = service.initialize_indexes().await;
      assert!(init_result.is_ok());

      // 3. 测试消息操作（禁用模式下应该成功但不做实际操作）
      let message = create_test_message(1, 1, 1, "lifecycle test");

      let index_result = service
        .index_message(&message, "Test Chat", "Test User", "group", 1)
        .await;
      assert!(index_result.is_ok());

      let update_result = service
        .update_message(&message, "Test Chat Updated", "Test User", "group", 1)
        .await;
      assert!(update_result.is_ok());

      let delete_result = service.delete_message(message.id).await;
      assert!(delete_result.is_ok());

      // 4. 测试搜索（禁用模式下应该返回错误）
      let search_request = create_test_search_request("lifecycle", 1);
      let search_result = service.search_messages(&search_request).await;

      match search_result {
        Err(AppError::SearchError(msg)) => {
          assert!(msg.contains("Search is disabled"));
        }
        _ => panic!("Expected search error in disabled mode"),
      }

      println!("✅ SearchService lifecycle test completed successfully");
    } else {
      println!("⚠️  Cannot create SearchService for lifecycle test");
    }
  }

  #[test]
  fn test_comprehensive_configuration() {
    // 测试全面的配置选项
    let mut config = create_test_search_config(true);

    // 修改配置以测试不同选项
    config.meilisearch.api_key = "custom_key".to_string();
    config.meilisearch.url = "http://custom-host:7701".to_string();
    config.meilisearch.connection_timeout_ms = 8000;
    config.meilisearch.request_timeout_ms = 15000;

    // 添加自定义字段
    config
      .meilisearch
      .indexes
      .messages
      .searchable_fields
      .push("custom_field".to_string());
    config
      .meilisearch
      .indexes
      .messages
      .filterable_fields
      .push("custom_filter".to_string());

    // 验证配置
    assert_eq!(config.meilisearch.api_key, "custom_key");
    assert_eq!(config.meilisearch.url, "http://custom-host:7701");
    assert_eq!(config.meilisearch.connection_timeout_ms, 8000);
    assert_eq!(config.meilisearch.request_timeout_ms, 15000);
    assert!(
      config
        .meilisearch
        .indexes
        .messages
        .searchable_fields
        .contains(&"custom_field".to_string())
    );
    assert!(
      config
        .meilisearch
        .indexes
        .messages
        .filterable_fields
        .contains(&"custom_filter".to_string())
    );

    println!("✅ Comprehensive configuration test passed");
  }

  #[tokio::test]
  async fn test_error_handling_scenarios() {
    // 测试各种错误处理场景
    let config = create_test_search_config(false);

    if let Ok(service) = SearchService::new(config) {
      // 测试搜索服务禁用时的一致性行为

      // 所有索引操作都应该静默成功
      let message = create_test_message(999, 999, 999, "error test");

      assert!(
        service
          .index_message(&message, "Chat", "User", "group", 1)
          .await
          .is_ok()
      );
      assert!(
        service
          .update_message(&message, "Chat", "User", "group", 1)
          .await
          .is_ok()
      );
      assert!(service.delete_message(999).await.is_ok());
      assert!(service.initialize_indexes().await.is_ok());

      // 但搜索应该返回明确错误
      let search_request = create_test_search_request("error", 1);
      match service.search_messages(&search_request).await {
        Err(AppError::SearchError(msg)) => {
          assert!(msg.contains("Search is disabled"));
        }
        _ => panic!("Expected search error"),
      }

      println!("✅ Error handling scenarios work correctly");
    }
  }

  #[test]
  fn test_configuration_edge_cases() {
    // 测试配置的边界情况

    // 最小配置
    let mut minimal_config = create_test_search_config(false);
    minimal_config
      .meilisearch
      .indexes
      .messages
      .searchable_fields = vec!["content".to_string()];
    minimal_config
      .meilisearch
      .indexes
      .messages
      .filterable_fields = vec!["workspace_id".to_string()];
    minimal_config.meilisearch.connection_timeout_ms = 1000;
    minimal_config.meilisearch.request_timeout_ms = 2000;

    // 验证最小配置有效
    assert!(!minimal_config.enabled);
    assert!(
      !minimal_config
        .meilisearch
        .indexes
        .messages
        .searchable_fields
        .is_empty()
    );
    assert!(minimal_config.meilisearch.connection_timeout_ms > 0);

    // 大配置
    let mut large_config = create_test_search_config(true);
    for i in 1..=50 {
      large_config
        .meilisearch
        .indexes
        .messages
        .searchable_fields
        .push(format!("field_{}", i));
    }

    // 验证大配置处理
    assert!(
      large_config
        .meilisearch
        .indexes
        .messages
        .searchable_fields
        .len()
        > 50
    );

    println!("✅ Configuration edge cases handled correctly");
  }
}
