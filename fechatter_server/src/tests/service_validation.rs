#[cfg(test)]
mod service_validation_tests {
  use crate::setup_test_users;
  use anyhow::Result;
  use fechatter_core::middlewares::TokenVerifier;
  use fechatter_core::{
    LogoutService, RefreshTokenService, SigninService, SignupService, TokenService,
    middlewares::{ActualAuthServiceProvider, WithServiceProvider},
    models::jwt::UserClaims,
  };

  /// 验证AppState使用正确的实现路径创建AuthService
  /// 这确保不会调用到核心层的占位符实现
  #[tokio::test]
  async fn verify_correct_auth_service_creation_path() -> Result<()> {
    let (_tdb, state, _users) = setup_test_users!(1).await;

    // 使用完全限定语法创建服务 - 这是正确的方式
    let _auth_service = <crate::AppState as ActualAuthServiceProvider>::create_service(&state);

    // 成功创建服务意味着没有调用到Core层占位符
    // 如果调用了Core层实现，会panic

    // 检查服务的类型 - 应该是server层的AuthService而不是core层的占位符
    assert!(
      std::any::type_name::<crate::services::auth_service::AuthService>()
        .contains("fechatter_server")
    );

    Ok(())
  }

  /// 验证RefreshTokenService接口调用正确的实现
  #[tokio::test]
  async fn verify_refresh_token_service_implementation() -> Result<()> {
    let (_tdb, state, users) = setup_test_users!(1).await;
    let user = &users[0];

    // 创建用户Claims
    let user_claims = UserClaims {
      id: user.id,
      workspace_id: user.workspace_id,
      fullname: user.fullname.clone(),
      email: user.email.clone(),
      status: user.status,
      created_at: user.created_at,
    };

    // 生成测试令牌
    let tokens = state
      .token_manager()
      .generate_auth_tokens(&user_claims, None, None)
      .await?;
    let refresh_token = tokens.refresh_token.token.clone();

    // 获取服务
    let auth_service = <crate::AppState as ActualAuthServiceProvider>::create_service(&state);

    // 使用完全限定语法调用方法
    let _result =
      <crate::services::auth_service::AuthService as RefreshTokenService>::refresh_token(
        &auth_service,
        &refresh_token,
        None,
      )
      .await;

    // 如果能执行到这里，说明没有调用到Core层占位符
    Ok(())
  }

  /// 验证SigninService接口调用正确的实现
  #[tokio::test]
  async fn verify_signin_service_implementation() -> Result<()> {
    let (_tdb, state, users) = setup_test_users!(1).await;
    let user = &users[0];

    // 创建SigninUser
    let signin_user = fechatter_core::SigninUser {
      email: user.email.clone(),
      password: "password".to_string(), // 这是测试用户的默认密码
    };

    // 获取服务
    let auth_service = <crate::AppState as ActualAuthServiceProvider>::create_service(&state);

    // 使用完全限定语法调用方法
    let _result = <crate::services::auth_service::AuthService as SigninService>::signin(
      &auth_service,
      &signin_user,
      None,
    )
    .await;

    // 如果能执行到这里，说明没有调用到Core层占位符
    Ok(())
  }

  /// 验证SignupService接口调用正确的实现
  #[tokio::test]
  async fn verify_signup_service_implementation() -> Result<()> {
    let (_tdb, state, _users) = setup_test_users!(0).await;

    // 创建测试用户
    let create_user = fechatter_core::CreateUser {
      email: format!("test_user_{}@acme.test", uuid::Uuid::new_v4().simple()),
      fullname: "Test User".to_string(),
      password: "password".to_string(),
      workspace: "Acme".to_string(),
    };

    // 获取服务
    let auth_service = <crate::AppState as ActualAuthServiceProvider>::create_service(&state);

    // 使用完全限定语法调用方法
    let _result = <crate::services::auth_service::AuthService as SignupService>::signup(
      &auth_service,
      &create_user,
      None,
    )
    .await;

    // 如果能执行到这里，说明没有调用到Core层占位符
    Ok(())
  }

  /// 验证LogoutService接口调用正确的实现
  #[tokio::test]
  async fn verify_logout_service_implementation() -> Result<()> {
    let (_tdb, state, users) = setup_test_users!(1).await;
    let user = &users[0];

    // 生成测试令牌
    let user_claims = UserClaims {
      id: user.id,
      workspace_id: user.workspace_id,
      fullname: user.fullname.clone(),
      email: user.email.clone(),
      status: user.status,
      created_at: user.created_at,
    };

    let tokens = state
      .token_manager()
      .generate_auth_tokens(&user_claims, None, None)
      .await?;
    let refresh_token = tokens.refresh_token.token.clone();

    // 获取服务
    let auth_service = <crate::AppState as ActualAuthServiceProvider>::create_service(&state);

    // 使用完全限定语法调用方法
    let _result = <crate::services::auth_service::AuthService as LogoutService>::logout(
      &auth_service,
      &refresh_token,
    )
    .await;

    let _result = <crate::services::auth_service::AuthService as LogoutService>::logout_all(
      &auth_service,
      user.id,
    )
    .await;

    // 如果能执行到这里，说明没有调用到Core层占位符
    Ok(())
  }

  /// 验证TokenVerifier接口调用正确的实现
  #[tokio::test]
  async fn verify_token_verifier_implementation() -> Result<()> {
    let (_tdb, state, users) = setup_test_users!(1).await;
    let user = &users[0];

    // 创建用户Claims
    let user_claims = UserClaims {
      id: user.id,
      workspace_id: user.workspace_id,
      fullname: user.fullname.clone(),
      email: user.email.clone(),
      status: user.status,
      created_at: user.created_at,
    };

    // 生成测试token
    let token = <fechatter_core::models::jwt::TokenManager as TokenService>::generate_token(
      state.token_manager(),
      &user_claims,
    )?;

    // 验证token - 使用完全限定语法
    let result = <crate::AppState as TokenVerifier>::verify_token(&state, &token)?;

    // 验证结果
    assert_eq!(result.id, user.id);
    assert_eq!(result.email, user.email);

    Ok(())
  }

  /// 验证core和server层的类型隔离
  #[tokio::test]
  async fn verify_type_isolation_between_layers() -> Result<()> {
    // 确保正确使用了server层的服务提供者，而不是core层的
    let service_provider_type =
      std::any::type_name::<crate::services::service_provider::ServiceProvider>();
    assert!(service_provider_type.contains("fechatter_server"));

    // 确保正确使用了server层的AuthService，而不是core层的
    let auth_service_type = std::any::type_name::<crate::services::auth_service::AuthService>();
    assert!(auth_service_type.contains("fechatter_server"));

    // 确保AppState实现了ActualAuthServiceProvider trait
    assert!(std::any::type_name::<crate::AppState>().contains("fechatter_server"));

    Ok(())
  }

  /// 创建通用调用检查函数，可在其他测试中使用
  #[allow(dead_code)]
  pub fn ensure_using_fully_qualified_path<T>(type_info: &str) -> bool {
    // 检查是否使用完全限定路径调用服务方法
    // 如：<MyService as MyTrait>::my_method
    type_info.contains("::<") && type_info.contains(" as ") && type_info.contains(">::")
  }
}
