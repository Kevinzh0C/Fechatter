//! Redis integration tests
//! Run with: cargo test --features integration_tests

#[cfg(feature = "integration_tests")]
mod tests {
    use fechatter_server::services::infrastructure::cache::redis::*;
    use std::sync::Arc;

    async fn get_cache() -> RedisCacheService {
        let url = std::env::var("REDIS_URL")
            .unwrap_or("redis://:fechatter_redis_pass@localhost:6379".into());
        RedisCacheService::new(&url, "test")
            .await
            .expect("Redis down?")
    }

    #[tokio::test]
    async fn can_connect() {
        let cache = get_cache().await;
        // If we got here, connection works
    }

    #[tokio::test]
    async fn basic_stuff() {
        let cache = get_cache().await;

        cache.set("foo", &"bar", 60).await.unwrap();
        let val: Option<String> = cache.get("foo").await.unwrap();
        assert_eq!(val, Some("bar".into()));

        assert!(cache.del("foo").await.unwrap());
        let val: Option<String> = cache.get("foo").await.unwrap();
        assert!(val.is_none());
    }

    #[tokio::test]
    async fn batch_works() {
        let cache = get_cache().await;

        cache
            .batch()
            .set("a", &"1", 60)
            .unwrap()
            .set("b", &"2", 60)
            .unwrap()
            .set("c", &"3", 60)
            .unwrap()
            .run()
            .await
            .unwrap();

        let vals: Vec<Option<String>> = cache.mget(&["a", "b", "c"]).await.unwrap();
        assert_eq!(
            vals,
            vec![Some("1".into()), Some("2".into()), Some("3".into())]
        );
    }

    #[tokio::test]
    async fn concurrent_ok() {
        let cache = Arc::new(get_cache().await);
        let mut handles = vec![];

        for i in 0..20 {
            let cache = cache.clone();
            let h = tokio::spawn(async move {
                cache.set(&format!("test:{}", i), &i, 60).await.unwrap();
                let val: Option<i32> = cache.get(&format!("test:{}", i)).await.unwrap();
                assert_eq!(val, Some(i));
            });
            handles.push(h);
        }

        for h in handles {
            h.await.unwrap();
        }
    }

    #[tokio::test]
    async fn ttl_constants() {
        let cache = get_cache().await;

        cache.set("short", &"data", ttl::SHORT).await.unwrap();
        cache.set("long", &"data", ttl::LONG).await.unwrap();

        let short: Option<String> = cache.get("short").await.unwrap();
        let long: Option<String> = cache.get("long").await.unwrap();

        assert!(short.is_some());
        assert!(long.is_some());
    }
}
