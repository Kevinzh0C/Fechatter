use crate::utils::jwt::TokenManager;
use sqlx::{Acquire, PgConnection, PgPool, Postgres};

pub trait WithDbPool {
  fn db_pool(&self) -> &PgPool;
}

impl WithDbPool for PgPool {
  fn db_pool(&self) -> &PgPool {
    self
  }
}

impl WithDbPool for &PgPool {
  fn db_pool(&self) -> &PgPool {
    self
  }
}

pub trait WithTokenManager {
  fn token_manager(&self) -> &TokenManager;
}

pub trait WithCache<K, V> {
  fn get_from_cache(&self, key: &K) -> Option<V>;
  fn insert_into_cache(&self, key: K, value: V, ttl_seconds: u64);
  fn remove_from_cache(&self, key: &K);
}

pub trait DbConnection {
  fn as_connection(&mut self) -> &mut PgConnection;
}

pub struct PgConnectionWrapper<'c> {
  conn: &'c mut PgConnection,
  pool: &'c PgPool,
}

impl<'c> PgConnectionWrapper<'c> {
  pub fn new(conn: &'c mut PgConnection, pool: &'c PgPool) -> Self {
    Self { conn, pool }
  }

  pub fn connection(&mut self) -> &mut PgConnection {
    self.conn
  }
}

impl<'c> WithDbPool for PgConnectionWrapper<'c> {
  fn db_pool(&self) -> &PgPool {
    self.pool
  }
}

pub fn wrap_connection<'c>(
  conn: &'c mut PgConnection,
  pool: &'c PgPool,
) -> PgConnectionWrapper<'c> {
  PgConnectionWrapper::new(conn, pool)
}
