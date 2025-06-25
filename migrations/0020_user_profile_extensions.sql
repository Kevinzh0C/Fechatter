-- User Profile Extensions
-- Migration: 0020_user_profile_extensions.sql
-- Purpose: Add profile fields to support comprehensive user profile management

-- Add profile-related columns to users table
ALTER TABLE users 
ADD COLUMN IF NOT EXISTS phone VARCHAR(20),
ADD COLUMN IF NOT EXISTS title VARCHAR(100),
ADD COLUMN IF NOT EXISTS department VARCHAR(100),
ADD COLUMN IF NOT EXISTS avatar_url VARCHAR(500),
ADD COLUMN IF NOT EXISTS bio TEXT,
ADD COLUMN IF NOT EXISTS timezone VARCHAR(50) DEFAULT 'UTC',
ADD COLUMN IF NOT EXISTS language VARCHAR(10) DEFAULT 'en',
ADD COLUMN IF NOT EXISTS last_active_at TIMESTAMPTZ;

-- Add user settings table for profile preferences
CREATE TABLE IF NOT EXISTS user_settings (
    user_id BIGINT PRIMARY KEY REFERENCES users(id) ON DELETE CASCADE,
    -- Notification preferences
    email_notifications BOOLEAN DEFAULT true,
    push_notifications BOOLEAN DEFAULT true,
    desktop_notifications BOOLEAN DEFAULT true,
    notification_sound VARCHAR(20) DEFAULT 'all' CHECK (notification_sound IN ('all', 'mentions', 'none')),
    
    -- Presence preferences
    show_online_status BOOLEAN DEFAULT true,
    auto_away BOOLEAN DEFAULT false,
    auto_away_minutes INTEGER DEFAULT 15 CHECK (auto_away_minutes >= 5 AND auto_away_minutes <= 120),
    
    -- UI preferences
    theme VARCHAR(10) DEFAULT 'light' CHECK (theme IN ('light', 'dark', 'auto')),
    message_display VARCHAR(15) DEFAULT 'comfortable' CHECK (message_display IN ('comfortable', 'compact')),
    
    -- Privacy preferences
    profile_visibility VARCHAR(20) DEFAULT 'workspace' CHECK (profile_visibility IN ('public', 'workspace', 'private')),
    show_email BOOLEAN DEFAULT false,
    show_phone BOOLEAN DEFAULT false,
    
    -- Advanced preferences (JSON for flexibility)
    custom_preferences JSONB DEFAULT '{}',
    
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Create user activity log table for tracking profile changes
CREATE TABLE IF NOT EXISTS user_activity_log (
    id BIGSERIAL PRIMARY KEY,
    user_id BIGINT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    activity_type VARCHAR(50) NOT NULL,
    description TEXT,
    ip_address INET,
    user_agent TEXT,
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Create indexes for performance
CREATE INDEX IF NOT EXISTS idx_users_department ON users(department) WHERE department IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_users_title ON users(title) WHERE title IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_users_timezone ON users(timezone);
CREATE INDEX IF NOT EXISTS idx_users_language ON users(language);
CREATE INDEX IF NOT EXISTS idx_users_last_active_at ON users(last_active_at);

CREATE INDEX IF NOT EXISTS idx_user_settings_theme ON user_settings(theme);
CREATE INDEX IF NOT EXISTS idx_user_settings_show_online_status ON user_settings(show_online_status);

CREATE INDEX IF NOT EXISTS idx_user_activity_log_user_id ON user_activity_log(user_id);
CREATE INDEX IF NOT EXISTS idx_user_activity_log_activity_type ON user_activity_log(activity_type);
CREATE INDEX IF NOT EXISTS idx_user_activity_log_created_at ON user_activity_log(created_at);

-- Function to automatically create user settings when a user is created
CREATE OR REPLACE FUNCTION create_default_user_settings()
RETURNS TRIGGER AS $$
BEGIN
    INSERT INTO user_settings (user_id) VALUES (NEW.id);
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Trigger to create default settings for new users
DROP TRIGGER IF EXISTS create_user_settings_trigger ON users;
CREATE TRIGGER create_user_settings_trigger
    AFTER INSERT ON users
    FOR EACH ROW
    EXECUTE FUNCTION create_default_user_settings();

-- Function to log user activity
CREATE OR REPLACE FUNCTION log_user_activity(
    p_user_id BIGINT,
    p_activity_type VARCHAR(50),
    p_description TEXT DEFAULT NULL,
    p_ip_address INET DEFAULT NULL,
    p_user_agent TEXT DEFAULT NULL,
    p_metadata JSONB DEFAULT '{}'
)
RETURNS void AS $$
BEGIN
    INSERT INTO user_activity_log (
        user_id, 
        activity_type, 
        description, 
        ip_address, 
        user_agent, 
        metadata
    ) VALUES (
        p_user_id, 
        p_activity_type, 
        p_description, 
        p_ip_address, 
        p_user_agent, 
        p_metadata
    );
END;
$$ LANGUAGE plpgsql;

-- Function to update user last_active_at timestamp
CREATE OR REPLACE FUNCTION update_user_last_active()
RETURNS TRIGGER AS $$
BEGIN
    UPDATE users 
    SET last_active_at = NOW() 
    WHERE id = NEW.sender_id;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Trigger to update last_active_at when user sends messages
DROP TRIGGER IF EXISTS update_last_active_on_message ON messages;
CREATE TRIGGER update_last_active_on_message
    AFTER INSERT ON messages
    FOR EACH ROW
    EXECUTE FUNCTION update_user_last_active();

-- Function to update user settings updated_at timestamp
CREATE OR REPLACE FUNCTION update_user_settings_timestamp()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Trigger to update user_settings timestamp
DROP TRIGGER IF EXISTS update_user_settings_timestamp_trigger ON user_settings;
CREATE TRIGGER update_user_settings_timestamp_trigger
    BEFORE UPDATE ON user_settings
    FOR EACH ROW
    EXECUTE FUNCTION update_user_settings_timestamp();

-- Function to log profile changes
CREATE OR REPLACE FUNCTION log_profile_changes()
RETURNS TRIGGER AS $$
DECLARE
    changes JSONB := '{}';
BEGIN
    -- Log significant profile changes
    IF OLD.fullname != NEW.fullname THEN
        changes := changes || jsonb_build_object('fullname', jsonb_build_object('old', OLD.fullname, 'new', NEW.fullname));
    END IF;
    
    IF OLD.email != NEW.email THEN
        changes := changes || jsonb_build_object('email', jsonb_build_object('old', OLD.email, 'new', NEW.email));
    END IF;
    
    IF (OLD.phone IS DISTINCT FROM NEW.phone) THEN
        changes := changes || jsonb_build_object('phone', jsonb_build_object('old', OLD.phone, 'new', NEW.phone));
    END IF;
    
    IF (OLD.title IS DISTINCT FROM NEW.title) THEN
        changes := changes || jsonb_build_object('title', jsonb_build_object('old', OLD.title, 'new', NEW.title));
    END IF;
    
    IF (OLD.department IS DISTINCT FROM NEW.department) THEN
        changes := changes || jsonb_build_object('department', jsonb_build_object('old', OLD.department, 'new', NEW.department));
    END IF;
    
    IF (OLD.bio IS DISTINCT FROM NEW.bio) THEN
        changes := changes || jsonb_build_object('bio', jsonb_build_object('old', 'redacted', 'new', 'redacted'));
    END IF;
    
    -- Log changes if any occurred
    IF changes != '{}' THEN
        PERFORM log_user_activity(
            NEW.id,
            'profile_updated',
            'User profile fields updated',
            NULL,
            NULL,
            changes
        );
    END IF;
    
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Trigger to log profile changes
DROP TRIGGER IF EXISTS log_profile_changes_trigger ON users;
CREATE TRIGGER log_profile_changes_trigger
    AFTER UPDATE ON users
    FOR EACH ROW
    EXECUTE FUNCTION log_profile_changes();

-- Add some helpful views for common queries
CREATE OR REPLACE VIEW user_profiles AS
SELECT 
    u.id,
    u.fullname,
    u.email,
    u.phone,
    u.title,
    u.department,
    u.avatar_url,
    u.bio,
    u.timezone,
    u.language,
    u.status,
    u.last_active_at,
    u.created_at,
    u.workspace_id,
    us.email_notifications,
    us.push_notifications,
    us.desktop_notifications,
    us.notification_sound,
    us.show_online_status,
    us.auto_away,
    us.auto_away_minutes,
    us.theme,
    us.message_display,
    us.profile_visibility,
    us.show_email,
    us.show_phone,
    up.status as presence_status,
    up.custom_status as presence_message,
    up.last_seen
FROM users u
LEFT JOIN user_settings us ON u.id = us.user_id
LEFT JOIN user_presence up ON u.id = up.user_id;

-- Create a view for user activity analytics
CREATE OR REPLACE VIEW user_activity_summary AS
SELECT 
    user_id,
    COUNT(*) as total_activities,
    COUNT(CASE WHEN activity_type = 'profile_updated' THEN 1 END) as profile_updates,
    COUNT(CASE WHEN activity_type = 'login' THEN 1 END) as login_count,
    MAX(created_at) as last_activity,
    MIN(created_at) as first_activity
FROM user_activity_log
GROUP BY user_id;

-- Add some sample data for existing users (if any exist)
-- This ensures existing users get default settings
INSERT INTO user_settings (user_id)
SELECT id FROM users 
WHERE id NOT IN (SELECT user_id FROM user_settings);

-- Add helpful comments
COMMENT ON TABLE user_settings IS 'User preferences and settings for profile customization';
COMMENT ON TABLE user_activity_log IS 'Audit log for user activities and profile changes';
COMMENT ON VIEW user_profiles IS 'Comprehensive view of user profiles with settings and presence';

COMMENT ON COLUMN users.phone IS 'User contact phone number (optional)';
COMMENT ON COLUMN users.title IS 'User job title or role';
COMMENT ON COLUMN users.department IS 'User department or team';
COMMENT ON COLUMN users.avatar_url IS 'URL to user profile picture';
COMMENT ON COLUMN users.bio IS 'User biography or description';
COMMENT ON COLUMN users.timezone IS 'User preferred timezone (default: UTC)';
COMMENT ON COLUMN users.language IS 'User preferred language code (default: en)';
COMMENT ON COLUMN users.last_active_at IS 'Last time user was active on the platform';