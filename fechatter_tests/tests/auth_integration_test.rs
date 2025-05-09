#[cfg(test)]
mod auth_integration_tests {
  use anyhow::Result;
  use axum::{
    Json, Router,
    body::{Body, HttpBody},
    extract::Extension,
    http::{Method, Request, StatusCode, header},
    response::{IntoResponse, Response},
  };
  use axum_extra::extract::cookie::Cookie;
  use fechatter_core::middlewares::custom_builder::CoreBuilder;
  use fechatter_core::{AuthTokens, CreateUser, SigninUser, services::AuthContext};
  use fechatter_server::{AppConfig, AppState};
  use http::HeaderValue;
  use hyper::body::to_bytes;
  use serde::{Deserialize, Serialize};
  use serde_json::{Value, json};
  use tower::ServiceExt;

  // 与服务器AuthResponse保持一致的结构
  #[derive(Debug, Serialize, Deserialize)]
  struct AuthResponse {
    access_token: String,
    expires_in: usize,
    refresh_token: Option<String>,
  }

  // Helper to create test data
  fn get_test_user() -> CreateUser {
    CreateUser {
      email: format!("test_user_{}@example.com", uuid::Uuid::new_v4()),
      fullname: "Test User".to_string(),
      password: "password123".to_string(),
      workspace: "TestWorkspace".to_string(),
    }
  }

  // 创建测试环境 - 返回AppState和Router
  async fn setup_test_environment() -> (AppState, Router) {
    // 创建配置
    let config = AppConfig::load().expect("Failed to load config");

    // 创建AppState
    let state = AppState::try_new(config)
      .await
      .expect("Failed to create AppState");

    // 创建完整的路由
    // 对于测试，我们创建自定义的路由和处理函数，而不是依赖于私有的handler
    let public_routes = Router::new()
      .route(
        "/signin",
        axum::routing::post(
          |axum::extract::State(state): axum::extract::State<AppState>,
           headers: axum::http::HeaderMap,
           Json(payload): Json<SigninUser>| async move {
            // 从请求头获取用户代理和IP
            let user_agent = headers
              .get(header::USER_AGENT)
              .and_then(|v| v.to_str().ok())
              .map(String::from);
            let auth_context = Some(AuthContext {
              user_agent,
              ip_address: Some("127.0.0.1".to_string()),
            });

            // 调用State的signin方法
            match state.signin(&payload, auth_context).await {
              Ok(Some(tokens)) => (StatusCode::OK, Json(tokens)),
              Ok(None) => {
                let error_json: Value = json!({"error": "Invalid credentials"});
                (StatusCode::UNAUTHORIZED, Json(error_json))
              }
              Err(e) => {
                let error_json: Value = json!({"error": e.to_string()});
                (StatusCode::INTERNAL_SERVER_ERROR, Json(error_json))
              }
            }
          },
        ),
      )
      .route(
        "/signup",
        axum::routing::post(
          |axum::extract::State(state): axum::extract::State<AppState>,
           headers: axum::http::HeaderMap,
           Json(payload): Json<CreateUser>| async move {
            // 从请求头获取用户代理和IP
            let user_agent = headers
              .get(header::USER_AGENT)
              .and_then(|v| v.to_str().ok())
              .map(String::from);
            let auth_context = Some(AuthContext {
              user_agent,
              ip_address: Some("127.0.0.1".to_string()),
            });

            // 调用State的signup方法
            match state.signup(&payload, auth_context).await {
              Ok(tokens) => (StatusCode::OK, Json(tokens)),
              Err(e) => {
                let error_json: Value = json!({"error": e.to_string()});
                (StatusCode::INTERNAL_SERVER_ERROR, Json(error_json))
              }
            }
          },
        ),
      )
      .route(
        "/refresh",
        axum::routing::post(
          |axum::extract::State(state): axum::extract::State<AppState>,
           headers: axum::http::HeaderMap,
           cookies: axum_extra::extract::cookie::CookieJar| async move {
            // 从Cookie获取refresh_token
            let refresh_token = cookies.get("refresh_token").map(|c| c.value().to_string());

            if let Some(token) = refresh_token {
              // 从请求头获取用户代理和IP
              let user_agent = headers
                .get(header::USER_AGENT)
                .and_then(|v| v.to_str().ok())
                .map(String::from);
              let auth_context = Some(AuthContext {
                user_agent,
                ip_address: Some("127.0.0.1".to_string()),
              });

              // 调用State的refresh_token方法
              match state.refresh_token(&token, auth_context).await {
                Ok(tokens) => {
                  let mut response = Json(tokens).into_response();

                  // 设置新的refresh_token cookie
                  let cookie = format!(
                    "refresh_token={}; Path=/; HttpOnly; SameSite=Strict",
                    tokens.refresh_token
                  );
                  response
                    .headers_mut()
                    .insert(header::SET_COOKIE, HeaderValue::from_str(&cookie).unwrap());

                  response
                }
                Err(e) => {
                  // 删除无效的refresh_token cookie
                  let mut response = Json(json!({"error": e.to_string()})).into_response();
                  response.headers_mut().insert(
                    header::SET_COOKIE,
                    HeaderValue::from_str(
                      "refresh_token=; Path=/; HttpOnly; SameSite=Strict; Max-Age=0",
                    )
                    .unwrap(),
                  );

                  (StatusCode::UNAUTHORIZED, response).into_response()
                }
              }
            } else {
              (
                StatusCode::UNAUTHORIZED,
                Json(json!({"error": "No refresh token"})),
              )
                .into_response()
            }
          },
        ),
      );

    // 认证路由
    let auth_routes = Router::new()
      .route(
        "/logout",
        axum::routing::post(
          |axum::extract::State(state): axum::extract::State<AppState>,
           cookies: axum_extra::extract::cookie::CookieJar| async move {
            // 从Cookie获取refresh_token
            let refresh_token = cookies.get("refresh_token").map(|c| c.value().to_string());

            if let Some(token) = refresh_token {
              // 调用State的logout方法
              match state.logout(&token).await {
                Ok(_) => {
                  // 删除refresh_token cookie
                  let mut response = Json(json!({"status": "success"})).into_response();
                  response.headers_mut().insert(
                    header::SET_COOKIE,
                    HeaderValue::from_str(
                      "refresh_token=; Path=/; HttpOnly; SameSite=Strict; Max-Age=0",
                    )
                    .unwrap(),
                  );

                  response
                }
                Err(e) => (
                  StatusCode::INTERNAL_SERVER_ERROR,
                  Json(json!({"error": e.to_string()})),
                )
                  .into_response(),
              }
            } else {
              (
                StatusCode::BAD_REQUEST,
                Json(json!({"error": "No refresh token"})),
              )
                .into_response()
            }
          },
        ),
      )
      .route(
        "/logout_all",
        axum::routing::post(
          |axum::extract::State(state): axum::extract::State<AppState>,
           Extension(user): Extension<fechatter_core::AuthUser>| async move {
            // 调用State的logout_all方法
            match state.logout_all(user.id).await {
              Ok(_) => {
                // 删除refresh_token cookie
                let mut response = Json(json!({"status": "success"})).into_response();
                response.headers_mut().insert(
                  header::SET_COOKIE,
                  HeaderValue::from_str(
                    "refresh_token=; Path=/; HttpOnly; SameSite=Strict; Max-Age=0",
                  )
                  .unwrap(),
                );

                response
              }
              Err(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": e.to_string()})),
              )
                .into_response(),
            }
          },
        ),
      );

    // Public routes不用添加认证中间件
    let public_router = public_routes.with_state(state.clone());

    // Auth routes需要添加认证中间件
    let auth_router = CoreBuilder::new(auth_routes, state.clone())
      .with_auth()
      .with_token_refresh()
      .build();

    // 合并路由
    let app = Router::new().merge(public_router).merge(auth_router);

    (state, app)
  }

  // 帮助函数 - 从响应体中提取JSON数据
  async fn get_response_json(response: Response) -> Result<Value> {
    let body = to_bytes(response.into_body()).await?.to_vec();
    Ok(serde_json::from_slice(&body)?)
  }

  // Parse AuthResponse from JSON
  fn parse_auth_response(json_value: &Value) -> Result<AuthResponse> {
    Ok(serde_json::from_value(json_value.clone())?)
  }

  // 帮助函数 - 提取cookie
  fn get_cookie<'a>(response: &'a Response, name: &str) -> Option<Cookie<'a>> {
    response
      .headers()
      .get_all(header::SET_COOKIE)
      .iter()
      .filter_map(|v| v.to_str().ok())
      .filter_map(|s| Cookie::parse(s).ok())
      .find(|c| c.name() == name)
  }

  #[tokio::test]
  async fn test_full_auth_flow_integration() -> Result<()> {
    let (state, app) = setup_test_environment().await;

    // 创建测试用户
    let test_user = get_test_user();

    // 1. 测试注册
    let signup_request = Request::builder()
      .uri("/signup")
      .method(Method::POST)
      .header("Content-Type", "application/json")
      .body(Body::from(serde_json::to_string(&test_user)?))?;

    let signup_response = app.clone().oneshot(signup_request).await?;
    assert_eq!(signup_response.status(), StatusCode::OK);

    let signup_json = get_response_json(signup_response).await?;
    let signup_auth = parse_auth_response(&signup_json)?;

    assert!(
      !signup_auth.access_token.is_empty(),
      "返回数据不包含access_token"
    );
    assert_eq!(
      signup_auth.expires_in,
      fechatter_core::models::jwt::ACCESS_TOKEN_EXPIRATION,
      "expires_in字段不正确"
    );

    // 获取access_token和refresh_token
    let access_token = signup_auth.access_token;
    let refresh_token = signup_auth.refresh_token.expect("应当返回refresh_token");

    // 2. 测试登录
    let signin_request = Request::builder()
      .uri("/signin")
      .method(Method::POST)
      .header("Content-Type", "application/json")
      .body(Body::from(serde_json::to_string(&SigninUser::new(
        &test_user.email,
        &test_user.password,
      ))?))?;

    let signin_response = app.clone().oneshot(signin_request).await?;
    assert_eq!(signin_response.status(), StatusCode::OK);

    let signin_json = get_response_json(signin_response).await?;
    let signin_auth = parse_auth_response(&signin_json)?;

    assert!(
      !signin_auth.access_token.is_empty(),
      "返回数据不包含access_token"
    );
    assert_eq!(
      signin_auth.expires_in,
      fechatter_core::models::jwt::ACCESS_TOKEN_EXPIRATION,
      "expires_in字段不正确"
    );

    // 获取新的access_token和refresh_token
    let signin_access_token = signin_auth.access_token;
    let signin_refresh_token = signin_auth.refresh_token.expect("应当返回refresh_token");

    // 3. 测试刷新token
    let refresh_request = Request::builder()
      .uri("/refresh")
      .method(Method::POST)
      .header(
        header::COOKIE,
        format!("refresh_token={}", signin_refresh_token),
      )
      .header("Content-Type", "application/json")
      .body(Body::empty())?;

    let refresh_response = app.clone().oneshot(refresh_request).await?;
    assert_eq!(refresh_response.status(), StatusCode::OK);

    let refresh_json = get_response_json(refresh_response).await?;
    let refresh_auth = parse_auth_response(&refresh_json)?;

    assert!(
      !refresh_auth.access_token.is_empty(),
      "返回数据不包含新的access_token"
    );
    assert_eq!(
      refresh_auth.expires_in,
      fechatter_core::models::jwt::ACCESS_TOKEN_EXPIRATION,
      "expires_in字段不正确"
    );

    // 获取新的access_token和refresh_token
    let new_access_token = refresh_auth.access_token;
    let new_refresh_token = refresh_auth.refresh_token.expect("应当返回refresh_token");

    // 4. 使用新token访问受保护资源
    let auth_request = Request::builder()
      .uri("/logout_all")
      .method(Method::POST)
      .header(
        header::AUTHORIZATION,
        format!("Bearer {}", new_access_token),
      )
      .header(
        header::COOKIE,
        format!("refresh_token={}", new_refresh_token),
      )
      .body(Body::empty())?;

    let auth_response = app.clone().oneshot(auth_request).await?;
    assert_eq!(auth_response.status(), StatusCode::OK);

    // 5. 测试没有token的未授权请求
    let unauth_request = Request::builder()
      .uri("/logout_all")
      .method(Method::POST)
      .body(Body::empty())?;

    let unauth_response = app.clone().oneshot(unauth_request).await?;
    assert_eq!(
      unauth_response.status(),
      StatusCode::UNAUTHORIZED,
      "未授权请求应返回401状态码"
    );

    Ok(())
  }

  #[tokio::test]
  async fn test_middleware_auth_chain() -> Result<()> {
    let (state, app) = setup_test_environment().await;

    // 创建测试用户
    let test_user = get_test_user();
    let signup_request = Request::builder()
      .uri("/signup")
      .method(Method::POST)
      .header("Content-Type", "application/json")
      .body(Body::from(serde_json::to_string(&test_user)?))?;

    let signup_response = app.clone().oneshot(signup_request).await?;
    assert_eq!(signup_response.status(), StatusCode::OK);

    let signup_json = get_response_json(signup_response).await?;
    let signup_auth = parse_auth_response(&signup_json)?;

    let access_token = signup_auth.access_token;
    let refresh_token = signup_auth.refresh_token.expect("应当返回refresh_token");

    // 测试认证中间件链
    let auth_request = Request::builder()
      .uri("/logout_all")
      .method(Method::POST)
      .header(header::AUTHORIZATION, format!("Bearer {}", access_token))
      .header(header::COOKIE, format!("refresh_token={}", refresh_token))
      .body(Body::empty())?;

    let auth_response = app.oneshot(auth_request).await?;
    assert_eq!(auth_response.status(), StatusCode::OK);

    Ok(())
  }
}
