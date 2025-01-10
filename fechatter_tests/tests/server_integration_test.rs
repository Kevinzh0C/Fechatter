mod server_integration_test {
  use anyhow::Result;
  use fechatter_core::{Chat, ChatType, Message};

  use axum::extract::State;

  use axum::{Extension, Json, Router, response::IntoResponse, routing::get};
  use fechatter_core::models::AuthUser;
  use fechatter_server::handlers::auth::AuthResponse;
  use fechatter_server::middlewares::RouterExt;
  use futures::StreamExt;
  use reqwest::StatusCode as ReqwestStatusCode;
  use reqwest::multipart::{Form, Part};
  use reqwest_eventsource::{Event, EventSource};
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

  struct ChatServer {
    addr: SocketAddr,
    token: String,
    client: reqwest::Client,
  }

  struct NotifyServer;

  const WILD_ADDR: &str = "0.0.0.0:0";

  // Helper functions
  async fn helper_create_user_and_signup(
    state: &fechatter_server::AppState,
    email: &str,
    password: &str,
    fullname: &str,
    company: &str,
  ) -> Result<impl Sized> {
    println!("Helper: Creating user {}...", email);
    let create_user_payload = fechatter_core::CreateUser::new(fullname, email, company, password);
    let auth_tokens = state.signup(&create_user_payload, None).await?;
    println!(
      "Helper: User created successfully. Access token length: {}, Refresh token available: {}",
      auth_tokens.access_token.len(),
      auth_tokens.refresh_token.token.len() > 0
    );
    Ok(auth_tokens)
  }

  async fn helper_setup_chat_server(
    state: fechatter_server::AppState,
    email: &str,
    password: &str,
  ) -> Result<ChatServer> {
    println!("Helper: Creating chat server...");
    let chat_server = match tokio::time::timeout(
      Duration::from_secs(5),
      ChatServer::new(state, email, password),
    )
    .await
    {
      Ok(result) => result?,
      Err(_) => {
        println!("Helper: Timeout while creating ChatServer");
        anyhow::bail!("Helper: Timeout while creating ChatServer");
      }
    };
    println!("Helper: Chat server created successfully");
    Ok(chat_server)
  }

  async fn helper_setup_notify_server(db_url: &str, token: &str) -> Result<NotifyServer> {
    println!("Helper: Setting up notify server with DB URL: {}", db_url);
    let notify_server =
      match tokio::time::timeout(Duration::from_secs(5), NotifyServer::new(db_url, token)).await {
        Ok(result) => result?,
        Err(_) => {
          println!("Helper: Timeout while creating NotifyServer");
          anyhow::bail!("Helper: Timeout while creating NotifyServer");
        }
      };
    println!("Helper: Notify server created successfully");
    Ok(notify_server)
  }

  async fn helper_create_chat(chat_server: &ChatServer) -> Result<Chat> {
    println!("Helper: Creating chat...");
    let chat = match tokio::time::timeout(Duration::from_secs(5), chat_server.create_chat()).await {
      Ok(result) => match result {
        Ok(chat) => chat,
        Err(e) => {
          println!("Helper: Error creating chat: {}", e);
          anyhow::bail!("Helper: Error creating chat: {}", e);
        }
      },
      Err(_) => {
        println!("Helper: Timeout while creating chat");
        anyhow::bail!("Helper: Timeout while creating chat");
      }
    };
    println!("Helper: Chat created successfully with ID: {}", chat.id);
    Ok(chat)
  }

  async fn helper_create_message(chat_server: &ChatServer, chat_id: u64) -> Result<Message> {
    println!("Helper: Creating message...");
    let msg = match tokio::time::timeout(
      Duration::from_secs(10),
      chat_server.create_message(chat_id),
    )
    .await
    {
      Ok(result) => match result {
        Ok(msg) => msg,
        Err(e) => {
          println!("Helper: Error creating message: {}", e);
          anyhow::bail!("Helper: Error creating message: {}", e);
        }
      },
      Err(_) => {
        println!("Helper: Timeout while creating message");
        anyhow::bail!("Helper: Timeout while creating message");
      }
    };
    println!("Helper: Message created successfully");
    Ok(msg)
  }

  // Define a handler for the /api/me route for testing purposes
  async fn test_me_handler(Extension(user): Extension<AuthUser>) -> impl IntoResponse {
    println!("Test /api/me handler called for user: {}", user.email);
    Json(json!({
        "id": user.id,
        "workspace_id": user.workspace_id,
        "email": user.email,
        "fullname": user.fullname,
        "status": user.status,
        "created_at": user.created_at.to_rfc3339() // Ensure consistent format
    }))
  }

  // Define a handler for testing workspace context
  async fn test_workspace_context_handler(
    Extension(user): Extension<AuthUser>,
    Extension(workspace): Extension<fechatter_core::models::Workspace>,
  ) -> impl IntoResponse {
    println!(
      "Test workspace context handler called for user: {} in workspace: {}",
      user.email, workspace.id
    );
    Json(json!({
        "user_id": user.id,
        "workspace_id": workspace.id,
        "workspace_name": workspace.name,
        "status": "Workspace context is working"
    }))
  }

  #[cfg(test)]
  #[tokio::test]
  async fn chat_server_should_work() -> Result<()> {
    println!("Starting chat_server_should_work test");

    let (tdb, state) = fechatter_server::AppState::test_new().await?;
    println!("AppState created successfully");

    let email = "tchen@acme.org";
    let password = "123456";
    let fullname = "Terry Chen";
    let company = "Acme";

    // 创建用户和凭据
    let _auth_tokens =
      helper_create_user_and_signup(&state, email, password, fullname, company).await?;

    // 创建聊天服务器，使用相同的凭据
    // Note: state is moved here, so it cannot be used afterwards unless cloned before this call.
    // If other helpers needed the original state, state.clone() would be passed to helper_setup_chat_server.
    let chat_server = helper_setup_chat_server(state.clone(), email, password).await?;

    // 设置通知服务器，使用 chat_server 获取的令牌
    let db_url = tdb.url();
    let _notify_server = helper_setup_notify_server(&db_url, &chat_server.token).await?;

    // 创建聊天
    let chat = helper_create_chat(&chat_server).await?;

    // 创建消息
    let chat_id = chat.id as u64;
    let _msg = helper_create_message(&chat_server, chat_id).await?;

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

      // 创建服务器路由
      let mut base_router = fechatter_server::get_router(state.clone()).await?;

      // Define and merge the /api/me route for the test server
      let me_api_router = Router::new()
        .route("/api/me", get(test_me_handler))
        .with_middlewares(state.clone())
        .with_auth()
        .build();
      base_router = base_router.merge(me_api_router);

      // Define a test endpoint for workspace context
      let workspace_test_router = Router::new()
        .route(
          "/api/debug/workspace_test",
          get(test_workspace_context_handler),
        )
        .with_middlewares(state.clone())
        .with_auth()
        .with_workspace()
        .build();

      base_router = base_router.merge(workspace_test_router);

      // 使用 ServerRouterBuilder 构建带有调试端点的路由
      let app = ServerRouterBuilder::new(state.clone(), base_router)
        .add_auth_debug_routes()
        .build();

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

        // 确保工作区存在，这是一个关键步骤
        println!("\nEnsuring workspace exists via debug endpoint...");
        let workspace_res = self
          .client
          .get(format!("http://{}/api/debug/ensure_workspace", self.addr))
          .header("Authorization", format!("Bearer {}", self.token))
          .send()
          .await?;

        println!(
          "Workspace ensure endpoint status: {}",
          workspace_res.status()
        );
        if workspace_res.status().is_success() {
          println!("Workspace ensure response: {}", workspace_res.text().await?);
        } else {
          let error_text = workspace_res.text().await.unwrap_or_default();
          println!("Workspace ensure error: {}", error_text);
          return Err(anyhow::anyhow!(
            "Failed to ensure workspace exists: {}",
            error_text
          ));
        }

        // Test workspace context if auth is successful
        println!("\nTesting workspace context with debug endpoint...");
        let workspace_test_res = self
          .client
          .get(format!("http://{}/api/debug/workspace_test", self.addr))
          .header("Authorization", format!("Bearer {}", self.token))
          .send()
          .await?;

        println!(
          "Workspace test endpoint status: {}",
          workspace_test_res.status()
        );
        if workspace_test_res.status().is_success() {
          println!(
            "Workspace test response: {}",
            workspace_test_res.text().await?
          );
        } else {
          println!(
            "Workspace test error: {}",
            workspace_test_res.text().await.unwrap_or_default()
          );
        }
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

        if status != ReqwestStatusCode::CREATED {
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
      assert_eq!(res.status(), ReqwestStatusCode::OK);
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
      assert_eq!(res.status(), ReqwestStatusCode::CREATED);
      let message: Message = res.json().await?;
      assert_eq!(message.content, "hello");
      assert_eq!(message.files.as_ref().unwrap().len(), 1);
      assert_eq!(message.sender_id, 1);
      assert_eq!(message.chat_id, chat_id as i64);
      Ok(message)
    }
  }

  // Definition for ServerRouterBuilder and its methods
  struct ServerRouterBuilder {
    router: axum::Router,
    state: fechatter_server::AppState,
  }

  impl ServerRouterBuilder {
    fn new(state: fechatter_server::AppState, router: axum::Router) -> Self {
      Self { router, state }
    }

    fn add_auth_debug_routes(self) -> Self {
      use axum::{Extension, Json, response::IntoResponse, routing::get};

      use fechatter_core::models::AuthUser;

      async fn debug_auth_handler(Extension(user): Extension<AuthUser>) -> impl IntoResponse {
        Json(serde_json::json!({
            "user_id": user.id,
            "workspace_id": user.workspace_id,
            "email": user.email,
            "fullname": user.fullname,
            "status": user.status,
        }))
      }

      // 新增: 测试API，确保用户的工作区存在
      async fn ensure_workspace_handler(
        Extension(user): Extension<AuthUser>,
        State(state): State<fechatter_server::AppState>,
      ) -> impl IntoResponse {
        println!("Ensuring workspace exists for user: {}", user.email);

        // 检查工作区是否存在
        let workspace = match state.find_by_id_with_pool(user.workspace_id).await {
          Ok(Some(workspace)) => {
            println!(
              "Workspace found: id={}, name={}",
              workspace.id, workspace.name
            );
            workspace
          }
          Ok(None) => {
            // 工作区不存在，创建一个
            println!(
              "Workspace id={} not found, creating a new one",
              user.workspace_id
            );
            let ws_name = format!("TestWorkspace-{}", user.id);
            match state.create_workspace_with_pool(&ws_name, user.id).await {
              Ok(new_ws) => {
                println!(
                  "Created new workspace: id={}, name={}",
                  new_ws.id, new_ws.name
                );
                new_ws
              }
              Err(e) => {
                println!("Error creating workspace: {:?}", e);
                return (
                  ReqwestStatusCode::INTERNAL_SERVER_ERROR,
                  Json(serde_json::json!({
                      "error": "Failed to create workspace",
                      "details": format!("{:?}", e)
                  })),
                )
                  .into_response();
              }
            }
          }
          Err(e) => {
            println!("Database error: {:?}", e);
            return (
              ReqwestStatusCode::INTERNAL_SERVER_ERROR,
              Json(serde_json::json!({
                  "error": "Database error",
                  "details": format!("{:?}", e)
              })),
            )
              .into_response();
          }
        };

        // 返回工作区信息
        Json(serde_json::json!({
            "workspace_id": workspace.id,
            "workspace_name": workspace.name,
            "owner_id": workspace.owner_id,
            "message": "Workspace verified and ready"
        }))
        .into_response()
      }

      // Create a specific router for /api/debug/auth and apply middleware
      let debug_auth_specific_router = axum::Router::new()
        .route("/api/debug/auth", get(debug_auth_handler))
        .with_middlewares(self.state.clone())
        .with_auth()
        .build();

      // 新增: 确保工作区存在的路由
      let ensure_workspace_router = axum::Router::new()
        .route("/api/debug/ensure_workspace", get(ensure_workspace_handler))
        // 使用正确的RouterExt实现
        // 使用server的RouterExt而不是core的
        .with_state(self.state.clone())
        .with_middlewares(self.state.clone())
        .with_auth_refresh_workspace() // 一次性添加所有中间件，确保正确的顺序
        .build();

      let router = self
        .router
        .merge(debug_auth_specific_router)
        .merge(ensure_workspace_router) // 合并新路由
        .nest_service(
          "/api/debug/auth_chain",
          axum::Router::new()
            .route("/", get(debug_auth_handler))
            .with_middlewares(self.state.clone())
            .with_auth()
            .build()
        )
        .merge(
          axum::Router::new()
            .route("/api/debug/workspace", get(debug_auth_handler))
            .with_middlewares(self.state.clone())
            .with_auth()
            .build()
        );

      Self {
        router,
        state: self.state,
      }
    }

    fn build(self) -> axum::Router {
      self.router
    }
  }
}
