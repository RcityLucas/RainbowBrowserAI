# Multi-stage build for minimal production image
FROM rust:1.75 as builder

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Create app directory
WORKDIR /app

# Copy manifest files
COPY Cargo.toml Cargo.lock ./

# Create dummy main to cache dependencies
RUN mkdir src && \
    echo "fn main() {}" > src/main.rs && \
    cargo build --release && \
    rm -rf src

# Copy source code
COPY src ./src
COPY workflows ./workflows

# Build release binary
RUN cargo build --release

# Production stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    chromium-driver \
    chromium \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN useradd -m -u 1001 rainbow && \
    mkdir -p /app/screenshots /app/workflows /app/data && \
    chown -R rainbow:rainbow /app

WORKDIR /app

# Copy binary from builder
COPY --from=builder /app/target/release/rainbow-poc /usr/local/bin/
COPY --from=builder --chown=rainbow:rainbow /app/workflows ./workflows

# Set Chrome options for container
ENV CHROME_DRIVER_URL=http://localhost:9515 \
    CHROME_NO_SANDBOX=true \
    DAILY_BUDGET=5.00 \
    RUST_LOG=info

# Switch to non-root user
USER rainbow

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD rainbow-poc report || exit 1

# Volume for persistent data
VOLUME ["/app/screenshots", "/app/data"]

# Default command
CMD ["rainbow-poc", "--help"]