-- Translation Quota Management Migration
-- Migration: 0027_translation_quota.sql
-- Purpose: Add translation quota tracking and bot translation logs

-- Create bot translation logs table for quota tracking
CREATE TABLE IF NOT EXISTS bot_translation_logs (
    id BIGSERIAL PRIMARY KEY,
    user_id BIGINT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    message_id BIGINT REFERENCES messages(id) ON DELETE SET NULL,
    source_language VARCHAR(10),
    target_language VARCHAR(10) NOT NULL,
    character_count INTEGER DEFAULT 0,
    translation_success BOOLEAN DEFAULT TRUE,
    api_response_time_ms INTEGER,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Create indexes for efficient quota queries
CREATE INDEX IF NOT EXISTS idx_bot_translation_logs_user_id ON bot_translation_logs(user_id);
CREATE INDEX IF NOT EXISTS idx_bot_translation_logs_created_at ON bot_translation_logs(created_at DESC);
CREATE INDEX IF NOT EXISTS idx_bot_translation_logs_user_date ON bot_translation_logs(user_id, DATE(created_at));
CREATE INDEX IF NOT EXISTS idx_bot_translation_logs_success ON bot_translation_logs(translation_success);

-- Create daily quota summary table for performance optimization
CREATE TABLE IF NOT EXISTS bot_daily_quota_summary (
    id BIGSERIAL PRIMARY KEY,
    user_id BIGINT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    date DATE NOT NULL,
    translation_count INTEGER DEFAULT 0,
    character_count INTEGER DEFAULT 0,
    last_updated TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(user_id, date)
);

-- Create indexes for quota summary
CREATE INDEX IF NOT EXISTS idx_bot_daily_quota_user_date ON bot_daily_quota_summary(user_id, date);
CREATE INDEX IF NOT EXISTS idx_bot_daily_quota_date ON bot_daily_quota_summary(date DESC);

-- Function to update daily quota summary
CREATE OR REPLACE FUNCTION update_daily_quota_summary()
RETURNS TRIGGER AS $$
DECLARE
    log_date DATE;
BEGIN
    -- Get the date of the translation log
    log_date := DATE(NEW.created_at);
    
    -- Update or insert daily summary
    INSERT INTO bot_daily_quota_summary (user_id, date, translation_count, character_count, last_updated)
    VALUES (NEW.user_id, log_date, 1, COALESCE(NEW.character_count, 0), NOW())
    ON CONFLICT (user_id, date)
    DO UPDATE SET 
        translation_count = bot_daily_quota_summary.translation_count + 1,
        character_count = bot_daily_quota_summary.character_count + COALESCE(NEW.character_count, 0),
        last_updated = NOW();
    
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Trigger to automatically update daily quota summary
CREATE TRIGGER update_daily_quota_summary_trigger
    AFTER INSERT ON bot_translation_logs
    FOR EACH ROW
    EXECUTE FUNCTION update_daily_quota_summary();

-- Create function to get user's daily quota usage
CREATE OR REPLACE FUNCTION get_user_daily_quota(
    p_user_id BIGINT,
    p_date DATE DEFAULT CURRENT_DATE
) RETURNS INTEGER AS $$
DECLARE
    quota_used INTEGER;
BEGIN
    SELECT COALESCE(translation_count, 0)
    INTO quota_used
    FROM bot_daily_quota_summary
    WHERE user_id = p_user_id AND date = p_date;
    
    RETURN COALESCE(quota_used, 0);
END;
$$ LANGUAGE plpgsql;

-- Create function to check if user can make translation request
CREATE OR REPLACE FUNCTION can_user_translate(
    p_user_id BIGINT,
    p_daily_limit INTEGER DEFAULT 20
) RETURNS BOOLEAN AS $$
DECLARE
    current_usage INTEGER;
BEGIN
    current_usage := get_user_daily_quota(p_user_id);
    RETURN current_usage < p_daily_limit;
END;
$$ LANGUAGE plpgsql;

-- Create view for quota monitoring
CREATE OR REPLACE VIEW bot_quota_status AS
SELECT 
    u.id as user_id,
    u.fullname,
    u.email,
    COALESCE(dqs.translation_count, 0) as translations_used_today,
    (20 - COALESCE(dqs.translation_count, 0)) as translations_remaining_today,
    CASE 
        WHEN COALESCE(dqs.translation_count, 0) >= 20 THEN 'QUOTA_EXCEEDED'
        WHEN COALESCE(dqs.translation_count, 0) >= 15 THEN 'QUOTA_WARNING'
        ELSE 'QUOTA_OK'
    END as quota_status,
    dqs.character_count as characters_translated_today,
    dqs.last_updated as last_translation_time
FROM users u
LEFT JOIN bot_daily_quota_summary dqs ON u.id = dqs.user_id 
    AND dqs.date = CURRENT_DATE
WHERE u.is_bot = FALSE  -- Only regular users have translation quotas
ORDER BY u.id;

-- Function to cleanup old quota logs (retention policy)
CREATE OR REPLACE FUNCTION cleanup_old_translation_logs(
    p_retention_days INTEGER DEFAULT 90
) RETURNS INTEGER AS $$
DECLARE
    deleted_count INTEGER;
    cutoff_date TIMESTAMPTZ;
BEGIN
    cutoff_date := NOW() - INTERVAL '1 day' * p_retention_days;
    
    DELETE FROM bot_translation_logs 
    WHERE created_at < cutoff_date;
    
    GET DIAGNOSTICS deleted_count = ROW_COUNT;
    
    -- Also cleanup old daily summaries
    DELETE FROM bot_daily_quota_summary 
    WHERE date < DATE(cutoff_date);
    
    RETURN deleted_count;
END;
$$ LANGUAGE plpgsql;

-- Add helpful comments
COMMENT ON TABLE bot_translation_logs IS 'Logs of all translation requests made by users for quota tracking';
COMMENT ON TABLE bot_daily_quota_summary IS 'Daily aggregated translation quota usage per user for performance';
COMMENT ON FUNCTION get_user_daily_quota(BIGINT, DATE) IS 'Get translation quota usage for a user on a specific date';
COMMENT ON FUNCTION can_user_translate(BIGINT, INTEGER) IS 'Check if user has remaining translation quota';
COMMENT ON VIEW bot_quota_status IS 'Real-time view of user translation quota status';
COMMENT ON FUNCTION cleanup_old_translation_logs(INTEGER) IS 'Cleanup old translation logs based on retention policy';

-- Insert some example data to verify the system works
-- Note: This would be removed in production
INSERT INTO bot_translation_logs (user_id, message_id, source_language, target_language, character_count, translation_success)
SELECT 
    1, -- Assuming user ID 1 exists
    NULL, 
    'en', 
    'zh', 
    50, 
    true
WHERE EXISTS (SELECT 1 FROM users WHERE id = 1)
LIMIT 1; 