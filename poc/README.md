# 🌈 RainbowBrowserAI - Proof of Concept

A pragmatic, production-ready browser automation tool with AI capabilities.

## Features

✅ **Core Capabilities**
- Natural language command processing via OpenAI GPT-4
- Browser automation with screenshot capture
- YAML/JSON workflow engine
- Session management
- Cost tracking and budget control

✅ **REST API & Web Dashboard**
- Full-featured REST API with 9 endpoints
- Interactive web dashboard with 6 tabs
- Real-time metrics and cost visualization
- Dark mode support
- Settings persistence

✅ **Production Features**
- Browser connection pooling
- Multi-layer caching
- Security middleware
- Rate limiting
- Comprehensive error handling

## Quick Start

See [QUICKSTART.md](QUICKSTART.md) for detailed setup instructions.

```bash
# 1. Set up environment
./setup_env.sh

# 2. Start ChromeDriver
chromedriver --port=9515 &

# 3. Start the server
cargo run --release -- serve

# 4. Open dashboard
# Navigate to http://localhost:3000/
```

## Project Structure

```
poc/
├── src/
│   ├── api.rs              # REST API implementation
│   ├── browser.rs          # WebDriver control
│   ├── llm_service.rs      # OpenAI integration
│   ├── workflow.rs         # Workflow engine
│   ├── browser_pool.rs     # Connection pooling
│   ├── cache.rs            # Caching layer
│   ├── metrics.rs          # Performance metrics
│   ├── security.rs         # Security middleware
│   └── main.rs             # CLI application
├── static/
│   ├── index.html          # Dashboard UI
│   ├── styles.css          # Dashboard styling
│   └── app.js              # Dashboard logic
├── examples/
│   └── workflows/          # Sample workflows
├── tests/                  # Test suite
└── docs/                   # Documentation
```

## API Endpoints

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/` | GET | Web dashboard |
| `/health` | GET | Health check |
| `/metrics` | GET | Performance metrics |
| `/cost` | GET | Cost report |
| `/command` | POST | Natural language command |
| `/navigate` | POST | Navigate to URL |
| `/screenshot` | POST | Take screenshot |
| `/workflow` | POST | Execute workflow |
| `/session` | POST | Manage sessions |

## Dashboard Features

- **Command Tab**: Execute natural language commands
- **Browse Tab**: Direct browser control
- **Workflow Tab**: Build and run workflows
- **Sessions Tab**: Manage browser sessions
- **Metrics Tab**: View performance and costs
- **Settings Tab**: Configure API keys

## Configuration

Environment variables (see `.env.example`):
- `OPENAI_API_KEY`: OpenAI API key for natural language
- `RAINBOW_DAILY_BUDGET`: Daily spending limit
- `CHROMEDRIVER_PORT`: ChromeDriver port
- `RAINBOW_API_PORT`: API server port

## Development

```bash
# Run tests
cargo test

# Run with logging
RUST_LOG=rainbow_poc=debug cargo run

# Build release version
cargo build --release

# Run benchmarks
cargo bench
```

## Documentation

- [Quick Start Guide](QUICKSTART.md)
- [Development Progress](../DEVELOPMENT_PROGRESS.md)
- [API Documentation](../docs/API.md)
- [Troubleshooting](../docs/TROUBLESHOOTING.md)

## Performance

- Navigation: <3s average
- Screenshot: <5s full page
- API response: <200ms
- Memory usage: 100-200MB
- Concurrent browsers: 100+

## Security

- Input validation and sanitization
- Rate limiting (100 req/min)
- URL validation (no local files)
- API key protection
- CORS support

## License

MIT - See [LICENSE](../LICENSE) for details