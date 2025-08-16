#!/bin/bash

# RainbowBrowserAI Environment Setup Script

echo "🌈 RainbowBrowserAI Environment Setup"
echo "====================================="
echo ""

# Check if .env file exists
if [ -f .env ]; then
    echo "⚠️  .env file already exists. Loading existing configuration..."
    source .env
else
    echo "Creating new .env file..."
    touch .env
fi

# Function to prompt for input with default value
prompt_with_default() {
    local prompt=$1
    local default=$2
    local varname=$3
    
    if [ -z "${!varname}" ]; then
        read -p "$prompt [$default]: " value
        value=${value:-$default}
        echo "export $varname=$value" >> .env
        export $varname=$value
    else
        echo "✓ $varname already set"
    fi
}

# OpenAI API Key
if [ -z "$OPENAI_API_KEY" ]; then
    echo ""
    echo "📝 OpenAI API Configuration"
    echo "You'll need an OpenAI API key for natural language commands."
    echo "Get your API key from: https://platform.openai.com/api-keys"
    echo ""
    read -s -p "Enter your OpenAI API key (or press Enter to skip): " api_key
    echo ""
    if [ ! -z "$api_key" ]; then
        echo "export OPENAI_API_KEY=$api_key" >> .env
        export OPENAI_API_KEY=$api_key
        echo "✓ OpenAI API key configured"
    else
        echo "⚠️  Skipping OpenAI API key (natural language commands will not work)"
        echo ""
        read -p "Enable mock mode for testing without API key? (y/n): " enable_mock
        if [ "$enable_mock" = "y" ] || [ "$enable_mock" = "Y" ]; then
            echo "export RAINBOW_MOCK_MODE=true" >> .env
            export RAINBOW_MOCK_MODE=true
            echo "✓ Mock mode enabled (for testing only)"
        fi
    fi
else
    echo "✓ OPENAI_API_KEY already set"
fi

echo ""
echo "📊 Budget Configuration"
prompt_with_default "Daily budget limit (USD)" "5.0" "RAINBOW_DAILY_BUDGET"

echo ""
echo "🌐 ChromeDriver Configuration"
prompt_with_default "ChromeDriver port" "9515" "CHROMEDRIVER_PORT"

echo ""
echo "🔧 API Server Configuration"
prompt_with_default "API server port" "3000" "RAINBOW_API_PORT"

echo ""
echo "✅ Environment configuration complete!"
echo ""
echo "Your configuration has been saved to .env"
echo ""
echo "To use these settings:"
echo "  source .env"
echo ""
echo "To start the API server with dashboard:"
echo "  cargo run --release -- serve"
echo ""
echo "Dashboard will be available at:"
echo "  http://localhost:${RAINBOW_API_PORT:-3000}/"
echo ""

# Check if ChromeDriver is installed
if ! command -v chromedriver &> /dev/null; then
    echo "⚠️  ChromeDriver not found!"
    echo "Install it with:"
    echo "  # macOS"
    echo "  brew install chromedriver"
    echo ""
    echo "  # Linux"
    echo "  sudo apt-get install chromium-chromedriver"
    echo ""
    echo "  # Or download from:"
    echo "  https://chromedriver.chromium.org/"
fi

# Offer to start ChromeDriver
echo ""
read -p "Would you like to start ChromeDriver now? (y/n): " start_driver
if [ "$start_driver" = "y" ] || [ "$start_driver" = "Y" ]; then
    echo "Starting ChromeDriver on port ${CHROMEDRIVER_PORT:-9515}..."
    chromedriver --port=${CHROMEDRIVER_PORT:-9515} &
    echo "✓ ChromeDriver started (PID: $!)"
fi