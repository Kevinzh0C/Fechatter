use async_trait::async_trait;
use sqlx::{Acquire, PgPool, Row};
use std::{mem, sync::Arc};

use crate::domains::workspace::repository::WorkspaceRepositoryImpl;
use fechatter_core::{
    contracts::UserRepository, error::CoreError, CreateUser, SigninUser, User, UserId, WorkspaceId,
};

use super::password::{hashed_password, verify_password};

/// User repository - 纯数据访问层
pub struct UserRepositoryImpl {
    pub pool: Arc<PgPool>,
    workspace_repo: Arc<WorkspaceRepositoryImpl>,
}

impl UserRepositoryImpl {
    pub fn new(pool: Arc<PgPool>) -> Self {
        let workspace_repo = Arc::new(WorkspaceRepositoryImpl::new(pool.clone()));
        Self {
            pool,
            workspace_repo,
        }
    }

    pub fn new_with_workspace_repo(
        pool: Arc<PgPool>,
        workspace_repo: Arc<WorkspaceRepositoryImpl>,
    ) -> Self {
        Self {
            pool,
            workspace_repo,
        }
    }
}

#[async_trait]
impl UserRepository for UserRepositoryImpl {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    async fn create(&self, input: &CreateUser) -> Result<User, CoreError> {
        // Check if email already exists
        let existing_user = self.exists_by_email(&input.email).await?;
        if existing_user {
            return Err(CoreError::UserAlreadyExists(format!(
                "User with email {} already exists",
                input.email
            )));
        }

        // Check if workspace exists (or create default)
        let workspace = self
            .workspace_repo
            .find_or_create_by_name(&input.workspace)
            .await?;

        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| CoreError::Internal(e.to_string()))?;

        let conn = tx
            .acquire()
            .await
            .map_err(|e| CoreError::Internal(e.to_string()))?;

        let mut is_new_workspace = false;
        if workspace.owner_id == UserId(0) {
            is_new_workspace = true;
        }

        let password_hash = hashed_password(&input.password)?;

        let user = sqlx::query_as::<_, User>(
            r#"
      INSERT INTO users (workspace_id, email, fullname, password_hash)
      VALUES ($1, $2, $3, $4)
      RETURNING id, fullname, email, password_hash, status, created_at, workspace_id,
                phone, title, department, avatar_url, bio, timezone, language, last_active_at
      "#,
        )
        .bind(workspace.id)
        .bind(&input.email)
        .bind(&input.fullname)
        .bind(password_hash)
        .fetch_one(&mut *conn)
        .await
        .map_err(|e| {
            if let Some(db_err) = e.as_database_error() {
                if db_err.is_unique_violation() {
                    CoreError::Validation(format!("User with email {} already exists", input.email))
                } else {
                    CoreError::Internal(e.to_string())
                }
            } else {
                CoreError::Internal(e.to_string())
            }
        })?;

        if is_new_workspace {
            let _res = sqlx::query("UPDATE workspaces SET owner_id = $1 WHERE id = $2")
                .bind(user.id)
                .bind(workspace.id)
                .execute(&mut *conn)
                .await
                .map_err(|e| CoreError::Internal(e.to_string()))?;
        }

        tx.commit()
            .await
            .map_err(|e| CoreError::Internal(e.to_string()))?;

        Ok(user)
    }

    async fn find_by_id(&self, id: UserId) -> Result<Option<User>, CoreError> {
        let user = sqlx::query_as::<_, User>(
            r#"SELECT id, fullname, email, password_hash, status, created_at, workspace_id,
         phone, title, department, avatar_url, bio, timezone, language, last_active_at 
         FROM users WHERE id = $1"#,
        )
        .bind(i64::from(id))
        .fetch_optional(&*self.pool)
        .await
        .map_err(|e| CoreError::Database(e.to_string()))?;

        Ok(user)
    }

    async fn find_by_email(&self, email: &str) -> Result<Option<User>, CoreError> {
        let user = sqlx::query_as::<_, User>(
            r#"SELECT id, fullname, email, password_hash, status, created_at, workspace_id,
         phone, title, department, avatar_url, bio, timezone, language, last_active_at 
         FROM users WHERE email = $1"#,
        )
        .bind(email)
        .fetch_optional(&*self.pool)
        .await
        .map_err(|e| CoreError::Database(e.to_string()))?;

        Ok(user)
    }

    async fn authenticate(&self, credentials: &SigninUser) -> Result<Option<User>, CoreError> {
        let user = self.find_by_email(&credentials.email).await?;

        match user {
            Some(mut user) => {
                let password_hash = match mem::take(&mut user.password_hash) {
                    Some(h) => h,
                    None => return Ok(None),
                };

                let is_valid = verify_password(&credentials.password, &password_hash)?;
                if is_valid {
                    Ok(Some(user))
                } else {
                    Ok(None)
                }
            }
            None => Ok(None),
        }
    }

    async fn update(&self, id: UserId, user_data: &User) -> Result<User, CoreError> {
        let updated_user = sqlx::query_as::<_, User>(
            r#"UPDATE users 
         SET fullname = $1, email = $2, status = $3, updated_at = NOW()
         WHERE id = $4
         RETURNING id, fullname, email, password_hash, status, created_at, workspace_id,
         phone, title, department, avatar_url, bio, timezone, language, last_active_at"#,
        )
        .bind(&user_data.fullname)
        .bind(&user_data.email)
        .bind(&user_data.status)
        .bind(i64::from(id))
        .fetch_one(&*self.pool)
        .await
        .map_err(|e| CoreError::Database(e.to_string()))?;

        Ok(updated_user)
    }

    async fn exists_by_email(&self, email: &str) -> Result<bool, CoreError> {
        let user = self.find_by_email(email).await?;
        Ok(user.is_some())
    }
}

// Repository扩展方法 - 仅数据访问，不是trait方法
impl UserRepositoryImpl {
    /// 验证用户ID列表是否都存在
    pub async fn validate_users_exist_by_ids(&self, ids: &[UserId]) -> Result<(), CoreError> {
        let query = r#"
      SELECT COUNT(DISTINCT id) as count FROM users WHERE id = ANY($1)
    "#;

        let count: i64 = sqlx::query_scalar(query)
            .bind(&ids.iter().map(|id| i64::from(*id)).collect::<Vec<_>>())
            .fetch_one(&*self.pool)
            .await
            .map_err(|e| CoreError::Database(e.to_string()))?;

        if count != ids.len() as i64 {
            let missing_ids = ids
                .iter()
                .map(|id| i64::from(*id).to_string())
                .collect::<Vec<_>>()
                .join(", ");

            return Err(CoreError::NotFound(format!(
                "Some users not found. IDs: {}",
                missing_ids
            )));
        }

        Ok(())
    }

    /// 更新用户密码哈希
    pub async fn update_password_hash(
        &self,
        user_id: UserId,
        new_hash: String,
    ) -> Result<(), CoreError> {
        let _result =
            sqlx::query("UPDATE users SET password_hash = $1, updated_at = NOW() WHERE id = $2")
                .bind(new_hash)
                .bind(i64::from(user_id))
                .execute(&*self.pool)
                .await
                .map_err(|e| CoreError::Database(e.to_string()))?;

        Ok(())
    }

    /// 检查用户邮箱是否存在 (别名方法)
    pub async fn email_user_exists(&self, email: &str) -> Result<Option<User>, CoreError> {
        self.find_by_email(email).await
    }

    /// 根据ID查找用户 (公共方法)
    pub async fn find_by_id_ext(&self, id: UserId) -> Result<Option<User>, CoreError> {
        self.find_by_id(id).await
    }

    /// 切换工作空间
    pub async fn switch_workspace(
        &self,
        user_id: UserId,
        workspace_id: WorkspaceId,
    ) -> Result<User, CoreError> {
        let updated_user = sqlx::query_as::<_, User>(
            r#"UPDATE users 
         SET workspace_id = $1, updated_at = NOW()
         WHERE id = $2
         RETURNING id, fullname, email, password_hash, status, created_at, workspace_id,
                   phone, title, department, avatar_url, bio, timezone, language, last_active_at"#,
        )
        .bind(workspace_id)
        .bind(i64::from(user_id))
        .fetch_one(&*self.pool)
        .await
        .map_err(|e| CoreError::Database(e.to_string()))?;

        Ok(updated_user)
    }

    /// Update user profile fields
    pub async fn update_profile(
        &self,
        user_id: UserId,
        fullname: Option<&str>,
        email: Option<&str>,
        phone: Option<&str>,
        title: Option<&str>,
        department: Option<&str>,
        avatar_url: Option<&str>,
        bio: Option<&str>,
        timezone: Option<&str>,
        language: Option<&str>,
    ) -> Result<User, CoreError> {
        // Build dynamic query based on provided fields
        let mut query_parts = Vec::new();
        let mut bind_index = 1;

        if fullname.is_some() {
            query_parts.push(format!("fullname = ${}", bind_index));
            bind_index += 1;
        }

        if email.is_some() {
            query_parts.push(format!("email = ${}", bind_index));
            bind_index += 1;
        }

        if phone.is_some() {
            query_parts.push(format!("phone = ${}", bind_index));
            bind_index += 1;
        }

        if title.is_some() {
            query_parts.push(format!("title = ${}", bind_index));
            bind_index += 1;
        }

        if department.is_some() {
            query_parts.push(format!("department = ${}", bind_index));
            bind_index += 1;
        }

        if avatar_url.is_some() {
            query_parts.push(format!("avatar_url = ${}", bind_index));
            bind_index += 1;
        }

        if bio.is_some() {
            query_parts.push(format!("bio = ${}", bind_index));
            bind_index += 1;
        }

        if timezone.is_some() {
            query_parts.push(format!("timezone = ${}", bind_index));
            bind_index += 1;
        }

        if language.is_some() {
            query_parts.push(format!("language = ${}", bind_index));
            bind_index += 1;
        }

        query_parts.push("updated_at = NOW()".to_string());

        if query_parts.len() == 1 {
            // Only updated_at was added
            return self
                .find_by_id(user_id)
                .await?
                .ok_or_else(|| CoreError::NotFound("User not found".to_string()));
        }

        let query = format!(
            r#"UPDATE users SET {} WHERE id = ${} 
         RETURNING id, fullname, email, password_hash, status, created_at, workspace_id,
         phone, title, department, avatar_url, bio, timezone, language, last_active_at"#,
            query_parts.join(", "),
            bind_index
        );

        let mut query_builder = sqlx::query_as::<_, User>(&query);

        if let Some(name) = fullname {
            query_builder = query_builder.bind(name);
        }

        if let Some(mail) = email {
            query_builder = query_builder.bind(mail);
        }

        if let Some(ph) = phone {
            query_builder = query_builder.bind(ph);
        }

        if let Some(t) = title {
            query_builder = query_builder.bind(t);
        }

        if let Some(dept) = department {
            query_builder = query_builder.bind(dept);
        }

        if let Some(avatar) = avatar_url {
            query_builder = query_builder.bind(avatar);
        }

        if let Some(b) = bio {
            query_builder = query_builder.bind(b);
        }

        if let Some(tz) = timezone {
            query_builder = query_builder.bind(tz);
        }

        if let Some(lang) = language {
            query_builder = query_builder.bind(lang);
        }

        query_builder = query_builder.bind(i64::from(user_id));

        let updated_user = query_builder
            .fetch_one(&*self.pool)
            .await
            .map_err(|e| CoreError::Database(e.to_string()))?;

        Ok(updated_user)
    }

    /// Get user profile with extended information
    pub async fn get_user_profile(&self, user_id: UserId) -> Result<Option<User>, CoreError> {
        self.find_by_id(user_id).await
    }

    /// Get user settings
    pub async fn get_user_settings(
        &self,
        user_id: UserId,
    ) -> Result<Option<fechatter_core::models::UserSettings>, CoreError> {
        let settings = sqlx::query_as::<_, fechatter_core::models::UserSettings>(
            r#"SELECT user_id, email_notifications, push_notifications, desktop_notifications, 
         notification_sound, show_online_status, auto_away, auto_away_minutes, theme, 
         message_display, profile_visibility, show_email, show_phone, 
         custom_preferences, created_at, updated_at
         FROM user_settings WHERE user_id = $1"#,
        )
        .bind(i64::from(user_id))
        .fetch_optional(&*self.pool)
        .await
        .map_err(|e| CoreError::Database(e.to_string()))?;

        Ok(settings)
    }

    /// Update user settings
    pub async fn update_user_settings(
        &self,
        user_id: UserId,
        settings: &fechatter_core::models::UserSettings,
    ) -> Result<fechatter_core::models::UserSettings, CoreError> {
        let updated_settings = sqlx::query_as::<_, fechatter_core::models::UserSettings>(
            r#"UPDATE user_settings SET 
         email_notifications = $1, push_notifications = $2, desktop_notifications = $3,
         notification_sound = $4, show_online_status = $5, auto_away = $6, 
         auto_away_minutes = $7, theme = $8, message_display = $9,
         profile_visibility = $10, show_email = $11, show_phone = $12,
         custom_preferences = $13, updated_at = NOW()
         WHERE user_id = $14
         RETURNING user_id, email_notifications, push_notifications, desktop_notifications, 
         notification_sound, show_online_status, auto_away, auto_away_minutes, theme, 
         message_display, profile_visibility, show_email, show_phone, 
         custom_preferences, created_at, updated_at"#,
        )
        .bind(settings.email_notifications)
        .bind(settings.push_notifications)
        .bind(settings.desktop_notifications)
        .bind(&settings.notification_sound)
        .bind(settings.show_online_status)
        .bind(settings.auto_away)
        .bind(settings.auto_away_minutes)
        .bind(&settings.theme)
        .bind(&settings.message_display)
        .bind(&settings.profile_visibility)
        .bind(settings.show_email)
        .bind(settings.show_phone)
        .bind(&settings.custom_preferences)
        .bind(i64::from(user_id))
        .fetch_one(&*self.pool)
        .await
        .map_err(|e| CoreError::Database(e.to_string()))?;

        Ok(updated_settings)
    }

    /// Log user activity
    pub async fn log_user_activity(
        &self,
        user_id: UserId,
        activity_type: &str,
        description: Option<&str>,
        ip_address: Option<std::net::IpAddr>,
        user_agent: Option<&str>,
        metadata: Option<serde_json::Value>,
    ) -> Result<(), CoreError> {
        // Convert IpAddr to String for database storage
        let ip_string = ip_address.map(|ip| ip.to_string());

        sqlx::query(
      r#"INSERT INTO user_activity_log (user_id, activity_type, description, ip_address, user_agent, metadata)
         VALUES ($1, $2, $3, $4, $5, $6)"#,
    )
    .bind(i64::from(user_id))
    .bind(activity_type)
    .bind(description)
    .bind(ip_string)
    .bind(user_agent)
    .bind(metadata)
    .execute(&*self.pool)
    .await
    .map_err(|e| CoreError::Database(e.to_string()))?;

        Ok(())
    }

    // =============================================================================
    // WORKSPACE MANAGEMENT
    // =============================================================================

    /// Get all users in a workspace
    pub async fn get_workspace_users(
        &self,
        workspace_id: WorkspaceId,
    ) -> Result<
        Vec<(
            i64,
            String,
            String,
            Option<String>,
            Option<String>,
            Option<String>,
            chrono::DateTime<chrono::Utc>,
            chrono::DateTime<chrono::Utc>,
        )>,
        CoreError,
    > {
        let rows = sqlx::query(
            r#"
      SELECT 
        id,
        email,
        fullname,
        bio,
        avatar_url,
        status,
        created_at,
        updated_at
      FROM users
      WHERE workspace_id = $1
      ORDER BY fullname ASC, email ASC
      "#,
        )
        .bind(i64::from(workspace_id))
        .fetch_all(&*self.pool)
        .await
        .map_err(|e| CoreError::Database(e.to_string()))?;

        let users = rows
            .into_iter()
            .map(|row| {
                (
                    row.get("id"),
                    row.get("email"),
                    row.get("fullname"),
                    row.get("bio"),
                    row.get("avatar_url"),
                    row.get("status"),
                    row.get("created_at"),
                    row.get::<Option<chrono::DateTime<chrono::Utc>>, _>("updated_at")
                        .unwrap_or_else(|| row.get("created_at")),
                )
            })
            .collect();

        Ok(users)
    }
}
