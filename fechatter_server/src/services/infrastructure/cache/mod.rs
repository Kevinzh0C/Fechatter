pub mod redis;

pub use redis::RedisCacheService;

pub type Cache = RedisCacheService;
