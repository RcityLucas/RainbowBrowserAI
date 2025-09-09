# Contributing to RainbowBrowserAI

Thank you for your interest in contributing to RainbowBrowserAI! This document provides guidelines and instructions for contributing to the project.

## 🚀 Getting Started

### Development Environment Setup

1. **Prerequisites**
   ```bash
   # Install Rust 1.75+
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   
   # Install Chrome/Chromium
   # Download from: https://www.google.com/chrome/
   
   # Install ChromeDriver (optional, auto-managed)
   # Download from: https://chromedriver.chromium.org/
   ```

2. **Fork and Clone**
   ```bash
   git clone https://github.com/your-username/RainbowBrowserAI.git
   cd RainbowBrowserAI
   ```

3. **Install Dependencies**
   ```bash
   cd poc-chromiumoxide
   cargo build
   ```

### Development Workflow

1. **Create a Feature Branch**
   ```bash
   git checkout -b feature/your-feature-name
   ```

2. **Make Your Changes**
   - Follow the coding standards below
   - Add tests for new functionality
   - Update documentation as needed

3. **Test Your Changes**
   ```bash
   # Run all tests
   cargo test --all-features
   
   # Run formatting
   cargo fmt
   
   # Run linter
   cargo clippy -- -D warnings
   
   # Test the application
   RAINBOW_MOCK_MODE=true cargo run -- serve --port 3001
   ```

4. **Commit and Push**
   ```bash
   git add .
   git commit -m "feat: add your feature description"
   git push origin feature/your-feature-name
   ```

5. **Create a Pull Request**
   - Go to GitHub and create a pull request
   - Provide a clear description of your changes
   - Link any related issues

## 📋 Contribution Guidelines

### Code Standards

1. **Rust Style**
   - Follow `cargo fmt` formatting
   - Pass all `cargo clippy` checks
   - Use meaningful variable and function names
   - Add doc comments for public APIs

2. **Error Handling**
   ```rust
   // ✅ Good: Specific error types
   pub enum PerceptionError {
       SessionNotFound(String),
       AnalysisFailed(String),
   }
   
   // ❌ Avoid: Generic errors in public APIs
   fn analyze() -> Result<(), Box<dyn Error>>
   ```

3. **Async/Await**
   ```rust
   // ✅ Good: Consistent async patterns
   pub async fn analyze_page(&mut self) -> Result<PageAnalysis> {
       // Implementation
   }
   
   // ✅ Good: Proper error propagation
   match self.browser.navigate(url).await {
       Ok(_) => Ok(analysis),
       Err(e) => Err(PerceptionError::NavigationFailed(e.to_string())),
   }
   ```

4. **Session Management**
   ```rust
   // ✅ Good: Always check session validity
   if let Some(session) = self.session_manager.get_session(&id).await {
       // Use session
   } else {
       return Err(PerceptionError::SessionNotFound(id));
   }
   ```

### Testing Requirements

1. **Unit Tests**
   ```rust
   #[cfg(test)]
   mod tests {
       use super::*;
       
       #[tokio::test]
       async fn test_session_creation() {
           // Test implementation
       }
   }
   ```

2. **Integration Tests**
   - Place in `tests/` directory
   - Test complete workflows
   - Use mock mode for CI/CD

3. **API Tests**
   ```bash
   # Test specific endpoints
   ./scripts/test-api-endpoints.sh
   ```

### Documentation

1. **Code Documentation**
   - Add doc comments for all public APIs
   - Include usage examples
   - Document error conditions

2. **README Updates**
   - Update feature lists when adding capabilities
   - Add new API endpoints to documentation
   - Update configuration options

3. **Architecture Documentation**
   - Update `docs/ARCHITECTURE.md` for structural changes
   - Document new perception modes or tools
   - Explain design decisions

## 🎯 Types of Contributions

### 🐛 Bug Fixes
- Search existing issues first
- Provide clear reproduction steps
- Include test cases that demonstrate the fix

### ✨ New Features
- Discuss large features in issues first
- Follow the layered perception architecture
- Maintain backward compatibility
- Add comprehensive tests

### 📚 Documentation
- Fix typos and improve clarity
- Add examples and use cases
- Translate documentation (future)

### 🔧 Performance Improvements
- Provide benchmarks showing improvement
- Ensure changes don't break existing functionality
- Document any trade-offs

## 🌟 Feature Development Guidelines

### Perception System Extensions

When adding new perception capabilities:

1. **Follow the Layered Pattern**
   ```rust
   // Add to appropriate perception layer
   impl LayeredPerception {
       pub async fn your_new_analysis(&mut self) -> Result<YourAnalysis> {
           // Implementation following performance targets
       }
   }
   ```

2. **Session Awareness**
   ```rust
   // Always support session-based operation
   async fn your_handler(
       State(state): State<AppState>,
       Json(req): Json<YourRequest>,
   ) -> impl IntoResponse {
       if let Some(session_id) = req.session_id {
           // Use session browser
       } else {
           // Fallback behavior
       }
   }
   ```

### API Endpoint Guidelines

1. **Consistent Response Format**
   ```rust
   #[derive(Serialize)]
   struct ApiResponse<T> {
       success: bool,
       data: Option<T>,
       error: Option<String>,
   }
   ```

2. **Error Handling**
   ```rust
   match operation().await {
       Ok(result) => Json(ApiResponse::success(result)).into_response(),
       Err(e) => (
           StatusCode::INTERNAL_SERVER_ERROR,
           Json(ApiResponse::<()>::error(e.to_string()))
       ).into_response(),
   }
   ```

## 🛠️ Development Tools

### Useful Commands

```bash
# Development server with debug logging
RUST_LOG=debug cargo run -- serve --port 3001

# Mock mode for testing without browser
RAINBOW_MOCK_MODE=true cargo run -- serve --port 3001

# Run specific tests
cargo test perception_tests

# Format and lint
cargo fmt && cargo clippy

# Build documentation
cargo doc --open

# Profile performance
cargo build --release
./target/release/rainbow-poc-chromiumoxide --benchmark
```

### VS Code Setup

Recommended extensions:
- `rust-analyzer`
- `CodeLLDB` (debugging)
- `Better TOML`
- `REST Client` (API testing)

## 📝 Commit Message Guidelines

Follow conventional commits format:

```
<type>(<scope>): <description>

[optional body]

[optional footer]
```

**Types:**
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation changes
- `style`: Code formatting
- `refactor`: Code refactoring
- `test`: Adding tests
- `chore`: Maintenance tasks

**Examples:**
```
feat(perception): add adaptive mode selection
fix(api): handle invalid session IDs correctly
docs(readme): update installation instructions
refactor(browser): optimize session management
test(integration): add session-aware perception tests
```

## 🔍 Code Review Process

### Before Submitting

- [ ] All tests pass
- [ ] Code follows style guidelines
- [ ] Documentation is updated
- [ ] No breaking changes (or clearly marked)
- [ ] Commit messages follow conventions

### Review Criteria

Reviewers will check:
1. **Functionality**: Does it work as intended?
2. **Design**: Is it well-architected?
3. **Performance**: Any performance implications?
4. **Security**: Any security concerns?
5. **Documentation**: Is it well-documented?
6. **Tests**: Are there adequate tests?

## 🚀 Release Process

### Version Management

We follow semantic versioning:
- **Major** (1.0.0): Breaking changes
- **Minor** (0.1.0): New features, backward compatible
- **Patch** (0.0.1): Bug fixes

### Release Checklist

- [ ] Update version in `Cargo.toml`
- [ ] Update `CHANGELOG.md`
- [ ] Run full test suite
- [ ] Build and test release binary
- [ ] Create GitHub release
- [ ] Update documentation

## 🤝 Community Guidelines

### Be Respectful
- Use welcoming and inclusive language
- Respect different viewpoints and experiences
- Accept constructive criticism gracefully

### Be Collaborative
- Help others learn and contribute
- Share knowledge and best practices
- Provide constructive feedback

### Be Patient
- Understand that contributors have different skill levels
- Take time to explain complex concepts
- Be patient with the review process

## 🆘 Getting Help

- **Issues**: Create GitHub issues for bugs and feature requests
- **Discussions**: Use GitHub Discussions for questions
- **Documentation**: Check the `docs/` directory
- **Examples**: Look at the `examples/` directory

## 📞 Contact

For urgent matters or security issues, contact the maintainers directly.

---

Thank you for contributing to RainbowBrowserAI! Your contributions help make browser automation more intelligent and accessible. 🌈