pub mod error;
pub mod middlewares;
pub mod models;
pub mod services;
pub mod state;
pub mod utils;

pub use error::*;
pub use middlewares::*;
pub use models::*;
pub use services::*;
pub use state::*;
pub use utils::*;

#[cfg(test)]
#[macro_export]
macro_rules! setup_test_users {
    ($n:expr) => {{
        use sqlx::{postgres::PgPoolOptions, PgPool};
        use std::sync::Arc;
        use crate::models::{User, CreateUser};
        use crate::state::{WithDbPool, WithCache, WithTokenManager};

        let db_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
            "postgres://postgres:postgres@localhost:5432/fechatter_test".to_string()
        });

        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(&db_url)
            .await
            .expect("Failed to connect to database");

        sqlx::query("DROP TABLE IF EXISTS refresh_tokens").execute(&pool).await.unwrap();
        sqlx::query("DROP TABLE IF EXISTS messages").execute(&pool).await.unwrap();
        sqlx::query("DROP TABLE IF EXISTS chat_members").execute(&pool).await.unwrap();
        sqlx::query("DROP TABLE IF EXISTS chats").execute(&pool).await.unwrap();
        sqlx::query("DROP TABLE IF EXISTS users").execute(&pool).await.unwrap();
        sqlx::query("DROP TABLE IF EXISTS workspaces").execute(&pool).await.unwrap();

        sqlx::query(
            r#"
            CREATE TABLE workspaces (
                id BIGSERIAL PRIMARY KEY,
                name TEXT NOT NULL,
                owner_id BIGINT NOT NULL,
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
            )
            "#,
        ).execute(&pool).await.unwrap();

        sqlx::query(
            r#"
            CREATE TYPE user_status AS ENUM ('suspended', 'active');

            CREATE TABLE users (
                id BIGSERIAL PRIMARY KEY,
                workspace_id BIGINT NOT NULL REFERENCES workspaces(id),
                email TEXT NOT NULL,
                fullname TEXT NOT NULL,
                password_hash TEXT,
                status user_status NOT NULL DEFAULT 'active',
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                UNIQUE(email)
            )
            "#,
        ).execute(&pool).await.unwrap();

        sqlx::query(
            r#"
            CREATE TYPE chat_type AS ENUM ('single', 'group', 'private_channel', 'public_channel');

            CREATE TABLE chats (
                id BIGSERIAL PRIMARY KEY,
                workspace_id BIGINT NOT NULL REFERENCES workspaces(id),
                name TEXT NOT NULL,
                chat_type chat_type NOT NULL,
                chat_members BIGINT[] NOT NULL,
                description TEXT DEFAULT '',
                created_by BIGINT NOT NULL REFERENCES users(id),
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
            )
            "#,
        ).execute(&pool).await.unwrap();

        sqlx::query(
            r#"
            CREATE TABLE chat_members (
                chat_id BIGINT NOT NULL REFERENCES chats(id),
                user_id BIGINT NOT NULL REFERENCES users(id),
                joined_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                PRIMARY KEY (chat_id, user_id)
            )
            "#,
        ).execute(&pool).await.unwrap();

        sqlx::query(
            r#"
            CREATE TABLE messages (
                id BIGSERIAL PRIMARY KEY,
                chat_id BIGINT NOT NULL REFERENCES chats(id),
                sender_id BIGINT NOT NULL REFERENCES users(id),
                content TEXT NOT NULL,
                files TEXT[] DEFAULT '{}',
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
            )
            "#,
        ).execute(&pool).await.unwrap();

        sqlx::query(
            r#"
            CREATE TABLE refresh_tokens (
                id BIGSERIAL PRIMARY KEY,
                user_id BIGINT NOT NULL REFERENCES users(id),
                token TEXT NOT NULL,
                expires_at TIMESTAMPTZ NOT NULL,
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
            )
            "#,
        ).execute(&pool).await.unwrap();

        use crate::utils::jwt::{TokenManager, AuthConfig};
        use crate::models::DatabaseModel;

        #[derive(Clone, Debug)]
        struct TestState {
            pool: PgPool,
            token_manager: TokenManager,
        }

        impl WithDbPool for TestState {
            fn db_pool(&self) -> &PgPool {
                &self.pool
            }
        }

        impl WithTokenManager for TestState {
            fn token_manager(&self) -> &TokenManager {
                &self.token_manager
            }
        }

        impl<K, V> WithCache<K, V> for TestState {
            fn get_from_cache(&self, _key: &K) -> Option<V> {
                None
            }

            fn insert_into_cache(&self, _key: K, _value: V, _ttl_seconds: u64) {
            }

            fn remove_from_cache(&self, _key: &K) {
            }
        }

        let auth_config = AuthConfig {
            sk: "-----BEGIN PRIVATE KEY-----\nMC4CAQAwBQYDK2VwBCIEIJ+DYvh6SEqVTm50DFtMDoQikTmiCqirVv9mWG9qfSnF\n-----END PRIVATE KEY-----".to_string(),
            pk: "-----BEGIN PUBLIC KEY-----\nMCowBQYDK2VwAyEAHrnbu7wEfAP9cGBOAHHwmH4Wsot1ciXBHmCRcXLBUUQ=\n-----END PUBLIC KEY-----".to_string(),
        };
        let token_manager = TokenManager::from_config(&auth_config).unwrap();

        let state = TestState {
            pool: pool.clone(),
            token_manager,
        };

        let mut users = Vec::new();
        for i in 0..$n {
            let input = CreateUser::new(
                &format!("User {}", i),
                &format!("user{}@test.com", i),
                "Test Workspace",
                "password",
            );
            let user = User::create(&input, &state).await.unwrap();
            users.push(user);
        }

        (pool, state, users)
    }};
}
