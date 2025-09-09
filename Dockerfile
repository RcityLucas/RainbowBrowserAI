# Multi-stage build for RainbowBrowserAI
FROM rust:1.75-slim-bullseye AS builder

# Install system dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    curl \
    wget \
    gnupg \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Set working directory
WORKDIR /app

# Copy manifests first for better layer caching
COPY Cargo.toml Cargo.lock ./
COPY poc-chromiumoxide/Cargo.toml ./poc-chromiumoxide/

# Create dummy main.rs to cache dependencies
RUN mkdir -p poc-chromiumoxide/src && echo "fn main() {}" > poc-chromiumoxide/src/main.rs

# Build dependencies
WORKDIR /app/poc-chromiumoxide
RUN cargo build --release --bin rainbow-poc-chromiumoxide
RUN rm src/main.rs

# Copy source code
COPY poc-chromiumoxide/src ./src/
COPY poc-chromiumoxide/static ./static/

# Build the application
RUN cargo build --release --bin rainbow-poc-chromiumoxide

# Runtime stage
FROM debian:bullseye-slim

# Install Chrome/Chromium and dependencies
RUN apt-get update && apt-get install -y \
    wget \
    gnupg \
    ca-certificates \
    fonts-liberation \
    libasound2 \
    libatk-bridge2.0-0 \
    libatk1.0-0 \
    libatspi2.0-0 \
    libdrm2 \
    libgtk-3-0 \
    libnspr4 \
    libnss3 \
    libwayland-client0 \
    libxcomposite1 \
    libxdamage1 \
    libxfixes3 \
    libxrandr2 \
    xdg-utils \
    libu2f-udev \
    libvulkan1 \
    && wget -q -O - https://dl-ssl.google.com/linux/linux_signing_key.pub | apt-key add - \
    && echo "deb [arch=amd64] http://dl.google.com/linux/chrome/deb/ stable main" >> /etc/apt/sources.list.d/google-chrome.list \
    && apt-get update \
    && apt-get install -y google-chrome-stable \
    && rm -rf /var/lib/apt/lists/*

# Install ChromeDriver
RUN CHROME_VERSION=$(google-chrome --version | awk '{print $3}' | cut -d'.' -f1-3) \
    && CHROMEDRIVER_VERSION=$(wget -qO- "https://chromedriver.storage.googleapis.com/LATEST_RELEASE_${CHROME_VERSION}") \
    && wget -O /tmp/chromedriver.zip "https://chromedriver.storage.googleapis.com/${CHROMEDRIVER_VERSION}/chromedriver_linux64.zip" \
    && unzip /tmp/chromedriver.zip -d /usr/local/bin/ \
    && rm /tmp/chromedriver.zip \
    && chmod +x /usr/local/bin/chromedriver

# Create non-root user
RUN useradd -m -u 1001 rainbow && \
    mkdir -p /app && \
    chown -R rainbow:rainbow /app

USER rainbow
WORKDIR /app

# Copy the built binary
COPY --from=builder --chown=rainbow:rainbow /app/poc-chromiumoxide/target/release/rainbow-poc-chromiumoxide ./

# Copy static files
COPY --from=builder --chown=rainbow:rainbow /app/poc-chromiumoxide/static ./static/

# Set environment variables
ENV RAINBOW_MOCK_MODE=false
ENV BROWSER_HEADLESS=true
ENV SERVER_PORT=3001
ENV CHROMEDRIVER_PORT=9515

# Expose port
EXPOSE 3001

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:3001/api/health || exit 1

# Run the application
CMD ["./rainbow-poc-chromiumoxide", "serve", "--port", "3001"]