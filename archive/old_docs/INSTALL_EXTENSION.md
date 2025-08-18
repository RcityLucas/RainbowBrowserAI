# 🌈 RainbowBrowserAI Browser Extension Installation Guide

## Quick Install (3 Steps)

### Step 1: Build the Server (Optional)
If you want the full AI capabilities, build and run the server:
```bash
# Build with web server feature
cargo build --release --features web-server

# Run the server
cargo run --features web-server
# Or run the compiled binary:
./target/release/rainbow-browser
```

### Step 2: Install the Browser Extension

#### Chrome/Edge:
1. Open browser extension management page:
   - Chrome: `chrome://extensions`
   - Edge: `edge://extensions`
2. Enable "Developer mode" (toggle in top right)
3. Click "Load unpacked"
4. Navigate to and select: `/mnt/d/github/RainbowBrowserAI/src/browser_extension`
5. The Rainbow icon 🌈 should appear in your toolbar!

#### Firefox:
1. Open `about:debugging`
2. Click "This Firefox"
3. Click "Load Temporary Add-on"
4. Navigate to `/mnt/d/github/RainbowBrowserAI/src/browser_extension`
5. Select `manifest.json`

### Step 3: Test the Extension
1. Open the test page: `file:///mnt/d/github/RainbowBrowserAI/test_extension.html`
2. Click the 🌈 icon in your browser toolbar
3. Or press `Ctrl+Shift+R` (Windows/Linux) or `Cmd+Shift+R` (Mac)
4. Try commands like:
   - "搜索人工智能"
   - "点击登录按钮"
   - "填写表单"
   - "提取所有价格"

## Features

### 🎯 Natural Language Commands
- **Chinese**: "搜索最新科技新闻", "点击下一页", "滚动到底部"
- **English**: "search for AI news", "click login", "extract all links"

### 🖱️ Smart Actions
- Automatic element detection
- Form filling with smart field recognition
- Data extraction from tables and lists
- Screenshot capture
- Page navigation

### ⚡ Two Modes

#### Server Mode (Full AI)
- Start the server first: `cargo run --features web-server`
- Extension connects to `http://localhost:8888`
- Full LLM-powered intelligence

#### Browser-Only Mode (Lightweight)
- Works without server
- Pattern-based smart actions
- Perfect for simple automation

## Troubleshooting

### Extension Not Loading?
- Make sure Developer Mode is enabled
- Check that all files are present in `src/browser_extension/`
- Try reloading the extension

### Server Connection Failed?
- Ensure server is running on port 8888
- Check firewall settings
- Extension will work in browser-only mode as fallback

### Commands Not Working?
- Make sure you're on a regular webpage (not chrome:// or about:// pages)
- Try refreshing the page
- Check browser console for errors (F12 → Console tab)

## Advanced Usage

### Custom Server Port
Edit `src/browser_extension/background.js`:
```javascript
const SERVER_URL = 'http://localhost:YOUR_PORT';
```

### Keyboard Shortcut
Default: `Ctrl+Shift+R` (Windows/Linux) or `Cmd+Shift+R` (Mac)

To change:
1. Go to `chrome://extensions/shortcuts`
2. Find "RainbowBrowserAI Assistant"
3. Set your preferred shortcut

## Security Note

This extension requires broad permissions to work on all websites. It:
- ✅ Runs locally on your computer
- ✅ Does not send data to external servers
- ✅ Open source and auditable
- ⚠️ Has access to webpage content (required for automation)

## Support

- 📧 Email: support@rainbow-browser.ai
- 📖 Full Guide: [USER_GUIDE.md](USER_GUIDE.md)
- 🐛 Issues: [GitHub Issues](https://github.com/rainbow-city/browser-ai/issues)

---

**Enjoy your AI-powered browsing experience!** 🌈✨