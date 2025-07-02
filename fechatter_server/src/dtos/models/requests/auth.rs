use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::{Validate, ValidationError};

/// 用户登录请求
#[derive(Debug, Serialize, Deserialize, Validate, ToSchema)]
pub struct LoginRequest {
    #[validate(email(message = "Invalid email format"))]
    #[schema(example = "user@example.com")]
    pub email: String,

    #[validate(length(min = 6, message = "Password must be at least 6 characters"))]
    #[schema(example = "password123")]
    pub password: String,

    #[schema(example = "web")]
    pub device_type: Option<String>,
}

/// 用户注册请求
#[derive(Debug, Serialize, Deserialize, Validate, ToSchema)]
pub struct RegisterRequest {
    #[validate(email(message = "Invalid email format"))]
    #[schema(example = "newuser@example.com")]
    pub email: String,

    #[validate(length(min = 6, message = "Password must be at least 6 characters"))]
    #[schema(example = "password123")]
    pub password: String,

    #[validate(length(
        min = 2,
        max = 50,
        message = "Full name must be between 2 and 50 characters"
    ))]
    #[schema(example = "John Doe")]
    pub fullname: String,

    #[schema(example = "My Company")]
    pub workspace_name: Option<String>,
}

/// 刷新令牌请求
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct RefreshTokenRequest {
    #[schema(example = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...")]
    pub refresh_token: String,
}

/// 忘记密码请求
#[derive(Debug, Serialize, Deserialize, Validate, ToSchema)]
pub struct ForgotPasswordRequest {
    #[validate(email(message = "Invalid email format"))]
    #[schema(example = "user@example.com")]
    pub email: String,
}

/// 重置密码请求
#[derive(Debug, Serialize, Deserialize, Validate, ToSchema)]
pub struct ResetPasswordRequest {
    #[schema(example = "reset_token_here")]
    pub token: String,

    #[validate(length(min = 6, message = "Password must be at least 6 characters"))]
    #[schema(example = "newpassword123")]
    pub new_password: String,
}

/// 修改密码请求
#[derive(Debug, Serialize, Deserialize, Validate, ToSchema)]
pub struct ChangePasswordRequest {
    #[validate(length(min = 6, message = "Current password is required"))]
    #[schema(example = "oldpassword123")]
    pub current_password: String,

    #[validate(length(min = 6, message = "New password must be at least 6 characters"))]
    #[schema(example = "newpassword123")]
    pub new_password: String,
}

/// 验证邮箱请求
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct VerifyEmailRequest {
    #[schema(example = "verification_token_here")]
    pub token: String,
}

/// 自定义验证器：检查密码强度
fn validate_password_strength(password: &str) -> Result<(), ValidationError> {
    let has_upper = password.chars().any(|c| c.is_uppercase());
    let has_lower = password.chars().any(|c| c.is_lowercase());
    let has_digit = password.chars().any(|c| c.is_numeric());

    if !has_upper || !has_lower || !has_digit {
        return Err(ValidationError::new("password_strength"));
    }

    Ok(())
}

/// 强密码注册请求（可选的更严格版本）
#[derive(Debug, Serialize, Deserialize, Validate, ToSchema)]
pub struct SecureRegisterRequest {
    #[validate(email(message = "Invalid email format"))]
    #[schema(example = "newuser@example.com")]
    pub email: String,

    #[validate(
        length(min = 8, message = "Password must be at least 8 characters"),
        custom(function = "validate_password_strength")
    )]
    #[schema(example = "SecurePass123")]
    pub password: String,

    #[validate(length(
        min = 2,
        max = 50,
        message = "Full name must be between 2 and 50 characters"
    ))]
    #[schema(example = "John Doe")]
    pub fullname: String,

    #[schema(example = "My Company")]
    pub workspace_name: Option<String>,

    #[schema(example = true)]
    pub accept_terms: bool,
}
