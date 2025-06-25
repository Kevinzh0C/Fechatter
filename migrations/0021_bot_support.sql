-- Bot Support Migration
-- Migration: 0021_bot_support.sql
-- Purpose: Add bot support to the users table

-- Add is_bot column to users table
ALTER TABLE users 
ADD COLUMN IF NOT EXISTS is_bot BOOLEAN DEFAULT FALSE NOT NULL;

-- Create index for efficient bot queries
CREATE INDEX IF NOT EXISTS idx_users_is_bot ON users(is_bot) WHERE is_bot = TRUE;

-- Create bot-specific configuration table
CREATE TABLE IF NOT EXISTS bot_config (
    user_id BIGINT PRIMARY KEY REFERENCES users(id) ON DELETE CASCADE,
    ai_model VARCHAR(100) DEFAULT 'gpt-4o-mini',
    embed_model VARCHAR(100) DEFAULT 'text-embedding-3-small',
    max_response_length INTEGER DEFAULT 2000,
    response_delay_ms INTEGER DEFAULT 1000,
    vector_size INTEGER DEFAULT 1536,
    enabled BOOLEAN DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Create bot analytics table for tracking bot performance
CREATE TABLE IF NOT EXISTS bot_analytics (
    id BIGSERIAL PRIMARY KEY,
    bot_id BIGINT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    chat_id BIGINT NOT NULL REFERENCES chats(id) ON DELETE CASCADE,
    request_type VARCHAR(50) NOT NULL,
    response_time_ms INTEGER NOT NULL,
    tokens_used INTEGER DEFAULT 0,
    success BOOLEAN NOT NULL,
    error_message TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Create indexes for bot analytics
CREATE INDEX IF NOT EXISTS idx_bot_analytics_bot_id ON bot_analytics(bot_id);
CREATE INDEX IF NOT EXISTS idx_bot_analytics_chat_id ON bot_analytics(chat_id);
CREATE INDEX IF NOT EXISTS idx_bot_analytics_created_at ON bot_analytics(created_at DESC);
CREATE INDEX IF NOT EXISTS idx_bot_analytics_success ON bot_analytics(success);

-- Insert default bot user if not exists
INSERT INTO users (id, fullname, email, password_hash, status, is_bot, workspace_id)
VALUES (
    999999,
    'Fechatter Bot',
    'bot@fechatter.com',
    'bot_no_login',
    'active',
    TRUE,
    1
) ON CONFLICT (id) DO UPDATE SET
    is_bot = TRUE,
    fullname = 'Fechatter Bot',
    email = 'bot@fechatter.com';

-- Insert default bot configuration
INSERT INTO bot_config (user_id)
VALUES (999999)
ON CONFLICT (user_id) DO NOTHING;

-- Function to automatically create bot config when bot user is created
CREATE OR REPLACE FUNCTION create_bot_config_on_bot_user()
RETURNS TRIGGER AS $$
BEGIN
    IF NEW.is_bot = TRUE THEN
        INSERT INTO bot_config (user_id) VALUES (NEW.id)
        ON CONFLICT (user_id) DO NOTHING;
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Trigger to auto-create bot config
DROP TRIGGER IF EXISTS create_bot_config_trigger ON users;
CREATE TRIGGER create_bot_config_trigger
    AFTER INSERT OR UPDATE ON users
    FOR EACH ROW
    WHEN (NEW.is_bot = TRUE)
    EXECUTE FUNCTION create_bot_config_on_bot_user();

-- Function to update bot config timestamp
CREATE OR REPLACE FUNCTION update_bot_config_timestamp()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Trigger to update bot config timestamp
CREATE TRIGGER update_bot_config_timestamp
    BEFORE UPDATE ON bot_config
    FOR EACH ROW
    EXECUTE FUNCTION update_bot_config_timestamp();

-- Add helpful comments
COMMENT ON COLUMN users.is_bot IS 'Indicates whether this user is a bot (automated agent)';
COMMENT ON TABLE bot_config IS 'Configuration settings for bot users';
COMMENT ON TABLE bot_analytics IS 'Performance and usage analytics for bot interactions';

-- Create view for bot users with configuration
CREATE OR REPLACE VIEW bot_users AS
SELECT 
    u.id,
    u.fullname,
    u.email,
    u.status,
    u.created_at,
    u.workspace_id,
    bc.ai_model,
    bc.embed_model,
    bc.max_response_length,
    bc.response_delay_ms,
    bc.vector_size,
    bc.enabled as bot_enabled,
    bc.updated_at as config_updated_at
FROM users u
LEFT JOIN bot_config bc ON u.id = bc.user_id
WHERE u.is_bot = TRUE;

COMMENT ON VIEW bot_users IS 'Comprehensive view of bot users with their configurations'; 