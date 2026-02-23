# KAMI - Progress Tracker

## Current State

### Phase 0: Foundations - COMPLETE

| Module | Status | Notes |
|--------|--------|-------|
| **Workspace** | Done | 11 crates, workspace deps, rust-toolchain |
| **kami-types** | Done | ToolId, ToolManifest, Capability, KamiError, DomainEvent |
| **kami-protocol** | Done | JSON-RPC 2.0, MCP tools/prompts/resources, schema validation |
| **kami-registry** | Done | ToolRepository trait, ToolQuery |
| **kami-config** | Done | Layered config (defaults + TOML + env), KamiConfig schema |
| **kami-guest** | Done | ABI helpers, kami_tool! macro, ToolMetadata |
| **WIT** | Done | world.wit, tool.wit, host.wit |

### Phase 1: Minimal Engine - COMPLETE

| Module | Status | Notes |
|--------|--------|-------|
| **kami-engine** | Done | Component Model (wasmtime v27), async instantiate, TypedFunc call, fuel metering |
| **kami-sandbox** | Done | Real WasiCtxBuilder, socket_addr_check network, fs perms, env isolation |
| **kami-runtime** | Done | WasmToolExecutor pipeline (ctx → store → instantiate → call → result) |
| **kami-cli run** | Done | Wired to engine, loads .wasm, executes with deny-all security |

### Phase 2: Isolation & Security - COMPLETE

| Module | Status | Notes |
|--------|--------|-------|
| **kami-types** | Done | `env_allow_list` on SecurityConfig |
| **kami-engine** | Done | StoreLimits in HostState, epoch interruption, store.limiter() |
| **kami-sandbox** | Done | Enhanced CapabilityChecker (env vars), validate_security_config() |
| **kami-runtime** | Done | Full pipeline: validate → sandbox → limits → epoch → timeout → execute |

### Phase 3: Registry & tool.toml - COMPLETE

| Module | Status | Notes |
|--------|--------|-------|
| **kami-types** | Done | `manifest.rs` - tool.toml parser (ToolToml → ToolManifest) |
| **kami-store-sqlite** | Done | Full CRUD, migrations (v1), in-memory support, JSON columns |
| **kami-cli install** | Done | Parse tool.toml, verify .wasm, register in SQLite |
| **kami-cli list** | Done | Query registry with optional name filter, tabular output |
| **kami-cli inspect** | Done | Detailed tool info (security, limits, arguments) |

### Phase 4: Runtime Orchestration - COMPLETE

| Module | Status | Notes |
|--------|--------|-------|
| **kami-runtime/cache** | Done | ComponentCache with LRU eviction, Mutex-based, thread-safe |
| **kami-runtime/resolver** | Done | ToolResolver: registry → WASM load → compile → cache |
| **kami-runtime/scheduler** | Done | Semaphore-based concurrency control, RAII permits |
| **kami-runtime/orchestrator** | Done | KamiRuntime: resolve + schedule + execute pipeline |
| **kami-cli exec** | Done | Execute registered tool by ID via full runtime pipeline |

### Phase 5: MCP Protocol & Transport - COMPLETE

| Module | Status | Notes |
|--------|--------|-------|
| **kami-protocol/initialize** | Done | MCP initialize handshake types, PROTOCOL_VERSION constant |
| **kami-transport-stdio/handler** | Done | McpHandler: dispatch initialize, tools/list, tools/call |
| **kami-transport-stdio/transport** | Done | StdioTransport: generic line-delimited JSON I/O |
| **kami-transport-stdio/server** | Done | McpServer: read → dispatch → write loop |
| **kami-cli serve** | Done | `kami serve` starts MCP server on stdio |

### Phase 6: SDK & Documentation - COMPLETE

| Module | Status | Notes |
|--------|--------|-------|
| **kami-guest/abi** | Done | parse_input, to_output, text_result, error_result, ToolMetadata |
| **kami-guest/macros** | Done | `kami_tool!` declarative macro for handler wiring |
| **docs/ARCHITECTURE.md** | Done | 7 ADRs (hexagonal, security, wasmtime, async, MCP, SDK, cache) |
| **README.md** | Done | Visual GitHub layout, diagrams, crate map, CLI reference |

### Phase 7: Stabilisation - COMPLETE

| Item | Status | Notes |
|------|--------|-------|
| `init.rs` compilation fix | Done | Raw string `r##"..."##` + `__PLACEHOLDER__` template substitution |
| Clean Architecture: parse_tool_manifest | Done | Moved from `kami-types` to `kami-config::manifest_loader` |
| Security: Path Traversal (HIGH) | Done | 3-layer defense in `FsJail::validate_path` |
| Security: IP bypass (HIGH) | Done | `is_addr_allowed(SocketAddr)` in `network.rs` |
| Security: env_allow_list | Done | Filtering wasi_config.env_vars against allow-list |
| Security: SQL LIMIT/OFFSET | Done | Bound parameters via `push(Box::new(limit as i64))` |
| Security: unwrap_or_default | Done | Replaced with `FromSqlConversionFailure` propagation |
| CLI: tokio::main | Done | All commands now `pub async fn execute()`, no `block_on` |
| CLI: shared helpers | Done | `shared::open_repository()` + `shared::create_runtime()` |
| Files > 150 lines | Done | handler.rs split (dispatch modules), repository.rs split (row_mapping + integration tests) |
| MCP notifications/initialized | Done | `handle_notification()` + server fallback parse |
| Dead code removal | Done | `pool.rs` removed, `ToolExecutor` trait properly implemented |
| WASM SHA-256 integrity | Done | `integrity.rs`, `wasm_sha256` field, DB migration v2, `kami verify` command |
| `#[tracing::instrument]` | Done | `executor.rs`, `resolver.rs`, `orchestrator.rs`, `handler.rs` |
| Tests: all missing modules | Done | config/loader, runtime/resolver, runtime/orchestrator |
| CI/CD pipeline | Done | GitHub Actions ci.yml (doc check + coverage) + release.yml (4 targets) |

### Phase 8: Observability & Productisation - IN PROGRESS

| Item | Status | Notes |
|------|--------|-------|
| Execution Metrics | Done | `metrics.rs` with `AtomicU64` counters, `MetricsSnapshot`, integrated into `KamiRuntime` |
| JSON structured logging | Done | `--log-format plain\|json` global flag in `kami-cli` |
| Graceful shutdown | Done | `KamiRuntime::shutdown()` + `Scheduler::drain()` + `tokio::select!` in `kami serve` |
| `kami init` end-to-end | Done | `execute_at(args, base_dir)` for testability; 2 unit tests verify all generated files + correct paths |
| HTTP/SSE transport | Done | `kami-transport-http` crate: axum router, bearer auth, `POST /mcp`, `GET /health`; `kami serve --transport http --port 3000 --token` |
| `kami status` command | Done | Shows tool registry stats (total/enabled/disabled) + runtime config; wired into CLI |
| `kami dev --watch` | Done | `notify 6.1`, debounced rebuild, `commands/dev.rs` (3 tests) |
| Functional examples | Done | `examples/hello-world` + `examples/echo` + `examples/json-transform` + `examples/http-fetch` |
| DiagnosticError trait | Done | `hint()` + `fix()` on EngineError, SandboxError, RuntimeError; wired into `kami exec` |
| Rate Limiter | Done | `rate_limiter.rs` token-bucket, per-tool + global limits, 4 tests |
| Pipeline module | Done | `pipeline.rs` multi-tool chaining, `execute_pipeline()`, 4 tests |

### v0.2 Compliance Audit - COMPLETE

| Item | Status | Notes |
|------|--------|-------|
| Zero `unwrap()`/`expect()` in prod | Done | All instances confirmed in `#[cfg(test)]` only |
| Zero `#[allow(dead_code)]` | Done | Removed from `main.rs` (output module is used) |
| All files ≤ 150 lines | Done | 7 files split: dev.rs, metrics.rs, rate_limiter.rs, capability.rs, repository.rs, integration.rs, repository tests |
| LRU cache eviction | Done | Replaced FIFO with proper LRU (order Vec + touch-on-access) |
| CONTRIBUTING.md | Done | Code standards, commit format, PR checklist |

### Sprint 7.2: Versioning & Update - COMPLETE

| Item | Status | Notes |
|------|--------|-------|
| DB migration v3 | Done | `pinned_version TEXT` + `updated_at TEXT` columns |
| `ToolRepository::update()` | Done | New trait method in registry port |
| SQLite `update()` impl | Done | Full UPDATE SQL for all 12 columns |
| `kami update` command | Done | Update single tool or `--all`, respects pins |
| `kami pin` command | Done | Pin/unpin tool versions |
| `query_builder.rs` extract | Done | Keeps `repository_impl.rs` under 150 lines |
| 3rd example (json-transform) | Done | pick/flatten/count actions, 5 tests |
| MCP round-trip e2e tests | Done | 4 integration tests in `mcp_roundtrip.rs` |
| AI agent integration docs | Done | INTEGRATION.md, GETTING_STARTED.md |
| Community files | Done | Issue templates (bug/feature/security), CODE_OF_CONDUCT.md |

### Build Status
- `cargo build` - PASS
- `cargo test` - **448 tests PASS** (+9 since last commit)
- `cargo clippy --all-targets` - CLEAN (0 warnings)
- `cargo fmt --check` - CLEAN
- `cargo audit` - CLEAN (4 known wasmtime advisories ignored via `.cargo/audit.toml`)
- `cargo tarpaulin` - **71.19% coverage** (1132/1590 lines)
- `wasm32-wasip2` echo tool - COMPILES & EXECUTES E2E
- `wasm32-wasip2` http-fetch tool - COMPILES & EXECUTES E2E (WASI HTTP outgoing)

## Tasks Accomplished

### Session 1 (Phase 0)
- Created full workspace with 11 crates
- Implemented domain layer types (kami-types) with tests
- Implemented protocol types (kami-protocol) with JSON-RPC 2.0 and MCP
- Implemented registry port traits (kami-registry)
- Implemented sandbox capability checker with deny-all defaults
- Set up layered configuration (figment)
- Created CLI with clap derive
- Created WIT interface definitions

### Session 2 (Phase 1)
- Implemented kami-engine with wasmtime v27 Component Model
- Created HostState with WasiView trait implementation
- Implemented async component loading, linking, instantiation
- Solved Canonical ABI for `result<string, string>` (indirect retptr)
- Built real WasiCtxBuilder with socket_addr_check network control
- Created WasmToolExecutor async pipeline in kami-runtime
- Wired CLI `run` command to full execution pipeline
- Created WAT echo component for integration tests

### Session 3 (Phase 2 + Phase 3)
- **Phase 2 - Isolation:**
  - env_allow_list, StoreLimits, epoch interruption, validate_security_config()
  - Full executor pipeline with timeout enforcement
  - CLI --max-memory-mb and --timeout-ms flags
- **Phase 3 - Registry:**
  - tool.toml parser with TOML → ToolManifest mapping
  - SQLite CRUD repository with migrations (PRAGMA user_version)
  - JSON serialization for security config and arguments columns
  - install command: parse manifest, verify wasm, check duplicates, register
  - list command: tabular output, name filter
  - inspect command: full tool details with security and arguments
- 67 tests passing, clippy clean

### Session 4 (Phase 4)
- ComponentCache: thread-safe compiled WASM caching with FIFO eviction
- ToolResolver: registry lookup → WASM file load → compile → cache pipeline
- Scheduler: tokio::sync::Semaphore concurrency control with RAII permits
- KamiRuntime: top-level orchestrator combining resolver + scheduler + executor
- `kami exec` command: execute registered tools by ID via full runtime pipeline
- 73 tests passing, clippy clean

### Session 5 (Phase 5)
- MCP initialize handshake types (InitializeParams/Result, ServerCapabilities)
- McpHandler: JSON-RPC dispatch for initialize, tools/list, tools/call
- StdioTransport: generic async line-delimited JSON reader/writer
- McpServer: main event loop (read → parse → dispatch → respond)
- `kami serve` CLI command to start MCP server on stdio
- inputSchema generation from ToolArgument definitions
- 81 tests passing, clippy clean

### Session 6 (Phase 6)
- kami-guest SDK: ABI helpers (parse_input, to_output, text_result, error_result)
- kami_tool! declarative macro for handler wiring + ToolMetadata
- ARCHITECTURE.md: ADR-005 (MCP stdio), ADR-006 (Guest SDK), ADR-007 (Component Cache)
- README.md: full rewrite with visual diagrams, security model, crate map, CLI reference
- 89 tests passing, clippy clean

### Session 8 (Phase 7: Stabilisation)
- **Sprint 0**: Fixed `init.rs` compilation error — raw string template conflict resolved with `r##"..."##` and `__PLACEHOLDER__` substitution pattern (`templates.rs` module)
- **Sprint 1 – Clean Architecture**: Moved `parse_tool_manifest_file` from `kami-types` (domain) to `kami-config` (adapter), removed `toml` dep from domain layer
- **Sprint 1 – CLI Harmonisation**: All 7 async commands converted from `Runtime::new().block_on()` to `pub async fn execute()` via `#[tokio::main]` on `main()`; `shared.rs` module for `open_repository()` and `create_runtime()` helpers
- **Sprint 2 – Security (4 fixes)**:
  - Path Traversal (HIGH): `FsJail::validate_path` now uses 3-layer defense (absolute/root rejection + `..` component check + `canonicalize()` containment)
  - IP bypass (HIGH): Added `is_addr_allowed(SocketAddr)` — direct IP connections require explicit IP in allow-list, hostname patterns cannot match IP literals
  - `env_allow_list` not enforced: WasiCtxBuilder now filters env vars against the allow-list before building context
  - SQL LIMIT/OFFSET injection: Replaced `format!()` with bound parameters (`Box<dyn ToSql>`)
  - `unwrap_or_default()` on deserialisation: Replaced with `FromSqlConversionFailure` error propagation; added `DataCorruption` variant to `RepositoryError`
- **File splitting** (>150 lines rule): `handler.rs` split into `handler.rs` + `dispatch/{initialize,tools_list,tools_call}.rs`; `repository.rs` split into `repository.rs` + `row_mapping.rs` + `tests/repository.rs`
- **MCP `notifications/initialized`**: `McpHandler::handle_notification()` added; server fallback parses unknown messages as `JsonRpcNotification` (no response sent)
- **Windows portability**: `has_root()` check added alongside `is_absolute()` in `FsJail` (Unix-style `/foo` paths not considered absolute by Rust on Windows)
- **113 tests passing, clippy clean**

### Session 7 (Post-Phase: CLI Enhancements)
- `--input-file` / `-f` flag on `run` and `exec` for JSON file input
- Stdin support via `--input-file -` (pipe JSON from another command)
- JSON validation on file/stdin input before execution
- Shared `input.rs` helper module (resolve_input with 6 tests)
- Documentation updates (DEPLOYMENT.md, DEVELOPER.md, README.md)
- 95 tests passing, clippy clean

## Blockers
- None

## All Phases Complete

All 6 phases of the KAMI roadmap are implemented:
- 11 crates, hexagonal architecture
- Wasmtime v27 Component Model with WASI P2
- Capability-based security with deny-all defaults
- SQLite registry with tool.toml manifests
- Runtime orchestrator with cache + scheduler
- MCP server over stdio (JSON-RPC 2.0)
- Guest SDK with kami_tool! macro

### Session 14 (v0.2 Compliance Audit)
- **150-line rule enforcement**: Split 7 oversized files
  - `dev.rs` (155→109) — tests extracted to `dev_tests.rs`
  - `metrics.rs` (156→81) — tests extracted to `tests/metrics.rs`
  - `rate_limiter.rs` (161→104) — tests extracted to `tests/rate_limiter.rs`
  - `capability.rs` (201→77) — tests extracted to `tests/capability.rs`
  - `repository.rs` (221→45) — trait impl extracted to `repository_impl.rs` (125 lines)
  - `integration.rs` (343 lines) — split into `engine_basic.rs` + `engine_isolation.rs` + `common/mod.rs` shared WAT fixture
  - `tests/repository.rs` (195 lines) — split into `repository_crud.rs` + `repository_query.rs`
- **Dead code**: Removed `#[allow(dead_code)]` from `main.rs` (output module is actively used)
- **LRU cache**: Replaced FIFO eviction in `ComponentCache` with proper LRU tracking (order Vec + touch-on-access, Mutex instead of RwLock); new `lru_evicts_least_recently_used` test
- **CONTRIBUTING.md**: Created with code standards, commit format, PR checklist, architecture overview
- **158 tests passing, clippy clean**

### Session 15 (Sprint 7.2: Versioning & Update + v0.2 Finalization)
- **cargo fmt**: Fixed formatting diffs across entire codebase
- **Sprint 7.2 — Versioning & Update**:
  - Domain: Added `pinned_version` and `updated_at` fields to `Tool` struct (`kami-types`)
  - Port: Added `update()` to `ToolRepository` trait (`kami-registry`)
  - Adapter: DB migration v3 (`pinned_version TEXT`, `updated_at TEXT`), updated `row_to_tool()`, `COLS`, full `update()` impl
  - Extracted `query_builder.rs` from `repository_impl.rs` to stay under 150 lines
  - CLI: `kami update <tool-id>` / `kami update --all` (respects pins, rebuilds tool.toml, recomputes SHA-256)
  - CLI: `kami pin <tool-id> [version]` / `kami pin --unpin <tool-id>`
  - Updated all `Tool {}` constructions (8+ files) with new fields
- **3rd example tool**: `examples/json-transform/` — pick keys, flatten nested objects, count keys (5 tests)
- **End-to-end MCP integration tests**: `mcp_roundtrip.rs` with 4 tests (initialize, tools/list, unknown method error, full CRUD lifecycle)
- **Documentation**:
  - `docs/INTEGRATION.md` — Claude Desktop, Cursor, HTTP transport, LangChain/Python, troubleshooting
  - `docs/GETTING_STARTED.md` — Install, quick start, tool.toml reference, CLI commands, examples
  - `.github/ISSUE_TEMPLATE/` — bug_report.md, feature_request.md, security.md
  - `CODE_OF_CONDUCT.md` — Contributor Covenant 2.1
- **176 tests passing, clippy clean, fmt clean**

### Session 16 (Sprint 3.3 + 3.4: Fuzzing & Benchmarks)
- **cargo audit**: Installed and ran `cargo-audit v0.22.1`
  - 2 low-severity wasmtime advisories (RUSTSEC-2025-0046, RUSTSEC-2025-0118) — mitigated (WASIp2 only, no shared memory)
  - 2 unmaintained transitive deps (paste, fxhash) — wasmtime internal
  - Created `.cargo/audit.toml` with documented ignore list
- **Sprint 3.3 — Property-based fuzzing with proptest**:
  - `kami-sandbox/tests/fuzz_filesystem.rs` (4 tests): arbitrary paths, absolute rejection, `..` rejection, simple relative acceptance
  - `kami-config/tests/fuzz_manifest.rs` (4 tests): arbitrary input no-panic, valid TOML parses, missing sections fail
  - `kami-protocol/tests/fuzz_jsonrpc.rs` (5 tests): arbitrary JSON no-panic, round-trip, missing fields fail
- **Sprint 3.4 — Criterion benchmarks**:
  - `kami-runtime/benches/cache_bench.rs` (4 benchmarks): insert_100, get_hit, get_miss, evict_lru_10
  - Results: insert 100 = ~1.8ms, get hit = ~472ns, get miss = ~141ns, evict 10 = ~374µs
- **4th example tool**: `examples/http-fetch/` — URL validation, scheme check, max_bytes, 6 tests
- **150-line fixes**: Compacted `orchestrator.rs` (154→147) and `tool.rs` (154→149)
- **194 tests passing, clippy clean, fmt clean, audit clean**

### Session 17 (Annexe B v1.0 progress)
- **SECURITY.md**: Root-level responsible disclosure policy (scope, process, timelines)
- **CI benchmarks**: `benchmarks` job in ci.yml with `github-action-benchmark` (regression detection at 150% threshold)
- **wit-bindgen 0.51 integration**:
  - Fixed workspace dep from 0.36 → 0.51 (matches wasmtime 27)
  - `component-model` feature flag on kami-guest (optional, gated)
  - `bindings.rs` module with `wit_bindgen::generate!` and `Guest` trait re-export
  - Comprehensive documentation for tool authors (dual-mode: native testing + WASM component)
- **Test coverage**: 25 new tests across 8 modules
  - `kami-engine/error.rs` (3): error kind mapping, diagnostic hints
  - `kami-engine/memory.rs` (3): MemoryStats edge cases
  - `kami-registry/query.rs` (3): builder pattern, chaining
  - `kami-transport-stdio/error.rs` (3): From conversions
  - `kami-transport-http/error.rs` (2): Display formatting
  - `kami-runtime/error.rs` (3): error mapping, fix suggestions
  - `kami-runtime/types.rs` (2): ExecutionResult Clone + Debug
  - `kami-runtime/runtime_config.rs` (2): Default values, Clone
  - `kami-types/error.rs` (4): constructors, DiagnosticError defaults
  - `kami-types/event.rs` (3 new + 1 replaced): all constructors + serde roundtrip
  - `kami-sandbox/error.rs` (6): all From mappings + diagnostic hints/fixes
  - `kami-runtime/pipeline.rs` tests (2 new in external file): serde roundtrip, defaults
- **Coverage measurement**: cargo-tarpaulin installed & configured, 49.09% (759/1546 lines)
- **219 tests passing, clippy clean, fmt clean, audit clean**

### Session 18 (E2E Demo — Full Pipeline Working)
- **Host-side WIT bindings** (`kami-engine/src/bindings.rs`):
  - `wasmtime::component::bindgen!` generating typed `KamiTool` struct
  - `Host` trait impl on `HostState` for `kami:tool/host@0.1.0` (log function dispatches to `tracing`)
  - `create_tool_linker()` — WASI + host imports in one linker
  - `instantiate_tool()` — typed component instantiation
  - `call_run()` — typed export call via `kami_tool_tool().call_run()`
- **Engine linker upgrade**: `create_linker()` now includes host interface imports alongside WASI
- **Executor dual-path**: `WasmToolExecutor` tries typed `instantiate_tool` first (WIT components), falls back to `instantiate_component` + `call_tool_run` (flat exports for test WAT)
- **WIT fixes**: Removed duplicate `package` doc comments that blocked `bindgen!` parsing
- **Echo tool guest** (`tests/fixtures/echo-tool/`):
  - Standalone Rust project (`cdylib` crate-type)
  - `wit_bindgen::generate!` for guest-side `kami-tool` world
  - Implements `Guest` trait (run echoes input, describe returns metadata JSON)
  - Compiled to `wasm32-wasip2` target (66KB component)
  - Excluded from workspace via `exclude = ["tests/fixtures/echo-tool"]`
- **E2E integration tests** (`kami-runtime/tests/e2e_echo.rs`):
  - `echo_tool_returns_input()` — full pipeline: parse tool.toml → register in SQLite → resolve → compile → instantiate → execute → verify result
  - `echo_tool_with_empty_input()` — empty JSON through same pipeline
  - Both tests use real wasm32-wasip2 component (not hand-crafted WAT)
- **221 tests passing, clippy clean, fmt clean**

### Session 19 (Coverage Campaign: 49% → 71%)
- **Target**: 70%+ test coverage, prioritizing security modules
- **Result**: 49.09% → 71.19% (+22.1%), 221 → 382 tests (+161)
- **Tool Author Guide**: Created `docs/TOOL_AUTHOR_GUIDE.md` (8-step guide for tool developers)
- **Transport tests**:
  - `kami-transport-stdio/tests/server_integration.rs` (6 tests): request loop, notifications, parse errors, EOF shutdown
  - `kami-transport-stdio/tests/tools_list.rs` (3 tests): empty registry, registered tool with inputSchema, disabled filter
  - `kami-transport-http/tests/router_integration.rs` (7 tests): health, parse error, notifications, auth 401/bearer, invalid request
- **Store tests**: `kami-store-sqlite/tests/repository_extended.rs` (6 tests): enabled_only, limit, offset, limit+offset, temp file, sha256
- **Runtime tests**:
  - `pipeline.rs` extended (+4): resolve_unknown, error displays
  - `error_diagnostic.rs` extended (+5): Sandbox/Engine variant coverage
- **Sandbox tests**: `wasi_security.rs` extended (+3): real tempdir preopens for read-only and sandbox modes
- **CLI inline tests** (21 new tests across 8 modules):
  - `validate.rs` (3): valid tool, missing wasm, missing manifest (tempfile-based)
  - `list.rs` (2): empty registry, name filter
  - `uninstall.rs` (2): missing tool, invalid ID
  - `status.rs` (1): empty registry stats
  - `inspect.rs` (3): missing tool, invalid ID, existing tool lifecycle
  - `pin.rs` (3): missing tool, full pin/unpin lifecycle
  - `output.rs` (5): print functions don't panic, default_db_path
  - `shared.rs` (2): open_repository with temp path, default path
- **Visibility fixes**: Made `router.rs` AppState/build_router pub, auth/router modules pub for external testing
- **All files ≤ 150 lines verified**
- **382 tests passing, clippy clean, fmt clean**

### Session 20 (Dead Code Cleanup)
- **Removed dead code** (linker.rs, context.rs, duplicate create_tool_linker, KamiTool re-export)
- **Coverage**: 71.51% (1132/1583 lines) — 380 tests

### Session 21 (Remote Plugin Install & Search)
- **Remote install**: `kami install` now accepts URLs, GitHub shorthand (`owner/repo@tag`), or local paths
- **download.rs** (149 lines): HTTP download via `reqwest`, ZIP extraction with zip-slip prevention, `strip_top_dir` flattening
- **install.rs** enhanced (125 lines): `resolve_source()` detects URL/GitHub/local → delegates to `download_to_plugins()` or local flow
- **search.rs** (122 lines): `kami search <query>` queries a remote JSON registry index, fuzzy matches on name/description/id
- **shared.rs** extended: `data_dir()`, `plugins_dir()`, `dirs_or_fallback()` for `~/.kami/plugins/` storage
- **New dependencies**: `reqwest` (rustls-tls, json, stream), `zip`, `serde` in kami-cli
- **CLI changes**: `Install` arg renamed `path` → `source`; new `Search` subcommand with `--registry` flag
- **389 tests passing, clippy clean, fmt clean**

### Session 22 (Registry à la Homebrew)
- **`kami publish` command** (144 lines): reads `tool.toml`, computes SHA-256, generates `index.json` entry, prints submission instructions
- **Registry template** (`registry/`): `index.json` (3 seed entries), `schema.json` (JSON Schema), `validate.yml` CI workflow
- **CI validation pipeline**: JSON syntax, schema validation, duplicate ID check, source URL reachability
- **391 tests passing, clippy clean, fmt clean**

### Session 24 (Documentation Completion)
- **`GET /health/ready`** readiness probe: new endpoint in `kami-transport-http/src/router.rs` + test `ready_endpoint_returns_ok` in `tests/router_integration.rs` — completes Annexe B v1.0 health probes item
- **Continue.dev integration guide**: new section in `docs/INTEGRATION.md` covering stdio + custom registry config, version requirements
- **Stale placeholder fixes** in `docs/GETTING_STARTED.md`: `your-org` → `Hypijump31`, duplicate step "5." → "6.", added `http-fetch` example to table
- **Test count corrected** in `docs/DEPLOYMENT.md`: `89+` → `402+`
- **403 tests passing**, clippy clean, fmt clean

### Session 25 (Sprint 1.1 + Sprint 1.2 — v1.0 Roadmap)

#### Sprint 1.1 — WASI HTTP Outgoing
- **`wasmtime-wasi-http` integration** in `kami-engine`:
  - Added `WasiHttpCtx` to `HostState` alongside `WasiCtx`; implemented `WasiHttpView` trait
  - `send_request()` override enforces `net_allow_list` — `ConnectionRefused` when host not in list; deny-all when list is empty
  - `add_only_http_to_linker_async()` registered in `create_linker()` for all WASM components
  - `is_http_host_allowed()` supports exact and wildcard (`*.example.com`) patterns
  - `host_state.set_net_allow_list()` wired into `executor.rs` from `SecurityConfig`
- **http-fetch-tool fixture** (`tests/fixtures/http-fetch-tool/`):
  - Real WASM component making HTTP GET via `wasi:http/outgoing-handler@0.2.2`
  - WIT world with full WASI HTTP + io + clocks dependency tree; `generate_all` option for wit-bindgen
  - `tool.toml` with deny-all security defaults
- **E2E tests** (`kami-runtime/tests/e2e_http.rs`, 4 tests):
  - `http_fetch_blocked_by_empty_allow_list` — no network, verifies deny-all
  - `http_fetch_blocked_for_unlisted_host` — partial list blocks unmatched host
  - `http_fetch_allowed_for_listed_host` — local tokio TCP server, verifies real HTTP round-trip
  - `http_fetch_wildcard_allow_list` — IP not matched by `*.localhost` wildcard

#### Sprint 1.2 — Extract McpHandler to `kami-mcp`
- **New crate `kami-mcp`** (APPLICATION layer, 12th crate):
  - `McpHandler` + `JsonRpcOutput` moved from `kami-transport-stdio` (ADAPTER) to `kami-mcp`
  - `dispatch/{initialize, tools_list, tools_call}.rs` — unchanged logic, new home
  - `to_json()` now returns `Result<String, serde_json::Error>` (no crate-specific error type)
  - Fixes architecture violation: ADAPTER (`kami-transport-http`) no longer depends on ADAPTER (`kami-transport-stdio`) for the handler
- **`kami-transport-stdio` simplified**:
  - Removed: `handler.rs`, `dispatch/` module (moved to `kami-mcp`)
  - Added: `kami-mcp` dep; re-exports `McpHandler` and `JsonRpcOutput` for backward compat
  - `server.rs` updated to use `kami_mcp::McpHandler`
- **`kami-transport-http` updated**:
  - Replaced `kami-transport-stdio` dep with `kami-mcp` in both `[dependencies]` and `[dev-dependencies]`
  - `router.rs` and `server.rs` import from `kami_mcp` directly
  - Integration tests updated to `use kami_mcp::McpHandler`
- **439 tests passing, clippy clean**

### Session 23 (Cryptographic Plugin Signatures)
- **Ed25519 plugin signing**: Full cryptographic signature workflow (keygen → sign → verify → enforce at execution)
- **`kami-runtime/signature.rs`** (144 lines): `generate_keypair()`, `sign_file()`, `verify_file_signature()`, `public_key_from_secret()` — 6 unit tests
- **`kami keygen`** (146 lines): Generates Ed25519 keypair to `~/.kami/keys/`, Unix 0o600 perms on secret key, `--force` overwrite — 3 unit tests
- **`kami sign`** (115 lines): Signs WASM plugin, prints signature + public key + instructions — 2 unit tests
- **`kami verify` enhanced** (108 lines): Now checks SHA-256 AND Ed25519 signature, `--public-key` flag (hex or file path)
- **Domain types**: Added `signature` and `signer_public_key` (both `Option<String>`) to `ToolManifest` with serde skip_serializing_if
- **SQLite migration v4**: Added `signature TEXT` and `signer_public_key TEXT` columns
- **Resolver enhanced**: Step 5 verifies Ed25519 signature at execution time (if present on manifest)
- **Registry schema.json**: Added optional `signature` and `signer_public_key` properties (backward compatible)
- **New dependencies**: `ed25519-dalek` v2 (std, rand_core), `rand` v0.8 in workspace
- **15+ test files updated** with new ToolManifest fields (`signature: None, signer_public_key: None`)
- **402 tests passing, clippy clean, fmt clean**

### Session 26 (Code Quality Improvements)
- **Rate Limiter Integration**: Wired `RateLimiter` into `KamiRuntime::execute()` — token-bucket check runs before any expensive work; returns `RuntimeError::RateLimited` with tool_id/limit/window. Added `rate_limit: RateLimitConfig` field to `RuntimeConfig` with sensible defaults.
- **Dead Code Cleanup**: Deleted `pool.rs` (15-line stub). Removed `Priority` enum from `scheduler.rs` (was never used for actual scheduling). Updated lib.rs exports and scheduler tests.
- **MCP Protocol Completeness**: Added `prompts/list`, `resources/list`, `resources/read` dispatch handlers. New files: `prompts_list.rs`, `resources_list.rs`. Server now advertises `tools`, `prompts`, and `resources` capabilities in `initialize` response. Added `PromptsCapability` and `ResourcesCapability` to `kami-protocol`.
- **WIT describe() Export**: Added `call_describe()` in `kami-engine/bindings.rs` to invoke the `describe()` WIT export. Exported from `kami-engine/lib.rs`.
- **Tests**: 9 new tests — 1 rate limiter (orchestrator), 5 MCP dispatch (prompts, resources, initialize capabilities), 3 inline (prompts_list, resources_list). **448 tests passing, 0 failures.**
- **Validation**: `cargo clippy -- -D warnings` clean, `cargo fmt --check` clean, `cargo check` clean.

## Future Enhancements
- [x] CI/CD pipeline (GitHub Actions + cargo-audit + coverage) ✅
- [x] WASM SHA-256 integrity verification at install and exec ✅
- [x] Observability metrics (AtomicU64 counters, MetricsSnapshot) ✅
- [x] JSON structured logging (--log-format flag) ✅
- [x] Graceful shutdown (KamiRuntime::shutdown + Scheduler::drain) ✅
- [x] Binary releases (release.yml — 4 targets) ✅
- [x] LRU cache eviction ✅
- [x] End-to-end integration tests (install → exec → MCP round-trip) ✅
- [x] Benchmarks with `criterion` (Sprint 3.4) ✅
- [x] Fuzzing with `proptest` on security modules (Sprint 3.3) ✅
- [x] `cargo audit` security audit (Annexe A) ✅
- [x] `wit-bindgen` integration in kami-guest ✅
- [x] CI benchmark regression detection ✅
- [x] SECURITY.md responsible disclosure policy ✅
- [x] Real WASM component E2E demo (wasm32-wasip2 echo tool) ✅
- [x] Test coverage > 70% (71.19% — transport, CLI, sandbox, runtime) ✅
- [x] Remote plugin install from URL/GitHub shorthand ✅
- [x] `kami search` command with remote registry index ✅
- [x] Ed25519 cryptographic plugin signatures (keygen, sign, verify) ✅
- [ ] Test coverage > 75% (transport + CLI integration tests)
- [ ] Plugin marketplace exploration (Sprint 7.5)
