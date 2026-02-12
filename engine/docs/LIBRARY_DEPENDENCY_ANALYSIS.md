# Library dependency analysis: baml-runtime, baml-types, internal-baml-core

This document analyses dependency use for **baml-runtime**, **baml-types**, and **internal-baml-core** with the goal of enabling *minimal* library consumption (e.g. CFFI cdylib, type-only consumers) without pulling the full CLI/VM/LLM stack. It also covers semver-friendly version constraints.

## Current consumption model

- **Published library surface**: The `baml` crate (languages/rust/baml) depends on **baml-sys**, which uses **dynamic loading** (libloading) of the engine. End users do **not** compile baml-runtime/baml-types/internal-baml-core; they get a prebuilt engine binary.
- **In-repo library consumers** of these crates:
  - **language_client_cffi** (cdylib): baml-runtime (default-features = false, features = ["internal"]), baml-types, internal-baml-core.
  - **language_client_python / typescript / ruby**: same trio.
  - **generators** (all languages): baml-types, internal-baml-core (not baml-runtime).
  - **cli, language_server, playground-server, baml-vm, baml-schema-wasm**: full stack.

So the “minimal library” use case is: **CFFI / Python / TS / Ruby clients and codegen** that need types + runtime glue, without CLI, LSP, or optional heavy features (AWS, tracing, etc.).

---

## 1. baml-types

**Role**: Shared type definitions (BamlValue, IR types, generator enums, expr, tracing types).

**Current dependencies** (all required in default build):

| Dependency | Used for | Minimal? |
|------------|----------|----------|
| anyhow | Error context | Yes (or thiserror only) |
| clap | `ValueEnum` for `GeneratorOutputType` (CLI) | **No** – CLI only |
| derive_builder | In Cargo.toml; **no usage found in src** | **Remove or optional** |
| internal-baml-diagnostics | `Span` in expr.rs | Optional if we gate or re-export a minimal span |
| itertools | join, Itertools in expr + ir_type | Could be optional or replaced with std |
| log | Logging | Optional for types-only |
| minijinja | `JinjaExpression`, `From<BamlValue> for minijinja::Value` | Optional (template features) |
| once_cell, pretty, secrecy, serde, serde_json, strum, time, tracing-core, web-time, tokio, uuid, baml-ids | Core types, IDs, serde | **Yes** (core) |
| indexmap | Optional via feature `stable_sort` | Already optional |

**Recommendations**

- Add a **`minimal`** (or **`library`**) feature that:
  - Makes **clap** optional (gate `impl ValueEnum for GeneratorOutputType` behind `feature = "cli"`).
  - Makes **minijinja** optional (gate `minijinja` module and `From<BamlValue> for minijinja::Value` behind `feature = "minijinja"`).
  - Makes **internal-baml-diagnostics** optional (e.g. `feature = "diagnostics"`; provide a minimal `Span`-like type or re-export when disabled).
  - Makes **pretty** optional (gate `RcDoc` / debug formatting in baml_value behind `feature = "pretty"` or leave as default).
  - Removes **derive_builder** if unused; otherwise make optional.
- Keep **default** feature set compatible with current CLI/generators (so existing dependants unchanged).
- **Semver**: Use caret ranges for minor/patch where possible, e.g. `time = "0.3"`, `uuid = "1"`, `serde = "1"`, `serde_json = "1"`. Keep `baml-ids` path or version in sync.

---

## 2. internal-baml-core

**Role**: Validation pipeline (parse → AST → parser-database → validation), configuration, IR helpers, generator loading. No LLM calls; used by runtime, CFFI, generators, LSP.

**Current dependencies**: baml-compiler, baml-derive, baml-types, bstd, chrono, derive_builder, either, enumflags2, log, indexmap, internal-baml-ast, internal-baml-diagnostics, internal-llm-client, internal-baml-jinja-types, internal-baml-parser-database, internal-baml-prompt-parser, baml-rpc, minijinja, rayon, regex, semver, serde, serde_json, shellwords, strsim, strum, textwrap, whoami, itertools, once_cell.

**Observations**

- Core validation and configuration are inherently needed by any consumer that “understands” BAML (CFFI, generators, LSP). There is no tiny “types only” slice of this crate; the minimal consumer still needs validation + IR.
- Heavy optional candidates:
  - **internal-llm-client**: Only needed if code paths touch LLM config/build. Could be made optional behind `feature = "llm"`.
  - **rayon**: Used for parallel parsing; could be optional with a single-threaded fallback.
  - **whoami**: Likely for env/UX; could be optional.
- **baml-compiler** is central (HIR, watch); making it optional would require a larger refactor (e.g. separate “validation-only” vs “full compiler” builds).

**Recommendations**

- Introduce a **`minimal`** feature that:
  - Makes **internal-llm-client** optional (if possible without breaking CFFI/generators).
  - Makes **rayon** optional (fallback to `iter` in hot paths).
  - Makes **whoami** optional.
- Prefer **semver** in workspace: e.g. `chrono = "0.4"`, `rayon = "1"`, `semver = "1"`. Keep internal path deps as-is; version them if ever published.

---

## 3. baml-runtime

**Role**: Full runtime: VM/interpreter, LLM clients, HTTP/SSE, AWS/GCP, tracing, CLI, codegen driver, watch. This is the “application” crate, not a thin library.

**Current dependencies**: Very large (baml-compiler, baml-vm, internal-baml-core, internal-llm-client, internal-baml-jinja, generators-lib, generators-openapi, baml-viz-events, plus tokio, reqwest, AWS SDK, tracing, axum, etc.). Many are **cfg(not(wasm32))** (e.g. axum, ring, notify, indicatif, ratatui).

**Observations**

- **language_client_cffi** already uses `default-features = false, features = ["internal"]`. The **default** feature set only adds `skip-integ-tests`; it does **not** trim the dependency tree. So CFFI still compiles the full runtime (including baml-compiler, VM, LLM, etc.).
- A true “minimal library” build would require a **feature that disables most of the crate** and only keeps:
  - Types and interfaces needed by the C FFI (e.g. runtime handle, type builder, request/response types).
  - Possibly a stub or “external” mode where execution is delegated to an external process/prebuilt binary (similar to how the published `baml` crate uses baml-sys + dynamic lib).

**Recommendations**

- Add a **`library`** (or **`minimal`**) feature that:
  - Makes optional: **baml-compiler**, **baml-vm**, **generators-lib**, **generators-openapi**, **internal-llm-client**, **internal-baml-jinja**, **baml-viz-events**, and all **non-wasm** binary-only deps (axum, criterion, notify-debouncer-full, ring, indicatif, ratatui, reedline, etc.) where possible.
  - Keeps: **baml-types**, **baml-ids**, **baml-rpc**, **baml-log**, **internal-baml-core** (or a minimal subset if we add minimal feature there), **serde**, **serde_json**, **tokio** (minimal features), and the minimal runtime interface used by CFFI.
- This implies **conditional compilation** in `lib.rs` and many modules (e.g. no `baml_compiler`, `generators_lib`, or LLM code when `library` is on). Design so that CFFI/Python/TS/Ruby only enable the modules they need.
- **Semver**: Broaden workspace deps to caret where safe (e.g. `tokio = "1"`, `serde = "1"`). Keep AWS SDK and similar pinned where the comments indicate compatibility issues (e.g. ring vs aws_lc_rs).

---

## 4. Semver and version strategy

- **Workspace `[workspace.dependencies]`**: Prefer **caret** for minor/patch compatibility, e.g.  
  `serde = "1"`, `serde_json = "1"`, `time = "0.3"`, `uuid = "1"`, `tokio = "1"`, `anyhow = "1"`, `thiserror = "2"`.  
  This keeps the tree flexible for library consumers.
- **Pinned deps**: Keep exact pins only where necessary (e.g. AWS SDK, wasm-bindgen, or deps with known breakage). Document the reason (e.g. in Cargo.toml or this doc).
- **Internal crates**: baml-ids, baml-rpc, internal-baml-* are path dependencies; when/if published, use proper semver and re-export from a single “baml” or “baml-core” facade if desired.

---

## 5. Summary table

| Crate | Minimal feature idea | Main optional deps | Semver |
|-------|----------------------|--------------------|--------|
| **baml-types** | `minimal` or `library` | clap, minijinja, internal-baml-diagnostics, pretty; remove unused derive_builder | Caret for serde, time, uuid, etc. |
| **internal-baml-core** | `minimal` | internal-llm-client, rayon, whoami | Caret for chrono, semver, rayon |
| **baml-runtime** | `library` / `minimal` | baml-compiler, baml-vm, generators-*, internal-llm-client, internal-baml-jinja, viz-events, and all binary-only (axum, ratatui, etc.) | Caret where safe; keep pins for AWS/wasm where documented |

Implementing the **baml-types** minimal feature is the lowest-risk and immediately reduces the tree for any consumer that only needs types (e.g. generated code or thin wrappers). **baml-runtime**’s library feature is the highest impact but requires careful conditional compilation and testing (CFFI, Python, TS, Ruby).

---

## 6. Minimal feature surface (baml-rt-quickjs / baml-rt-builder)

The following is the **exact public API surface** required by consumers such as **baml-rt-quickjs** and **baml-rt-builder**. All paths below are public and stable for library use.

### 6.1 baml-runtime

| Surface | Path | Purpose |
|--------|------|--------|
| **BamlRuntime** | `baml_runtime::BamlRuntime` | Load IL, run functions, build_request, extract signatures |
| **BamlRuntime::from_directory** | `BamlRuntime::from_directory` | Load runtime from baml_src |
| **BamlRuntime::build_request** | `BamlRuntime::build_request` (async), `build_request_sync` | Build HTTP request for LLM interception |
| **FunctionResultStream** | `baml_runtime::FunctionResultStream` | Streaming function results |
| **RuntimeContextManager** | `baml_runtime::RuntimeContextManager` | Context for build_request |
| **TripWire** | `baml_runtime::TripWire` | Cancellation |
| **FunctionResult** | `baml_runtime::FunctionResult` | Callback for function results |
| **Collector** | `baml_runtime::tracingv2::storage::storage::Collector` | LLM call trace collection |
| **LLMCall** | `baml_runtime::tracingv2::storage::storage::LLMCall` | LLM call details and post-hoc interception |

Re-exports: `FunctionResultStream`, `RuntimeContextManager`, `FunctionResult`, `TripWire` come from `pub use types::*` in `lib.rs`. `Collector` and `LLMCall` are in `tracingv2::storage::storage` (use full path).

### 6.2 baml-types

| Surface | Path | Purpose |
|--------|------|--------|
| **BamlMap** | `baml_types::BamlMap` | Function params (JSON to BAML value map) |
| **BamlValue** | `baml_types::BamlValue` | BAML value representation |
| **HTTPRequest** | `baml_types::tracing::events::HTTPRequest` | HTTP request from build_request for LLM interception |
| **LiteralValue** | `baml_types::ir_type::LiteralValue` | IR type information for TypeScript generation |
| **TypeNonStreaming** | `baml_types::ir_type::TypeNonStreaming` | IR type (non-streaming) |
| **TypeValue** | `baml_types::TypeValue` (re-exported from ir_type) | Primitive type enum |
| **UnionTypeViewGeneric** | `baml_types::ir_type::UnionTypeViewGeneric` | Union view for IR to TS generation |

### 6.3 internal-baml-core

| Surface | Path | Purpose |
|--------|------|--------|
| **FeatureFlags** | `internal_baml_core::feature_flags::FeatureFlags` | Flags for from_directory / runtime init |
| **IRSignature** | `internal_baml_core::ir::ir_hasher::IRSignature` | IR signature for TS generation |
| **FunctionSignature** | `internal_baml_core::ir::ir_hasher::FunctionSignature` | Function signature for TS generation |

### 6.4 Consumer crates (e.g. baml-rt-core)

If a crate (e.g. **baml-rt-core**) declares **baml-runtime**, **baml-types**, and **internal-baml-core** but does **not** use them in its source, those dependencies are unused and should be **removed** to keep the dependency tree smaller. Only depend on the crates that provide the types/symbols you actually use (see tables above).

### 6.5 Consuming this fork as a git dependency (minimal-runtime)

To depend on this BAML fork with a **minimal runtime** surface (no CLI, no codegen), use:

```toml
[dependencies]
baml-runtime = { git = "https://github.com/YOUR_ORG/baml", default-features = false, features = ["minimal-runtime"] }
# If you also need the "internal" API (e.g. InternalRuntimeInterface):
# baml-runtime = { git = "...", default-features = false, features = ["minimal-runtime", "internal"] }
```

With `default-features = false` and `features = ["minimal-runtime"]`, the following are **not** included:

- **cli** (clap, axum, ratatui, indicatif, reedline, notify-debouncer-full, walkdir, which, dirs, crossterm, **minijinja**, **colored**, **pretty**, etc.)
- **codegen** (generators-lib, generators-openapi)
- **baml-viz-events**
- **colored** and **pretty** (terminal styling and RcDoc-based control-flow rendering; minimal uses plain strings and `Debug` for expressions)
- **env_logger** (moved to dev-dependencies; only used in tests)

**Brutal trim (minimal surface only):** When `cli` is off, the full `test_constraints` module (which uses minijinja for evaluating test checks) is not compiled. A stub provides `TestConstraintsResult` and `evaluate_test_constraints` (no-op returning empty), so `run_test` / `run_expr_test` still compile and run but do not evaluate constraints. This keeps **minijinja** out of the minimal dependency tree. A **style** module provides colored-vs-plain output: with `colored` feature, terminal colors; without, plain strings. Control-flow expression rendering uses **pretty** (RcDoc) when enabled, or `format!("{:?}", expr)` when not.

**internal-baml-core:** Optional **rayon** (feature `rayon`, on by default) for parallel parsing in `validate()`; use `default-features = false` for sequential-only. The unused **whoami** dependency was removed.

The runtime still includes: VM/interpreter, LLM clients, `from_directory`, `build_request`, streaming, tracing storage (Collector, LLMCall), and all types needed for the surface in §6.1–6.3.
