use async_trait::async_trait;
use std::sync::Arc;

use fechatter_core::contracts::UserRepository;
use fechatter_core::{error::CoreError, User, UserId};

use super::{
    password::{hashed_password, verify_password},
    repository::UserRepositoryImpl,
};

/// 用户域服务 - 业务逻辑层
#[async_trait]
pub trait UserDomainService: Send + Sync {
    /// 修改用户密码（带当前密码验证）
    async fn change_password(
        &self,
        user_id: UserId,
        current_password: &str,
        new_password: &str,
    ) -> Result<(), CoreError>;

    /// 更新用户资料
    async fn update_profile(&self, user_id: UserId, fullname: &str) -> Result<User, CoreError>;

    /// 验证用户是否存在
    async fn validate_users_exist(&self, user_ids: &[UserId]) -> Result<(), CoreError>;
}

#[derive(Debug, Clone)]
pub struct UserConfig {
    pub min_password_length: usize,
    pub max_password_length: usize,
    pub min_fullname_length: usize,
    pub max_fullname_length: usize,
}

impl Default for UserConfig {
    fn default() -> Self {
        Self {
            min_password_length: 8,
            max_password_length: 128,
            min_fullname_length: 1,
            max_fullname_length: 100,
        }
    }
}

pub struct UserDomainServiceImpl {
    repository: Arc<UserRepositoryImpl>,
    config: UserConfig,
}

impl UserDomainServiceImpl {
    pub fn new(repository: Arc<UserRepositoryImpl>, config: UserConfig) -> Self {
        Self { repository, config }
    }

    /// 验证密码强度
    fn validate_password(&self, password: &str) -> Result<(), CoreError> {
        if password.len() < self.config.min_password_length {
            return Err(CoreError::Validation(format!(
                "Password must be at least {} characters long",
                self.config.min_password_length
            )));
        }

        if password.len() > self.config.max_password_length {
            return Err(CoreError::Validation(format!(
                "Password must be no more than {} characters long",
                self.config.max_password_length
            )));
        }

        // 可以添加更多密码复杂度规则
        // if !password.chars().any(|c| c.is_uppercase()) {
        //   return Err(CoreError::Validation("Password must contain at least one uppercase letter".into()));
        // }

        Ok(())
    }

    /// 验证用户名
    fn validate_fullname(&self, fullname: &str) -> Result<(), CoreError> {
        let trimmed = fullname.trim();

        if trimmed.len() < self.config.min_fullname_length {
            return Err(CoreError::Validation(format!(
                "Full name must be at least {} characters long",
                self.config.min_fullname_length
            )));
        }

        if trimmed.len() > self.config.max_fullname_length {
            return Err(CoreError::Validation(format!(
                "Full name must be no more than {} characters long",
                self.config.max_fullname_length
            )));
        }

        Ok(())
    }
}

#[async_trait]
impl UserDomainService for UserDomainServiceImpl {
    async fn change_password(
        &self,
        user_id: UserId,
        current_password: &str,
        new_password: &str,
    ) -> Result<(), CoreError> {
        // 验证新密码强度
        self.validate_password(new_password)?;

        // 获取当前用户
        let user = (&*self.repository)
            .find_by_id(user_id)
            .await?
            .ok_or(CoreError::NotFound(format!(
                "User {} not found",
                i64::from(user_id)
            )))?;

        // 验证当前密码
        let current_hash = user
            .password_hash
            .ok_or(CoreError::Validation("User has no password set".into()))?;

        let is_valid = verify_password(current_password, &current_hash)?;
        if !is_valid {
            return Err(CoreError::Validation(
                "Current password is incorrect".into(),
            ));
        }

        // 生成新密码哈希
        let new_hash = hashed_password(new_password)?;

        // 更新密码
        self.repository
            .update_password_hash(user_id, new_hash)
            .await?;

        Ok(())
    }

    async fn update_profile(&self, user_id: UserId, fullname: &str) -> Result<User, CoreError> {
        // 验证输入
        self.validate_fullname(fullname)?;

        // 获取当前用户
        let mut user =
            (&*self.repository)
                .find_by_id(user_id)
                .await?
                .ok_or(CoreError::NotFound(format!(
                    "User {} not found",
                    i64::from(user_id)
                )))?;

        // 更新用户信息
        user.fullname = fullname.trim().to_string();

        // 保存更新
        let updated_user = (&*self.repository).update(user_id, &user).await?;

        Ok(updated_user)
    }

    async fn validate_users_exist(&self, user_ids: &[UserId]) -> Result<(), CoreError> {
        if user_ids.is_empty() {
            return Ok(());
        }

        self.repository.validate_users_exist_by_ids(user_ids).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fechatter_core::contracts::UserRepository;

    #[tokio::test]
    async fn validate_password_should_enforce_length_limits() {
        let config = UserConfig::default();
        let service = UserDomainServiceImpl {
            repository: Arc::new(MockUserRepository::new()),
            config: config.clone(),
        };

        // Test minimum length - should fail
        let short_password = "1234567"; // 7 chars, below min of 8
        let result = service.validate_password(short_password);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("at least 8 characters"));

        // Test valid length - should pass
        let valid_password = "12345678"; // 8 chars, exactly min
        assert!(service.validate_password(valid_password).is_ok());

        // Test maximum length - should fail
        let long_password = "a".repeat(129); // 129 chars, above max of 128
        let result = service.validate_password(&long_password);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("no more than 128 characters"));
    }

    #[tokio::test]
    async fn validate_password_should_handle_edge_cases() {
        let config = UserConfig::default();
        let service = UserDomainServiceImpl {
            repository: Arc::new(MockUserRepository::new()),
            config,
        };

        // Test empty password
        assert!(service.validate_password("").is_err());

        // Test exact boundary values
        let min_valid = "a".repeat(8);
        assert!(service.validate_password(&min_valid).is_ok());

        let max_valid = "a".repeat(128);
        assert!(service.validate_password(&max_valid).is_ok());
    }

    #[tokio::test]
    async fn validate_fullname_should_enforce_length_limits() {
        let config = UserConfig::default();
        let service = UserDomainServiceImpl {
            repository: Arc::new(MockUserRepository::new()),
            config: config.clone(),
        };

        // Test empty name - should fail
        let result = service.validate_fullname("");
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("at least 1 characters"));

        // Test whitespace only - should fail
        let result = service.validate_fullname("   ");
        assert!(result.is_err());

        // Test valid name - should pass
        assert!(service.validate_fullname("John Doe").is_ok());

        // Test too long name - should fail
        let long_name = "a".repeat(101); // 101 chars, above max of 100
        let result = service.validate_fullname(&long_name);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("no more than 100 characters"));
    }

    #[tokio::test]
    async fn validate_fullname_should_handle_whitespace_correctly() {
        let config = UserConfig::default();
        let service = UserDomainServiceImpl {
            repository: Arc::new(MockUserRepository::new()),
            config,
        };

        // Test leading/trailing whitespace gets trimmed
        assert!(service.validate_fullname("  John Doe  ").is_ok());

        // Test internal whitespace is preserved
        assert!(service.validate_fullname("John  Doe").is_ok());

        // Test minimum after trimming
        assert!(service.validate_fullname(" a ").is_ok()); // Trims to "a" (1 char)
    }

    #[tokio::test]
    async fn validate_fullname_with_custom_config() {
        let config = UserConfig {
            min_fullname_length: 3,
            max_fullname_length: 20,
            ..Default::default()
        };
        let service = UserDomainServiceImpl {
            repository: Arc::new(MockUserRepository::new()),
            config,
        };

        // Test below custom minimum
        assert!(service.validate_fullname("Jo").is_err()); // 2 chars, below min of 3

        // Test valid with custom config
        assert!(service.validate_fullname("Joe").is_ok()); // 3 chars, exactly min

        // Test above custom maximum
        let long_name = "a".repeat(21); // 21 chars, above max of 20
        assert!(service.validate_fullname(&long_name).is_err());
    }

    // Mock repository for testing validation only
    struct MockUserRepository;

    impl MockUserRepository {
        fn new() -> Self {
            Self
        }
    }

    #[async_trait]
    impl UserRepository for MockUserRepository {
        async fn find_by_id(&self, _id: UserId) -> Result<Option<User>, CoreError> {
            unimplemented!("Mock for testing validation only")
        }

        async fn find_by_email(&self, _email: &str) -> Result<Option<User>, CoreError> {
            unimplemented!("Mock for testing validation only")
        }

        async fn create(&self, _input: &CreateUser) -> Result<User, CoreError> {
            unimplemented!("Mock for testing validation only")
        }

        async fn update(&self, _id: UserId, _user_data: &User) -> Result<User, CoreError> {
            unimplemented!("Mock for testing validation only")
        }

        async fn authenticate(&self, _credentials: &SigninUser) -> Result<Option<User>, CoreError> {
            unimplemented!("Mock for testing validation only")
        }

        async fn exists_by_email(&self, _email: &str) -> Result<bool, CoreError> {
            unimplemented!("Mock for testing validation only")
        }
    }
}
