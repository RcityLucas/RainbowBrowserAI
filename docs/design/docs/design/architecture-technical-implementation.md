**Architecture Summary**
- Services & Engines
  - unified-kernel: session/state/health/resource/tab coordination; event bus; adaptive orchestration.
  - layered-perception: 3-mode external API backed by internal staged pipelines; adaptive scheduler.
  - intelligent-action: tool execution engine with planner, queue, limiter, retry/verification.
  - optimized-persistence: repository abstraction; default multi‑model store (e.g., SurrealDB); compression + indices + cache.
  - performance/stability engines: metrics/tracing collection; health checks; fault detection/recovery.

**Workspace Layout (Rust)**
- Crates
  - `unified-kernel/` (lifecycle, event bus, schedulers)
  - `layered-perception/` (fast/standard/deep, adaptive)
  - `intelligent-action/` (executor, locator, verification, retry, concurrency)
  - `optimized-persistence/` (storage, query optimizer, cache, ER model)
  - `shared-types/` (sessions, perception, actions, executions, errors, metrics)
  - Optional `web/` for Leptos UI (observability/control panel)
- Dependencies
  - Async: `tokio`
  - Web/API: `axum` + `tower`
  - Browser control: driver abstraction (e.g., Fantoccini/Playwright behind trait)
  - DB: `surrealdb`
  - Serde: `serde` + `bincode`
  - Perf/conc: `rayon`, `dashmap`
  - WASM: `wasm-bindgen`, `web-sys` (where applicable)
  - Telemetry: `tracing`, `tracing-subscriber`

**Key Interfaces (Traits) & Contracts**
- Perception
  - `trait PerceptionEngine { async fn perceive(&self, ctx: PerceptionCtx) -> Result<PerceptionResult>; }`
  - `enum PerceptionMode { Fast, Standard, Deep, Adaptive }` with SLA enforcement in middleware.
- Action
  - `trait ToolExecutor { async fn execute(&self, call: ToolCall) -> Result<ToolResult>; }`
  - Planner selects strategy: `Sequential | Parallel | Intelligent`; scheduler enforces limits.
- Persistence
  - `trait Repository { async fn store_session(..); async fn store_perception(..); async fn store_execution(..); ... }`
  - `CacheLayer` in front; `QueryOptimizer` routes lookups; compression for snapshots.
- Kernel
  - `trait ModuleInterface { get_state; handle_request; subscribe_events; shutdown }`
  - `EventBus` with envelope, filters, and async subscribers; module health states exposed.

**External API (OpenAPI 3.1 alignment)**
- Endpoints
  - `POST /sessions` → create; returns Session
  - `GET /sessions/{id}` → session state + perf/decision (when enabled)
  - `DELETE /sessions/{id}` → graceful close
  - `POST /sessions/{id}/perception` → IntelligentPerceptionResult
  - `POST /sessions/{id}/tools` → start execution; returns execution_id and plan
  - `GET /sessions/{id}/executions/{execId}` → status/results/metrics/errors
  - `GET /health` → health snapshot and alerts
  - `GET /metrics` → Prometheus exposition (text/plain)
  - `GET /version` → version and compatibility info
- Security: Bearer token; per-session RBAC claims; rate limiting per token.

**Data Models (Schema‑First)**
- Align to project JSON Schemas under `schemas/json/`; generate typed models for Rust, TS, Python where needed.
- Wire Types
  - SessionConfig/Session, PerceptionMode, IntelligentPerceptionResult(Fast/Standard/Deep), StandardTool/ExecutionOptions, ExecutionStatus, HealthMetrics, PerformanceSummary, ErrorInfo.
- Validation
  - Request validation via JSON schema and serde; response conformance tests.

**Core Flows**
- Adaptive Perception Selection
  - Inputs: page signals (ready, layout stability), task hints, perf preferences, historical metrics.
  - Policy: choose lowest sufficient mode to meet accuracy; escalate on ambiguity; degrade under pressure.
  - Telemetry: decision factors with weights and confidence (when tracing on).
- Tool Execution Pipeline
  - Intake → Validate → Plan (dependencies, potential parallelization) → Queue → Acquire budget → Execute
  - Verification: post‑action checks; re‑perceive if required; smart retry with backoff/jitter.
  - Concurrency: thread pool + async tasks; per-session and global limiters; cooperative cancellation.
- Persistence Strategy
  - Perception: compress + index by session/time/url; store shallow fast_data separately for quick recall.
  - Executions: append‑only events; aggregate summaries for dashboard; TTL for verbose traces.
  - Caching: two‑tier (in‑proc + optional Redis) with invalidation on navigation and DOM mutation.

**Performance & Stability Tactics**
- SLA Guards
  - Time budgeters around perception; early return with partials; adaptive sampling in low energy mode.
- Backpressure
  - Queue caps; shed low‑priority tasks; degrade deep→standard under high load.
- Zero‑copy Paths
  - Share immutable page snapshots via Arc; binary serde for hot paths.
- Health & Recovery
  - Liveness/readiness; self‑healing (restart tabs, recycle sessions); detect flapping and quarantine.

**Error Handling & Taxonomy**
- Categories: `timeout | not_found | conflict | precondition_failed | execution_error | resource_exhausted | internal`.
- Policies
  - Retryable: network errors, transient timeouts; max attempts with exponential backoff + jitter.
  - Recoverable: element not found → re‑perceive; stale element → refresh locator; dynamic UI → wait/retry.
  - Irrecoverable: auth failures, invalid parameters; fast‑fail with guidance.

**Security**
- AuthZ
  - Bearer tokens; per-session isolation; optional scopes for perception/action/history.
- Data Protection
  - Sensitive fields redacted in logs; PII minimization; encryption at rest (DB capability) and TLS in transit.
- Abuse Controls
  - Rate limits; execution quotas; sandbox policies for actions.

**Observability**
- Tracing
  - Spans: session, perception, execution, tool, DB call; trace ids returned when decision trace enabled.
- Metrics (Prometheus)
  - `organism_perception_duration_seconds{mode}` histogram
  - `tool_success_rate{tool}` gauge / counters
  - `execution_queue_depth`, `cache_hit_rate`, `cpu_usage_percent`, `memory_usage_mb`
- Logs
  - Structured, leveled, correlation with trace ids; PII‑aware redaction.

**Deployment**
- Container
  - Multi‑stage build; runtime image with browser engine (e.g., Chromium) and CA certs.
  - Env flags: `PERCEPTION_LEVEL=adaptive`, `MEMORY_SHARING=enabled`.
- Kubernetes
  - StatefulSet or Deployment; resources: CPU limits to stabilize latency; persistent volume for history/snapshots where needed.
  - Probes: readiness on core services; liveness with recovery.
  - HPA: scale on `execution_queue_depth` and latency percentiles.

**Testing Strategy**
- Unit
  - Perception mode SLA tests (clock‑bounded); tool parameter validation; repository contract fakes.
- Integration (E2E)
  - Full workflow on controlled pages; verify perception→action→persistence; version/migration tests.
- Performance
  - Latency budgets per mode; throughput under concurrency; tool success P95/P99.
- Chaos
  - Induced timeouts, navigation stalls, DOM mutations; verify recovery and tails.

**Compatibility & Migration**
- Fresh start: no legacy behavior required. Provide semantic versioning from day one and clear deprecation policy.

**Risks & Mitigations**
- Risk: Divergence between external 3‑tier and internal 4‑tier sensing.
  - Mitigation: Strict adapter at API boundary; conformance tests; feature flag to phase out internal Quick path.
- Risk: Headless engine variability (Chromium/Firefox/WebKit).
  - Mitigation: Abstraction layer; compliance suite per engine; per‑engine capability matrix.
- Risk: Storage hot spots and cost.
  - Mitigation: Snapshot compression; TTLs; tiered storage; indices; cache read‑through.

**Next Steps**
- Finalize OpenAPI spec and publish under `docs/api/openapi.yaml`.
- Codegen clients (TS/Python/Rust) from schemas and OpenAPI.
- Land crate skeletons and shared types; implement perception Fast path and minimal tools (MVP).
- Stand up CI for schema conformance and SLA tests.
