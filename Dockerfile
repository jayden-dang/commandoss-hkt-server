# Multi-stage build for ZK-Guardian with GitHub Integration
FROM --platform=linux/amd64 rust:1.87.0-bullseye AS builder

# Add metadata labels
LABEL maintainer="Jayden Dang <jayden.dangvu@gmail.com>"
LABEL version="1.0.0"
LABEL description="ZK-Guardian GitHub Integration Service"

WORKDIR /app

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    libpq-dev \
    binutils \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Copy dependency files first for better caching
COPY Cargo.toml Cargo.lock ./
COPY rustfmt.toml ./

# Copy all Cargo.toml files from crates (including new services)
COPY crates/core/jd_core/Cargo.toml ./crates/core/jd_core/
COPY crates/gateways/api_gateway/Cargo.toml ./crates/gateways/api_gateway/
COPY crates/gateways/web_server/Cargo.toml ./crates/gateways/web_server/
COPY crates/infrastructure/jd_infra/Cargo.toml ./crates/infrastructure/jd_infra/
COPY crates/infrastructure/jd_messaging/Cargo.toml ./crates/infrastructure/jd_messaging/
COPY crates/infrastructure/jd_storage/Cargo.toml ./crates/infrastructure/jd_storage/
COPY crates/infrastructure/jd_tracing/Cargo.toml ./crates/infrastructure/jd_tracing/
COPY crates/shared/jd_domain/Cargo.toml ./crates/shared/jd_domain/
COPY crates/shared/jd_rpc_core/Cargo.toml ./crates/shared/jd_rpc_core/
COPY crates/shared/jd_utils/Cargo.toml ./crates/shared/jd_utils/

# GitHub Integration and new services
COPY crates/services/github_service/Cargo.toml ./crates/services/github_service/
COPY crates/services/auth_service/Cargo.toml ./crates/services/auth_service/
COPY crates/services/analytics_service/Cargo.toml ./crates/services/analytics_service/
COPY crates/services/behavior_service/Cargo.toml ./crates/services/behavior_service/
COPY crates/services/developer_service/Cargo.toml ./crates/services/developer_service/
COPY crates/services/patch_service/Cargo.toml ./crates/services/patch_service/
COPY crates/services/scoring_service/Cargo.toml ./crates/services/scoring_service/
COPY crates/services/sui_service/Cargo.toml ./crates/services/sui_service/
COPY crates/services/vulnerability_service/Cargo.toml ./crates/services/vulnerability_service/
COPY crates/services/zkproof_service/Cargo.toml ./crates/services/zkproof_service/

# Create dummy source files for dependency caching
RUN mkdir -p crates/core/jd_core/src && \
    echo "pub fn dummy() {}" > crates/core/jd_core/src/lib.rs && \
    mkdir -p crates/gateways/api_gateway/src && \
    echo "pub fn dummy() {}" > crates/gateways/api_gateway/src/lib.rs && \
    mkdir -p crates/gateways/web_server/src && \
    echo "fn main() {}" > crates/gateways/web_server/src/main.rs && \
    mkdir -p crates/infrastructure/jd_infra/src && \
    echo "pub fn dummy() {}" > crates/infrastructure/jd_infra/src/lib.rs && \
    mkdir -p crates/infrastructure/jd_messaging/src && \
    echo "pub fn dummy() {}" > crates/infrastructure/jd_messaging/src/lib.rs && \
    mkdir -p crates/infrastructure/jd_storage/src && \
    echo "pub fn dummy() {}" > crates/infrastructure/jd_storage/src/lib.rs && \
    mkdir -p crates/infrastructure/jd_tracing/src && \
    echo "pub fn dummy() {}" > crates/infrastructure/jd_tracing/src/lib.rs && \
    mkdir -p crates/shared/jd_domain/src && \
    echo "pub fn dummy() {}" > crates/shared/jd_domain/src/lib.rs && \
    mkdir -p crates/shared/jd_rpc_core/src && \
    echo "pub fn dummy() {}" > crates/shared/jd_rpc_core/src/lib.rs && \
    mkdir -p crates/shared/jd_utils/src && \
    echo "pub fn dummy() {}" > crates/shared/jd_utils/src/lib.rs && \
    mkdir -p crates/services/github_service/src && \
    echo "pub fn dummy() {}" > crates/services/github_service/src/lib.rs && \
    mkdir -p crates/services/auth_service/src && \
    echo "pub fn dummy() {}" > crates/services/auth_service/src/lib.rs && \
    mkdir -p crates/services/analytics_service/src && \
    echo "pub fn dummy() {}" > crates/services/analytics_service/src/lib.rs && \
    mkdir -p crates/services/behavior_service/src && \
    echo "pub fn dummy() {}" > crates/services/behavior_service/src/lib.rs && \
    mkdir -p crates/services/developer_service/src && \
    echo "pub fn dummy() {}" > crates/services/developer_service/src/lib.rs && \
    mkdir -p crates/services/patch_service/src && \
    echo "pub fn dummy() {}" > crates/services/patch_service/src/lib.rs && \
    mkdir -p crates/services/scoring_service/src && \
    echo "pub fn dummy() {}" > crates/services/scoring_service/src/lib.rs && \
    mkdir -p crates/services/sui_service/src && \
    echo "pub fn dummy() {}" > crates/services/sui_service/src/lib.rs && \
    mkdir -p crates/services/vulnerability_service/src && \
    echo "pub fn dummy() {}" > crates/services/vulnerability_service/src/lib.rs && \
    mkdir -p crates/services/zkproof_service/src && \
    echo "pub fn dummy() {}" > crates/services/zkproof_service/src/lib.rs

# Build dependencies
RUN --mount=type=cache,target=/app/target \
    --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git \
    cargo build --release

# Copy actual source code
COPY . .

# Build the application (native AMD64)
RUN --mount=type=cache,target=/app/target \
    --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git \
    cargo build --workspace --release && \
    echo "=== Built binaries ===" && \
    ls -la target/release/ && \
    echo "=== Copying web_server binary ===" && \
    cp target/release/web_server ./app && \
    ls -la ./app && \
    strip ./app

# Runtime stage - Ubuntu for compatibility
FROM --platform=linux/amd64 ubuntu:22.04 AS deploy

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    curl \
    libssl3 \
    libpq5 \
    jq \
    openssl \
    git \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN useradd -m -u 1000 appuser

WORKDIR /deploy

# Copy binary from builder
COPY --from=builder /app/app ./app

# Copy SQL migrations and other necessary files
COPY --from=builder /app/sql ./sql
COPY --from=builder /app/docs ./docs

# Copy GitHub integration test scripts and documentation
COPY --from=builder /app/test_*.sh ./
COPY --from=builder /app/setup_github_app_env.sh ./
COPY --from=builder /app/GITHUB_INTEGRATION_README.md ./

# Make test scripts executable
RUN chmod +x ./test_*.sh ./setup_github_app_env.sh

# Create default environment file for GitHub integration
RUN echo '# Database Configuration' > .env && \
    echo 'DATABASE_URL=postgres://jayden:postgres@localhost:5432/jaydenblog' >> .env && \
    echo 'REDIS_URL=redis://localhost:6379/' >> .env && \
    echo '' >> .env && \
    echo '# Server Configuration' >> .env && \
    echo 'WEB_ADDR=0.0.0.0:8080' >> .env && \
    echo 'HOST=0.0.0.0' >> .env && \
    echo 'PORT=8080' >> .env && \
    echo '' >> .env && \
    echo '# GitHub Integration (Configure these for production)' >> .env && \
    echo '# Option 1: GitHub App (Recommended)' >> .env && \
    echo '# GITHUB_APP_ID=your_app_id' >> .env && \
    echo '# GITHUB_PRIVATE_KEY_PATH=/deploy/github-app-private-key.pem' >> .env && \
    echo '# GITHUB_WEBHOOK_SECRET=your_webhook_secret' >> .env && \
    echo '' >> .env && \
    echo '# Option 2: Personal Access Token' >> .env && \
    echo '# GITHUB_TOKEN=ghp_your_personal_token' >> .env && \
    echo '# GITHUB_WEBHOOK_SECRET=your_webhook_secret' >> .env && \
    echo '' >> .env && \
    echo '# GitHub Service Configuration' >> .env && \
    echo 'WEBHOOK_BASE_URL=https://your-domain.com' >> .env && \
    echo 'GITHUB_MAX_QUEUE_SIZE=1000' >> .env && \
    echo 'GITHUB_RATE_LIMIT_PER_HOUR=5000' >> .env && \
    echo '' >> .env && \
    echo '# Logging' >> .env && \
    echo 'RUST_LOG=info' >> .env && \
    echo 'RUST_BACKTRACE=1' >> .env

# Create directories for logs and GitHub keys
RUN mkdir -p /deploy/logs /deploy/data && \
    chmod +w /deploy && \
    chown -R appuser:appuser /deploy

# Set permissions
RUN chmod +x ./app

# Switch to non-root user
USER appuser

# Set environment variables (fallback if .env doesn't work)
ENV DATABASE_URL=postgres://jayden:postgres@localhost:5432/jaydenblog
ENV REDIS_URL=redis://localhost:6379/
ENV WEB_ADDR=0.0.0.0:8080
ENV HOST=0.0.0.0
ENV PORT=8080
ENV RUST_LOG=info
ENV RUST_BACKTRACE=1

# GitHub Integration defaults
ENV WEBHOOK_BASE_URL=https://your-domain.com
ENV GITHUB_MAX_QUEUE_SIZE=1000
ENV GITHUB_RATE_LIMIT_PER_HOUR=5000

# Expose port
EXPOSE 8080

# Health check with GitHub integration
HEALTHCHECK --interval=30s --timeout=3s --start-period=10s --retries=3 \
    CMD curl -f http://localhost:8080/api/v1/health || exit 1

# Run application
ENTRYPOINT ["./app"]
