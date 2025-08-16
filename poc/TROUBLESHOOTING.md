# 🔧 RainbowBrowserAI Troubleshooting Guide

Common issues and solutions for RainbowBrowserAI PoC.

## 🚨 Common Error Messages

### "OpenAI API key not configured"

**Error**: `{"error":"OpenAI API key not configured","code":503}`

**Cause**: No OpenAI API key is set in the environment.

**Solutions**:

1. **Set API key in environment**:
   ```bash
   export OPENAI_API_KEY=sk-your-key-here
   cargo run --release -- serve
   ```

2. **Use interactive setup**:
   ```bash
   ./setup_env.sh
   source .env
   ```

3. **Enable mock mode for testing**:
   ```bash
   export RAINBOW_MOCK_MODE=true
   cargo run --release -- serve
   ```

4. **Configure in dashboard**:
   - Go to Settings tab
   - Enter your API key
   - Save settings

---

### "Failed to connect to ChromeDriver"

**Error**: `{"error":"Internal server error","details":"Failed to connect to ChromeDriver after retries","code":500}`

**Cause**: ChromeDriver is not running or not accessible.

**Solutions**:

1. **Start ChromeDriver**:
   ```bash
   # Default port 9515
   chromedriver &
   
   # Custom port
   chromedriver --port=9516 &
   ```

2. **Install ChromeDriver**:
   ```bash
   # macOS
   brew install chromedriver
   
   # Ubuntu/Debian
   sudo apt-get install chromium-chromedriver
   
   # Manual install
   # Download from https://chromedriver.chromium.org/
   ```

3. **Check ChromeDriver status**:
   ```bash
   # Check if running
   ps aux | grep chromedriver
   
   # Test connection
   curl http://localhost:9515/status
   ```

4. **Configure custom port**:
   ```bash
   export CHROMEDRIVER_PORT=9516
   ```

---

### "Address already in use"

**Error**: `Address already in use (os error 98)`

**Cause**: Another server is running on the same port.

**Solutions**:

1. **Use different port**:
   ```bash
   cargo run --release -- serve --port 3001
   ```

2. **Kill existing processes**:
   ```bash
   # Find process using port
   lsof -i :3000
   
   # Kill specific process
   kill PID
   
   # Kill all rainbow processes
   pkill -f rainbow-poc
   ```

3. **Check what's using the port**:
   ```bash
   netstat -tlnp | grep 3000
   ```

---

### "Daily budget exceeded"

**Error**: `Cannot afford LLM operation: $0.0050`

**Cause**: Cost tracking has reached the daily limit.

**Solutions**:

1. **Check cost report**:
   ```bash
   cargo run --release -- report
   ```

2. **Increase budget**:
   ```bash
   export RAINBOW_DAILY_BUDGET=10.0
   ```

3. **Reset cost tracker** (new day):
   ```bash
   rm cost_tracker.json
   ```

---

### "Invalid OpenAI API key"

**Error**: `{"error":"Invalid OpenAI API key","code":401}`

**Cause**: API key is incorrect, expired, or has no credits.

**Solutions**:

1. **Verify API key**:
   - Check https://platform.openai.com/api-keys
   - Ensure key starts with `sk-`
   - Check billing and usage limits

2. **Test API key manually**:
   ```bash
   curl https://api.openai.com/v1/models \
     -H "Authorization: Bearer $OPENAI_API_KEY"
   ```

3. **Generate new key**:
   - Go to OpenAI Platform
   - Create new API key
   - Update environment

---

## 🔧 Installation Issues

### Rust Compilation Errors

**Issue**: Build fails with dependency errors.

**Solutions**:

1. **Update Rust**:
   ```bash
   rustup update
   ```

2. **Clear cache**:
   ```bash
   cargo clean
   cargo build --release
   ```

3. **Check Rust version**:
   ```bash
   rustc --version  # Should be 1.70+
   ```

### Missing Dependencies

**Issue**: System dependencies not found.

**Solutions**:

1. **Install build tools**:
   ```bash
   # Ubuntu/Debian
   sudo apt-get update
   sudo apt-get install build-essential pkg-config libssl-dev
   
   # macOS
   xcode-select --install
   ```

2. **Install Chrome/Chromium**:
   ```bash
   # Ubuntu/Debian
   sudo apt-get install chromium-browser
   
   # macOS
   brew install --cask google-chrome
   ```

---

## 🌐 Network Issues

### Proxy/Firewall Blocking

**Issue**: Cannot reach OpenAI API or download dependencies.

**Solutions**:

1. **Configure proxy**:
   ```bash
   export HTTP_PROXY=http://proxy:port
   export HTTPS_PROXY=http://proxy:port
   ```

2. **Bypass proxy for local**:
   ```bash
   export NO_PROXY=localhost,127.0.0.1
   ```

3. **Use alternative registry**:
   ```bash
   # For Rust dependencies
   export CARGO_NET_GIT_FETCH_WITH_CLI=true
   ```

### SSL Certificate Issues

**Issue**: SSL verification errors.

**Solutions**:

1. **Update certificates**:
   ```bash
   # Ubuntu/Debian
   sudo apt-get update && sudo apt-get install ca-certificates
   
   # macOS
   brew install ca-certificates
   ```

2. **Temporary bypass** (not recommended):
   ```bash
   export CARGO_NET_GIT_FETCH_WITH_CLI=true
   ```

---

## 📊 Performance Issues

### Slow Response Times

**Issue**: API responses taking >5 seconds.

**Solutions**:

1. **Check system resources**:
   ```bash
   htop
   free -h
   df -h
   ```

2. **Enable performance logging**:
   ```bash
   RUST_LOG=rainbow_poc=debug cargo run -- serve
   ```

3. **Optimize browser pool**:
   - Increase pool size in config
   - Reduce idle timeout
   - Use headless mode

### High Memory Usage

**Issue**: Process using >1GB RAM.

**Solutions**:

1. **Monitor browser instances**:
   ```bash
   # Check browser processes
   ps aux | grep chrome
   ```

2. **Limit concurrent operations**:
   - Reduce browser pool size
   - Implement request queuing
   - Clear browser sessions regularly

3. **Enable cleanup**:
   ```bash
   # Regular cleanup
   curl -X POST http://localhost:3000/session \
     -d '{"action":"cleanup"}'
   ```

---

## 🔍 Debugging

### Enable Debug Logging

```bash
# Full debug output
RUST_LOG=rainbow_poc=debug cargo run -- serve

# Specific modules
RUST_LOG=rainbow_poc::api=debug,rainbow_poc::llm_service=info cargo run -- serve

# Save logs to file
RUST_LOG=debug cargo run -- serve 2>&1 | tee debug.log
```

### Test Individual Components

```bash
# Test browser automation only
cargo run --release -- navigate "https://example.com" --screenshot

# Test natural language (with API key)
cargo run --release -- ask "navigate to github"

# Test workflow execution
cargo run --release -- workflow examples/workflows/simple.yaml
```

### API Endpoint Testing

```bash
# Health check
curl http://localhost:3000/health

# Test with mock mode
RAINBOW_MOCK_MODE=true cargo run -- serve &
curl -X POST http://localhost:3000/command \
  -H "Content-Type: application/json" \
  -d '{"command":"test command"}'
```

---

## 📋 Configuration Checklist

Before reporting issues, verify:

- [ ] Rust 1.70+ installed
- [ ] ChromeDriver installed and running
- [ ] OpenAI API key set (or mock mode enabled)
- [ ] Required ports (3000, 9515) available
- [ ] Network connectivity to api.openai.com
- [ ] Sufficient disk space and memory
- [ ] Environment variables loaded (`source .env`)

---

## 🆘 Getting Help

### Collect Debug Information

```bash
# System info
uname -a
rustc --version
cargo --version

# Process info
ps aux | grep rainbow
ps aux | grep chrome

# Network info
netstat -tlnp | grep -E "(3000|9515)"

# Log recent errors
tail -n 50 debug.log
```

### Report Issues

Include in your issue report:
1. Error message (exact text)
2. Steps to reproduce
3. System information
4. Debug logs
5. Configuration used

### Links

- [GitHub Issues](https://github.com/yourusername/RainbowBrowserAI/issues)
- [Quick Start Guide](QUICKSTART.md)
- [API Documentation](../docs/API.md)
- [Development Progress](../DEVELOPMENT_PROGRESS.md)

---

**Pro Tip**: Use mock mode (`RAINBOW_MOCK_MODE=true`) to test the system without OpenAI API key during development!