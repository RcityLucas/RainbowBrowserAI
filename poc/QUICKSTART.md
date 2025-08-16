# 🌈 RainbowBrowserAI Quick Start Guide

Get up and running with RainbowBrowserAI in 5 minutes!

## Prerequisites

1. **Rust** (1.70+)
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

2. **ChromeDriver**
   ```bash
   # macOS
   brew install chromedriver
   
   # Ubuntu/Debian
   sudo apt-get install chromium-chromedriver
   
   # Or download from https://chromedriver.chromium.org/
   ```

3. **OpenAI API Key** (optional, for natural language commands)
   - Get your key from [OpenAI Platform](https://platform.openai.com/api-keys)

## Installation

1. **Clone the repository**
   ```bash
   git clone https://github.com/yourusername/RainbowBrowserAI.git
   cd RainbowBrowserAI/poc
   ```

2. **Set up environment**
   ```bash
   # Run the interactive setup script
   ./setup_env.sh
   
   # Or manually copy and edit the environment file
   cp .env.example .env
   # Edit .env with your API key and preferences
   ```

3. **Build the project**
   ```bash
   cargo build --release
   ```

## Quick Start

### Option 1: Web Dashboard (Recommended)

1. **Start ChromeDriver**
   ```bash
   chromedriver --port=9515 &
   ```

2. **Start the API server with dashboard**
   ```bash
   source .env  # Load environment variables
   cargo run --release -- serve
   ```

3. **Open the dashboard**
   - Navigate to http://localhost:3000/
   - The dashboard provides a user-friendly interface for all features

### Option 2: Command Line Interface

1. **Basic navigation**
   ```bash
   # Navigate to a website
   cargo run --release -- navigate "https://github.com"
   
   # Navigate and take a screenshot
   cargo run --release -- navigate "https://github.com" --screenshot
   ```

2. **Natural language commands** (requires OpenAI API key)
   ```bash
   # Use natural language
   cargo run --release -- ask "go to github and take a screenshot"
   
   # More complex commands
   cargo run --release -- ask "navigate to google, search for rust programming, and capture the results"
   ```

3. **Run workflows**
   ```bash
   # Execute a workflow file
   cargo run --release -- workflow workflow.yaml
   ```

## Dashboard Features

Once you open http://localhost:3000/, you'll find:

- **🔧 Command Tab**: Execute natural language commands
- **🌐 Browse Tab**: Direct browser control with screenshots
- **📊 Workflow Tab**: Build and run automation workflows
- **💻 Sessions Tab**: Manage browser sessions
- **📈 Metrics Tab**: View performance and cost metrics
- **⚙️ Settings Tab**: Configure API keys and preferences

## Common Issues & Solutions

### Issue: "OpenAI API key not configured"
**Solution**: Set your API key in the environment
```bash
export OPENAI_API_KEY=sk-your-key-here
# Or add it to your .env file
```

### Issue: "Failed to connect to ChromeDriver"
**Solution**: Ensure ChromeDriver is running
```bash
# Check if ChromeDriver is running
ps aux | grep chromedriver

# Start ChromeDriver
chromedriver --port=9515 &
```

### Issue: "Daily budget exceeded"
**Solution**: Check your cost report and adjust budget
```bash
# View cost report
cargo run --release -- report

# Adjust budget in .env
export RAINBOW_DAILY_BUDGET=10.0
```

## Next Steps

1. **Explore workflows**: Check out `examples/workflows/` for automation templates
2. **Read the docs**: See [DEVELOPMENT_PROGRESS.md](DEVELOPMENT_PROGRESS.md) for detailed features
3. **Customize settings**: Edit `config.yaml` for advanced configuration
4. **Join the community**: Report issues and contribute on GitHub

## Getting Help

- **Documentation**: Check the `docs/` folder
- **Examples**: See `examples/` for sample workflows
- **Issues**: Report bugs on [GitHub Issues](https://github.com/yourusername/RainbowBrowserAI/issues)
- **API Reference**: View at http://localhost:3000/docs (when server is running)

## Tips for Success

1. **Start simple**: Try basic navigation before complex workflows
2. **Use the dashboard**: It's the easiest way to explore features
3. **Monitor costs**: Check the metrics tab regularly
4. **Save workflows**: Build reusable automation templates
5. **Experiment safely**: Use `--dry-run` flag to test workflows without execution

---

**Ready to automate?** 🚀 Start with the dashboard at http://localhost:3000/