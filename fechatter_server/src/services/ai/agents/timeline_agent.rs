use super::summary_agent::{ChatPeriodSummary, TopicCluster};
use crate::error::AppError;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Timeline Index Agent
/// Creates navigable timeline with topic-based indexing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimelineIndexAgent;

/// Timeline entry representing a significant period or event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimelineEntry {
  pub id: String,
  pub chat_id: i64,
  pub start_time: DateTime<Utc>,
  pub end_time: DateTime<Utc>,
  pub entry_type: TimelineEntryType,
  pub title: String,
  pub summary: String,
  pub participants: Vec<i64>,
  pub message_range: (i64, i64), // (first_message_id, last_message_id)
  pub tags: Vec<String>,
  pub importance_score: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TimelineEntryType {
  TopicDiscussion(String),        // Topic name
  ImportantDecision(String),      // Decision summary
  ActionItems(Vec<String>),       // List of action items
  HighActivity,                   // Period of high message volume
  SentimentShift(String, String), // (from_sentiment, to_sentiment)
}

/// Timeline index for quick navigation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimelineIndex {
  pub chat_id: i64,
  pub entries: Vec<TimelineEntry>,
  pub topic_map: HashMap<String, Vec<i64>>,
  pub participant_map: HashMap<i64, String>,
  pub generated_at: DateTime<Utc>,
}

impl TimelineIndexAgent {
  pub fn new() -> Self {
    Self
  }

  /// Build timeline index from summaries and topic clusters
  pub async fn build_timeline_index(
    &self,
    chat_id: i64,
    daily_summaries: Vec<ChatPeriodSummary>,
    topic_clusters: Vec<TopicCluster>,
    messages: Vec<(i64, i64, String, DateTime<Utc>)>,
  ) -> Result<TimelineIndex, AppError> {
    let mut entries = Vec::new();
    let mut topic_map = HashMap::new();
    let mut participant_map = HashMap::new();
    
    // 1. Create entries from daily summaries
    for summary in daily_summaries {
      let entry = TimelineEntry {
        id: format!("summary-{}", summary.start_time.timestamp()),
        chat_id,
        start_time: summary.start_time,
        end_time: summary.end_time,
        entry_type: TimelineEntryType::TopicDiscussion(summary.key_topics.join(", ")),
        title: format!("Daily Summary - {}", summary.start_time.format("%Y-%m-%d")),
        summary: summary.summary,
        participants: summary.participants,
        message_range: (0, 0), // Will be filled by LLM if needed
        tags: summary.key_topics,
        importance_score: 0.8,
      };
      entries.push(entry);
    }
    
    // 2. Create entries from topic clusters  
    for topic in topic_clusters {
      topic_map.insert(topic.topic.clone(), topic.message_ids.clone());
      
      if !topic.message_ids.is_empty() {
        let entry = TimelineEntry {
          id: format!("topic-{}", topic.topic.replace(" ", "-")),
          chat_id,
          start_time: Utc::now(), // Simple fallback, LLM can enhance
          end_time: Utc::now(),
          entry_type: TimelineEntryType::TopicDiscussion(topic.topic.clone()),
          title: format!("Topic: {}", topic.topic),
          summary: format!("Discussion about {} with {} messages", topic.topic, topic.message_ids.len()),
          participants: vec![], // LLM can fill this
          message_range: (*topic.message_ids.first().unwrap_or(&0), *topic.message_ids.last().unwrap_or(&0)),
          tags: topic.keywords,
          importance_score: topic.relevance_score,
        };
        entries.push(entry);
      }
    }
    
    // 3. Detect high activity periods (simple version)
    let high_activity_entries = self.detect_high_activity_periods(&messages, chat_id)?;
    entries.extend(high_activity_entries);
    
    // 4. Build participant map (simple user ID -> name mapping)
    for (_, user_id, _, _) in &messages {
      participant_map.insert(*user_id, format!("User-{}", user_id));
    }
    
    Ok(TimelineIndex {
      chat_id,
      entries,
      topic_map,
      participant_map,
      generated_at: Utc::now(),
    })
  }

  /// Search timeline by query
  pub async fn search_timeline(
    &self,
    index: &TimelineIndex,
    query: &str,
  ) -> Result<Vec<TimelineEntry>, AppError> {
    let query_lower = query.to_lowercase();

    let mut results: Vec<(TimelineEntry, f32)> = index
      .entries
      .iter()
      .filter_map(|entry| {
        let mut score = 0.0;

        // Check title
        if entry.title.to_lowercase().contains(&query_lower) {
          score += 2.0;
        }

        // Check summary
        if entry.summary.to_lowercase().contains(&query_lower) {
          score += 1.0;
        }

        // Check tags
        for tag in &entry.tags {
          if tag.to_lowercase().contains(&query_lower) {
            score += 0.5;
          }
        }

        if score > 0.0 {
          Some((entry.clone(), score))
        } else {
          None
        }
      })
      .collect();

    // Sort by score
    results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

    Ok(results.into_iter().map(|(entry, _)| entry).collect())
  }

  /// Get timeline entries for a specific time range
  pub async fn get_timeline_range(
    &self,
    index: &TimelineIndex,
    start: DateTime<Utc>,
    end: DateTime<Utc>,
  ) -> Result<Vec<TimelineEntry>, AppError> {
    Ok(
      index
        .entries
        .iter()
        .filter(|entry| {
          (entry.start_time >= start && entry.start_time <= end)
            || (entry.end_time >= start && entry.end_time <= end)
            || (entry.start_time <= start && entry.end_time >= end)
        })
        .cloned()
        .collect(),
    )
  }

  /// Detect periods of high activity
  fn detect_high_activity_periods(
    &self,
    messages: &[(i64, i64, String, DateTime<Utc>)],
    chat_id: i64,
  ) -> Result<Vec<TimelineEntry>, AppError> {
    let mut entries = Vec::new();

    // Group messages by hour
    let mut hourly_counts = std::collections::HashMap::new();
    for (_, _, _, timestamp) in messages {
      let hour_key = timestamp.format("%Y-%m-%d %H:00").to_string();
      *hourly_counts.entry(hour_key).or_insert(0) += 1;
    }

    // Calculate average messages per hour
    let total_count: usize = hourly_counts.values().sum();
    let avg_count = total_count as f32 / hourly_counts.len() as f32;
    let threshold = avg_count * 2.0; // 2x average = high activity

    // Find high activity periods
    let mut high_activity_hours: Vec<_> = hourly_counts
      .into_iter()
      .filter(|(_, count)| *count as f32 > threshold)
      .collect();
    high_activity_hours.sort_by_key(|(hour, _)| hour.clone());

    // Group consecutive hours
    if !high_activity_hours.is_empty() {
      let mut current_start = DateTime::parse_from_str(
        &format!("{} +00:00", high_activity_hours[0].0),
        "%Y-%m-%d %H:%M %z",
      )
      .unwrap()
      .with_timezone(&Utc);
      let mut current_end = current_start;
      let mut current_messages: Vec<(i64, i64, String, DateTime<Utc>)> = vec![];

      for (hour_str, _) in high_activity_hours {
        let hour = DateTime::parse_from_str(&format!("{} +00:00", hour_str), "%Y-%m-%d %H:%M %z")
          .unwrap()
          .with_timezone(&Utc);

        if hour - current_end <= chrono::Duration::hours(1) {
          current_end = hour;
        } else {
          // Create entry for previous period
          if !current_messages.is_empty() {
            let entry = self.create_high_activity_entry(
              chat_id,
              current_start,
              current_end + chrono::Duration::hours(1),
              messages,
            )?;
            entries.push(entry);
          }

          current_start = hour;
          current_end = hour;
          current_messages.clear();
        }
      }

      // Create entry for last period
      if current_start != current_end {
        let entry = self.create_high_activity_entry(
          chat_id,
          current_start,
          current_end + chrono::Duration::hours(1),
          messages,
        )?;
        entries.push(entry);
      }
    }

    Ok(entries)
  }

  /// Create high activity timeline entry
  fn create_high_activity_entry(
    &self,
    chat_id: i64,
    start: DateTime<Utc>,
    end: DateTime<Utc>,
    messages: &[(i64, i64, String, DateTime<Utc>)],
  ) -> Result<TimelineEntry, AppError> {
    let period_messages: Vec<_> = messages
      .iter()
      .filter(|(_, _, _, timestamp)| timestamp >= &start && timestamp < &end)
      .collect();

    let participants: Vec<i64> = period_messages
      .iter()
      .map(|(_, sender_id, _, _)| *sender_id)
      .collect::<std::collections::HashSet<_>>()
      .into_iter()
      .collect();

    let first_msg = period_messages
      .first()
      .map(|(id, _, _, _)| *id)
      .unwrap_or(0);
    let last_msg = period_messages
      .last()
      .map(|(id, _, _, _)| *id)
      .unwrap_or(first_msg);

    Ok(TimelineEntry {
      id: format!("activity-{}", start.timestamp()),
      chat_id,
      start_time: start,
      end_time: end,
      entry_type: TimelineEntryType::HighActivity,
      title: format!("High Activity Period ({} messages)", period_messages.len()),
      summary: format!(
        "Intense discussion period with {} messages from {} participants",
        period_messages.len(),
        participants.len()
      ),
      participants,
      message_range: (first_msg, last_msg),
      tags: vec!["high-activity".to_string()],
      importance_score: 0.7,
    })
  }
}
