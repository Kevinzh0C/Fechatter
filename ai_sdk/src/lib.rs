mod adapters;

pub use adapters::*;

use std::fmt;

pub enum AiAdapter {
  Openai(OpenaiAdapter),
  Ollama(OllamaAdapter),
}

#[derive(Debug, Clone)]
pub enum Role {
  User,
  Assistant,
  System,
}

#[derive(Debug, Clone)]
pub struct Message {
  pub role: Role,
  pub content: String,
}

#[allow(async_fn_in_trait)]
pub trait AiService {
  /// Basic chat completion
  async fn complete(&self, messages: &[Message]) -> anyhow::Result<String>;
  
  /// Generate embeddings for texts
  async fn embed_texts(&self, texts: Vec<String>) -> anyhow::Result<Vec<Vec<f32>>>;
  
  /// Generate single embedding
  async fn generate_embedding(&self, text: &str) -> anyhow::Result<Vec<f32>> {
    let embeddings = self.embed_texts(vec![text.to_string()]).await?;
    embeddings
      .into_iter()
      .next()
      .ok_or_else(|| anyhow::anyhow!("Failed to generate embedding"))
  }
  
  /// Generate summary
  async fn generate_summary(&self, text: &str) -> anyhow::Result<String> {
    let messages = vec![
      Message::system("You are a helpful assistant that creates concise summaries."),
      Message::user(format!("Please summarize the following text:\n\n{}", text)),
    ];
    self.complete(&messages).await
  }
  
  /// Suggest replies based on context  
  async fn suggest_replies(&self, context: &str) -> anyhow::Result<Vec<String>> {
    let messages = vec![
      Message::system("You are a helpful assistant that suggests appropriate replies to messages. Provide 3 different reply options, each on a new line."),
      Message::user(format!("Based on this conversation context, suggest 3 possible replies:\n\n{}", context)),
    ];
    let response = self.complete(&messages).await?;
    Ok(
      response
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(|s| s.to_string())
        .collect(),
    )
  }
  
  /// Moderate content (check if content is appropriate)
  async fn moderate_content(&self, content: &str) -> anyhow::Result<bool>;
}

// TODO: in future, use enum_dispatch crate to dispatch the methods for different adapters
impl AiService for AiAdapter {
  async fn complete(&self, messages: &[Message]) -> anyhow::Result<String> {
    match self {
      AiAdapter::Openai(adapter) => adapter.complete(messages).await,
      AiAdapter::Ollama(adapter) => adapter.complete(messages).await,
    }
  }
  
  async fn embed_texts(&self, texts: Vec<String>) -> anyhow::Result<Vec<Vec<f32>>> {
    match self {
      AiAdapter::Openai(adapter) => adapter.embed_texts(texts).await,
      AiAdapter::Ollama(adapter) => adapter.embed_texts(texts).await,
    }
  }
  
  async fn moderate_content(&self, content: &str) -> anyhow::Result<bool> {
    match self {
      AiAdapter::Openai(adapter) => adapter.moderate_content(content).await,
      AiAdapter::Ollama(adapter) => adapter.moderate_content(content).await,
    }
  }
}

impl fmt::Display for Role {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Role::User => write!(f, "user"),
      Role::Assistant => write!(f, "assistant"),
      Role::System => write!(f, "system"),
    }
  }
}

impl Message {
  pub fn new(role: Role, content: impl Into<String>) -> Self {
    Self {
      role,
      content: content.into(),
    }
  }

  pub fn user(content: impl Into<String>) -> Self {
    Self::new(Role::User, content)
  }

  pub fn assistant(content: impl Into<String>) -> Self {
    Self::new(Role::Assistant, content)
  }

  pub fn system(content: impl Into<String>) -> Self {
    Self::new(Role::System, content)
  }
}
