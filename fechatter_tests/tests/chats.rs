use anyhow::Result;
use fechatter_core::{Chat, ChatType, Message};

use futures::StreamExt;
use reqwest::{
  StatusCode,
  multipart::{Form, Part},
};
use reqwest_eventsource::{Event, EventSource};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::{net::SocketAddr, time::Duration};
use tokio::{net::TcpListener, time::sleep};

/*
test1:
    name: user 1 create chat
    steps:
        - signin
            email: tchen@acme.org
            password: 123456
        - create_chat
            name: test
            members: [1, 2]
        - create_message
            chat_id: 1
            content: hello
            files: [Cargo.toml]
*/

#[derive(Debug, Deserialize)]
struct AuthToken {
  token: String,
}

#[derive(Serialize, Deserialize)]
struct AuthResponse {
  access_token: String,
  expires_in: usize,
  refresh_token: Option<String>,
}

struct ChatServer {
  addr: SocketAddr,
  token: String,
  client: reqwest::Client,
}

struct NotifyServer;

const WILD_ADDR: &str = "0.0.0.0:0";

#[cfg(test)]
#[tokio::test]
async fn chat_server_should_work() -> Result<()> {
  println!("Starting chat_server_should_work test");

  let (tdb, state) = fechatter_server::AppState::test_new().await?;
  println!("AppState created successfully");

  // 创建用户和凭据
  let email = "tchen@acme.org";
  let password = "123456";

  let create_user = fechatter_core::CreateUser::new("Terry Chen", email, "Acme", password);
  println!("Creating user: {}", email);

  let auth_tokens = state.signup(&create_user, None).await?;
  println!(
    "User created successfully. Access token length: {}, Refresh token available: {}",
    auth_tokens.access_token.len(),
    auth_tokens.refresh_token.token.len() > 0
  );

  // 创建聊天服务器，使用相同的凭据
  println!("Creating chat server...");
  let chat_server = match tokio::time::timeout(
    Duration::from_secs(5),
    ChatServer::new(state, email, password),
  )
  .await
  {
    Ok(result) => result?,
    Err(_) => {
      println!("Timeout while creating ChatServer");
      anyhow::bail!("Timeout while creating ChatServer");
    }
  };
  println!("Chat server created successfully");

  // 设置通知服务器，使用 chat_server 获取的令牌
  let db_url = tdb.url();
  println!("Setting up notify server with DB URL: {}", db_url);

  let notify_server = match tokio::time::timeout(
    Duration::from_secs(5),
    NotifyServer::new(&db_url, &chat_server.token),
  )
  .await
  {
    Ok(result) => result?,
    Err(_) => {
      println!("Timeout while creating NotifyServer");
      anyhow::bail!("Timeout while creating NotifyServer");
    }
  };
  println!("Notify server created successfully");

  // 创建聊天
  println!("Creating chat...");
  let chat = match tokio::time::timeout(Duration::from_secs(5), chat_server.create_chat()).await {
    Ok(result) => match result {
      Ok(chat) => chat,
      Err(e) => {
        println!("Error creating chat: {}", e);
        anyhow::bail!("Error creating chat: {}", e);
      }
    },
    Err(_) => {
      println!("Timeout while creating chat");
      anyhow::bail!("Timeout while creating chat");
    }
  };
  println!("Chat created successfully with ID: {}", chat.id);

  // 创建消息
  println!("Creating message...");
  let chat_id = chat.id as u64;
  let _msg = match tokio::time::timeout(
    Duration::from_secs(10),
    chat_server.create_message(chat_id),
  )
  .await
  {
    Ok(result) => match result {
      Ok(msg) => msg,
      Err(e) => {
        println!("Error creating message: {}", e);
        anyhow::bail!("Error creating message: {}", e);
      }
    },
    Err(_) => {
      println!("Timeout while creating message");
      anyhow::bail!("Timeout while creating message");
    }
  };
  println!("Message created successfully");

  // 等待事件处理
  println!("Waiting for events...");
  sleep(Duration::from_secs(1)).await;
  println!("Test completed successfully");

  Ok(())
}

impl NotifyServer {
  async fn new(db_url: &str, token: &str) -> Result<Self> {
    println!("Setting up notify server. DB URL: {}", db_url);
    if token.len() > 10 {
      println!(
        "Using token: {}... (length: {})",
        &token[0..10],
        token.len()
      );
    } else {
      println!("WARNING: Token is too short: {}", token);
    }

    let mut config = notify_server::AppConfig::load()?;
    println!("Notify server config loaded");

    config.server.db_url = db_url.to_string();
    println!("DB URL updated in config");

    let app = notify_server::get_router(config).await?;
    println!("Notify server router created");

    let listener = TcpListener::bind(WILD_ADDR).await?;
    let addr = listener.local_addr()?;
    println!("Notify server listening on {}", addr);

    tokio::spawn(async move {
      println!("Notify server thread started");
      axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
    });

    println!(
      "Creating EventSource connection to http://{}/events?token={}",
      addr, token
    );
    let mut es = EventSource::get(format!("http://{}/events?token={}", addr, token));
    println!("EventSource created");

    tokio::spawn(async move {
      println!("EventSource listener thread started");
      while let Some(event) = es.next().await {
        match event {
          Ok(Event::Open) => println!("EventSource connection open"),
          Ok(Event::Message(message)) => {
            println!("Received event: {}", message.event);
            match message.event.as_str() {
              "NewChat" => {
                println!("Processing NewChat event: {}", message.data);
                let chat: Chat = serde_json::from_str(&message.data).unwrap();
                assert_eq!(chat.name.as_str(), "test");
                assert_eq!(chat.chat_members, vec![1, 2]);
                assert_eq!(chat.chat_type, ChatType::PrivateChannel);
                println!("Chat event validated successfully");
              }

              "NewMessage" => {
                println!("Processing NewMessage event: {}", message.data);
                let msg: Message = serde_json::from_str(&message.data).unwrap();
                assert_eq!(msg.content, "hello");
                assert_eq!(msg.files.as_ref().unwrap().len(), 1);
                assert_eq!(msg.sender_id, 1);
                println!("Message event validated successfully");
              }
              _ => {
                println!("Unexpected event type: {}", message.event);
                panic!("unexpected event: {:?}", message);
              }
            }
          }
          Err(err) => {
            println!("EventSource error: {}", err);
            es.close();
          }
        }
      }
      println!("EventSource listener thread terminated");
    });

    println!("NotifyServer setup completed");
    Ok(Self)
  }
}

impl ChatServer {
  async fn new(state: fechatter_server::AppState, email: &str, password: &str) -> Result<Self> {
    println!("Starting ChatServer::new with provided credentials");

    // 创建服务器路由，添加一个调试端点
    let mut app = fechatter_server::get_router(state.clone()).await?;

    // 添加一个调试端点，用于验证用户认证状态
    use axum::{Extension, Json, response::IntoResponse, routing::get};
    use fechatter_core::middlewares::verify_token_middleware;
    use fechatter_core::models::AuthUser;

    async fn debug_auth_handler(Extension(user): Extension<AuthUser>) -> impl IntoResponse {
      // 返回用户信息以及工作区ID
      Json(serde_json::json!({
        "user_id": user.id,
        "workspace_id": user.workspace_id,
        "email": user.email,
        "fullname": user.fullname,
        "status": user.status,
      }))
    }

    // 添加一个测试路由，具有与创建聊天相同的中间件链
    app = app
      .route("/api/debug/auth", get(debug_auth_handler))
      .nest_service(
        "/api/debug/auth_chain",
        axum::Router::new()
          .route("/", get(debug_auth_handler))
          .layer(axum::middleware::from_fn_with_state(
            state.clone(),
            verify_token_middleware::<fechatter_server::AppState>,
          )),
      )
      .merge(
        axum::Router::new()
          .route("/api/debug/workspace", get(debug_auth_handler))
          .route_layer(axum::middleware::from_fn_with_state(
            state.clone(),
            verify_token_middleware::<fechatter_server::AppState>,
          )),
      );

    println!("Router with debug endpoint created successfully");

    let listener = TcpListener::bind(WILD_ADDR).await?;
    let addr = listener.local_addr()?;
    println!("Server listening on {}", addr);

    tokio::spawn(async move {
      println!("Server thread started");
      axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
    });

    let client = reqwest::Client::new();
    println!("Client created");

    let mut ret = Self {
      addr,
      client,
      token: "".to_string(),
    };

    // 使用提供的凭据登录
    println!("Signing in with credentials: {}", email);
    ret.token = ret.signin(email, password).await?;
    println!("Signin completed");

    println!("ChatServer initialized successfully");
    Ok(ret)
  }

  async fn signin(&self, email: &str, password: &str) -> Result<String> {
    println!("Attempting to sign in with {}", email);

    // 构建登录请求体
    let signin_body = format!(r#"{{"email": "{}", "password": "{}"}}"#, email, password);

    let res = self
      .client
      .post(&format!("http://{}/api/signin", self.addr))
      .header("Content-Type", "application/json")
      .body(signin_body)
      .send()
      .await?;

    println!("Signin response status: {}", res.status());
    assert_eq!(res.status(), 200);

    // 获取响应文本
    let response_text = res.text().await?;
    println!("Raw signin response: {}", response_text);

    // 解析更新后的响应格式
    let auth_resp: AuthResponse = serde_json::from_str(&response_text)?;

    // 打印令牌信息（只显示前20个字符和长度，避免泄露完整令牌）
    if auth_resp.access_token.len() > 20 {
      println!(
        "Token received: {}... (length: {})",
        &auth_resp.access_token[0..20],
        auth_resp.access_token.len()
      );
    } else {
      println!("Token received: [EMPTY or TOO SHORT]");
    }

    Ok(auth_resp.access_token)
  }

  async fn create_chat(&self) -> Result<Chat> {
    // 首先检查调试端点，确保认证有效
    println!("\nTesting authentication with debug endpoint...");
    let debug_res = self
      .client
      .get(format!("http://{}/api/debug/auth", self.addr))
      .header("Authorization", format!("Bearer {}", self.token))
      .send()
      .await?;

    let debug_status = debug_res.status();
    println!("Debug endpoint status: {}", debug_status);

    if debug_status.is_success() {
      // 打印认证信息
      let auth_info = debug_res.text().await?;
      println!("Authentication info: {}", auth_info);
    } else {
      println!("Debug endpoint returned error: {}", debug_status);
      let error_text = debug_res.text().await.unwrap_or_default();
      if !error_text.is_empty() {
        println!("Error details: {}", error_text);
      }

      // 测试中间件链
      println!("Testing auth middleware chain...");
      let chain_res = self
        .client
        .get(format!("http://{}/api/debug/auth_chain", self.addr))
        .header("Authorization", format!("Bearer {}", self.token))
        .send()
        .await?;

      println!("Auth chain test status: {}", chain_res.status());
      if chain_res.status().is_success() {
        println!("Auth chain test response: {}", chain_res.text().await?);
      } else {
        println!(
          "Auth chain test error: {}",
          chain_res.text().await.unwrap_or_default()
        );
      }
    }

    // 尝试两次不同的请求格式，以确定哪一种更合适
    for attempt in 1..=2 {
      println!("\nAttempt {} to create chat", attempt);

      // 第一次尝试最小化请求，第二次添加members
      let request_body = if attempt == 1 {
        r#"{"name": "test", "chat_type": "PrivateChannel"}"#
      } else {
        r#"{"name": "test", "chat_type": "PrivateChannel", "members": [2], "description": ""}"#
      };

      println!("Sending create chat request with body: {}", request_body);

      // 打印认证令牌信息
      if self.token.len() > 20 {
        println!(
          "Using token: {}... (length: {})",
          &self.token[0..20],
          self.token.len()
        );
      } else {
        println!("WARNING: Token is empty or too short: '{}'", self.token);
      }

      let auth_header = format!("Bearer {}", self.token);
      println!("Authorization header: {}", &auth_header[..30]); // 只显示前30个字符

      // 首先，获取一下当前用户信息，检查工作区上下文
      let user_info_res = self
        .client
        .get(format!("http://{}/api/me", self.addr))
        .header("Authorization", &auth_header)
        .send()
        .await;

      match user_info_res {
        Ok(res) => {
          println!("User info response status: {}", res.status());
          if res.status().is_success() {
            let user_info = res.text().await?;
            println!("User info: {}", user_info);
          } else {
            let error = res
              .text()
              .await
              .unwrap_or_else(|_| "Failed to read error response".to_string());
            println!("Failed to get user info: {}", error);
          }
        }
        Err(e) => println!("Error fetching user info: {}", e),
      }

      // 然后尝试创建聊天
      let req = self
        .client
        .post(format!("http://{}/api/chat", self.addr))
        .header("Authorization", &auth_header)
        .header("Content-Type", "application/json")
        .body(request_body);

      println!("Sending request...");
      let res = req.send().await?;
      let status = res.status();
      println!("Response status: {}", status);

      // 获取并打印所有响应头
      println!("Response headers:");
      for (name, value) in res.headers() {
        println!(
          "  {}: {}",
          name,
          value.to_str().unwrap_or("Non-utf8 header value")
        );
      }

      if status != StatusCode::CREATED {
        let error_text = res.text().await?;
        println!(
          "Failed to create chat. Status: {}, Error: '{}'",
          status, error_text
        );

        if attempt < 2 {
          println!("Will retry with different request format");
          continue;
        }

        anyhow::bail!(
          "Failed to create chat after multiple attempts: {}",
          error_text
        );
      }

      let chat: Chat = res.json().await?;
      println!("Successfully created chat: {:?}", chat);
      assert_eq!(chat.name, "test");
      assert_eq!(chat.chat_type, ChatType::PrivateChannel);

      return Ok(chat);
    }

    // 不应该到这里，但为了编译通过
    anyhow::bail!("Unexpected control flow in create_chat")
  }

  async fn create_message(&self, chat_id: u64) -> Result<Message> {
    // upload file
    let data = include_bytes!("../Cargo.toml");
    let files = Part::bytes(data)
      .file_name("Cargo.toml")
      .mime_str("text/plain")?;
    let form = Form::new().part("file", files);

    let res = self
      .client
      .post(&format!("http://{}/api/upload", self.addr))
      .header("Authorization", format!("Bearer {}", self.token))
      .multipart(form)
      .send()
      .await?;
    assert_eq!(res.status(), StatusCode::OK);
    let ret: Vec<String> = res.json().await?;

    let body = serde_json::to_string(&json!({
        "content": "hello",
        "files": ret,
    }))?;
    let res = self
      .client
      .post(format!(
        "http://{}/api/chat/{}/messages",
        self.addr, chat_id
      ))
      .header("Authorization", format!("Bearer {}", self.token))
      .header("Content-Type", "application/json")
      .body(body);
    let res = res.send().await?;
    assert_eq!(res.status(), StatusCode::CREATED);
    let message: Message = res.json().await?;
    assert_eq!(message.content, "hello");
    assert_eq!(message.files.as_ref().unwrap().len(), 1);
    assert_eq!(message.sender_id, 1);
    assert_eq!(message.chat_id, chat_id as i64);
    Ok(message)
  }
}
