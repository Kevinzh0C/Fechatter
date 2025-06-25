use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::{
  error::AppError,
  services::{
    ai::agents::{
      summary_agent::{ChatPeriodSummary, ChatSummaryAgent},
      timeline_agent::{TimelineIndex, TimelineIndexAgent},
    },
    ai::{
      OpenAIClient,
      hybrid_search::{HybridSearchResult, HybridSearchService},
    },
  },
};

/// RAG-based Message Indexer
/// Inspired by LangChain's document indexing approach
pub struct RAGMessageIndexer {
  hybrid_search: Arc<HybridSearchService>,
  openai_client: Arc<OpenAIClient>,
  summary_agent: Arc<ChatSummaryAgent>,
  timeline_agent: Arc<TimelineIndexAgent>,
}

/// Document representation for RAG
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RAGDocument {
  pub id: String,
  pub content: String,
  pub metadata: DocumentMetadata,
  pub embeddings: Option<Vec<f32>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentMetadata {
  pub message_id: i64,
  pub chat_id: i64,
  pub sender_id: i64,
  pub timestamp: DateTime<Utc>,
  pub doc_type: DocumentType,
  pub parent_id: Option<String>,
  pub summary_level: u8, // 0 = raw message, 1 = hourly, 2 = daily, etc.
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum DocumentType {
  RawMessage,
  MessageChunk { chunk_index: usize },
  HourlySummary,
  DailySummary,
  TopicCluster { topic: String },
  TimelineEvent { event_type: String },
}

/// Retrieval chain for RAG
#[derive(Debug, Clone)]
pub struct RetrievalChain {
  pub query: String,
  pub context_window: usize,
  pub include_summaries: bool,
  pub time_range: Option<(DateTime<Utc>, DateTime<Utc>)>,
}

/// RAG Query Result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RAGQueryResult {
  pub answer: String,
  pub source_documents: Vec<RAGDocument>,
  pub confidence_score: f32,
  pub reasoning_chain: Vec<String>,
}

impl RAGMessageIndexer {
  pub fn new(
    hybrid_search: Arc<HybridSearchService>,
    openai_client: Arc<OpenAIClient>,
    summary_agent: Arc<ChatSummaryAgent>,
    timeline_agent: Arc<TimelineIndexAgent>,
  ) -> Self {
    Self {
      hybrid_search,
      openai_client,
      summary_agent,
      timeline_agent,
    }
  }

  /// Build hierarchical index for a chat
  pub async fn build_chat_index(
    &self,
    chat_id: i64,
    messages: Vec<(i64, i64, String, DateTime<Utc>)>,
  ) -> Result<ChatIndex, AppError> {
    // Let LLM handle complex logic, we just organize data
    let summary_agent = ChatSummaryAgent::new(self.openai_client.clone());
    let timeline_agent = TimelineIndexAgent::new();
    
    // 1. Generate summaries (let LLM do the work)
    // Generate rolling summaries instead of daily summaries
    let daily_summaries = summary_agent.generate_rolling_summaries(chat_id, messages.clone(), 24, 24).await?;
    
    // 2. Extract topics (let LLM do the work)  
    let _message_data: Vec<(i64, String)> = messages.iter()
      .map(|(id, _, content, _)| (*id, content.clone()))
      .collect();
    let topic_clusters = summary_agent.identify_topic_clusters(chat_id, daily_summaries.clone(), messages.clone()).await?;
    
    // 3. Build timeline (let LLM enhance) - clone messages to avoid borrow issues
    let timeline = timeline_agent.build_timeline_index(
      chat_id, 
      daily_summaries.clone(), 
      topic_clusters, 
      messages.clone()
    ).await?;
    
    Ok(ChatIndex {
      chat_id,
      message_count: messages.len(),
      hourly_summaries: vec![], // Simple for now
      daily_summaries,
      timeline,
      created_at: Utc::now(),
    })
  }

  /// Query using RAG approach
  pub async fn query_with_rag(
    &self,
    chain: RetrievalChain,
    chat_id: i64,
  ) -> Result<RAGQueryResult, AppError> {
    let mut reasoning_chain = Vec::new();

    // 1. Retrieve relevant documents
    reasoning_chain.push("Searching for relevant messages and summaries...".to_string());
    let search_results = self
      .hybrid_search
      .search(&chain.query, Some(chat_id), chain.context_window)
      .await?;

    // 2. Build context from search results
    let _context = self.build_context_from_results(&search_results, &chain)?;
    reasoning_chain.push(format!("Found {} relevant documents", search_results.len()));

    // 3. Generate answer using LLM with context (let LLM do all the heavy lifting)
    let context_text = search_results.iter()
      .map(|r| format!("Message: {}", r.content))
      .collect::<Vec<_>>()
      .join("\n");
    
    let prompt = format!(
      "Based on the following chat context, answer this question: {}\n\nContext:\n{}\n\nAnswer:",
      chain.query, context_text
    );
    reasoning_chain.push("Generating answer based on context...".to_string());

    let answer = self.openai_client.generate_summary(&prompt).await?;

    // 4. Extract source documents
    let source_documents = self.results_to_documents(search_results);

    // 5. Calculate confidence
    let confidence_score = self.calculate_confidence(&source_documents);

    Ok(RAGQueryResult {
      answer,
      source_documents,
      confidence_score,
      reasoning_chain,
    })
  }

  /// Create Q&A chains for common queries
  pub async fn create_qa_chain(&self, chat_id: i64) -> Result<QAChain, AppError> {
    Ok(QAChain {
      chat_id,
      chains: vec![
        (
          "What were the main topics discussed?".to_string(),
          RetrievalChain {
            query: "main topics discussions".to_string(),
            context_window: 20,
            include_summaries: true,
            time_range: None,
          },
        ),
        (
          "What decisions were made?".to_string(),
          RetrievalChain {
            query: "decisions made important".to_string(),
            context_window: 15,
            include_summaries: true,
            time_range: None,
          },
        ),
        (
          "What are the action items?".to_string(),
          RetrievalChain {
            query: "action items tasks todo".to_string(),
            context_window: 15,
            include_summaries: true,
            time_range: None,
          },
        ),
      ],
    })
  }

  /// Generate conversational summary using RAG
  pub async fn generate_conversational_summary(
    &self,
    chat_id: i64,
    style: SummaryStyle,
  ) -> Result<String, AppError> {
    // Retrieve context based on style
    let query = match style {
      SummaryStyle::Executive => "key decisions action items outcomes",
      SummaryStyle::Technical => "technical details implementation specifics",
      SummaryStyle::Narrative => "conversation flow story progression",
    };

    let chain = RetrievalChain {
      query: query.to_string(),
      context_window: 30,
      include_summaries: true,
      time_range: None,
    };

    let rag_result = self.query_with_rag(chain, chat_id).await?;

    // Post-process based on style
    let styled_summary = self.apply_summary_style(&rag_result.answer, style).await?;

    Ok(styled_summary)
  }

  /// Index summaries for retrieval
  async fn index_summaries(
    &self,
    hourly: &[ChatPeriodSummary],
    _daily: &[ChatPeriodSummary],
  ) -> Result<(), AppError> {
    // Index hourly summaries
    for summary in hourly {
      let _doc_id = format!(
        "hourly-{}-{}",
        summary.chat_id,
        summary.start_time.timestamp()
      );

      let content = format!(
        "Hourly Summary ({}): {}\nTopics: {}\nDecisions: {}\nActions: {}",
        summary.start_time.format("%Y-%m-%d %H:%M"),
        summary.summary,
        summary.key_topics.join(", "),
        summary.important_decisions.join("; "),
        summary.action_items.join("; ")
      );

      // Store in search system
      self
        .hybrid_search
        .index_message(
          -1, // Special ID for summaries
          summary.chat_id,
          0, // System sender
          &content,
          summary.start_time,
        )
        .await?;
    }

    Ok(())
  }

  /// Index timeline entries
  async fn index_timeline(&self, timeline: &TimelineIndex) -> Result<(), AppError> {
    for entry in &timeline.entries {
      let content = format!("Timeline Event: {} - {}", entry.title, entry.summary);

      self
        .hybrid_search
        .index_message(
          -2, // Special ID for timeline
          timeline.chat_id,
          0,
          &content,
          entry.start_time,
        )
        .await?;
    }

    Ok(())
  }

  /// Build context from search results
  fn build_context_from_results(
    &self,
    results: &[HybridSearchResult],
    chain: &RetrievalChain,
  ) -> Result<String, AppError> {
    let mut context_parts = Vec::new();

    for result in results.iter().take(chain.context_window) {
      let timestamp = result.timestamp.format("%Y-%m-%d %H:%M");
      context_parts.push(format!(
        "[{}] {}: {}",
        timestamp, result.message_id, result.snippet
      ));
    }

    Ok(context_parts.join("\n\n"))
  }

  /// Build RAG prompt
  fn build_rag_prompt(&self, query: &str, context: &str) -> String {
    format!(
      "Based on the following conversation context, please answer the question.\n\n\
             Context:\n{}\n\n\
             Question: {}\n\n\
             Please provide a comprehensive answer based solely on the given context. \
             If the context doesn't contain enough information, say so.",
      context, query
    )
  }

  /// Convert search results to RAG documents
  fn results_to_documents(&self, results: Vec<HybridSearchResult>) -> Vec<RAGDocument> {
    results
      .into_iter()
      .map(|result| {
        RAGDocument {
          id: format!("msg-{}", result.message_id),
          content: result.content,
          metadata: DocumentMetadata {
            message_id: result.message_id,
            chat_id: result.chat_id,
            sender_id: 0, // Would need to fetch this
            timestamp: result.timestamp,
            doc_type: DocumentType::RawMessage,
            parent_id: None,
            summary_level: 0,
          },
          embeddings: None,
        }
      })
      .collect()
  }

  /// Calculate confidence score
  fn calculate_confidence(&self, documents: &[RAGDocument]) -> f32 {
    if documents.is_empty() {
      return 0.0;
    }

    // Simple confidence based on document count and recency
    let count_score = (documents.len() as f32 / 10.0).min(1.0);
    let recency_score = if let Some(latest) = documents.first() {
      let age_hours = (Utc::now() - latest.metadata.timestamp).num_hours();
      (1.0 - (age_hours as f32 / 168.0)).max(0.0) // 168 hours = 1 week
    } else {
      0.0
    };

    (count_score * 0.7 + recency_score * 0.3).min(1.0)
  }

  /// Apply summary style
  async fn apply_summary_style(
    &self,
    summary: &str,
    style: SummaryStyle,
  ) -> Result<String, AppError> {
    let style_prompt = match style {
      SummaryStyle::Executive => {
        "Reformat this summary for executives: focus on decisions, outcomes, and action items. Use bullet points."
      }
      SummaryStyle::Technical => {
        "Reformat this summary for technical readers: include implementation details, technical decisions, and specifications."
      }
      SummaryStyle::Narrative => {
        "Reformat this summary as a narrative: tell the story of the conversation in a flowing, readable format."
      }
    };

    let prompt = format!("{}\n\nOriginal summary:\n{}", style_prompt, summary);
    self.openai_client.generate_summary(&prompt).await
  }
}

/// Chat index structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatIndex {
  pub chat_id: i64,
  pub message_count: usize,
  pub hourly_summaries: Vec<ChatPeriodSummary>,
  pub daily_summaries: Vec<ChatPeriodSummary>,
  pub timeline: TimelineIndex,
  pub created_at: DateTime<Utc>,
}

/// Q&A Chain collection
#[derive(Debug, Clone)]
pub struct QAChain {
  pub chat_id: i64,
  pub chains: Vec<(String, RetrievalChain)>,
}

/// Summary styles
#[derive(Debug, Clone, Copy)]
pub enum SummaryStyle {
  Executive,
  Technical,
  Narrative,
}
