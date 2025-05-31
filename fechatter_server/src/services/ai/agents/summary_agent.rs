use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::{error::AppError, services::ai::OpenAIClient};

/// Chat Summary Agent
/// Generates periodic summaries and topic extraction
#[derive(Debug, Clone)]
pub struct ChatSummaryAgent {
  openai_client: Arc<OpenAIClient>,
}

/// Summary of a chat period
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatPeriodSummary {
  pub chat_id: i64,
  pub start_time: DateTime<Utc>,
  pub end_time: DateTime<Utc>,
  pub message_count: usize,
  pub participants: Vec<i64>,
  pub summary: String,
  pub key_topics: Vec<String>,
  pub sentiment: String,
  pub important_decisions: Vec<String>,
  pub action_items: Vec<String>,
}

/// Topic cluster across time
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopicCluster {
  pub topic: String,
  pub keywords: Vec<String>,
  pub time_ranges: Vec<(DateTime<Utc>, DateTime<Utc>)>,
  pub message_ids: Vec<i64>,
  pub summary: String,
  pub relevance_score: f32,
}

impl ChatSummaryAgent {
  pub fn new(openai_client: Arc<OpenAIClient>) -> Self {
    Self { openai_client }
  }

  /// Generate summary for a specific time period
  pub async fn summarize_period(
    &self,
    chat_id: i64,
    messages: Vec<(i64, i64, String, DateTime<Utc>)>, // (message_id, sender_id, content, timestamp)
    period_hours: i64,
  ) -> Result<ChatPeriodSummary, AppError> {
    if messages.is_empty() {
      return Err(AppError::InvalidInput(
        "No messages to summarize".to_string(),
      ));
    }

    // Group messages by time periods
    let start_time = messages.first().unwrap().3;
    let end_time = messages.last().unwrap().3;

    // Extract participants
    let participants: Vec<i64> = messages
      .iter()
      .map(|(_, sender_id, _, _)| *sender_id)
      .collect::<std::collections::HashSet<_>>()
      .into_iter()
      .collect();

    // Prepare context for summary
    let conversation = messages
      .iter()
      .map(|(_, sender_id, content, timestamp)| {
        format!(
          "[{}] User {}: {}",
          timestamp.format("%H:%M"),
          sender_id,
          content
        )
      })
      .collect::<Vec<_>>()
      .join("\n");

    // Generate comprehensive summary
    let summary_prompt = format!(
      "Analyze this chat conversation and provide a structured summary:\n\n{}\n\n\
            Provide:\n\
            1. Overall summary (2-3 sentences)\n\
            2. Key topics discussed (comma-separated)\n\
            3. Overall sentiment (positive/neutral/negative)\n\
            4. Important decisions made (if any, bullet points)\n\
            5. Action items mentioned (if any, bullet points)\n\
            \n\
            Format your response as JSON with keys: summary, topics, sentiment, decisions, action_items",
      conversation
    );

    let response = self.openai_client.generate_summary(&summary_prompt).await?;

    // Parse the response (in production, use a more robust JSON parser)
    let parsed = self.parse_summary_response(&response)?;

    Ok(ChatPeriodSummary {
      chat_id,
      start_time,
      end_time,
      message_count: messages.len(),
      participants,
      summary: parsed.0,
      key_topics: parsed.1,
      sentiment: parsed.2,
      important_decisions: parsed.3,
      action_items: parsed.4,
    })
  }

  /// Identify and cluster topics across time
  pub async fn identify_topic_clusters(
    &self,
    chat_id: i64,
    summaries: Vec<ChatPeriodSummary>,
    messages: Vec<(i64, i64, String, DateTime<Utc>)>,
  ) -> Result<Vec<TopicCluster>, AppError> {
    // Extract all topics from summaries
    let mut topic_map: std::collections::HashMap<
      String,
      Vec<(DateTime<Utc>, DateTime<Utc>, Vec<i64>)>,
    > = std::collections::HashMap::new();

    for summary in summaries {
      for topic in &summary.key_topics {
        // Find messages related to this topic in the time range
        let related_messages: Vec<i64> = messages
          .iter()
          .filter(|(_, _, content, timestamp)| {
            timestamp >= &summary.start_time
              && timestamp <= &summary.end_time
              && content.to_lowercase().contains(&topic.to_lowercase())
          })
          .map(|(id, _, _, _)| *id)
          .collect();

        topic_map
          .entry(topic.clone())
          .or_insert_with(Vec::new)
          .push((summary.start_time, summary.end_time, related_messages));
      }
    }

    // Create topic clusters
    let mut clusters = Vec::new();
    for (topic, time_ranges) in topic_map {
      let all_message_ids: Vec<i64> = time_ranges
        .iter()
        .flat_map(|(_, _, ids)| ids.clone())
        .collect();

      let time_ranges_only: Vec<(DateTime<Utc>, DateTime<Utc>)> = time_ranges
        .iter()
        .map(|(start, end, _)| (*start, *end))
        .collect();

      // Generate topic summary
      let topic_messages: Vec<String> = messages
        .iter()
        .filter(|(id, _, _, _)| all_message_ids.contains(id))
        .map(|(_, _, content, _)| content.clone())
        .collect();

      let topic_summary = self.generate_topic_summary(&topic, &topic_messages).await?;

      clusters.push(TopicCluster {
        topic: topic.clone(),
        keywords: self.extract_keywords(&topic_messages).await?,
        time_ranges: time_ranges_only,
        message_ids: all_message_ids,
        summary: topic_summary,
        relevance_score: (topic_messages.len() as f32) / (messages.len() as f32),
      });
    }

    // Sort by relevance
    clusters.sort_by(|a, b| b.relevance_score.partial_cmp(&a.relevance_score).unwrap());

    Ok(clusters)
  }

  /// Generate rolling summaries (e.g., daily, weekly)
  pub async fn generate_rolling_summaries(
    &self,
    chat_id: i64,
    messages: Vec<(i64, i64, String, DateTime<Utc>)>,
    window_hours: i64,
    step_hours: i64,
  ) -> Result<Vec<ChatPeriodSummary>, AppError> {
    if messages.is_empty() {
      return Ok(Vec::new());
    }

    let mut summaries = Vec::new();
    let start_time = messages.first().unwrap().3;
    let end_time = messages.last().unwrap().3;

    let mut current_start = start_time;
    let window_duration = Duration::hours(window_hours);
    let step_duration = Duration::hours(step_hours);

    while current_start < end_time {
      let current_end = current_start + window_duration;

      // Get messages in this window
      let window_messages: Vec<_> = messages
        .iter()
        .filter(|(_, _, _, timestamp)| timestamp >= &current_start && timestamp < &current_end)
        .cloned()
        .collect();

      if !window_messages.is_empty() {
        let summary = self
          .summarize_period(chat_id, window_messages, window_hours)
          .await?;
        summaries.push(summary);
      }

      current_start = current_start + step_duration;
    }

    Ok(summaries)
  }

  /// Generate topic summary
  async fn generate_topic_summary(
    &self,
    topic: &str,
    messages: &[String],
  ) -> Result<String, AppError> {
    let context = messages.join("\n");
    let prompt = format!(
      "Summarize the discussion about '{}' based on these messages:\n{}\n\n\
            Provide a concise summary (2-3 sentences) focusing on key points and conclusions.",
      topic, context
    );

    self.openai_client.generate_summary(&prompt).await
  }

  /// Extract keywords from messages
  async fn extract_keywords(&self, messages: &[String]) -> Result<Vec<String>, AppError> {
    // Simple implementation - in production, use TF-IDF or similar
    let text = messages.join(" ").to_lowercase();
    let words: Vec<&str> = text.split_whitespace().collect();

    // Count word frequencies
    let mut word_count = std::collections::HashMap::new();
    for word in words {
      if word.len() > 4 {
        // Filter short words
        *word_count.entry(word).or_insert(0) += 1;
      }
    }

    // Get top keywords
    let mut keywords: Vec<_> = word_count.into_iter().collect();
    keywords.sort_by_key(|&(_, count)| std::cmp::Reverse(count));

    Ok(
      keywords
        .into_iter()
        .take(5)
        .map(|(word, _)| word.to_string())
        .collect(),
    )
  }

  /// Parse summary response from AI
  fn parse_summary_response(
    &self,
    response: &str,
  ) -> Result<(String, Vec<String>, String, Vec<String>, Vec<String>), AppError> {
    // Try to parse as JSON first
    if let Ok(json) = serde_json::from_str::<serde_json::Value>(response) {
      let summary = json["summary"].as_str().unwrap_or("").to_string();
      let topics = json["topics"]
        .as_str()
        .map(|s| s.split(',').map(|t| t.trim().to_string()).collect())
        .unwrap_or_default();
      let sentiment = json["sentiment"].as_str().unwrap_or("neutral").to_string();
      let decisions = json["decisions"]
        .as_array()
        .map(|arr| {
          arr
            .iter()
            .filter_map(|v| v.as_str().map(|s| s.to_string()))
            .collect()
        })
        .unwrap_or_default();
      let action_items = json["action_items"]
        .as_array()
        .map(|arr| {
          arr
            .iter()
            .filter_map(|v| v.as_str().map(|s| s.to_string()))
            .collect()
        })
        .unwrap_or_default();

      return Ok((summary, topics, sentiment, decisions, action_items));
    }

    // Fallback to simple parsing
    Ok((
      response.to_string(),
      vec!["general discussion".to_string()],
      "neutral".to_string(),
      Vec::new(),
      Vec::new(),
    ))
  }
}
