#!/bin/bash

# Docker Build Script for ZK-Guardian GitHub Integration
set -e

# Configuration
IMAGE_NAME="zkguardian-github"
IMAGE_TAG="latest"
DOCKER_REGISTRY="jaydendang"  # Change this to your Docker registry
FULL_IMAGE_NAME="${DOCKER_REGISTRY}/${IMAGE_NAME}:${IMAGE_TAG}"

echo "🚀 Building ZK-Guardian GitHub Integration Docker Image"
echo "================================================="

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to print colored output
print_status() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

print_step() {
    echo -e "${BLUE}[STEP]${NC} $1"
}

# Check if Docker is running
if ! docker info > /dev/null 2>&1; then
    print_error "Docker is not running. Please start Docker and try again."
    exit 1
fi

# Check if we're in the right directory
if [ ! -f "Dockerfile" ]; then
    print_error "Dockerfile not found. Please run this script from the server directory."
    exit 1
fi

print_step "1. Pre-build validation"

# Check if essential files exist
essential_files=(
    "Cargo.toml"
    "Cargo.lock"
    "crates/services/github_service/Cargo.toml"
    "crates/gateways/web_server/src/main.rs"
    "test_github_integration_full.sh"
    "GITHUB_INTEGRATION_README.md"
)

for file in "${essential_files[@]}"; do
    if [ ! -f "$file" ]; then
        print_error "Essential file missing: $file"
        exit 1
    fi
done

print_status "All essential files found ✓"

print_step "2. Building Docker image"

# Build the Docker image
print_status "Building image: $FULL_IMAGE_NAME"
print_warning "This may take 10-15 minutes for the first build..."

docker build \
    --platform linux/amd64 \
    --tag "$FULL_IMAGE_NAME" \
    --tag "${IMAGE_NAME}:${IMAGE_TAG}" \
    --tag "${IMAGE_NAME}:latest" \
    --progress=plain \
    .

if [ $? -eq 0 ]; then
    print_status "Docker image built successfully! ✓"
else
    print_error "Docker build failed!"
    exit 1
fi

print_step "3. Image information"

# Show image size and details
docker images | grep -E "(REPOSITORY|${IMAGE_NAME})"

print_step "4. Quick validation"

# Test that the image can start
print_status "Testing image startup..."
CONTAINER_ID=$(docker run -d --name zkguardian-test -p 8081:8080 "$FULL_IMAGE_NAME")

if [ $? -eq 0 ]; then
    print_status "Container started successfully"

    # Wait a moment for startup
    sleep 5

    # Test health endpoint
    if curl -f http://localhost:8081/api/v1/health > /dev/null 2>&1; then
        print_status "Health check passed ✓"
    else
        print_warning "Health check failed (this might be expected without database)"
    fi

    # Stop and remove test container
    docker stop "$CONTAINER_ID" > /dev/null
    docker rm "$CONTAINER_ID" > /dev/null
    print_status "Test container cleaned up"
else
    print_error "Failed to start test container"
fi

print_step "5. Build complete!"

echo
echo "================================================="
print_status "Image built successfully: $FULL_IMAGE_NAME"
echo
echo "Next steps:"
echo "1. Push to registry: docker push $FULL_IMAGE_NAME"
echo "2. Deploy to EC2: Use the deployment scripts provided"
echo "3. Configure environment: Update .env with your GitHub App credentials"
echo
echo "Local testing:"
echo "  docker run -p 8080:8080 $FULL_IMAGE_NAME"
echo
echo "With custom environment:"
echo "  docker run -p 8080:8080 --env-file .env $FULL_IMAGE_NAME"
echo "================================================="
