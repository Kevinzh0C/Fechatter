#!/bin/bash

# User Profile Migration Script
# This script applies the user profile extensions migration

set -euo pipefail

# Configuration
DATABASE_URL="${DATABASE_URL:-postgresql://postgres:password@localhost:5432/fechatter}"
MIGRATION_FILE="migrations/0020_user_profile_extensions.sql"

echo "🚀 Starting User Profile Migration..."
echo "Database URL: ${DATABASE_URL}"
echo "Migration file: ${MIGRATION_FILE}"

# Check if migration file exists
if [ ! -f "${MIGRATION_FILE}" ]; then
    echo "❌ Migration file not found: ${MIGRATION_FILE}"
    exit 1
fi

# Check if psql is available
if ! command -v psql &> /dev/null; then
    echo "❌ psql command not found. Please install PostgreSQL client."
    exit 1
fi

# Test database connection
echo "🔍 Testing database connection..."
if ! psql "${DATABASE_URL}" -c "SELECT 1;" &> /dev/null; then
    echo "❌ Failed to connect to database. Please check your DATABASE_URL."
    exit 1
fi

echo "✅ Database connection successful."

# Check current schema version (if migration tracking exists)
echo "🔍 Checking current database schema..."

# Create migrations tracking table if it doesn't exist
psql "${DATABASE_URL}" -c "
CREATE TABLE IF NOT EXISTS schema_migrations (
    version VARCHAR(255) PRIMARY KEY,
    applied_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
"

# Check if this migration has already been applied
MIGRATION_VERSION="0020_user_profile_extensions"
ALREADY_APPLIED=$(psql "${DATABASE_URL}" -t -c "
SELECT COUNT(*) FROM schema_migrations WHERE version = '${MIGRATION_VERSION}';
" | tr -d ' ')

if [ "${ALREADY_APPLIED}" -gt 0 ]; then
    echo "⚠️  Migration ${MIGRATION_VERSION} has already been applied."
    echo "Use --force flag to reapply (this will drop and recreate tables)."
    
    if [ "${1:-}" != "--force" ]; then
        exit 0
    fi
    
    echo "🔄 Force flag detected. Reapplying migration..."
    # Remove from tracking
    psql "${DATABASE_URL}" -c "DELETE FROM schema_migrations WHERE version = '${MIGRATION_VERSION}';"
fi

# Apply the migration
echo "🔧 Applying migration: ${MIGRATION_VERSION}..."

# Execute the migration in a transaction
psql "${DATABASE_URL}" -v ON_ERROR_STOP=1 << EOF
BEGIN;

-- Apply the migration
\i ${MIGRATION_FILE}

-- Record that migration was applied
INSERT INTO schema_migrations (version) VALUES ('${MIGRATION_VERSION}');

COMMIT;
EOF

if [ $? -eq 0 ]; then
    echo "✅ Migration applied successfully!"
    
    # Show some statistics
    echo ""
    echo "📊 Migration Statistics:"
    
    # Count users
    USER_COUNT=$(psql "${DATABASE_URL}" -t -c "SELECT COUNT(*) FROM users;" | tr -d ' ')
    echo "   - Users in database: ${USER_COUNT}"
    
    # Count user settings
    SETTINGS_COUNT=$(psql "${DATABASE_URL}" -t -c "SELECT COUNT(*) FROM user_settings;" | tr -d ' ')
    echo "   - User settings records: ${SETTINGS_COUNT}"
    
    # Show new columns
    echo ""
    echo "📋 New user profile columns added:"
    psql "${DATABASE_URL}" -c "
    SELECT column_name, data_type, is_nullable 
    FROM information_schema.columns 
    WHERE table_name = 'users' 
    AND column_name IN ('phone', 'title', 'department', 'avatar_url', 'bio', 'timezone', 'language', 'last_active_at')
    ORDER BY column_name;
    "
    
    echo ""
    echo "🎉 User profile migration completed successfully!"
    echo ""
    echo "Next steps:"
    echo "   1. Restart your application server"
    echo "   2. Test the new profile endpoints:"
    echo "      - GET /api/users/profile"
    echo "      - PUT /api/users/profile"
    echo "   3. Update your frontend to use the new profile fields"
    
else
    echo "❌ Migration failed!"
    exit 1
fi