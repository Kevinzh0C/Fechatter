#!/bin/bash

echo "🧪 Testing NATS message logging across all services..."
echo "======================================================="

# Test analytics events (for analytics_server)
echo ""
echo "📊 Testing Analytics Events..."
nats pub fechatter.analytics.user.login '{
  "user_id": 123,
  "timestamp": "2024-01-01T12:00:00Z",
  "session_id": "test_session_001",
  "ip_address": "127.0.0.1",
  "user_agent": "test-agent/1.0"
}'

nats pub fechatter.analytics.message.sent '{
  "user_id": 123,
  "chat_id": 1,
  "message_type": "text",
  "size": 256,
  "timestamp": "2024-01-01T12:01:00Z"
}'

# Test notify events (for notify_server)
echo ""
echo "🔔 Testing Notify Events..."
nats pub fechatter.messages.created '{
  "event_type": "new_message",
  "chat_id": 1,
  "sender_id": 123,
  "content": "Hello from NATS test!",
  "timestamp": "2024-01-01T12:02:00Z"
}'

nats pub fechatter.realtime.chat.1 '{
  "event_type": "typing_started",
  "user_id": 123,
  "chat_id": 1,
  "user_name": "Test User",
  "timestamp": "2024-01-01T12:03:00Z"
}'

nats pub fechatter.chat.member.added '{
  "event_type": "member_added",
  "chat_id": 1,
  "user_id": 456,
  "added_by": 123,
  "timestamp": "2024-01-01T12:04:00Z"
}'

# Test bot events (for bot_server)
echo ""
echo "🤖 Testing Bot Events..."
nats pub fechatter.messages.created '{
  "msg": {
    "id": 999,
    "chat_id": 1,
    "sender_id": 123,
    "content": "Hey bot, can you help me?",
    "created_at": "2024-01-01T12:05:00Z"
  },
  "members": [123, 2]
}'

nats pub fechatter.chats.member.joined '{
  "chat_id": 1,
  "user_id": 2,
  "bot_id": 2,
  "timestamp": "2024-01-01T12:06:00Z"
}'

echo ""
echo "✅ All test events published!"
echo ""
echo "📋 Expected Log Output:"
echo "- analytics_server: Should show 'Received NATS event' with protobuf parsing"
echo "- notify_server: Should show 'Received NATS event' with JSON parsing and routing"
echo "- bot_server: Should show 'Received NATS event' with message processing"
echo ""
echo "🔍 Check the service logs for detailed NATS message processing information." 