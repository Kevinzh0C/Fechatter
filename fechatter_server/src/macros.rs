#[macro_export]
macro_rules! test_setup {
  () => {{
    let config = AppConfig::load()?;
    let (_tdb, state) = AppState::test_new(config).await?;
    state
  }};
}

#[macro_export]
macro_rules! assert_response {
  ($response:expr, $status:expr) => {{
    let response = $response.into_response();
    assert_eq!(response.status(), $status);
    response
  }};
}

#[macro_export]
macro_rules! assert_json_response {
  ($response:expr, $status:expr, $type:ty) => {{
    let response = assert_response!($response, $status);
    let body = response.into_body().collect().await?.to_bytes();
    let body: $type = serde_json::from_slice(&body)?;
    body
  }};
}
