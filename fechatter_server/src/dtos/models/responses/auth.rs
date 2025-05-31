use fechatter_core::{User, Workspace};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// 登录响应
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct LoginResponse {
  #[schema(example = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...")]
  pub access_token: String,

  #[schema(example = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...")]
  pub refresh_token: String,

  #[schema(example = "Bearer")]
  pub token_type: String,

  #[schema(example = 3600)]
  pub expires_in: i64, // seconds

  pub user: User,

  pub workspace: Workspace,

  #[schema(example = "2024-01-01T12:00:00Z")]
  pub login_time: chrono::DateTime<chrono::Utc>,
}

/// 注册响应
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct RegisterResponse {
  pub user: User,

  pub workspace: Option<Workspace>,

  #[schema(example = "User registered successfully. Please check your email for verification.")]
  pub message: String,

  #[schema(example = true)]
  pub email_verification_required: bool,

  #[schema(example = "2024-01-01T12:00:00Z")]
  pub created_at: chrono::DateTime<chrono::Utc>,
}

/// 刷新令牌响应
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct RefreshTokenResponse {
  #[schema(example = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...")]
  pub access_token: String,

  #[schema(example = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...")]
  pub refresh_token: Option<String>, // 可选的新刷新令牌

  #[schema(example = "Bearer")]
  pub token_type: String,

  #[schema(example = 3600)]
  pub expires_in: i64,

  #[schema(example = "2024-01-01T12:00:00Z")]
  pub issued_at: chrono::DateTime<chrono::Utc>,
}

/// 忘记密码响应
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ForgotPasswordResponse {
  #[schema(example = "Password reset email sent successfully")]
  pub message: String,

  #[schema(example = "user@example.com")]
  pub email: String,

  #[schema(example = "2024-01-01T13:00:00Z")]
  pub reset_link_expires_at: chrono::DateTime<chrono::Utc>,

  #[schema(example = "2024-01-01T12:00:00Z")]
  pub sent_at: chrono::DateTime<chrono::Utc>,
}

/// 重置密码响应
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ResetPasswordResponse {
  #[schema(example = "Password reset successfully")]
  pub message: String,

  #[schema(example = true)]
  pub success: bool,

  #[schema(example = "2024-01-01T12:00:00Z")]
  pub reset_at: chrono::DateTime<chrono::Utc>,
}

/// 修改密码响应
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ChangePasswordResponse {
  #[schema(example = "Password changed successfully")]
  pub message: String,

  #[schema(example = true)]
  pub success: bool,

  #[schema(example = "2024-01-01T12:00:00Z")]
  pub changed_at: chrono::DateTime<chrono::Utc>,

  #[schema(example = true)]
  pub logout_other_sessions: bool,
}

/// 邮箱验证响应
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct VerifyEmailResponse {
  #[schema(example = "Email verified successfully")]
  pub message: String,

  #[schema(example = true)]
  pub success: bool,

  pub user: User,

  #[schema(example = "2024-01-01T12:00:00Z")]
  pub verified_at: chrono::DateTime<chrono::Utc>,
}

/// 登出响应
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct LogoutResponse {
  #[schema(example = "Logged out successfully")]
  pub message: String,

  #[schema(example = true)]
  pub success: bool,

  #[schema(example = "2024-01-01T12:00:00Z")]
  pub logout_time: chrono::DateTime<chrono::Utc>,
}

/// 用户会话信息响应
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct SessionInfoResponse {
  pub user: User,

  pub workspace: Workspace,

  #[schema(example = "2024-01-01T12:00:00Z")]
  pub login_time: chrono::DateTime<chrono::Utc>,

  #[schema(example = "2024-01-01T13:00:00Z")]
  pub expires_at: chrono::DateTime<chrono::Utc>,

  #[schema(example = "192.168.1.100")]
  pub ip_address: Option<String>,

  #[schema(example = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")]
  pub user_agent: Option<String>,

  #[schema(example = "web")]
  pub device_type: Option<String>,
}

/// 活跃会话列表响应
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ActiveSessionsResponse {
  pub sessions: Vec<SessionInfo>,

  #[schema(example = 3)]
  pub total_sessions: i32,

  #[schema(example = "sess_current_123")]
  pub current_session_id: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct SessionInfo {
  #[schema(example = "sess_123456")]
  pub session_id: String,

  #[schema(example = "2024-01-01T12:00:00Z")]
  pub created_at: chrono::DateTime<chrono::Utc>,

  #[schema(example = "2024-01-01T12:30:00Z")]
  pub last_activity: chrono::DateTime<chrono::Utc>,

  #[schema(example = "192.168.1.100")]
  pub ip_address: String,

  #[schema(example = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")]
  pub user_agent: String,

  #[schema(example = "web")]
  pub device_type: String,

  #[schema(example = "New York, NY")]
  pub location: Option<String>,

  #[schema(example = true)]
  pub is_current: bool,
}

/// 撤销会话响应
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct RevokeSessionResponse {
  #[schema(example = "Session revoked successfully")]
  pub message: String,

  #[schema(example = "sess_123456")]
  pub revoked_session_id: String,

  #[schema(example = "2024-01-01T12:00:00Z")]
  pub revoked_at: chrono::DateTime<chrono::Utc>,
}
