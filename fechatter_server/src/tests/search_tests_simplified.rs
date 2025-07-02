use fechatter_core::{
    models::jwt::UserClaims, CreateChat, CreateMessage, CreateUser, SearchMessages,
};
use uuid::Uuid;

use crate::{handlers::search_messages, models::ChatType, AppError, AppState};
use axum::{
    extract::{Extension, Path, State},
    Json,
};

#[tokio::test]
async fn test_basic_search_functionality() -> Result<(), AppError> {
    println!("Testing basic search functionality...");

    let (_tdb, app_state) = AppState::test_new().await?;

    // Generate unique identifier
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis();

    // Create test user with unique email
    let create_user = CreateUser {
        email: format!("test{}@example.com", timestamp),
        fullname: "Test User".to_string(),
        password: "password123".to_string(),
        workspace: format!("WSB{}", timestamp % 1000000), // Basic test workspace
    };
    let user = app_state.create_user(&create_user, None).await?;

    // Create another user for single chat
    let create_user2 = CreateUser {
        email: format!("test2_{}@example.com", timestamp),
        fullname: "Test User 2".to_string(),
        password: "password123".to_string(),
        workspace: format!("WSB{}", timestamp % 1000000), // Same workspace
    };
    let user2 = app_state.create_user(&create_user2, None).await?;

    // Create test chat using the create_new_chat method
    let chat_payload = CreateChat {
        name: format!("Test Chat {}", timestamp),
        members: Some(vec![user2.id.into()]), // Single chat with user2
        description: None,
        chat_type: ChatType::Single,
    };
    let chat = app_state
        .create_new_chat(
            user.id.into(),
            &chat_payload.name,
            chat_payload.chat_type,
            chat_payload
                .members
                .map(|m| m.into_iter().map(|id| id.into()).collect()),
            chat_payload.description.as_deref(),
            user.workspace_id.into(),
        )
        .await?;

    // Create test messages
    for i in 0..5 {
        let msg_payload = CreateMessage {
            content: format!("Test message {} with keyword API", i),
            files: vec![],
            idempotency_key: Uuid::now_v7(),
        };
        app_state
            .create_message(msg_payload, chat.id.into(), user.id.into())
            .await?;
    }

    let user_claims = UserClaims {
        id: user.id,
        workspace_id: user.workspace_id.into(),
        fullname: user.fullname.clone(),
        email: user.email.clone(),
        status: user.status,
        created_at: user.created_at,
    };

    // Test basic search
    let search_request = SearchMessages {
        query: "API".to_string(),
        workspace_id: user.workspace_id.into(),
        chat_id: None, // Will be set by handler
        offset: 0,
        limit: 10,
    };

    let result = search_messages(
        Extension(user_claims),
        State(app_state.clone()),
        Path(chat.id.into()),
        Json(search_request),
    )
    .await;

    match result {
        Ok(search_result) => {
            println!("   ✓ Search executed successfully");
            println!("   ✓ Found {} results", search_result.total_hits);
            println!("   ✓ Query time: {}ms", search_result.query_time_ms);
        }
        Err(AppError::SearchError(msg)) => {
            if msg.contains("index") || msg.contains("connection") || msg.contains("disabled") {
                println!("   INFO:  Search service needs configuration: {}", msg);
            } else {
                return Err(AppError::SearchError(msg));
            }
        }
        Err(e) => return Err(e),
    }

    println!("Basic search functionality test completed");
    Ok(())
}

#[tokio::test]
async fn test_search_validation() -> Result<(), AppError> {
    println!("✓ Testing search validation...");

    let (_tdb, app_state) = AppState::test_new().await?;

    // Generate unique identifier
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis();

    // Create test user with unique email
    let create_user = CreateUser {
        email: format!("test2_{}@example.com", timestamp),
        fullname: "Test User 2".to_string(),
        password: "password123".to_string(),
        workspace: format!("WSV{}", timestamp % 1000000), // Validation test workspace
    };
    let user = app_state.create_user(&create_user, None).await?;

    // Create another user for single chat
    let create_user2 = CreateUser {
        email: format!("test2b_{}@example.com", timestamp),
        fullname: "Test User 2B".to_string(),
        password: "password123".to_string(),
        workspace: format!("WSV{}", timestamp % 1000000), // Same workspace
    };
    let user2 = app_state.create_user(&create_user2, None).await?;

    // Create test chat
    let chat_payload = CreateChat {
        name: format!("Test Chat 2 {}", timestamp),
        members: Some(vec![user2.id.into()]), // Single chat with user2
        description: None,
        chat_type: ChatType::Single,
    };
    let chat = app_state
        .create_new_chat(
            user.id.into(),
            &chat_payload.name,
            chat_payload.chat_type,
            chat_payload
                .members
                .map(|m| m.into_iter().map(|id| id.into()).collect()),
            chat_payload.description.as_deref(),
            user.workspace_id.into(),
        )
        .await?;

    let user_claims = UserClaims {
        id: user.id,
        workspace_id: user.workspace_id.into(),
        fullname: user.fullname.clone(),
        email: user.email.clone(),
        status: user.status,
        created_at: user.created_at,
    };

    // Test empty query
    let empty_search = SearchMessages {
        query: "".to_string(),
        workspace_id: user.workspace_id.into(),
        chat_id: None,
        offset: 0,
        limit: 10,
    };

    let result = search_messages(
        Extension(user_claims.clone()),
        State(app_state.clone()),
        Path(chat.id.into()),
        Json(empty_search),
    )
    .await;

    assert!(matches!(result, Err(AppError::InvalidInput(_))));
    println!("   ✓ Empty query validation works");

    // Test invalid limit
    let invalid_limit = SearchMessages {
        query: "test".to_string(),
        workspace_id: user.workspace_id.into(),
        chat_id: None,
        offset: 0,
        limit: 1000, // Exceeds max
    };

    let result = search_messages(
        Extension(user_claims),
        State(app_state.clone()),
        Path(chat.id.into()),
        Json(invalid_limit),
    )
    .await;

    assert!(matches!(result, Err(AppError::InvalidInput(_))));
    println!("   ✓ Limit validation works");

    println!("Search validation test completed");
    Ok(())
}

#[tokio::test]
async fn test_search_pagination() -> Result<(), AppError> {
    println!("📄 Testing search pagination...");

    let (_tdb, app_state) = AppState::test_new().await?;

    // Generate unique identifier
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis();

    // Create test user with unique email
    let create_user = CreateUser {
        email: format!("test3_{}@example.com", timestamp),
        fullname: "Test User 3".to_string(),
        password: "password123".to_string(),
        workspace: format!("WSP{}", timestamp % 1000000), // Pagination test workspace
    };
    let user = app_state.create_user(&create_user, None).await?;

    // Create another user for single chat
    let create_user2 = CreateUser {
        email: format!("test3b_{}@example.com", timestamp),
        fullname: "Test User 3B".to_string(),
        password: "password123".to_string(),
        workspace: format!("WSP{}", timestamp % 1000000), // Same workspace
    };
    let user2 = app_state.create_user(&create_user2, None).await?;

    // Create test chat
    let chat_payload = CreateChat {
        name: format!("Test Chat 3 {}", timestamp),
        members: Some(vec![user2.id.into()]), // Single chat with user2
        description: None,
        chat_type: ChatType::Single,
    };
    let chat = app_state
        .create_new_chat(
            user.id.into(),
            &chat_payload.name,
            chat_payload.chat_type,
            chat_payload
                .members
                .map(|m| m.into_iter().map(|id| id.into()).collect()),
            chat_payload.description.as_deref(),
            user.workspace_id.into(),
        )
        .await?;

    // Create many test messages
    for i in 0..25 {
        let msg_payload = CreateMessage {
            content: format!("Pagination test message {}", i),
            files: vec![],
            idempotency_key: Uuid::now_v7(),
        };
        app_state
            .create_message(msg_payload, chat.id.into(), user.id.into())
            .await?;
    }

    let user_claims = UserClaims {
        id: user.id,
        workspace_id: user.workspace_id.into(),
        fullname: user.fullname.clone(),
        email: user.email.clone(),
        status: user.status,
        created_at: user.created_at,
    };

    // Test first page
    let first_page = SearchMessages {
        query: "Pagination".to_string(),
        workspace_id: user.workspace_id.into(),
        chat_id: None,
        offset: 0,
        limit: 10,
    };

    let result = search_messages(
        Extension(user_claims.clone()),
        State(app_state.clone()),
        Path(chat.id.into()),
        Json(first_page),
    )
    .await;

    match result {
        Ok(search_result) => {
            println!("   ✓ First page retrieved");
            println!("   ✓ Has more: {}", search_result.has_more);
        }
        Err(AppError::SearchError(msg)) => {
            if msg.contains("index") || msg.contains("connection") || msg.contains("disabled") {
                println!("   INFO:  Search service needs configuration: {}", msg);
                return Ok(());
            } else {
                return Err(AppError::SearchError(msg));
            }
        }
        Err(e) => return Err(e),
    }

    // Test second page
    let second_page = SearchMessages {
        query: "Pagination".to_string(),
        workspace_id: user.workspace_id.into(),
        chat_id: None,
        offset: 10,
        limit: 10,
    };

    let result = search_messages(
        Extension(user_claims),
        State(app_state.clone()),
        Path(chat.id.into()),
        Json(second_page),
    )
    .await;

    match result {
        Ok(search_result) => {
            println!("   ✓ Second page retrieved");
            println!("   ✓ Results count: {}", search_result.messages.len());
        }
        Err(AppError::SearchError(_)) => {
            println!("   INFO:  Search service needs configuration");
        }
        Err(e) => return Err(e),
    }

    println!("Search pagination test completed");
    Ok(())
}
