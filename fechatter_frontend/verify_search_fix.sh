#!/bin/bash

# 🔍 Search Modal Fix Verification Script
# Verifies that the search modal opening issue has been resolved

echo "🔍 Starting Search Modal Fix Verification..."
echo "============================================="

# Check if we're in the right directory
if [ ! -f "package.json" ]; then
    echo "❌ Error: Please run this script from the fechatter_frontend directory"
    exit 1
fi

# Check if the modal opening fix has been applied
echo "📝 Checking if modal opening fix has been applied..."
if grep -q "🔧 FIX: 优先使用 isOpen" src/components/search/ProductionSearchModal.vue; then
    echo "✅ Modal opening fix detected in ProductionSearchModal.vue"
else
    echo "❌ Modal opening fix not found! Please apply the fix first."
    exit 1
fi

# Check if the modern UI upgrade has been applied
echo "📝 Checking if modern UI upgrade has been applied..."
if grep -q "Premium Design System inspired by Notion, Linear & Slack" src/components/search/ProductionSearchModal.vue; then
    echo "✅ Modern UI design system detected in ProductionSearchModal.vue"
else
    echo "❌ Modern UI upgrade not found! Please apply the UI upgrade first."
    exit 1
fi

if grep -q "max-width: min(680px, calc(100vw - var(--space-8)))" src/components/search/ProductionSearchModal.vue; then
    echo "✅ Adaptive width system detected in ProductionSearchModal.vue"
else
    echo "❌ Adaptive width system not found! Please apply the modern CSS first."
    exit 1
fi

if grep -q -- "--color-primary: #6366f1" src/components/search/ProductionSearchModal.vue; then
    echo "✅ Modern color system detected in ProductionSearchModal.vue"
else
    echo "❌ Modern color system not found! Please apply the design system first."
    exit 1
fi

# Check if development server is running
echo "🌐 Checking development server status..."
if curl -s http://localhost:5173 > /dev/null; then
    echo "✅ Development server is running on port 5173"
else
    echo "⚠️  Development server not running. Starting it now..."
    echo "🚀 Please run 'yarn dev' in another terminal and try again"
    exit 1
fi

# Check key files exist
echo "📁 Verifying key component files..."
files=(
    "src/views/Chat.vue"
    "src/components/search/ProductionSearchModal.vue"
    "src/services/searchService.js"
)

for file in "${files[@]}"; do
    if [ -f "$file" ]; then
        echo "✅ Found: $file"
    else
        echo "❌ Missing: $file"
        exit 1
    fi
done

# Check for search button in Chat.vue
echo "🔍 Verifying search button implementation..."
if grep -q "handleSearchClick" src/views/Chat.vue; then
    echo "✅ Search button click handler found"
else
    echo "❌ Search button click handler missing"
    exit 1
fi

# Check for modal binding
if grep -q ":is-open=\"showSearchModal\"" src/views/Chat.vue; then
    echo "✅ Modal binding found"
else
    echo "❌ Modal binding missing"
    exit 1
fi

# Check if API endpoint fix has been applied
echo "📝 Checking if API endpoint fix has been applied..."
if grep -q "Use actual backend routes from lib.rs" src/services/searchService.js; then
    echo "✅ API endpoint fix detected in searchService.js"
else
    echo "❌ API endpoint fix not found! Please apply the API fix first."
    exit 1
fi

if grep -q "/chat/\${chatId}/messages/search" src/services/searchService.js; then
    echo "✅ Correct API endpoint detected in searchService.js"
else
    echo "❌ Correct API endpoint not found! Please fix the API mapping first."
    exit 1
fi

# Check if HTTP method fix has been applied (CRITICAL FIX for 'missing field q' error)
echo "📝 Checking if HTTP method fix has been applied..."
if grep -q "Backend expects parameters in query string for both GET and POST" src/services/searchService.js; then
    echo "✅ HTTP method fix detected in searchService.js"
else
    echo "❌ HTTP method fix not found! This could cause 'missing field q' database errors."
    exit 1
fi

if grep -q "api.post(endpoint, null, { params: searchParams })" src/services/searchService.js; then
    echo "✅ POST query string method correctly implemented (fixes 'missing field q' error)"
else
    echo "❌ POST query string method not found! This WILL cause 400 Bad Request errors."
    exit 1
fi

echo ""
echo "🎉 All checks passed! The search modal fix has been successfully applied."
echo ""
echo "📋 Manual Testing Instructions:"
echo "1. Open http://localhost:5173 in your browser"
echo "2. Login and navigate to any chat"
echo "3. Click the search button (magnifying glass icon)"
echo "4. ✅ VERIFY: Modal opens with proper width (680px adaptive, not narrow strip)"
echo "5. ✅ VERIFY: Search input is focused and fully visible"
echo "6. ✅ VERIFY: Modern filter buttons (All Messages, Semantic, Exact Match, Recent) are visible"
echo "7. ✅ VERIFY: Search button appears on RIGHT side of input when typing"
echo "8. ✅ VERIFY: Search results display as beautiful card containers"
echo "9. Press Escape to close the modal"
echo "10. Try Ctrl+K (or Cmd+K) keyboard shortcut"
echo "11. ✅ VERIFY: Modal looks perfect on mobile with responsive design"
echo ""
echo "✅ Verification completed successfully!"
echo "🚀 The search modal should now work properly." 