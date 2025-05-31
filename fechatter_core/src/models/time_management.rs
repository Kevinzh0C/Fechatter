use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// 统一时间管理模块
/// 提供应用层统一的时间处理，避免数据库方言差异
pub struct TimeManager;

impl TimeManager {
  /// 获取当前UTC时间
  /// 这是应用层统一的时间获取入口
  #[inline]
  pub fn now() -> DateTime<Utc> {
    Utc::now()
  }

  /// 为新实体创建时间戳
  pub fn create_timestamps() -> EntityTimestamps {
    let now = Self::now();
    EntityTimestamps {
      created_at: now,
      updated_at: now,
    }
  }

  /// 为更新实体创建时间戳
  pub fn update_timestamp() -> DateTime<Utc> {
    Self::now()
  }

  /// 格式化时间为ISO字符串
  pub fn format_iso(time: DateTime<Utc>) -> String {
    time.format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string()
  }
}

/// 实体时间戳结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityTimestamps {
  pub created_at: DateTime<Utc>,
  pub updated_at: DateTime<Utc>,
}

/// 为任何需要时间戳的实体提供的trait
pub trait WithTimestamps {
  fn created_at(&self) -> DateTime<Utc>;
  fn updated_at(&self) -> DateTime<Utc>;
  fn touch(&mut self) {
    self.set_updated_at(TimeManager::now());
  }
  fn set_updated_at(&mut self, time: DateTime<Utc>);
}

/// 时间相关的SQL查询助手
pub struct TimeQuery;

impl TimeQuery {
  /// 生成创建实体的时间字段绑定（用于INSERT）
  pub fn create_bindings() -> EntityTimestamps {
    TimeManager::create_timestamps()
  }

  /// 生成更新实体的时间字段绑定（用于UPDATE）
  pub fn update_binding() -> DateTime<Utc> {
    TimeManager::update_timestamp()
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use std::thread::sleep;
  use std::time::Duration;

  #[test]
  fn test_time_manager_consistency() {
    let time1 = TimeManager::now();
    sleep(Duration::from_millis(10));
    let time2 = TimeManager::now();

    assert!(time2 > time1);
  }

  #[test]
  fn test_entity_timestamps() {
    let timestamps = TimeManager::create_timestamps();
    assert_eq!(timestamps.created_at, timestamps.updated_at);
  }

  #[test]
  fn test_format_iso() {
    let now = TimeManager::now();
    let formatted = TimeManager::format_iso(now);
    assert!(formatted.contains("T"));
    assert!(formatted.ends_with("Z"));
  }
}
