use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Unified time management module
/// Provides application-level unified time handling to avoid database dialect differences
pub struct TimeManager;

impl TimeManager {
  /// Get current UTC time
  /// This is the unified application-level entry point for time retrieval
  #[inline]
  pub fn now() -> DateTime<Utc> {
    Utc::now()
  }

  /// Create timestamps for new entities
  pub fn create_timestamps() -> EntityTimestamps {
    let now = Self::now();
    EntityTimestamps {
      created_at: now,
      updated_at: now,
    }
  }

  /// Create timestamp for entity updates
  pub fn update_timestamp() -> DateTime<Utc> {
    Self::now()
  }

  /// Format time as ISO string
  pub fn format_iso(time: DateTime<Utc>) -> String {
    time.format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string()
  }
}

/// Entity timestamp structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityTimestamps {
  pub created_at: DateTime<Utc>,
  pub updated_at: DateTime<Utc>,
}

/// Trait for any entity that requires timestamps
pub trait WithTimestamps {
  fn created_at(&self) -> DateTime<Utc>;
  fn updated_at(&self) -> DateTime<Utc>;
  fn touch(&mut self) {
    self.set_updated_at(TimeManager::now());
  }
  fn set_updated_at(&mut self, time: DateTime<Utc>);
}

/// SQL query helper for time-related operations
pub struct TimeQuery;

impl TimeQuery {
  /// Generate time field bindings for entity creation (for INSERT)
  pub fn create_bindings() -> EntityTimestamps {
    TimeManager::create_timestamps()
  }

  /// Generate time field bindings for entity updates (for UPDATE)
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
