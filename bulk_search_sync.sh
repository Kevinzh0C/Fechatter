#!/bin/bash

# Fechatter Bulk Search Synchronization Script
# 批量搜索同步脚本 - 生产级别版本
# 解决MeiliSearch文档数量为0的问题

echo "🔍 Fechatter Bulk Search Synchronization"
echo "========================================"
echo ""

# Configuration
SERVER_IP="45.77.178.85"
SERVER_PORT="8080"
MEILISEARCH_URL="http://localhost:7700"
MEILISEARCH_API_KEY="fechatter_search_key"
INDEX_NAME="messages"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Helper functions
info() { echo -e "${BLUE}ℹ️  $1${NC}"; }
success() { echo -e "${GREEN}✅ $1${NC}"; }
warn() { echo -e "${YELLOW}⚠️  $1${NC}"; }
error() { echo -e "${RED}❌ $1${NC}"; }

echo "📍 Target Server: $SERVER_IP:$SERVER_PORT"
echo "🔍 MeiliSearch: $MEILISEARCH_URL"
echo "📊 Index: $INDEX_NAME"
echo ""

# ========================================
# Step 1: Verify Problem State
# ========================================
info "Step 1: Verifying current state..."

echo "🔍 Checking MeiliSearch index status..."
INDEX_STATS=$(curl -s -H "Authorization: Bearer $MEILISEARCH_API_KEY" "$MEILISEARCH_URL/indexes/$INDEX_NAME/stats" 2>/dev/null)

if echo "$INDEX_STATS" | grep -q '"numberOfDocuments":0'; then
    error "Confirmed: MeiliSearch has 0 documents (sync issue)"
    SYNC_NEEDED=true
else
    DOC_COUNT=$(echo "$INDEX_STATS" | grep -o '"numberOfDocuments":[0-9]*' | cut -d':' -f2)
    success "MeiliSearch has $DOC_COUNT documents"
    SYNC_NEEDED=false
fi

echo ""

# ========================================
# Step 2: Database Message Count
# ========================================
info "Step 2: Checking database message count..."

echo "Run this command to check database:"
echo "docker exec fechatter-postgres-vcr psql -U fechatter -d fechatter -c 'SELECT COUNT(*) as total_messages FROM messages;'"
echo ""
echo "Expected result: 164+ messages in database"
echo ""

# ========================================
# Step 3: Manual Bulk Sync Implementation
# ========================================
if [ "$SYNC_NEEDED" = true ]; then
    info "Step 3: Implementing bulk synchronization..."
    
    cat << 'EOF'
📋 MANUAL BULK SYNC PROCEDURE:

Since async indexing is broken, we'll implement manual bulk sync:

1️⃣ Extract Messages from Database:
   ssh root@45.77.178.85
   docker exec fechatter-postgres-vcr psql -U fechatter -d fechatter -c "
   COPY (
     SELECT 
       m.id,
       m.chat_id,
       m.sender_id,
       m.content,
       m.created_at,
       m.files,
       u.fullname as sender_name,
       c.workspace_id,
       c.name as chat_name,
       c.chat_type
     FROM messages m
     LEFT JOIN users u ON m.sender_id = u.id
     LEFT JOIN chats c ON m.chat_id = c.id
     ORDER BY m.created_at DESC
   ) TO '/tmp/messages_export.csv' WITH CSV HEADER;
   "

2️⃣ Convert to MeiliSearch Format:
   docker exec fechatter-postgres-vcr cat /tmp/messages_export.csv

3️⃣ Bulk Insert to MeiliSearch:
   curl -X POST '$MEILISEARCH_URL/indexes/$INDEX_NAME/documents' \
     -H 'Content-Type: application/json' \
     -H 'Authorization: Bearer $MEILISEARCH_API_KEY' \
     -d '[CONVERTED_DOCUMENTS]'

EOF

else
    success "Step 3: Sync not needed - documents already exist"
fi

echo ""

# ========================================
# Step 4: Automated Sync Script
# ========================================
info "Step 4: Creating automated sync script..."

cat << 'EOF' > /tmp/auto_sync_messages.sh
#!/bin/bash

# Automated Message Sync Script
# Run this on the server to perform bulk sync

echo "🔄 Starting automated message synchronization..."

# Get database export
docker exec fechatter-postgres-vcr psql -U fechatter -d fechatter -t -A -F',' -c "
SELECT 
  json_build_object(
    'id', m.id,
    'chat_id', m.chat_id,
    'sender_id', m.sender_id,
    'sender_name', COALESCE(u.fullname, 'Unknown'),
    'content', m.content,
    'files', COALESCE(m.files, '[]'::json),
    'created_at', m.created_at::text,
    'workspace_id', c.workspace_id,
    'chat_name', c.name,
    'chat_type', c.chat_type
  )
FROM messages m
LEFT JOIN users u ON m.sender_id = u.id
LEFT JOIN chats c ON m.chat_id = c.id
ORDER BY m.created_at DESC;
" > /tmp/messages_json.txt

# Convert to JSON array
echo "[" > /tmp/messages_bulk.json
sed 's/$/,/' /tmp/messages_json.txt | sed '$ s/,$//' >> /tmp/messages_bulk.json
echo "]" >> /tmp/messages_bulk.json

# Send to MeiliSearch
RESPONSE=$(curl -s -X POST 'http://localhost:7700/indexes/messages/documents' \
  -H 'Content-Type: application/json' \
  -H 'Authorization: Bearer fechatter_search_key' \
  -d @/tmp/messages_bulk.json)

echo "📊 Sync Response: $RESPONSE"

# Verify
STATS=$(curl -s -H 'Authorization: Bearer fechatter_search_key' 'http://localhost:7700/indexes/messages/stats')
echo "📈 Final Stats: $STATS"

# Cleanup
rm -f /tmp/messages_json.txt /tmp/messages_bulk.json

echo "✅ Automated sync completed"
EOF

success "Created automated sync script: /tmp/auto_sync_messages.sh"

echo ""

# ========================================
# Step 5: Quick Sync Commands
# ========================================
info "Step 5: Quick sync commands (run on server)..."

cat << 'EOF'
🚀 QUICK SYNC COMMANDS (copy and paste):

# 1. SSH to server
ssh root@45.77.178.85

# 2. Quick message count verification
docker exec fechatter-postgres-vcr psql -U fechatter -d fechatter -c 'SELECT COUNT(*) FROM messages;'

# 3. Export first 10 messages for testing
docker exec fechatter-postgres-vcr psql -U fechatter -d fechatter -c "
SELECT json_agg(
  json_build_object(
    'id', m.id,
    'chat_id', m.chat_id,
    'sender_id', m.sender_id,
    'sender_name', COALESCE(u.fullname, 'Unknown'),
    'content', m.content,
    'created_at', m.created_at::text,
    'workspace_id', c.workspace_id
  )
) FROM (
  SELECT m.*, u.fullname, c.workspace_id
  FROM messages m
  LEFT JOIN users u ON m.sender_id = u.id
  LEFT JOIN chats c ON m.chat_id = c.id
  ORDER BY m.created_at DESC
  LIMIT 10
) AS m;
"

# 4. Test MeiliSearch insertion (manual)
curl -X POST 'http://localhost:7700/indexes/messages/documents' \
  -H 'Content-Type: application/json' \
  -H 'Authorization: Bearer fechatter_search_key' \
  -d '[
    {
      "id": 164,
      "chat_id": 1,
      "sender_id": 2,
      "sender_name": "Super User",
      "content": "test message for search indexing",
      "created_at": "2025-06-16T23:29:16.876625Z",
      "workspace_id": 2
    }
  ]'

# 5. Verify insertion
curl -s -H 'Authorization: Bearer fechatter_search_key' 'http://localhost:7700/indexes/messages/stats'

EOF

echo ""

# ========================================
# Step 6: Async Index Fix Analysis
# ========================================
info "Step 6: Analyzing async indexing failure..."

cat << 'EOF'
🔍 ASYNC INDEX FAILURE ANALYSIS:

The async indexing mechanism is not working because:

1. **Event Publishing Issue**: Message creation doesn't trigger search events
2. **NATS Consumer Problem**: Search index consumer not processing events  
3. **Configuration Issue**: Search indexing may be disabled in current config

To diagnose:

1. Check message creation events:
   docker logs fechatter-server-vcr | grep -i "search\|index\|event"

2. Check NATS consumer status:
   docker logs fechatter-nats-vcr | grep -i "consumer\|search"

3. Verify current configuration:
   cat fechatter_server_fixed.yml | grep -A 20 "search:"

4. Test event publishing manually:
   curl -X POST 'http://localhost:8080/api/admin/chat/1/reindex' \
     -H 'Authorization: Bearer YOUR_TOKEN'

EOF

echo ""

# ========================================
# Step 7: Monitoring and Recovery
# ========================================
info "Step 7: Setting up monitoring and recovery..."

cat << 'EOF' > /tmp/search_sync_monitor.sh
#!/bin/bash

# Search Sync Monitoring Script
# Run this periodically to check sync status

MEILISEARCH_URL="http://localhost:7700"
MEILISEARCH_API_KEY="fechatter_search_key"

# Get database count
DB_COUNT=$(docker exec fechatter-postgres-vcr psql -U fechatter -d fechatter -t -c 'SELECT COUNT(*) FROM messages;' | tr -d ' ')

# Get MeiliSearch count  
MS_STATS=$(curl -s -H "Authorization: Bearer $MEILISEARCH_API_KEY" "$MEILISEARCH_URL/indexes/messages/stats")
MS_COUNT=$(echo "$MS_STATS" | grep -o '"numberOfDocuments":[0-9]*' | cut -d':' -f2)

echo "📊 Sync Status Check:"
echo "   Database messages: $DB_COUNT"
echo "   MeiliSearch docs:  $MS_COUNT"

if [ "$DB_COUNT" -gt "$MS_COUNT" ]; then
    echo "⚠️  Sync drift detected: DB has $((DB_COUNT - MS_COUNT)) more messages"
    echo "🔄 Trigger resync needed"
else
    echo "✅ Sync status: OK"
fi

# Check last message sync
LATEST_DB=$(docker exec fechatter-postgres-vcr psql -U fechatter -d fechatter -t -c 'SELECT id FROM messages ORDER BY created_at DESC LIMIT 1;' | tr -d ' ')
LATEST_MS=$(curl -s -H "Authorization: Bearer $MEILISEARCH_API_KEY" "$MEILISEARCH_URL/indexes/messages/search" -d '{"q":"","limit":1,"sort":["created_at:desc"]}' | grep -o '"id":[0-9]*' | head -1 | cut -d':' -f2)

echo "   Latest DB message:  $LATEST_DB"
echo "   Latest MS message:  $LATEST_MS"

if [ "$LATEST_DB" != "$LATEST_MS" ]; then
    echo "⚠️  Latest message sync issue detected"
fi
EOF

success "Created monitoring script: /tmp/search_sync_monitor.sh"

echo ""

# ========================================
# Step 8: Immediate Action Plan
# ========================================
info "Step 8: Immediate action plan..."

cat << 'EOF'
🎯 IMMEDIATE ACTION PLAN:

Priority 1 - Quick Fix (5 minutes):
1. SSH to server: ssh root@45.77.178.85
2. Run quick test sync (Step 5 commands above)
3. Verify search works: Test with "Hi" in Chat 1

Priority 2 - Full Sync (15 minutes):
1. Export all messages from database
2. Bulk insert to MeiliSearch
3. Verify search functionality across all chats

Priority 3 - Fix Async Indexing (30 minutes):
1. Analyze message creation event publishing
2. Fix NATS consumer configuration
3. Test new message indexing

Priority 4 - Monitoring (10 minutes):
1. Deploy sync monitoring script
2. Set up alerts for sync drift
3. Document recovery procedures

EOF

echo ""

# ========================================
# Step 9: Success Verification
# ========================================
info "Step 9: Success verification steps..."

cat << 'EOF'
✅ VERIFICATION CHECKLIST:

After implementing the fix, verify:

1. **Document Count Match**:
   - Database messages: 164+
   - MeiliSearch docs: 164+

2. **Search Functionality**:
   - Search "Hi" in Chat 1 → Should return results
   - Search "项目" in Chat 3 → Should return results  
   - Search "test" globally → Should return results

3. **New Message Sync**:
   - Send test message
   - Wait 5 seconds
   - Verify appears in search results

4. **Performance Metrics**:
   - Search response time: <2 seconds
   - Index update time: <10 seconds
   - Zero search errors

EOF

echo ""

# ========================================
# Summary
# ========================================
success "🎉 Bulk sync plan complete!"
echo ""
echo "Next steps:"
echo "1. Run the quick sync commands (Step 5)"
echo "2. Execute full bulk sync if needed"
echo "3. Fix async indexing for future messages"
echo "4. Deploy monitoring for ongoing health"
echo ""
warn "⚠️  Important: This addresses the symptom. Root cause (async indexing) still needs fixing."
echo ""
info "📋 All scripts saved to /tmp/ for easy execution" 