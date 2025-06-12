-- ClickHouse Analytics Database Initialization for Fechatter
-- This script is executed during ClickHouse container startup

-- Note: In Docker, CREATE DATABASE is handled automatically by CLICKHOUSE_DB environment variable
-- This script focuses on table and view creation

-- Create analytics events table
CREATE TABLE IF NOT EXISTS fechatter_analytics.analytics_events
(
    -- EventContext fields
    client_id String,
    session_id String,
    duration UInt32,
    app_version String,
    system_os String,
    system_arch String,
    system_locale String,
    system_timezone String,
    system_browser Nullable(String),
    system_browser_version Nullable(String),
    user_id Nullable(String),
    ip Nullable(String),
    user_agent Nullable(String),
    geo_country Nullable(String),
    geo_region Nullable(String),
    geo_city Nullable(String),
    client_ts Int64,
    server_ts Int64,
    
    -- Common fields
    event_type String,
    
    -- AppExitEvent fields
    exit_code Nullable(String),
    
    -- UserLoginEvent
    login_email Nullable(String),
    login_method Nullable(String),
    
    -- UserLogoutEvent
    logout_email Nullable(String),
    
    -- UserRegisterEvent
    register_email Nullable(String),
    register_workspace_id Nullable(String),
    register_method Nullable(String),
    
    -- ChatCreatedEvent
    chat_created_workspace_id Nullable(String),
    chat_created_type Nullable(String),
    chat_created_members_count Nullable(Int32),
    
    -- MessageSentEvent
    message_chat_id Nullable(String),
    message_type Nullable(String),
    message_size Nullable(Int32),
    message_total_files Nullable(Int32),
    message_has_mentions Nullable(UInt8),
    message_has_links Nullable(UInt8),
    
    -- ChatJoinedEvent
    chat_joined_id Nullable(String),
    chat_joined_method Nullable(String),
    
    -- ChatLeftEvent
    chat_left_id Nullable(String),
    chat_left_reason Nullable(String),
    
    -- NavigationEvent
    navigation_from Nullable(String),
    navigation_to Nullable(String),
    navigation_duration_ms Nullable(Int32),
    
    -- FileUploadedEvent
    file_uploaded_type Nullable(String),
    file_uploaded_size Nullable(Int64),
    file_uploaded_method Nullable(String),
    file_uploaded_duration_ms Nullable(Int32),
    
    -- FileDownloadedEvent
    file_downloaded_type Nullable(String),
    file_downloaded_size Nullable(Int64),
    file_downloaded_duration_ms Nullable(Int32),
    
    -- SearchPerformedEvent
    search_type Nullable(String),
    search_query_length Nullable(String),
    search_results_count Nullable(Int32),
    search_duration_ms Nullable(Int32),
    search_has_filters Nullable(UInt8),
    
    -- NotificationReceivedEvent
    notification_type Nullable(String),
    notification_source Nullable(String),
    notification_was_clicked Nullable(UInt8),
    
    -- ErrorOccurredEvent
    error_type Nullable(String),
    error_code Nullable(String),
    error_message Nullable(String),
    error_stack_trace Nullable(String),
    error_context Nullable(String),
    
    -- BotResponseEvent
    bot_response_bot_id Nullable(String),
    bot_response_chat_id Nullable(String),
    bot_response_type Nullable(String),
    bot_response_time_ms Nullable(UInt64),
    bot_response_tokens_used Nullable(UInt32),
    bot_response_success Nullable(UInt8),
    bot_response_error_message Nullable(String),
    
    -- Indexes
    INDEX idx_user_id user_id TYPE bloom_filter GRANULARITY 1,
    INDEX idx_session_id session_id TYPE bloom_filter GRANULARITY 1,
    INDEX idx_event_type event_type TYPE bloom_filter GRANULARITY 1
)
ENGINE = MergeTree()
PARTITION BY toYYYYMM(toDateTime(server_ts / 1000))
ORDER BY (event_type, server_ts, client_id)
TTL toDateTime(server_ts / 1000) + INTERVAL 90 DAY;

-- Create materialized views for common analytics (fixed nullable column issues)

-- Daily active users (using coalesce to handle nullable user_id)
CREATE MATERIALIZED VIEW IF NOT EXISTS fechatter_analytics.daily_active_users
ENGINE = SummingMergeTree()
PARTITION BY toYYYYMM(date)
ORDER BY (date, user_id_key)
AS
SELECT
    toDate(toDateTime(server_ts / 1000)) AS date,
    coalesce(user_id, '') AS user_id_key,
    user_id,
    count() AS events_count
FROM fechatter_analytics.analytics_events
WHERE user_id IS NOT NULL AND user_id != ''
GROUP BY date, user_id_key, user_id;

-- Event counts by type
CREATE MATERIALIZED VIEW IF NOT EXISTS fechatter_analytics.event_counts_by_type
ENGINE = SummingMergeTree()
PARTITION BY toYYYYMM(date)
ORDER BY (date, event_type)
AS
SELECT
    toDate(toDateTime(server_ts / 1000)) AS date,
    event_type,
    count() AS count
FROM fechatter_analytics.analytics_events
GROUP BY date, event_type;

-- Message statistics (fixed nullable column handling)
CREATE MATERIALIZED VIEW IF NOT EXISTS fechatter_analytics.message_statistics
ENGINE = SummingMergeTree()
PARTITION BY toYYYYMM(date)
ORDER BY (date, message_type_key)
AS
SELECT
    toDate(toDateTime(server_ts / 1000)) AS date,
    coalesce(message_type, 'unknown') AS message_type_key,
    message_type,
    count() AS message_count,
    sum(coalesce(message_size, 0)) AS total_size,
    sum(coalesce(message_total_files, 0)) AS total_files
FROM fechatter_analytics.analytics_events
WHERE event_type = 'message_sent'
GROUP BY date, message_type_key, message_type;

-- Error tracking (fixed nullable columns)
CREATE MATERIALIZED VIEW IF NOT EXISTS fechatter_analytics.error_tracking
ENGINE = MergeTree()
PARTITION BY toYYYYMM(date)
ORDER BY (date, error_type_key, error_code_key)
AS
SELECT
    toDateTime(server_ts / 1000) AS timestamp,
    toDate(timestamp) AS date,
    coalesce(error_type, 'unknown') AS error_type_key,
    coalesce(error_code, 'unknown') AS error_code_key,
    error_type,
    error_code,
    error_message,
    error_context,
    user_id,
    client_id
FROM fechatter_analytics.analytics_events
WHERE event_type = 'error_occurred';

-- Session analytics (fixed nullable user_id)
CREATE MATERIALIZED VIEW IF NOT EXISTS fechatter_analytics.session_analytics
ENGINE = AggregatingMergeTree()
PARTITION BY toYYYYMM(date)
ORDER BY (date, session_id, client_id)
AS
SELECT
    toDate(toDateTime(server_ts / 1000)) AS date,
    session_id,
    client_id,
    user_id,
    min(server_ts) AS session_start,
    max(server_ts) AS session_end,
    max(duration) AS session_duration,
    count() AS events_count,
    groupArray(event_type) AS event_types
FROM fechatter_analytics.analytics_events
GROUP BY date, session_id, client_id, user_id;

-- Create aggregated sessions table (fixed structure)
CREATE TABLE IF NOT EXISTS fechatter_analytics.sessions(
    date Date,
    client_id String,
    session_id String,
    app_version String,
    system_os String,
    system_arch String,
    system_locale String,
    system_timezone String,
    user_id_key String,  -- Non-nullable key for sorting
    user_id Nullable(String),  -- Original nullable field
    ip Nullable(String),
    user_agent Nullable(String),
    geo_country Nullable(String),
    geo_region Nullable(String),
    geo_city Nullable(String),
    session_start SimpleAggregateFunction(min, DateTime64(3)),
    session_end SimpleAggregateFunction(max, DateTime64(3)),
    session_length SimpleAggregateFunction(sum, UInt64),
    total_events UInt32
) ENGINE = SummingMergeTree()
ORDER BY (date, client_id, session_id, user_id_key);

-- Create materialized view to aggregate sessions (fixed nullable handling)
CREATE MATERIALIZED VIEW IF NOT EXISTS fechatter_analytics.sessions_mv TO fechatter_analytics.sessions AS
SELECT
    toDate(server_ts) AS date,
    client_id,
    session_id,
    any(app_version) AS app_version,
    any(system_os) AS system_os,
    any(system_arch) AS system_arch,
    any(system_locale) AS system_locale,
    any(system_timezone) AS system_timezone,
    coalesce(any(user_id), '') AS user_id_key,
    any(user_id) AS user_id,
    any(ip) AS ip,
    any(user_agent) AS user_agent,
    any(geo_country) AS geo_country,
    any(geo_region) AS geo_region,
    any(geo_city) AS geo_city,
    min(server_ts) AS session_start,
    max(server_ts) AS session_end,
    sum(duration) / 1000 AS session_length,
    count(1) AS total_events
FROM fechatter_analytics.analytics_events
GROUP BY date, client_id, session_id, user_id;

-- Create daily sessions summary table
CREATE TABLE IF NOT EXISTS fechatter_analytics.daily_sessions(
    date Date,
    client_id String,
    total_session_length SimpleAggregateFunction(sum, UInt64),
    total_session_events SimpleAggregateFunction(sum, UInt64),
    unique_users AggregateFunction(uniq, String)  -- Non-nullable for aggregation
) ENGINE = SummingMergeTree()
ORDER BY (date, client_id);

CREATE MATERIALIZED VIEW IF NOT EXISTS fechatter_analytics.daily_sessions_mv TO fechatter_analytics.daily_sessions AS
SELECT
    date,
    client_id,
    sum(session_length) AS total_session_length,
    sum(total_events) AS total_session_events,
    uniqState(user_id_key) AS unique_users
FROM fechatter_analytics.sessions
WHERE user_id_key != ''
GROUP BY date, client_id;

-- Insert sample data for testing (optional - remove in production)
INSERT INTO fechatter_analytics.analytics_events(client_id, session_id, app_version, system_os, system_arch, system_locale, system_timezone, client_ts, server_ts, event_type)
    VALUES ('client_001', 'session_001', '1.0.0', 'macOS', 'x86_64', 'en-US', 'America/New_York', toUnixTimestamp(now()) * 1000, toUnixTimestamp(now()) * 1000, 'AppStart');

INSERT INTO fechatter_analytics.analytics_events(client_id, session_id, app_version, system_os, system_arch, system_locale, system_timezone, user_id, ip, client_ts, server_ts, event_type, login_email)
    VALUES ('client_002', 'session_002', '1.0.1', 'Windows', 'x86_64', 'en-GB', 'Europe/London', 'user_123', '192.168.1.1', toUnixTimestamp(now()) * 1000, toUnixTimestamp(now()) * 1000, 'UserLogin', 'user@example.com');

