#!/bin/bash

# Fechatter Deprecated HTML Cleanup Script
echo "🧹 Fechatter HTML Cleanup - Removing deprecated development artifacts"
echo "======================================================================="

# Check for dry-run flag
DRY_RUN=false
if [ "$1" == "--dry-run" ] || [ "$1" == "-n" ]; then
    DRY_RUN=true
    echo "🔍 DRY RUN MODE - No files will be deleted"
    echo ""
fi

# Counter for deleted files
deleted_count=0

# Essential files to preserve - be more specific about what to keep
PRESERVE_PATTERNS="^[^/]*index\.html$|favicon\.html$|^[^/]*/index\.html$"

# Function to delete files matching pattern
delete_pattern() {
    local pattern="$1"
    local description="$2"
    
    echo -e "\n🗂️  Processing: $description"
    echo "   Pattern: $pattern"
    
    # Find files matching pattern but exclude preserved files
    local files=$(find . -name "*.html" -type f \
        -not -path "./target/*" \
        -not -path "./node_modules/*" \
        -not -path "./.venv/*" \
        -not -path "./fechatter_frontend/node_modules/*" \
        | grep -E "$pattern" \
        | grep -v -E "$PRESERVE_PATTERNS")
    
    if [ -n "$files" ]; then
        local count=$(echo "$files" | wc -l)
        echo "   Found: $count files"
        
        # Show first 5 files as preview
        echo "   Preview:"
        echo "$files" | head -5 | sed 's/^/     - /'
        if [ $count -gt 5 ]; then
            echo "     ... and $((count - 5)) more"
        fi
        
        # Delete files (or just show what would be deleted in dry-run)
        if [ "$DRY_RUN" = true ]; then
            echo "   🔍 Would delete: $count files (DRY RUN)"
        else
            echo "$files" | xargs rm -f
            echo "   ✅ Deleted: $count files"
        fi
        deleted_count=$((deleted_count + count))
    else
        echo "   ℹ️  No files found"
    fi
}

# Create backup directory
backup_dir="backups/html_cleanup_$(date +%Y%m%d_%H%M%S)"
mkdir -p "$backup_dir"
echo "📦 Backup directory created: $backup_dir"

# Backup critical files before cleanup
echo "📋 Creating backup of file list..."
find . -name "*.html" -type f \
    -not -path "./target/*" \
    -not -path "./node_modules/*" \
    -not -path "./.venv/*" \
    -not -path "./fechatter_frontend/node_modules/*" \
    > "$backup_dir/all_html_files_before_cleanup.txt"

# More specific patterns to avoid false positives
# Category 1: Test files (be specific)
delete_pattern ".*(test|Test).*\.html$" "Test Files"

# Category 2: Debug files  
delete_pattern ".*(debug|Debug).*\.html$" "Debug Files"

# Category 3: Fix files (be more specific)
delete_pattern ".*(fix|Fix)-.*\.html$|.*-fix\.html$" "Fix Files"

# Category 4: Verification files
delete_pattern ".*(verification|Verification).*\.html$" "Verification Files"

# Category 5: SSE diagnostic files (very specific)
delete_pattern "(^|/)sse-.*\.html$" "SSE Diagnostic Files"

# Category 6: Analysis files with specific patterns
delete_pattern ".*(analysis|Analysis|diagnostic|Diagnostic).*\.html$" "Analysis & Diagnostic Files"

# Category 7: DAG files
delete_pattern ".*(dag|DAG).*\.html$" "DAG Files"

# Category 8: Very specific development artifacts
delete_pattern "(bot-test|ngrok-|auth.*test|message.*test|file.*test|search.*test|complete.*test|quick.*test).*\.html$" "Specific Dev Artifacts"

# Summary
echo -e "\n�� Cleanup Complete!"
echo "======================================================================="
if [ "$DRY_RUN" = true ]; then
    echo "📊 Total files that would be deleted: $deleted_count"
    echo "🔍 This was a DRY RUN - no files were actually deleted"
    echo "💡 Run without --dry-run to perform actual deletion"
else
    echo "📊 Total files deleted: $deleted_count"
fi
echo "📦 Backup location: $backup_dir"

# Show remaining HTML files
remaining=$(find . -name "*.html" -type f \
    -not -path "./target/*" \
    -not -path "./node_modules/*" \
    -not -path "./.venv/*" \
    -not -path "./fechatter_frontend/node_modules/*" \
    | wc -l)

echo "📋 Remaining HTML files: $remaining"

echo -e "\n🔍 Remaining files preview:"
find . -name "*.html" -type f \
    -not -path "./target/*" \
    -not -path "./node_modules/*" \
    -not -path "./.venv/*" \
    -not -path "./fechatter_frontend/node_modules/*" \
    | head -10 | sed 's/^/   /'

if [ "$DRY_RUN" = true ]; then
    echo -e "\n💡 To actually perform the cleanup, run:"
    echo "   ./cleanup_deprecated_html.sh"
else
    echo -e "\n✅ HTML cleanup completed successfully!"
fi 