# Repository Guidelines

## Project Structure & Module Organization
- `src/` — Rust sources: `api/` (Axum handlers), `browser/` (chromiumoxide pool), `perception/`, `tools/`, `coordination/`, `llm/`, `intelligence/`, `action/`, `main.rs`.
- `static/` — Dashboard assets (`index.html`, `app.js`, `styles.css`).
- `scripts/` — Dev/test utilities (smoke tests, helpers).
- `tests/` and top‑level `test_*.sh` — integration/smoke scripts.
- `examples/` — usage samples and client demos.

## Build, Test, and Development Commands
- `cargo build --release` — optimized build.
- `./start.sh [--port 3001] [--headless] [--no-browser] [--cargo-run]` — build and run server; opens dashboard unless `--no-browser`.
- `cargo run --release -- serve --port 3001 [--headless]` — run via Cargo (alternative to `start.sh`).
- `curl -s http://localhost:3001/api/health` — health check.
- `cargo test` — run unit tests.
- `scripts/test_navigate_perceive.sh <port> <url>` — smoke test (navigate → perceive).
- Example (legacy combined): `curl -X POST http://localhost:<port>/api/perception/analyze -H 'Content-Type: application/json' -d '{"url":"https://example.com"}'`.

## Coding Style & Naming Conventions
- Format: `cargo fmt --all`; Lint: `cargo clippy -- -D warnings`.
- Indentation: 4 spaces for Rust; match surrounding style in JS/HTML.
- Naming: `snake_case` modules/files/functions; `UpperCamelCase` types; `SCREAMING_SNAKE_CASE` consts; HTTP routes use `kebab-case`.

## Testing Guidelines
- Co-locate unit tests where practical; run with `cargo test`.
- Prefer targeted smoke scripts in `scripts/` for API flows.
- Name tests by behavior (clear intent, include edge cases).

## Commit & Pull Request Guidelines
- Commits: short, imperative subject (≤72 chars). Optional scope prefix, e.g., `api: add navigate-perceive endpoint`. Group related changes only.
- PRs: explain rationale and testing notes (commands + sample responses). Link issues; screenshots/gifs for UI changes. Ensure `cargo fmt`, `cargo clippy`, and smoke tests pass.

## Security & Configuration Tips
- Server binds to `127.0.0.1` and retries nearby ports; do not expose publicly.
- CI: prefer `./start.sh --headless`.
- Useful env vars: `RAINBOW_TOOL_TIMEOUT_SECS`, `RAINBOW_NAV_TIMEOUT_SECS`.

## Architecture Overview
- Axum API ↔ Browser Pool (chromiumoxide) ↔ Tools Registry ↔ Perception Engine.
- When adding endpoints, register in both routers (normal + legacy) and update `static/app.js`, `scripts/` smoke tests, and `/api/routes`.

