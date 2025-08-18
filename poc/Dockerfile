# Multi-stage build for RainbowBrowserAI
# Stage 1: Builder
FROM rust:1.75-slim as builder

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    build-essential \
    && rm -rf /var/lib/apt/lists/*

# Create app directory
WORKDIR /usr/src/rainbow-poc

# Copy manifests
COPY Cargo.toml Cargo.lock ./

# Create dummy main to cache dependencies
RUN mkdir src && \
    echo "fn main() {}" > src/main.rs && \
    cargo build --release --bin rainbow-poc && \
    rm -rf src

# Copy source code
COPY src ./src
COPY static ./static

# Build the application in release mode
RUN cargo build --release --bin rainbow-poc

# Stage 2: Runtime
FROM debian:bookworm-slim

# Install runtime dependencies including Chrome
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    chromium \
    chromium-driver \
    wget \
    gnupg \
    fonts-liberation \
    libasound2 \
    libatk-bridge2.0-0 \
    libatk1.0-0 \
    libatspi2.0-0 \
    libcups2 \
    libdbus-1-3 \
    libdrm2 \
    libgbm1 \
    libgtk-3-0 \
    libnspr4 \
    libnss3 \
    libxcomposite1 \
    libxdamage1 \
    libxfixes3 \
    libxkbcommon0 \
    libxrandr2 \
    xdg-utils \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN useradd -m -u 1001 rainbow && \
    mkdir -p /app/plugins /app/screenshots /app/data /app/static && \
    chown -R rainbow:rainbow /app

# Copy the binary from builder
COPY --from=builder /usr/src/rainbow-poc/target/release/rainbow-poc /app/rainbow-poc

# Copy static files
COPY --chown=rainbow:rainbow static /app/static

# Copy plugin examples if they exist
COPY --chown=rainbow:rainbow plugins/examples /app/plugins/examples 2>/dev/null || true

# Set environment variables
ENV RUST_LOG=info \
    RUST_BACKTRACE=1 \
    CHROMIUM_PATH=/usr/bin/chromium \
    CHROMEDRIVER_PATH=/usr/bin/chromedriver \
    PLUGIN_DIR=/app/plugins \
    SCREENSHOT_DIR=/app/screenshots \
    DATA_DIR=/app/data \
    API_HOST=0.0.0.0 \
    API_PORT=3000 \
    DAILY_BUDGET=10.00 \
    CHROME_NO_SANDBOX=true

# Switch to non-root user
USER rainbow

# Set working directory
WORKDIR /app

# Expose ports
EXPOSE 3000

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD wget --no-verbose --tries=1 --spider http://localhost:3000/health || exit 1

# Volume for persistent data
VOLUME ["/app/screenshots", "/app/plugins", "/app/data"]

# Default command
CMD ["./rainbow-poc", "api"]