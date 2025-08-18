# 🌈 RainbowBrowserAI

Browser automation with AI-powered natural language commands.

## Features

- 🤖 **Mock Mode**: Works without API keys
- 🌐 **Web Interface**: User-friendly dashboard  
- 📸 **Screenshots**: Automatic capture
- 🔄 **Browser Control**: Navigate, click, extract data
- 💸 **Cost Tracking**: Built-in budget management
- 🔌 **Plugin System**: Extensible architecture

## Quick Start

1. **Install ChromeDriver**: Run `download_direct.ps1`
2. **Start Application**: Run `start.bat`  
3. **Open Browser**: Go to http://localhost:3000
4. **Test Commands**: Try `navigate to github.com`

## Commands

### Basic Commands
- `navigate to github.com`
- `go to google and take screenshot`
- `open stackoverflow`

### Advanced Commands  
- `test google.com,github.com,stackoverflow.com with screenshots`
- `navigate to github.com with 1920x1080 and take screenshot`
- `extract data from news.ycombinator.com`
- `monitor mysite.com for changes`

**See [EXAMPLES.md](EXAMPLES.md) for more complex examples and API usage.**

## Known Issues & Fixes

- **URL Parsing Issue**: If "go to stackoverflow" navigates to go.com instead of stackoverflow.com, see [URL_PARSING_FIX.md](URL_PARSING_FIX.md) for the solution.

## Files

- **start.bat** - Start the application
- **download_direct.ps1** - Download ChromeDriver
- **FIX_CHROMEDRIVER.bat** - Fix version issues

## API

Full REST API available at:
- `POST /command` - Execute commands
- `GET /health` - System status
- `GET /metrics` - Performance data
- `POST /extract` - Data extraction

See [API_DOCUMENTATION.md](API_DOCUMENTATION.md) for details.

---

**Note**: This is a Proof of Concept. For production use, consider security implications.