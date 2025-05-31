// AI service implementations
pub mod agents;
pub mod cohere;
pub mod huggingface;
pub mod hybrid_search;
pub mod openai;
pub mod rag_indexer;

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
