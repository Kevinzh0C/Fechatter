//! # Query Processor - High Performance Chat Message Query Processor
//!
//! Specialized processor for search query preprocessing, optimization and intelligent features
//!
//! ## Features
//! - Query string cleaning and normalization
//! - Keyword extraction and stop word filtering
//! - Intelligent query building
//! - Multi-language support (Chinese, English, Japanese, Korean etc)
//! - Search suggestion generation
//! - Query caching and performance optimization
//! - Context-aware search
//! - Query intent detection

use dashmap::DashMap;
use jieba_rs::Jieba;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, OnceLock, RwLock};
use std::time::{Duration, Instant};
use tracing::{debug, warn};
use unicode_segmentation::UnicodeSegmentation;

// ================================================================================================
// Global Static Instances - Reduce Repeated Initialization Overhead
// ================================================================================================

/// Global Chinese tokenizer instance
static JIEBA: OnceLock<Jieba> = OnceLock::new();

/// Global query cache
static QUERY_CACHE: OnceLock<Arc<QueryCache>> = OnceLock::new();

// ================================================================================================
// Query Type Definition
// ================================================================================================

/// Query type enumeration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum QueryType {
  /// Exact phrase search (quoted)
  ExactPhrase,
  /// Boolean search (contains AND, OR, NOT)
  Boolean,
  /// Fuzzy search (contains wildcards)
  Fuzzy,
  /// Normal keyword search
  Keywords,
  /// Empty query
  Empty,
}

// ================================================================================================
// Data Structure Definitions
// ================================================================================================

/// Language type
#[derive(Debug, Clone, PartialEq)]
pub enum LanguageType {
  Chinese,
  English,
  Japanese,
  Korean,
  Mixed,
  Other,
}

/// Keyword information
#[derive(Debug, Clone)]
pub struct KeywordInfo {
  /// Keyword
  pub word: String,
  /// Weight
  pub weight: f32,
  /// Language type
  pub language: LanguageType,
  /// Position in original text
  pub position: usize,
}

/// Query intent
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum QueryIntent {
  /// Content search
  ContentSearch,
  /// Time range search
  TimeRange,
  /// File search
  FileSearch,
  /// Person search
  PersonSearch,
  /// Mixed search
  Mixed,
}

/// Query context
#[derive(Debug, Clone)]
pub struct QueryContext {
  /// Current chat ID
  pub chat_id: Option<i64>,
  /// Current user ID
  pub user_id: i64,
  /// Is in file view
  pub is_in_file_view: bool,
  /// Recent search history
  pub recent_searches: Vec<String>,
  /// Current timezone
  pub timezone: String,
}

/// Optimized query result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizedQuery {
  /// Original query
  pub original: String,
  /// Optimized query
  pub optimized: String,
  /// Extracted keywords
  pub keywords: Vec<String>,
  /// Query type
  pub query_type: QueryType,
  /// Optimization confidence (0.0 - 1.0)
  pub confidence: f32,
  /// Query intent
  pub intent: Option<QueryIntent>,
  /// Processing time
  pub processing_time: Duration,
}

impl OptimizedQuery {
  /// Whether to suggest using optimized query
  pub fn should_use_optimized(&self) -> bool {
    self.confidence > 0.6 && !self.optimized.is_empty()
  }

  /// Get best query string
  pub fn best_query(&self) -> &str {
    if self.should_use_optimized() {
      &self.optimized
    } else {
      &self.original
    }
  }
}

// ================================================================================================
// Query Preprocessor - Core Optimization Version
// ================================================================================================

/// Query Preprocessor - Intelligent search query optimization
pub struct QueryProcessor {
  /// Legacy query cache (deprecated)
  legacy_cache: Arc<QueryCache>,
  /// Search cache service (preferred)
  search_cache: Option<Arc<super::cache::SearchCacheService>>,
  /// Query statistics
  stats: Arc<QueryStats>,
}

impl Default for QueryProcessor {
  fn default() -> Self {
    Self::new()
  }
}

impl QueryProcessor {
  /// Create new query processor instance with legacy cache
  pub fn new() -> Self {
    Self {
      legacy_cache: QUERY_CACHE
        .get_or_init(|| Arc::new(QueryCache::new(10000)))
        .clone(),
      search_cache: None,
      stats: Arc::new(QueryStats::new()),
    }
  }

  /// Create new query processor with search cache service
  pub fn new_with_search_cache(cache: Arc<super::cache::SearchCacheService>) -> Self {
    Self {
      legacy_cache: QUERY_CACHE
        .get_or_init(|| Arc::new(QueryCache::new(10000)))
        .clone(),
      search_cache: Some(cache),
      stats: Arc::new(QueryStats::new()),
    }
  }

  /// Preprocess query string to improve search accuracy
  pub fn preprocess_query(query: &str) -> String {
    static CLEAN_REGEX: OnceLock<Regex> = OnceLock::new();
    static NORMALIZE_REGEX: OnceLock<Regex> = OnceLock::new();

    let clean_regex =
      CLEAN_REGEX.get_or_init(|| Regex::new(r"[^\p{L}\p{N}\s\-_@#.$:/?&=+%]").unwrap());

    let normalize_regex = NORMALIZE_REGEX.get_or_init(|| Regex::new(r"\s+").unwrap());

    let mut processed = query.to_lowercase();

    // 1. Clean special characters
    processed = clean_regex.replace_all(&processed, " ").to_string();

    // 2. Normalize whitespace
    processed = normalize_regex
      .replace_all(&processed, " ")
      .trim()
      .to_string();

    debug!("Query preprocessed: '{}' -> '{}'", query, processed);
    processed
  }

  /// Extract search keywords
  pub fn extract_keywords(query: &str) -> Vec<String> {
    let keywords: Vec<String> = query
      .split_whitespace()
      .filter(|word| word.len() >= 2)
      .filter(|word| !Self::is_stop_word(word))
      .filter(|word| !Self::is_common_symbol(word))
      .map(|word| word.to_string())
      .collect();

    debug!("Keywords extracted from '{}': {:?}", query, keywords);
    keywords
  }

  /// Advanced keyword extraction - Support multi-language and intelligent tokenization
  pub fn extract_keywords_advanced(&self, query: &str) -> Vec<KeywordInfo> {
    let jieba = JIEBA.get_or_init(|| Jieba::new());
    let mut keywords = Vec::new();

    // Detect language type
    let lang_type = Self::detect_language(query);

    match lang_type {
      LanguageType::Chinese | LanguageType::Mixed => {
        // Use jieba for Chinese tokenization
        let words = jieba.cut_for_search(query, false);
        for word in words {
          if !Self::is_stop_word(word) && word.chars().count() >= 1 {
            keywords.push(KeywordInfo {
              word: word.to_string(),
              weight: Self::calculate_keyword_weight(word, query),
              language: LanguageType::Chinese,
              position: query.find(word).unwrap_or(0),
            });
          }
        }
      }
      LanguageType::English => {
        // English tokenization
        for word in query.split_whitespace() {
          if !Self::is_stop_word(word) && word.len() >= 2 {
            keywords.push(KeywordInfo {
              word: word.to_string(),
              weight: Self::calculate_keyword_weight(word, query),
              language: LanguageType::English,
              position: query.find(word).unwrap_or(0),
            });
          }
        }
      }
      _ => {
        // Use Unicode tokenization for other languages
        for word in query.unicode_words() {
          if !Self::is_stop_word(word) && word.len() >= 2 {
            keywords.push(KeywordInfo {
              word: word.to_string(),
              weight: Self::calculate_keyword_weight(word, query),
              language: lang_type.clone(),
              position: query.find(word).unwrap_or(0),
            });
          }
        }
      }
    }

    // Sort by weight
    keywords.sort_by(|a, b| b.weight.partial_cmp(&a.weight).unwrap());
    keywords
  }

  /// Detect language type
  fn detect_language(text: &str) -> LanguageType {
    let mut has_chinese = false;
    let mut has_english = false;
    let mut has_japanese = false;
    let mut has_korean = false;

    for ch in text.chars() {
      match ch {
        '\u{4e00}'..='\u{9fff}' => has_chinese = true,
        'a'..='z' | 'A'..='Z' => has_english = true,
        '\u{3040}'..='\u{309f}' | '\u{30a0}'..='\u{30ff}' => has_japanese = true,
        '\u{ac00}'..='\u{d7af}' => has_korean = true,
        _ => {}
      }
    }

    if has_chinese && has_english {
      LanguageType::Mixed
    } else if has_chinese {
      LanguageType::Chinese
    } else if has_japanese {
      LanguageType::Japanese
    } else if has_korean {
      LanguageType::Korean
    } else if has_english {
      LanguageType::English
    } else {
      LanguageType::Other
    }
  }

  /// Calculate keyword weight
  fn calculate_keyword_weight(word: &str, query: &str) -> f32 {
    let mut weight = 1.0f32;

    // Frequency weight
    let frequency = query.matches(word).count() as f32;
    weight *= (1.0f32 + frequency.ln()).min(2.0f32);

    // Position weight - Words at front are more important
    if let Some(pos) = query.find(word) {
      let position_ratio = pos as f32 / query.len() as f32;
      weight *= 1.5 - position_ratio * 0.5;
    }

    // Length weight - Longer words are usually more specific
    weight *= (word.len() as f32 / 10.0f32).min(1.5f32);

    // Capitalization weight - Capitalized words in original text may be proper nouns
    if word.chars().any(|c| c.is_uppercase()) {
      weight *= 1.2;
    }

    weight
  }

  /// Check if word is a stop word
  pub fn is_stop_word(word: &str) -> bool {
    static CHINESE_STOP_WORDS: &[&str] = &[
      "的",
      "了",
      "在",
      "是",
      "我",
      "有",
      "和",
      "就",
      "不",
      "人",
      "都",
      "一",
      "一个",
      "上",
      "也",
      "很",
      "到",
      "说",
      "要",
      "去",
      "你",
      "会",
      "着",
      "没有",
      "看",
      "好",
      "自己",
      "这",
      "那",
      "他",
      "她",
      "它",
      "我们",
      "你们",
      "他们",
      "什么",
      "怎么",
      "为什么",
      "哪里",
      "什么时候",
      "如何",
      "能够",
      "可以",
      "应该",
      "必须",
      "需要",
      "想要",
      "喜欢",
      "知道",
      "认为",
      "觉得",
      "感觉",
      "听说",
      "看见",
      "发现",
    ];

    static ENGLISH_STOP_WORDS: &[&str] = &[
      "the", "a", "an", "and", "or", "but", "in", "on", "at", "to", "for", "of", "with", "by",
      "is", "are", "was", "were", "be", "been", "have", "has", "had", "do", "does", "did", "will",
      "would", "can", "could", "should", "may", "might", "must", "shall", "ought", "i", "you",
      "he", "she", "it", "we", "they", "me", "him", "her", "us", "them", "my", "your", "his",
      "her", "its", "our", "their", "mine", "yours", "ours", "theirs", "this", "that", "these",
      "those", "here", "there", "where", "when", "why", "how", "what", "which", "who", "whom",
      "whose", "all", "any", "some", "no", "not", "very", "so", "too", "more", "most", "much",
      "many", "few", "little", "less", "up", "down", "out", "off", "over", "under", "again",
      "further", "then", "once",
    ];

    CHINESE_STOP_WORDS.contains(&word) || ENGLISH_STOP_WORDS.contains(&word)
  }

  /// Check if word is a common symbol or meaningless character
  fn is_common_symbol(word: &str) -> bool {
    static COMMON_SYMBOLS: &[&str] = &[
      "@", "#", "$", "%", "&", "*", "+", "-", "=", "_", "|", "\\", "/", "?", "!", "(", ")", "[",
      "]", "{", "}", "<", ">", "~", "`", "^", ":", ";", "'", "\"", ",", ".", "...", "。", "，",
      "？", "！", "；", "：", "、", "——", "……",
    ];

    COMMON_SYMBOLS.contains(&word) || word.chars().all(|c| !c.is_alphanumeric())
  }

  /// Build intelligent search query
  pub fn build_smart_query(original_query: &str, keywords: &[String]) -> String {
    if keywords.is_empty() {
      warn!("No keywords extracted from query: '{}'", original_query);
      return original_query.to_string();
    }

    // If only one keyword, return directly
    if keywords.len() == 1 {
      return keywords[0].clone();
    }

    // Multi-keyword strategy
    let smart_query = if keywords.len() <= 3 {
      // Few keywords: use AND for exact match
      format!("({})", keywords.join(" AND "))
    } else {
      // Many keywords: use top 3 important keywords + OR other keywords
      let important_keywords = &keywords[..3];
      let other_keywords = &keywords[3..];

      if other_keywords.is_empty() {
        format!("({})", important_keywords.join(" AND "))
      } else {
        format!(
          "({}) OR ({})",
          important_keywords.join(" AND "),
          other_keywords.join(" OR ")
        )
      }
    };

    debug!(
      "Smart query built: '{}' -> '{}'",
      original_query, smart_query
    );
    smart_query
  }

  /// Detect query type
  pub fn detect_query_type(query: &str) -> QueryType {
    let trimmed = query.trim();

    if trimmed.is_empty() {
      return QueryType::Empty;
    }

    // Detect exact phrase search
    if trimmed.starts_with('"') && trimmed.ends_with('"') && trimmed.len() > 2 {
      return QueryType::ExactPhrase;
    }

    // Detect boolean search
    if trimmed.to_uppercase().contains(" AND ")
      || trimmed.to_uppercase().contains(" OR ")
      || trimmed.to_uppercase().contains(" NOT ")
    {
      return QueryType::Boolean;
    }

    // Detect fuzzy search
    if trimmed.contains('*') || trimmed.contains('?') {
      return QueryType::Fuzzy;
    }

    QueryType::Keywords
  }

  /// Optimize query based on type
  pub fn optimize_by_type(query: &str, query_type: QueryType) -> String {
    match query_type {
      QueryType::ExactPhrase => {
        // Keep exact phrase unchanged
        query.to_string()
      }
      QueryType::Boolean => {
        // Keep boolean query unchanged but clean extra spaces
        query.trim().to_string()
      }
      QueryType::Fuzzy => {
        // Keep fuzzy query unchanged
        query.to_string()
      }
      QueryType::Keywords => {
        // Perform full keyword optimization
        let processed = Self::preprocess_query(query);
        let keywords = Self::extract_keywords(&processed);
        Self::build_smart_query(query, &keywords)
      }
      QueryType::Empty => String::new(),
    }
  }

  /// Complete query optimization process
  pub fn optimize_query(query: &str) -> OptimizedQuery {
    let processor = Self::new();
    processor.optimize_query_advanced(query, None)
  }

  /// Advanced query optimization with context and intent
  pub fn optimize_query_advanced(
    &self,
    query: &str,
    context: Option<&QueryContext>,
  ) -> OptimizedQuery {
    let start_time = Instant::now();

    // 1. Check search cache first (if available) - Make it synchronous fallback
    let user_id = context.map(|c| c.user_id);
    if let Some(ref search_cache) = self.search_cache {
      // Try to get from cache without blocking
      // In a real async context, this would be awaited properly
      // For now, skip cache to avoid runtime nesting
      debug!("Skipping search cache to avoid runtime nesting in sync context");
    }

    // 2. Fallback to legacy cache
    let cache_key = format!("{}-{:?}", query, context);
    if let Some(cached) = self.legacy_cache.get(&cache_key) {
      self.stats.record_cache_hit();
      return cached;
    }

    let original = query.to_string();
    let query_type = Self::detect_query_type(query);

    let optimized = match query_type {
      QueryType::Empty => {
        return OptimizedQuery {
          original,
          optimized: String::new(),
          keywords: Vec::new(),
          query_type,
          confidence: 0.0,
          intent: None,
          processing_time: start_time.elapsed(),
        };
      }
      _ => Self::optimize_by_type(query, query_type.clone()),
    };

    let keywords = if query_type == QueryType::Keywords {
      let processed = Self::preprocess_query(query);
      Self::extract_keywords(&processed)
    } else {
      Vec::new()
    };

    // Calculate optimization confidence
    let confidence: f32 = Self::calculate_confidence(&original, &optimized, &keywords);

    // 2. Query intent detection
    let intent = self.detect_query_intent(query, &keywords, context);

    // 3. Adjust query strategy based on intent
    let adjusted_optimized = self.adjust_query_by_intent(&optimized, &intent);

    let result = OptimizedQuery {
      original,
      optimized: adjusted_optimized,
      keywords,
      query_type,
      confidence,
      intent: Some(intent),
      processing_time: start_time.elapsed(),
    };

    // 4. Update caches - Skip search cache to avoid runtime issues
    if self.search_cache.is_some() {
      debug!("Skipping search cache update to avoid runtime nesting in sync context");
    } else {
      self.legacy_cache.insert(cache_key, result.clone());
    }

    // 5. Record statistics
    self.stats.record_query_processed(start_time.elapsed());

    result
  }

  /// Query intent detection
  fn detect_query_intent(
    &self,
    query: &str,
    _keywords: &[String],
    context: Option<&QueryContext>,
  ) -> QueryIntent {
    // Time-related patterns
    static TIME_PATTERNS: &[&str] = &[
      "今天",
      "昨天",
      "本周",
      "上周",
      "本月",
      "today",
      "yesterday",
      "this week",
      "last week",
    ];

    // File type patterns
    static FILE_PATTERNS: &[&str] = &[
      "文件", "图片", "文档", "PDF", "Word", "Excel", "图像", "照片", "file", "image", "document",
      "photo", "picture",
    ];

    // Person-related patterns
    static PERSON_PATTERNS: &[&str] = &["@", "发送", "来自", "sent by", "from", "作者", "author"];

    let query_lower = query.to_lowercase();

    // Detect time intent
    if TIME_PATTERNS.iter().any(|p| query_lower.contains(p)) {
      return QueryIntent::TimeRange;
    }

    // Detect file search intent
    if FILE_PATTERNS.iter().any(|p| query_lower.contains(p)) {
      return QueryIntent::FileSearch;
    }

    // Detect person search intent
    if PERSON_PATTERNS.iter().any(|p| query_lower.contains(p)) || query.contains('@') {
      return QueryIntent::PersonSearch;
    }

    // Infer from context
    if let Some(ctx) = context {
      if ctx.is_in_file_view {
        return QueryIntent::FileSearch;
      }
      if ctx.recent_searches.iter().any(|s| s.contains("@")) {
        return QueryIntent::PersonSearch;
      }
    }

    // Default to content search
    QueryIntent::ContentSearch
  }

  /// Adjust query based on intent
  fn adjust_query_by_intent(&self, query: &str, intent: &QueryIntent) -> String {
    match intent {
      QueryIntent::TimeRange => {
        // Add time range filter for time search
        format!("{} filter:time", query)
      }
      QueryIntent::FileSearch => {
        // Add file type filter for file search
        format!("{} has:file", query)
      }
      QueryIntent::PersonSearch => {
        // Optimize for person search
        query.to_string()
      }
      _ => query.to_string(),
    }
  }

  /// Calculate query optimization confidence
  fn calculate_confidence(original: &str, optimized: &str, keywords: &[String]) -> f32 {
    let mut confidence: f32 = 0.5f32; // Base confidence

    // If keywords were extracted, increase confidence
    if !keywords.is_empty() {
      confidence += 0.2;
    }

    // If query was optimized (length changed), increase confidence
    if original.len() != optimized.len() {
      confidence += 0.1;
    }

    // If query contains multiple words, increase confidence
    if original.split_whitespace().count() > 1 {
      confidence += 0.1;
    }

    // If query length is reasonable, increase confidence
    if original.len() >= 3 && original.len() <= 50 {
      confidence += 0.1;
    }

    confidence.min(1.0f32)
  }
}

// ================================================================================================
// Query Cache
// ================================================================================================

/// High-performance query cache
pub struct QueryCache {
  /// Use DashMap for concurrent safety
  cache: DashMap<String, (OptimizedQuery, Instant)>,
  /// Maximum cache size
  max_size: usize,
  /// Cache expiration time
  ttl: Duration,
}

impl QueryCache {
  pub fn new(max_size: usize) -> Self {
    Self {
      cache: DashMap::with_capacity(max_size),
      max_size,
      ttl: Duration::from_secs(300), // 5 minutes
    }
  }

  pub fn get(&self, query: &str) -> Option<OptimizedQuery> {
    if let Some(entry) = self.cache.get(query) {
      let (result, timestamp) = entry.value();
      if timestamp.elapsed() < self.ttl {
        return Some(result.clone());
      } else {
        // Expired, remove
        drop(entry);
        self.cache.remove(query);
      }
    }
    None
  }

  pub fn insert(&self, query: String, result: OptimizedQuery) {
    // Simple LRU implementation
    if self.cache.len() >= self.max_size {
      // Remove oldest entry
      if let Some(oldest_key) = self
        .cache
        .iter()
        .min_by_key(|entry| entry.value().1)
        .map(|entry| entry.key().clone())
      {
        self.cache.remove(&oldest_key);
      }
    }

    self.cache.insert(query, (result, Instant::now()));
  }
}

// ================================================================================================
// Query Statistics
// ================================================================================================

/// Query statistics information
pub struct QueryStats {
  /// Query count
  query_count: Arc<RwLock<u64>>,
  /// Cache hits
  cache_hits: Arc<RwLock<u64>>,
  /// Average processing time
  avg_processing_time: Arc<RwLock<Duration>>,
}

impl QueryStats {
  pub fn new() -> Self {
    Self {
      query_count: Arc::new(RwLock::new(0)),
      cache_hits: Arc::new(RwLock::new(0)),
      avg_processing_time: Arc::new(RwLock::new(Duration::ZERO)),
    }
  }

  pub fn record_query_processed(&self, processing_time: Duration) {
    let mut count = self.query_count.write().unwrap();
    *count += 1;

    let mut avg = self.avg_processing_time.write().unwrap();
    let total_time = avg.as_millis() * (*count - 1) as u128 + processing_time.as_millis();
    *avg = Duration::from_millis((total_time / *count as u128) as u64);
  }

  pub fn record_cache_hit(&self) {
    let mut hits = self.cache_hits.write().unwrap();
    *hits += 1;
  }

  pub fn get_stats(&self) -> (u64, u64, Duration) {
    let count = *self.query_count.read().unwrap();
    let hits = *self.cache_hits.read().unwrap();
    let avg_time = *self.avg_processing_time.read().unwrap();
    (count, hits, avg_time)
  }
}

/// Search suggestion generator
pub struct SearchSuggestionGenerator {
  /// Historical query cache
  history_cache: Arc<DashMap<String, Vec<String>>>,
  /// Trending searches
  trending_searches: Arc<RwLock<Vec<String>>>,
}

impl Default for SearchSuggestionGenerator {
  fn default() -> Self {
    Self::new()
  }
}

impl SearchSuggestionGenerator {
  pub fn new() -> Self {
    Self {
      history_cache: Arc::new(DashMap::new()),
      trending_searches: Arc::new(RwLock::new(Vec::new())),
    }
  }

  /// Generate query-based search suggestions
  pub fn generate_suggestions(
    &self,
    partial_query: &str,
    user_id: i64,
    limit: usize,
  ) -> Vec<String> {
    if partial_query.len() < 2 {
      return Vec::new();
    }

    let mut suggestions = Vec::new();
    let partial_lower = partial_query.to_lowercase();

    // 1. User history search suggestions
    if let Some(history) = self.history_cache.get(&user_id.to_string()) {
      for query in history.value() {
        if query.to_lowercase().starts_with(&partial_lower) {
          suggestions.push(query.clone());
        }
      }
    }

    // 2. Trending search suggestions
    let trending = self.trending_searches.read().unwrap();
    for query in trending.iter() {
      if query.to_lowercase().contains(&partial_lower) {
        suggestions.push(query.clone());
      }
    }

    // 3. Smart completion suggestions
    suggestions.extend(Self::generate_completion_suggestions(
      partial_query,
      limit / 3,
    ));
    suggestions.extend(Self::generate_semantic_suggestions(
      partial_query,
      limit / 3,
    ));
    suggestions.extend(Self::generate_context_suggestions(partial_query, limit / 3));

    // Deduplicate and limit
    suggestions.sort();
    suggestions.dedup();
    suggestions.truncate(limit);

    suggestions
  }

  /// Generate auto-completion suggestions
  fn generate_completion_suggestions(partial_query: &str, limit: usize) -> Vec<String> {
    let common_completions = [
      "文件", "图片", "文档", "报告", "会议", "项目", "任务", "计划", "file", "image", "document",
      "report", "meeting", "project", "task", "plan",
    ];

    common_completions
      .iter()
      .filter(|&completion| completion.starts_with(partial_query))
      .take(limit)
      .map(|&s| s.to_string())
      .collect()
  }

  /// Generate semantic-related suggestions
  fn generate_semantic_suggestions(partial_query: &str, limit: usize) -> Vec<String> {
    let mut suggestions = Vec::new();

    if partial_query.contains("文件") || partial_query.contains("file") {
      suggestions.extend_from_slice(&[
        "PDF文件".to_string(),
        "Word文档".to_string(),
        "Excel表格".to_string(),
        "图片文件".to_string(),
      ]);
    }

    if partial_query.contains("会议") || partial_query.contains("meeting") {
      suggestions.extend_from_slice(&[
        "会议纪要".to_string(),
        "会议室预订".to_string(),
        "视频会议".to_string(),
        "会议议程".to_string(),
      ]);
    }

    suggestions.truncate(limit);
    suggestions
  }

  /// Generate context-related suggestions
  fn generate_context_suggestions(_partial_query: &str, _limit: usize) -> Vec<String> {
    // This can generate suggestions based on current chat context
    Vec::new()
  }

  /// Record user search history
  pub fn record_search(&self, user_id: i64, query: String) {
    let key = user_id.to_string();
    let mut history = self.history_cache.entry(key).or_insert_with(Vec::new);

    // Remove duplicates
    history.retain(|q| q != &query);

    // Add to beginning
    history.insert(0, query);

    // Limit history size
    if history.len() > 20 {
      history.truncate(20);
    }
  }

  /// Update trending searches
  pub fn update_trending(&self, searches: Vec<String>) {
    let mut trending = self.trending_searches.write().unwrap();
    *trending = searches;
  }
}

// ================================================================================================
// Factory Functions - Query Processor with Optimizations
// ================================================================================================

/// Create high-performance query processor with search cache
pub fn create_optimized_query_processor(
  search_cache: Arc<super::cache::SearchCacheService>,
) -> QueryProcessor {
  QueryProcessor::new_with_search_cache(search_cache)
}

/// Create query processor with custom cache configuration
pub fn create_query_processor_with_config(
  search_cache: Arc<super::cache::SearchCacheService>,
) -> QueryProcessor {
  QueryProcessor::new_with_search_cache(search_cache)
}

/// Create legacy query processor (for backward compatibility)
pub fn create_legacy_query_processor() -> QueryProcessor {
  QueryProcessor::new()
}

// ================================================================================================
// Test Module
// ================================================================================================

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_preprocess_query() {
    assert_eq!(
      QueryProcessor::preprocess_query("  Hello@#$  World!!!  "),
      "hello world"
    );
    assert_eq!(QueryProcessor::preprocess_query("测试@#查询"), "测试 查询");
    assert_eq!(QueryProcessor::preprocess_query(""), "");
  }

  #[test]
  fn test_extract_keywords() {
    let keywords = QueryProcessor::extract_keywords("hello world test");
    assert!(keywords.contains(&"hello".to_string()));
    assert!(keywords.contains(&"world".to_string()));
    assert!(keywords.contains(&"test".to_string()));

    let keywords_with_stop_words = QueryProcessor::extract_keywords("the quick brown fox");
    assert!(!keywords_with_stop_words.contains(&"the".to_string()));
    assert!(keywords_with_stop_words.contains(&"quick".to_string()));
  }

  #[test]
  fn test_is_stop_word() {
    assert!(QueryProcessor::is_stop_word("the"));
    assert!(QueryProcessor::is_stop_word("的"));
    assert!(!QueryProcessor::is_stop_word("hello"));
    assert!(!QueryProcessor::is_stop_word("测试"));
  }

  #[test]
  fn test_detect_query_type() {
    assert_eq!(
      QueryProcessor::detect_query_type("\"exact phrase\""),
      QueryType::ExactPhrase
    );
    assert_eq!(
      QueryProcessor::detect_query_type("word1 AND word2"),
      QueryType::Boolean
    );
    assert_eq!(QueryProcessor::detect_query_type("test*"), QueryType::Fuzzy);
    assert_eq!(
      QueryProcessor::detect_query_type("normal query"),
      QueryType::Keywords
    );
    assert_eq!(QueryProcessor::detect_query_type(""), QueryType::Empty);
  }

  #[test]
  fn test_optimize_query() {
    let result = QueryProcessor::optimize_query("  hello   world  ");
    assert_eq!(result.optimized, "(hello AND world)");
    assert!(result.keywords.contains(&"hello".to_string()));
    assert!(result.keywords.contains(&"world".to_string()));
    assert!(result.confidence > 0.5);
  }

  #[test]
  fn test_search_suggestions() {
    let generator = SearchSuggestionGenerator::new();
    generator.record_search(1, "文件搜索".to_string());
    let suggestions = generator.generate_suggestions("文", 1, 5);
    assert!(!suggestions.is_empty());
  }

  #[test]
  fn test_language_detection() {
    assert_eq!(
      QueryProcessor::detect_language("hello world"),
      LanguageType::English
    );
    assert_eq!(
      QueryProcessor::detect_language("你好世界"),
      LanguageType::Chinese
    );
    assert_eq!(
      QueryProcessor::detect_language("hello 世界"),
      LanguageType::Mixed
    );
  }

  #[test]
  fn test_advanced_query_optimization() {
    let processor = QueryProcessor::new();
    let context = QueryContext {
      chat_id: Some(1),
      user_id: 1,
      is_in_file_view: false,
      recent_searches: vec![],
      timezone: "UTC".to_string(),
    };

    let result = processor.optimize_query_advanced("找文件 meeting notes", Some(&context));
    assert!(result.confidence > 0.5);
    assert!(result.intent == Some(QueryIntent::FileSearch));
  }
}
