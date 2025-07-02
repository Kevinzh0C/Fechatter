use fechatter_core::models::UserStatus;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Complete user profile response
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct UserProfileResponse {
    pub id: i64,
    pub fullname: String,
    pub email: String,
    pub status: UserStatus,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub workspace_id: i64,
    pub phone: Option<String>,
    pub title: Option<String>,
    pub department: Option<String>,
    pub avatar_url: Option<String>,
    pub bio: Option<String>,
    pub timezone: Option<String>,
    pub language: Option<String>,
    pub last_active_at: Option<chrono::DateTime<chrono::Utc>>,
    pub settings: Option<UserSettingsResponse>,
}

/// User settings response
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct UserSettingsResponse {
    pub email_notifications: bool,
    pub push_notifications: bool,
    pub desktop_notifications: bool,
    pub notification_sound: String,
    pub show_online_status: bool,
    pub auto_away: bool,
    pub auto_away_minutes: i32,
    pub theme: String,
    pub message_display: String,
}

/// Basic user info response (for lists and search)
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct UserInfoResponse {
    pub id: i64,
    pub fullname: String,
    pub email: String,
    pub avatar_url: Option<String>,
    pub title: Option<String>,
    pub department: Option<String>,
    pub status: UserStatus,
    pub last_active_at: Option<chrono::DateTime<chrono::Utc>>,
}

/// Profile update success response
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ProfileUpdateResponse {
    pub success: bool,
    pub message: String,
    pub updated_fields: Vec<String>,
    pub profile: UserProfileResponse,
}
