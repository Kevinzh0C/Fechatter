//! HTTP测试客户端
//!
//! 封装常用的HTTP测试操作

use anyhow::Result;
use reqwest::{Client, Response, StatusCode};
use serde::{de::DeserializeOwned, Serialize};
use std::collections::HashMap;

/// HTTP测试客户端
pub struct HttpClient {
  client: Client,
  base_url: String,
  auth_token: Option<String>,
}

impl HttpClient {
  /// 创建新的HTTP客户端
  pub fn new(base_url: String) -> Self {
    Self {
      client: Client::new(),
      base_url,
      auth_token: None,
    }
  }

  /// 设置认证令牌
  pub fn set_auth_token(&mut self, token: String) {
    self.auth_token = Some(token);
  }

  /// 清除认证令牌
  pub fn clear_auth_token(&mut self) {
    self.auth_token = None;
  }

  /// 构建请求构建器
  fn request_builder(&self, method: reqwest::Method, path: &str) -> reqwest::RequestBuilder {
    let url = format!("{}{}", self.base_url, path);
    let mut builder = self.client.request(method, &url);

    if let Some(token) = &self.auth_token {
      builder = builder.header("Authorization", format!("Bearer {}", token));
    }

    builder
  }

  /// GET请求
  pub async fn get(&self, path: &str) -> Result<Response> {
    let response = self
      .request_builder(reqwest::Method::GET, path)
      .send()
      .await?;
    Ok(response)
  }

  /// GET请求并解析JSON
  pub async fn get_json<T: DeserializeOwned>(&self, path: &str) -> Result<T> {
    let response = self.get(path).await?;
    let status = response.status();

    if !status.is_success() {
      let error_text = response.text().await.unwrap_or_default();
      anyhow::bail!("Request failed with status {}: {}", status, error_text);
    }

    let data = response.json::<T>().await?;
    Ok(data)
  }

  /// POST请求
  pub async fn post<B: Serialize>(&self, path: &str, body: &B) -> Result<Response> {
    let response = self
      .request_builder(reqwest::Method::POST, path)
      .json(body)
      .send()
      .await?;
    Ok(response)
  }

  /// POST请求并解析JSON
  pub async fn post_json<B: Serialize, T: DeserializeOwned>(
    &self,
    path: &str,
    body: &B,
  ) -> Result<T> {
    let response = self.post(path, body).await?;
    let status = response.status();

    if !status.is_success() {
      let error_text = response.text().await.unwrap_or_default();
      anyhow::bail!("Request failed with status {}: {}", status, error_text);
    }

    let data = response.json::<T>().await?;
    Ok(data)
  }

  /// PUT请求
  pub async fn put<B: Serialize>(&self, path: &str, body: &B) -> Result<Response> {
    let response = self
      .request_builder(reqwest::Method::PUT, path)
      .json(body)
      .send()
      .await?;
    Ok(response)
  }

  /// DELETE请求
  pub async fn delete(&self, path: &str) -> Result<Response> {
    let response = self
      .request_builder(reqwest::Method::DELETE, path)
      .send()
      .await?;
    Ok(response)
  }

  /// 文件上传
  pub async fn upload_file(
    &self,
    path: &str,
    file_name: &str,
    file_content: Vec<u8>,
  ) -> Result<Response> {
    let part = reqwest::multipart::Part::bytes(file_content)
      .file_name(file_name.to_string())
      .mime_str("application/octet-stream")?;

    let form = reqwest::multipart::Form::new().part("file", part);

    let response = self
      .request_builder(reqwest::Method::POST, path)
      .multipart(form)
      .send()
      .await?;

    Ok(response)
  }

  /// 检查响应状态
  pub fn check_status(response: &Response, expected: StatusCode) -> Result<()> {
    if response.status() != expected {
      anyhow::bail!("Expected status {}, got {}", expected, response.status());
    }
    Ok(())
  }
}

/// HTTP测试工具函数
pub mod utils {
  use super::*;

  /// 从响应中提取错误信息
  pub async fn extract_error_message(response: Response) -> String {
    response
      .text()
      .await
      .unwrap_or_else(|_| "Failed to read error response".to_string())
  }

  /// 构建查询参数
  pub fn build_query_params(params: &[(&str, &str)]) -> String {
    if params.is_empty() {
      return String::new();
    }

    let query = params
      .iter()
      .map(|(k, v)| format!("{}={}", k, urlencoding::encode(v)))
      .collect::<Vec<_>>()
      .join("&");

    format!("?{}", query)
  }
}
