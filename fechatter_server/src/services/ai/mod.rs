// AI service implementations
// Core AI services using ai_sdk (basic operations)
pub mod core;

// Simple AI implementations (delegate to LLM)
pub mod simple_topics;
pub mod simple_workflow;

// Specialized AI services (chat-specific features)
pub mod agents;
pub mod cohere;
pub mod huggingface;
pub mod hybrid_search;
pub mod openai;
pub mod rag_indexer;

// Optional workflow system
pub mod workflow;

// Re-export core types for convenience
pub use core::{AiServiceAdapter, AiAdapter, AiService, AiMessage, AiRole, OpenaiAdapter, OllamaAdapter};

// Re-export simple implementations
pub use simple_topics::SimpleTopicExtractor;
pub use simple_workflow::SimpleWorkflow;

// Re-export main types
pub use cohere::CohereClient;
pub use huggingface::HuggingFaceClient;
pub use hybrid_search::{HybridSearchConfig, HybridSearchResult, HybridSearchService};
pub use openai::OpenAIClient;
pub use rag_indexer::{
  ChatIndex, DocumentType, QAChain, RAGDocument, RAGMessageIndexer, RAGQueryResult, RetrievalChain,
  SummaryStyle,
};

// TODO: Add OpenAI, Anthropic, Local LLM implementations
