#!/bin/bash

# RainbowBrowserAI Development Setup Script

set -e

echo "🌈 Setting up RainbowBrowserAI development environment..."

# Check prerequisites
check_prerequisites() {
    echo "📋 Checking prerequisites..."
    
    if ! command -v rustc &> /dev/null; then
        echo "❌ Rust not found. Please install Rust 1.75+ from https://rustup.rs/"
        exit 1
    fi
    
    RUST_VERSION=$(rustc --version | awk '{print $2}')
    echo "✅ Rust version: $RUST_VERSION"
    
    if ! command -v google-chrome &> /dev/null && ! command -v chromium &> /dev/null; then
        echo "❌ Chrome/Chromium not found. Please install Chrome or Chromium browser."
        exit 1
    fi
    echo "✅ Chrome/Chromium found"
    
    if ! command -v cargo &> /dev/null; then
        echo "❌ Cargo not found. Please install Cargo with Rust."
        exit 1
    fi
    echo "✅ Cargo found"
}

# Install development dependencies
install_dev_dependencies() {
    echo "📦 Installing development dependencies..."
    
    # Install cargo tools
    cargo install cargo-watch cargo-audit cargo-outdated
    
    echo "✅ Development dependencies installed"
}

# Setup environment
setup_environment() {
    echo "🔧 Setting up environment..."
    
    # Copy environment file if it doesn't exist
    if [ ! -f .env ]; then
        cp .env.example .env
        echo "📄 Created .env file from .env.example"
        echo "✏️  Please edit .env file with your configuration"
    else
        echo "📄 .env file already exists"
    fi
    
    # Create necessary directories
    mkdir -p logs config tmp
    echo "📁 Created necessary directories"
}

# Build the project
build_project() {
    echo "🔨 Building the project..."
    
    cd poc-chromiumoxide
    cargo build --release --bin rainbow-poc-chromiumoxide
    
    echo "✅ Project built successfully"
}

# Run tests
run_tests() {
    echo "🧪 Running tests..."
    
    cd poc-chromiumoxide
    cargo test --all-features
    
    echo "✅ Tests completed"
}

# Setup git hooks (optional)
setup_git_hooks() {
    echo "🔧 Setting up git hooks..."
    
    cat > .git/hooks/pre-commit << 'EOF'
#!/bin/bash
echo "Running pre-commit checks..."
cd poc-chromiumoxide
cargo fmt --check || (echo "Please run 'cargo fmt' before committing" && exit 1)
cargo clippy -- -D warnings || (echo "Please fix clippy warnings before committing" && exit 1)
EOF
    
    chmod +x .git/hooks/pre-commit
    echo "✅ Git hooks setup complete"
}

# Main setup function
main() {
    check_prerequisites
    install_dev_dependencies
    setup_environment
    build_project
    run_tests
    setup_git_hooks
    
    echo ""
    echo "🎉 Development environment setup complete!"
    echo ""
    echo "Next steps:"
    echo "  1. Edit .env file with your configuration"
    echo "  2. Start development server:"
    echo "     cd poc-chromiumoxide"
    echo "     RUST_LOG=debug cargo run -- serve --port 3001"
    echo "  3. Visit http://localhost:3001 to see the web interface"
    echo ""
    echo "Development commands:"
    echo "  cargo fmt              # Format code"
    echo "  cargo clippy           # Run linter"
    echo "  cargo test             # Run tests"
    echo "  cargo watch -x run     # Auto-reload on changes"
    echo ""
    echo "Happy coding! 🚀"
}

main "$@"