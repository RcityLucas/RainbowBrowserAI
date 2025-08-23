# 🚀 RainbowBrowserAI - Quick Start

Get started in 2 simple steps!

## Prerequisites

1. **Rust** - Install from [rustup.rs](https://rustup.rs/)
2. **Chrome Browser** - Download from [google.com/chrome](https://www.google.com/chrome/)

## Quick Start

### Step 1: Download ChromeDriver
```cmd
download_direct.ps1
```
This downloads the correct ChromeDriver version for your Chrome.

### Step 2: Start the Application
```cmd
start.bat
```
The browser will automatically open to http://localhost:3000

## Usage

In the web interface, try these commands:
- `navigate to github.com`
- `go to google and take screenshot`  
- `open stackoverflow`

## Files You Need

- **start.bat** - Start the application
- **download_direct.ps1** - Download ChromeDriver  
- **FIX_CHROMEDRIVER.bat** - Fix version mismatches

## Features

- 🤖 **Mock Mode**: Works without API keys
- 🌐 **Web Interface**: Easy to use dashboard
- 📸 **Screenshots**: Automatic screenshot capture
- 🔄 **Browser Control**: Navigate, click, extract data

## Troubleshooting

**ChromeDriver version mismatch?**
Run: `FIX_CHROMEDRIVER.bat`

**Port 3000 busy?**
Edit start.bat and change the port number.

That's it! 🎉