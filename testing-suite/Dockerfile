# Multi-stage Dockerfile for RainbowBrowserAI Testing Suite
FROM node:18-bullseye as base

# Install system dependencies
RUN apt-get update && apt-get install -y \
    curl \
    jq \
    bc \
    git \
    build-essential \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Install Rust
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"

# Set working directory
WORKDIR /app

# Development stage
FROM base as development

# Install Playwright dependencies
RUN npx playwright install-deps

# Copy package files
COPY package*.json ./
COPY tsconfig.json ./
COPY jest.config.js ./

# Install Node.js dependencies
RUN npm ci

# Install Playwright browsers
RUN npx playwright install

# Copy source code
COPY src/ ./src/

# Set environment variables
ENV NODE_ENV=test
ENV RAINBOW_MOCK_MODE=true
ENV HEADLESS=true

# Expose port for test server
EXPOSE 3002

# Default command
CMD ["npm", "test"]

# Production testing stage
FROM base as production

# Copy package files
COPY package*.json ./
COPY tsconfig.json ./
COPY jest.config.js ./
COPY playwright.config.ts ./

# Install only production dependencies
RUN npm ci --only=production

# Install Playwright with dependencies
RUN npx playwright install-deps
RUN npx playwright install

# Copy built source
COPY src/ ./src/

# Create non-root user for security
RUN useradd -r -s /bin/false testuser && \
    chown -R testuser:testuser /app

USER testuser

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:3002/api/health || exit 1

# Set environment variables
ENV NODE_ENV=production
ENV RAINBOW_MOCK_MODE=true
ENV HEADLESS=true

# Run tests
CMD ["npm", "run", "test:all"]

# CI/CD stage for continuous integration
FROM base as ci

# Install all dependencies including dev dependencies
COPY package*.json ./
COPY tsconfig.json ./
COPY jest.config.js ./
COPY playwright.config.ts ./
RUN npm ci

# Install Playwright with all browsers
RUN npx playwright install-deps
RUN npx playwright install

# Copy all source files
COPY . .

# Create reports directory
RUN mkdir -p reports coverage test-results

# Set permissions
RUN chmod +x scripts/* || true

# Environment for CI
ENV CI=true
ENV NODE_ENV=test
ENV RAINBOW_MOCK_MODE=true
ENV HEADLESS=true

# Command for CI pipeline
CMD ["npm", "run", "test:ci"]

# Performance testing stage
FROM base as performance

# Install performance testing dependencies
RUN npm install -g artillery clinic autocannon

COPY package*.json ./
RUN npm ci

# Copy performance test files
COPY src/performance/ ./src/performance/
COPY performance/ ./performance/

ENV NODE_ENV=performance
ENV RAINBOW_MOCK_MODE=true

# Run performance tests
CMD ["npm", "run", "test:performance"]

# Debug stage for troubleshooting
FROM development as debug

# Install additional debug tools
RUN apt-get update && apt-get install -y \
    vim \
    htop \
    strace \
    tcpdump \
    netstat-nat \
    && rm -rf /var/lib/apt/lists/*

# Enable debug logging
ENV DEBUG=*
ENV RUST_LOG=debug
ENV VERBOSE_TESTS=true

# Interactive shell by default
CMD ["/bin/bash"]