#[cfg(test)]
mod macros {
  /// 确保使用完全限定语法调用服务方法的宏
  ///
  /// 用法示例:
  /// ```
  /// #[test]
  /// fn test_service_call() {
  ///   let auth_service = get_auth_service();
  ///   
  ///   // 正确的调用方式 - 使用宏
  ///   call_service!(auth_service, RefreshTokenService, refresh_token, "token", None);
  ///   
  ///   // 不推荐的方式 - 直接调用，可能会调用到核心层占位符
  ///   // auth_service.refresh_token("token", None).await;
  /// }
  /// ```
  #[macro_export]
  macro_rules! call_service {
    // 对于返回Result的无参方法
    ($service:expr, $trait:path, $method:ident) => {
      <crate::services::auth_service::AuthService as $trait>::$method(&$service).await
    };

    // 对于返回Result的单参数方法
    ($service:expr, $trait:path, $method:ident, $arg1:expr) => {
      <crate::services::auth_service::AuthService as $trait>::$method(&$service, $arg1).await
    };

    // 对于返回Result的双参数方法
    ($service:expr, $trait:path, $method:ident, $arg1:expr, $arg2:expr) => {
      <crate::services::auth_service::AuthService as $trait>::$method(&$service, $arg1, $arg2).await
    };

    // 对于返回Result的三参数方法
    ($service:expr, $trait:path, $method:ident, $arg1:expr, $arg2:expr, $arg3:expr) => {
      <crate::services::auth_service::AuthService as $trait>::$method(
        &$service, $arg1, $arg2, $arg3,
      )
      .await
    };
  }

  /// 确保使用完全限定语法创建服务的宏
  #[macro_export]
  macro_rules! create_auth_service {
    ($state:expr) => {
      <crate::AppState as fechatter_core::middlewares::ActualAuthServiceProvider>::create_service(
        &$state,
      )
    };
  }

  /// 确保使用完全限定语法验证Token的宏
  #[macro_export]
  macro_rules! verify_token {
    ($state:expr, $token:expr) => {
      <crate::AppState as fechatter_core::middlewares::TokenVerifier>::verify_token(&$state, $token)
    };
  }

  /// 示例宏使用的测试
  #[cfg(test)]
  mod tests {
    use crate::{call_service, create_auth_service, setup_test_users, verify_token};
    use anyhow::Result;
    use fechatter_core::{RefreshTokenService, TokenService, models::jwt::UserClaims};

    #[tokio::test]
    async fn test_service_macros() -> Result<()> {
      let (_tdb, state, users) = setup_test_users!(1).await;
      let user = &users[0];

      // 使用宏创建服务
      let auth_service = create_auth_service!(state);

      // 创建测试令牌
      let user_claims = UserClaims {
        id: user.id,
        workspace_id: user.workspace_id.into(),
        fullname: user.fullname.clone(),
        email: user.email.clone(),
        status: user.status,
        created_at: user.created_at,
      };

      let tokens =
        <fechatter_core::models::jwt::TokenManager as TokenService>::generate_auth_tokens(
          state.token_manager(),
          &user_claims,
          None,
          None,
        )
        .await?;

      // 使用宏验证token
      let _claims = verify_token!(state, &tokens.access_token)?;

      // 使用宏调用服务方法
      let _result = call_service!(
        auth_service,
        RefreshTokenService,
        refresh_token,
        &tokens.refresh_token.token,
        None
      );

      Ok(())
    }
  }
}
