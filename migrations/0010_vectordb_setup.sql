-- Vector database setup for embeddings and semantic search
-- Migration: 0010_vectordb_setup.sql

-- Install pgvector extension if not exists
CREATE EXTENSION IF NOT EXISTS vector;

-- Create embeddings table for message semantic search
CREATE TABLE IF NOT EXISTS message_embeddings (
    id BIGSERIAL PRIMARY KEY,
    message_id BIGINT NOT NULL REFERENCES messages(id) ON DELETE CASCADE,
    embedding VECTOR(1536), -- OpenAI embedding dimensions
    embedding_model VARCHAR(100) DEFAULT 'text-embedding-ada-002',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Create user embeddings for user similarity
CREATE TABLE IF NOT EXISTS user_embeddings (
    id BIGSERIAL PRIMARY KEY,
    user_id BIGINT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    profile_embedding VECTOR(1536),
    interests_embedding VECTOR(1536),
    embedding_model VARCHAR(100) DEFAULT 'text-embedding-ada-002',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Create chat embeddings for chat topic classification
CREATE TABLE IF NOT EXISTS chat_embeddings (
    id BIGSERIAL PRIMARY KEY,
    chat_id BIGINT NOT NULL REFERENCES chats(id) ON DELETE CASCADE,
    topic_embedding VECTOR(1536),
    summary_embedding VECTOR(1536),
    embedding_model VARCHAR(100) DEFAULT 'text-embedding-ada-002',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Create indexes for vector similarity search
CREATE INDEX IF NOT EXISTS idx_message_embeddings_message_id ON message_embeddings(message_id);
CREATE INDEX IF NOT EXISTS idx_message_embeddings_vector ON message_embeddings USING ivfflat (embedding vector_cosine_ops) WITH (lists = 100);

CREATE INDEX IF NOT EXISTS idx_user_embeddings_user_id ON user_embeddings(user_id);
CREATE INDEX IF NOT EXISTS idx_user_profile_embeddings_vector ON user_embeddings USING ivfflat (profile_embedding vector_cosine_ops) WITH (lists = 100);
CREATE INDEX IF NOT EXISTS idx_user_interests_embeddings_vector ON user_embeddings USING ivfflat (interests_embedding vector_cosine_ops) WITH (lists = 100);

CREATE INDEX IF NOT EXISTS idx_chat_embeddings_chat_id ON chat_embeddings(chat_id);
CREATE INDEX IF NOT EXISTS idx_chat_topic_embeddings_vector ON chat_embeddings USING ivfflat (topic_embedding vector_cosine_ops) WITH (lists = 100);
CREATE INDEX IF NOT EXISTS idx_chat_summary_embeddings_vector ON chat_embeddings USING ivfflat (summary_embedding vector_cosine_ops) WITH (lists = 100);

-- Create semantic search materialized view
CREATE MATERIALIZED VIEW IF NOT EXISTS semantic_search_index AS
SELECT 
    m.id as message_id,
    m.content,
    m.chat_id,
    m.sender_id,
    m.created_at,
    me.embedding,
    c.name as chat_name,
    u.fullname as sender_name
FROM messages m
JOIN message_embeddings me ON m.id = me.message_id
JOIN chats c ON m.chat_id = c.id
JOIN users u ON m.sender_id = u.id;

-- Create index on materialized view
CREATE INDEX IF NOT EXISTS idx_semantic_search_embedding ON semantic_search_index USING ivfflat (embedding vector_cosine_ops) WITH (lists = 100);
CREATE INDEX IF NOT EXISTS idx_semantic_search_chat_id ON semantic_search_index(chat_id);
CREATE INDEX IF NOT EXISTS idx_semantic_search_created_at ON semantic_search_index(created_at);