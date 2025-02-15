#!/bin/bash
# collect-x86-binaries.sh - Collect and organize x86_64 binaries for packaging
# This script moves compiled x86_64 binaries to packaging directories

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
SOURCE_DIR="target/main/x86_64-unknown-linux-musl/release"
PACKAGE_DIR="docker/binaries/x86_64"
BUILD_LOG="build-x86.log"

# Function to print colored output
print_status() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

print_header() {
    echo -e "${BLUE}=== $1 ===${NC}"
}

# Function to verify binary architecture
verify_binary_arch() {
    local binary_path="$1"
    local binary_name="$2"
    
    if [[ "$OSTYPE" == "darwin"* ]]; then
        # On macOS, use file command
        local arch_info=$(file "$binary_path")
        if echo "$arch_info" | grep -q "x86-64"; then
            print_status "✅ $binary_name: x86_64 architecture verified"
            return 0
        else
            print_error "❌ $binary_name: Wrong architecture detected"
            echo "  File info: $arch_info"
            return 1
        fi
    else
        # On Linux, use readelf
        if command -v readelf &> /dev/null; then
            local arch_info=$(readelf -h "$binary_path" | grep Machine)
            if echo "$arch_info" | grep -q "X86-64"; then
                print_status "✅ $binary_name: x86_64 architecture verified"
                return 0
            else
                print_error "❌ $binary_name: Wrong architecture detected"
                echo "  Architecture: $arch_info"
                return 1
            fi
        else
            print_warning "⚠️  Cannot verify architecture (readelf not available)"
            return 0
        fi
    fi
}

# Function to get binary size in human readable format
get_binary_size() {
    local binary_path="$1"
    if [[ "$OSTYPE" == "darwin"* ]]; then
        stat -f%z "$binary_path" | numfmt --to=iec-i --suffix=B
    else
        stat --printf="%s" "$binary_path" | numfmt --to=iec-i --suffix=B
    fi
}

# Binary definitions
BINARY_NAMES=("fechatter_server" "analytics_server" "notify_server" "fechatter_gateway" "bot" "indexer")
PACKAGE_NAMES=("fechatter_server" "analytics_server" "notify_server" "fechatter_gateway" "bot_server" "bot_server")

# Function to get package name for binary
get_package_name() {
    local binary_name="$1"
    case "$binary_name" in
        "fechatter_server") echo "fechatter_server" ;;
        "analytics_server") echo "analytics_server" ;;
        "notify_server") echo "notify_server" ;;
        "fechatter_gateway") echo "fechatter_gateway" ;;
        "bot") echo "bot_server" ;;
        "indexer") echo "bot_server" ;;
        *) echo "unknown" ;;
    esac
}

# Main collection function
collect_binaries() {
    print_header "Collecting x86_64 Binaries for Packaging"
    
    # Create package directory
    mkdir -p "$PACKAGE_DIR"
    
    # Initialize build log
    {
        echo "# Fechatter x86_64 Build Report"
        echo "Date: $(date)"
        echo "Target: x86_64-unknown-linux-musl"
        echo ""
    } > "$BUILD_LOG"
    
    local total_binaries=0
    local successful_copies=0
    local total_size=0
    
    # Process each binary
    for binary_name in "${BINARY_NAMES[@]}"; do
        local source_path="$SOURCE_DIR/$binary_name"
        local package_name=$(get_package_name "$binary_name")
        local dest_dir="$PACKAGE_DIR/$package_name"
        local dest_path="$dest_dir/$binary_name"
        
        total_binaries=$((total_binaries + 1))
        
        print_status "Processing binary: $binary_name"
        
        # Check if source binary exists
        if [[ ! -f "$source_path" ]]; then
            print_error "Binary not found: $source_path"
            echo "❌ $binary_name: Binary not found" >> "$BUILD_LOG"
            continue
        fi
        
        # Verify architecture
        if ! verify_binary_arch "$source_path" "$binary_name"; then
            echo "❌ $binary_name: Architecture verification failed" >> "$BUILD_LOG"
            continue
        fi
        
        # Create destination directory
        mkdir -p "$dest_dir"
        
        # Copy binary
        if cp "$source_path" "$dest_path"; then
            # Make executable
            chmod +x "$dest_path"
            
            # Get size information
            local size=$(get_binary_size "$dest_path")
            total_size=$((total_size + $(stat -c%s "$dest_path" 2>/dev/null || stat -f%z "$dest_path")))
            
            print_status "✅ $binary_name -> $dest_path ($size)"
            echo "✅ $binary_name: $size" >> "$BUILD_LOG"
            successful_copies=$((successful_copies + 1))
        else
            print_error "Failed to copy $binary_name"
            echo "❌ $binary_name: Copy failed" >> "$BUILD_LOG"
        fi
    done
    
    # Generate manifest
    generate_manifest
    
    # Summary
    print_header "Collection Summary"
    print_status "Total binaries processed: $total_binaries"
    print_status "Successfully copied: $successful_copies"
    print_status "Total size: $(echo $total_size | numfmt --to=iec-i --suffix=B)"
    
    if [[ $successful_copies -eq $total_binaries ]]; then
        print_status "🎉 All binaries collected successfully!"
        return 0
    else
        print_error "⚠️  Some binaries failed to collect"
        return 1
    fi
}

# Function to generate binary manifest
generate_manifest() {
    local manifest_file="$PACKAGE_DIR/MANIFEST.txt"
    
    {
        echo "# Fechatter x86_64 Binary Manifest"
        echo "# Generated: $(date)"
        echo "# Target: x86_64-unknown-linux-musl"
        echo ""
        
        for binary_name in "${BINARY_NAMES[@]}"; do
            local package_name=$(get_package_name "$binary_name")
            local binary_path="$PACKAGE_DIR/$package_name/$binary_name"
            
            if [[ -f "$binary_path" ]]; then
                local size=$(get_binary_size "$binary_path")
                local checksum=$(shasum -a 256 "$binary_path" | cut -d' ' -f1)
                echo "$package_name/$binary_name $size $checksum"
            fi
        done
    } > "$manifest_file"
    
    print_status "Manifest generated: $manifest_file"
}

# Function to show collected binaries
show_binaries() {
    print_header "Collected Binaries Structure"
    
    if [[ -d "$PACKAGE_DIR" ]]; then
        tree "$PACKAGE_DIR" 2>/dev/null || find "$PACKAGE_DIR" -type f
    else
        print_warning "No binaries collected yet. Run collection first."
    fi
}

# Function to clean previous builds
clean_binaries() {
    print_header "Cleaning Previous Builds"
    
    if [[ -d "$PACKAGE_DIR" ]]; then
        rm -rf "$PACKAGE_DIR"
        print_status "Removed: $PACKAGE_DIR"
    fi
    
    if [[ -f "$BUILD_LOG" ]]; then
        rm -f "$BUILD_LOG"
        print_status "Removed: $BUILD_LOG"
    fi
    
    print_status "Clean completed"
}

# Function to copy to Docker context
copy_to_docker() {
    print_header "Copying Binaries to Docker Context"
    
    local docker_context="docker/context/binaries"
    mkdir -p "$docker_context"
    
    if [[ -d "$PACKAGE_DIR" ]]; then
        cp -r "$PACKAGE_DIR"/* "$docker_context/"
        print_status "Binaries copied to Docker context: $docker_context"
    else
        print_error "No binaries found. Run collection first."
        return 1
    fi
}

# Main execution
main() {
    case "${1:-collect}" in
        "collect")
            collect_binaries
            ;;
        "show")
            show_binaries
            ;;
        "clean")
            clean_binaries
            ;;
        "docker")
            copy_to_docker
            ;;
        "all")
            clean_binaries
            collect_binaries
            copy_to_docker
            ;;
        *)
            echo "Usage: $0 {collect|show|clean|docker|all}"
            echo ""
            echo "Commands:"
            echo "  collect  - Collect x86_64 binaries from build output"
            echo "  show     - Show collected binaries structure"
            echo "  clean    - Clean previous collections"
            echo "  docker   - Copy binaries to Docker context"
            echo "  all      - Clean, collect, and copy to Docker context"
            exit 1
            ;;
    esac
}

# Check if script is being sourced or executed
if [[ "${BASH_SOURCE[0]}" == "${0}" ]]; then
    main "$@"
fi 