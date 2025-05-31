// Vector database implementations
pub mod pgvector;
pub mod pinecone;

// Re-export types from fechatter_core with explicit paths
pub use fechatter_core::models::vector_db::{
  MessageChunk, MetadataFilter, VectorDatabase, VectorSearchResult,
};

// Re-export implementations
pub use pgvector::PgVectorDatabase;
pub use pinecone::PineconeClient;

// Temporary placeholder for vector database functionality
pub struct PlaceholderVectorDb;

impl PlaceholderVectorDb {
  pub fn new() -> Self {
    Self
  }
}
