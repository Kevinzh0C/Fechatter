#!/bin/bash

# Docker build script for Fechatter
# Coordinates local build and Docker packaging

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
IMAGE_NAME="${IMAGE_NAME:-fechatter}"
IMAGE_TAG="${IMAGE_TAG:-latest}"
REGISTRY="${REGISTRY:-}"
DOCKERFILE="${DOCKERFILE:-Dockerfile.local}"
BUILD_CONTEXT="${BUILD_CONTEXT:-.}"
PLATFORM="${PLATFORM:-linux/amd64}"

# Function to print colored output
print_status() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Function to check Docker availability
check_docker() {
    print_status "Checking Docker availability..."
    
    if ! command -v docker &> /dev/null; then
        print_error "Docker not found. Please install Docker first."
        exit 1
    fi
    
    if ! docker info &> /dev/null; then
        print_error "Docker daemon not running. Please start Docker."
        exit 1
    fi
    
    print_success "Docker is available"
}

# Function to run local build
run_local_build() {
    print_status "Running local build..."
    
    if [[ ! -f "scripts/build.sh" ]]; then
        print_error "Build script not found at scripts/build.sh"
        exit 1
    fi
    
    # Make build script executable
    chmod +x scripts/build.sh
    
    # Run build with clean option if requested
    if [[ "${CLEAN_BUILD:-false}" == "true" ]]; then
        print_status "Running clean build..."
        CLEAN_BUILD=true ./scripts/build.sh build
    else
        ./scripts/build.sh build
    fi
    
    # Verify binaries exist
    if [[ ! -d "target/build" ]] || [[ -z "$(ls -A target/build 2>/dev/null)" ]]; then
        print_error "No binaries found in target/build. Build may have failed."
        exit 1
    fi
    
    print_success "Local build completed"
}

# Function to build Docker image
build_docker_image() {
    print_status "Building Docker image..."
    
    local full_image_name="${IMAGE_NAME}:${IMAGE_TAG}"
    
    if [[ -n "$REGISTRY" ]]; then
        full_image_name="${REGISTRY}/${full_image_name}"
    fi
    
    print_status "Image name: $full_image_name"
    print_status "Platform: $PLATFORM"
    print_status "Dockerfile: $DOCKERFILE"
    
    # Build arguments
    local build_args=()
    
    if [[ -n "${BUILD_DATE:-}" ]]; then
        build_args+=(--build-arg "BUILD_DATE=$BUILD_DATE")
    else
        build_args+=(--build-arg "BUILD_DATE=$(date -u +'%Y-%m-%dT%H:%M:%SZ')")
    fi
    
    if [[ -n "${GIT_COMMIT:-}" ]]; then
        build_args+=(--build-arg "GIT_COMMIT=$GIT_COMMIT")
    else
        local git_commit=$(git rev-parse --short HEAD 2>/dev/null || echo "unknown")
        build_args+=(--build-arg "GIT_COMMIT=$git_commit")
    fi
    
    # Build the image
    if docker buildx build \
        --platform "$PLATFORM" \
        --file "$DOCKERFILE" \
        --tag "$full_image_name" \
        "${build_args[@]}" \
        --progress=plain \
        "$BUILD_CONTEXT"; then
        print_success "Docker image built: $full_image_name"
    else
        print_error "Docker build failed"
        exit 1
    fi
    
    # Store full image name for later use
    echo "$full_image_name" > .docker-image-name
}

# Function to test Docker image
test_docker_image() {
    print_status "Testing Docker image..."
    
    local full_image_name
    full_image_name=$(cat .docker-image-name 2>/dev/null || echo "${IMAGE_NAME}:${IMAGE_TAG}")
    
    # Basic image inspection
    print_status "Image inspection:"
    docker image inspect "$full_image_name" --format '{{.Config.ExposedPorts}}' | head -1
    
    # Check image size
    local image_size
    image_size=$(docker image inspect "$full_image_name" --format '{{.Size}}' | awk '{print int($1/1024/1024)" MB"}')
    print_status "Image size: $image_size"
    
    # Test container creation (without running)
    if docker create --name fechatter-test "$full_image_name" > /dev/null 2>&1; then
        docker rm fechatter-test > /dev/null 2>&1
        print_success "Image test passed"
    else
        print_error "Image test failed"
        exit 1
    fi
}

# Function to push Docker image
push_docker_image() {
    if [[ -z "$REGISTRY" ]]; then
        print_warning "No registry specified, skipping push"
        return 0
    fi
    
    print_status "Pushing Docker image to registry..."
    
    local full_image_name
    full_image_name=$(cat .docker-image-name 2>/dev/null || echo "${REGISTRY}/${IMAGE_NAME}:${IMAGE_TAG}")
    
    if docker push "$full_image_name"; then
        print_success "Image pushed: $full_image_name"
    else
        print_error "Failed to push image"
        exit 1
    fi
}

# Function to cleanup temporary files
cleanup() {
    print_status "Cleaning up..."
    rm -f .docker-image-name
}

# Function to show usage
show_usage() {
    cat << EOF
Usage: $0 [OPTIONS] [COMMAND]

Commands:
  build     - Build locally and create Docker image (default)
  push      - Build and push to registry
  test      - Test the built image
  clean     - Clean build artifacts

Options:
  --image-name NAME     Docker image name (default: fechatter)
  --image-tag TAG       Docker image tag (default: latest)
  --registry REGISTRY   Docker registry URL
  --platform PLATFORM   Target platform (default: linux/amd64)
  --dockerfile FILE     Dockerfile to use (default: Dockerfile.local)
  --clean-build         Clean build before starting
  --no-cache            Build Docker image without cache
  --help               Show this help message

Environment Variables:
  IMAGE_NAME           Docker image name
  IMAGE_TAG            Docker image tag
  REGISTRY             Docker registry URL
  PLATFORM             Target platform
  DOCKERFILE           Dockerfile to use
  CLEAN_BUILD          Set to 'true' for clean build
  BUILD_DATE           Build timestamp
  GIT_COMMIT           Git commit hash

Examples:
  $0 build                                    # Basic build
  $0 build --clean-build                     # Clean build
  $0 push --registry hub.docker.com/myuser   # Build and push
  $0 --image-tag v1.0.0 build                # Build with specific tag
EOF
}

# Function to parse command line arguments
parse_args() {
    while [[ $# -gt 0 ]]; do
        case $1 in
            --image-name)
                IMAGE_NAME="$2"
                shift 2
                ;;
            --image-tag)
                IMAGE_TAG="$2"
                shift 2
                ;;
            --registry)
                REGISTRY="$2"
                shift 2
                ;;
            --platform)
                PLATFORM="$2"
                shift 2
                ;;
            --dockerfile)
                DOCKERFILE="$2"
                shift 2
                ;;
            --clean-build)
                CLEAN_BUILD="true"
                shift
                ;;
            --no-cache)
                DOCKER_NO_CACHE="true"
                shift
                ;;
            --help)
                show_usage
                exit 0
                ;;
            build|push|test|clean)
                COMMAND="$1"
                shift
                ;;
            *)
                print_error "Unknown option: $1"
                show_usage
                exit 1
                ;;
        esac
    done
}

# Main function
main() {
    local command="${COMMAND:-build}"
    
    echo "Fechatter Docker Build Script"
    echo "============================"
    echo "Command: $command"
    echo "Image: ${IMAGE_NAME}:${IMAGE_TAG}"
    echo "Platform: $PLATFORM"
    if [[ -n "$REGISTRY" ]]; then
        echo "Registry: $REGISTRY"
    fi
    echo ""
    
    case "$command" in
        "build")
            check_docker
            run_local_build
            build_docker_image
            test_docker_image
            print_success "Build completed successfully!"
            ;;
        "push")
            check_docker
            run_local_build
            build_docker_image
            test_docker_image
            push_docker_image
            print_success "Build and push completed successfully!"
            ;;
        "test")
            check_docker
            test_docker_image
            ;;
        "clean")
            print_status "Cleaning build artifacts..."
            rm -rf target/build
            docker image prune -f
            cleanup
            print_success "Cleanup completed"
            ;;
        *)
            print_error "Unknown command: $command"
            show_usage
            exit 1
            ;;
    esac
    
    cleanup
}

# Trap for cleanup on exit
trap cleanup EXIT

# Parse arguments and run main function
parse_args "$@"
main 