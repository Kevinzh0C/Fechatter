-- User features: presence, templates, file sharing
-- Migration: 0013_user_features.sql

-- Add user presence and activity tracking
CREATE TABLE IF NOT EXISTS user_presence (
    user_id BIGINT PRIMARY KEY REFERENCES users(id) ON DELETE CASCADE,
    status VARCHAR(20) DEFAULT 'offline' CHECK (status IN ('online', 'away', 'busy', 'offline')),
    custom_status VARCHAR(100),
    last_seen TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    is_mobile BOOLEAN DEFAULT FALSE,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Add typing indicators
CREATE TABLE IF NOT EXISTS typing_indicators (
    id BIGSERIAL PRIMARY KEY,
    chat_id BIGINT NOT NULL REFERENCES chats(id) ON DELETE CASCADE,
    user_id BIGINT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    started_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at TIMESTAMPTZ NOT NULL DEFAULT (NOW() + INTERVAL '10 seconds'),
    UNIQUE(chat_id, user_id)
);

-- Add chat templates for quick responses
CREATE TABLE IF NOT EXISTS chat_templates (
    id BIGSERIAL PRIMARY KEY,
    chat_id BIGINT REFERENCES chats(id) ON DELETE CASCADE,
    user_id BIGINT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    name VARCHAR(100) NOT NULL,
    content TEXT NOT NULL,
    category VARCHAR(50),
    usage_count INTEGER DEFAULT 0,
    is_shared BOOLEAN DEFAULT FALSE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Add file sharing permissions and tracking
CREATE TABLE IF NOT EXISTS file_shares (
    id BIGSERIAL PRIMARY KEY,
    message_id BIGINT NOT NULL REFERENCES messages(id) ON DELETE CASCADE,
    file_path VARCHAR(500) NOT NULL,
    file_name VARCHAR(255) NOT NULL,
    file_size BIGINT NOT NULL,
    mime_type VARCHAR(100),
    uploaded_by BIGINT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    access_count INTEGER DEFAULT 0,
    expires_at TIMESTAMPTZ,
    is_public BOOLEAN DEFAULT FALSE,
    checksum VARCHAR(64),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Create indexes for user features
CREATE INDEX IF NOT EXISTS idx_user_presence_status ON user_presence(status);
CREATE INDEX IF NOT EXISTS idx_typing_indicators_chat_id ON typing_indicators(chat_id);
CREATE INDEX IF NOT EXISTS idx_typing_indicators_expires_at ON typing_indicators(expires_at);
CREATE INDEX IF NOT EXISTS idx_chat_templates_user_id ON chat_templates(user_id);
CREATE INDEX IF NOT EXISTS idx_chat_templates_chat_id ON chat_templates(chat_id);
CREATE INDEX IF NOT EXISTS idx_file_shares_message_id ON file_shares(message_id);
CREATE INDEX IF NOT EXISTS idx_file_shares_uploaded_by ON file_shares(uploaded_by);

-- Function to clean expired typing indicators
CREATE OR REPLACE FUNCTION cleanup_expired_typing_indicators()
RETURNS void AS $$
BEGIN
    DELETE FROM typing_indicators WHERE expires_at < NOW();
END;
$$ LANGUAGE plpgsql;

-- Function to update user presence
CREATE OR REPLACE FUNCTION update_user_presence()
RETURNS TRIGGER AS $$
BEGIN
    INSERT INTO user_presence (user_id, last_seen)
    VALUES (NEW.sender_id, NEW.created_at)
    ON CONFLICT (user_id) 
    DO UPDATE SET 
        last_seen = NEW.created_at,
        updated_at = NEW.created_at;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Trigger to update user presence when sending messages
DROP TRIGGER IF EXISTS update_presence_on_message ON messages;
CREATE TRIGGER update_presence_on_message
    AFTER INSERT ON messages
    FOR EACH ROW
    EXECUTE FUNCTION update_user_presence();