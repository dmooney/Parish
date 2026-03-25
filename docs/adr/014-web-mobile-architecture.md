# ADR-014: Web & Mobile Architecture

> Back to [ADR Index](README.md) | [Docs Index](../index.md)

## Status

Accepted (2026-03-23), **revised 2026-03-25** to reflect Tauri GUI migration (ADR-016)

## Context

Parish now runs as a Tauri 2 desktop app with a Svelte 5 + TypeScript frontend (ADR-016, Phase 8). The game engine lives in `parish-core` as a reusable Rust library, and the Tauri backend (`src-tauri/`) bridges it to the Svelte frontend via typed IPC commands and events.

To reach players on **web browsers** and **mobile devices** (iOS/Android), we need to:

1. Serve the same Svelte frontend to browsers, backed by a remote game server instead of local Tauri IPC.
2. Build native mobile apps using Tauri v2's iOS/Android targets, reusing the same frontend.
3. Avoid maintaining separate frontends — one `ui/` codebase should serve all platforms.

Key constraints:
- The Svelte frontend already renders the full game UI (map, chat, sidebar, theme) using Tauri IPC.
- LLM inference for remote players must use cloud providers (ADR-013, ADR-017).
- NPC simulation and persistence are computationally significant and must run server-side for web play.
- Tauri v2 natively supports iOS and Android, making mobile a low-cost extension.

## Decision

Adopt a **unified frontend with a transport abstraction** across three deployment targets:

### Transport Abstraction

The Svelte frontend communicates through a `GameTransport` interface (`ui/src/lib/transport.ts`) with two implementations:

| Implementation | Backend | Used By |
|---|---|---|
| `TauriTransport` | Tauri IPC (`invoke`/`listen`) | Desktop, mobile-local |
| `WebSocketTransport` | axum WebSocket server | Web browser, mobile-remote |

At runtime, the frontend detects whether it's running inside a Tauri webview (via `window.__TAURI_INTERNALS__`) and selects the appropriate transport. All Svelte components, stores, and event handlers are transport-agnostic.

### Game Server

An **axum** HTTP/WebSocket server (`crates/parish-server/`) hosts game sessions for web and remote-mobile clients:

- Each connection gets an isolated `GameSession` backed by `parish-core`
- The WebSocket protocol mirrors the Tauri IPC contract: JSON command/response pairs for `invoke()`, and server-pushed events for `listen()`
- Cloud LLM providers handle inference (extending ADR-013/ADR-017)
- SQLite persistence per session (extending Phase 4)
- The server also serves the Svelte production build as static files

### Mobile

Tauri v2 mobile targets (iOS/Android) embed the same Svelte frontend and Rust backend:

- **Local mode** (default): `parish-core` runs on-device, same as desktop. Requires a capable device for LLM inference or a configured cloud provider.
- **Remote mode** (optional): Frontend connects to the game server via `WebSocketTransport`. Allows play on low-powered devices.
- Touch-optimized responsive layout via CSS media queries.

### Web

The Svelte frontend is built as a standalone SPA (no Tauri runtime) and served by the game server or a CDN. The `WebSocketTransport` connects to the server's `/ws` endpoint.

## Consequences

- **Code reuse**: 100% of the Svelte frontend is shared across desktop, web, and mobile. No parallel UI codebases.
- **Transport abstraction cost**: A thin interface layer (~100 LOC TypeScript) added to the frontend. Low complexity.
- **Server cost**: Cloud hosting + cloud LLM inference for web/remote-mobile players. Desktop and mobile-local remain free.
- **Latency**: WebSocket adds network round-trip latency for web players. Mitigated by token streaming and optimistic UI updates.
- **Mobile: two modes**: Supporting both local and remote play on mobile adds configuration complexity, but provides flexibility.
- **Shared command dispatch refactor**: The game command logic in `src-tauri/src/commands.rs` must be extracted into `parish-core` so both Tauri and the server can invoke it. This is the most significant refactoring cost.
- **No WASM game engine**: Unlike the original plan, the game engine does not compile to WASM. Web play requires the game server. This is simpler and avoids WASM limitations (no SQLite, no Ollama, no filesystem).

## Alternatives Considered

1. **egui compiled to WASM** (original ADR-014): The pre-Phase 8 plan. Rejected because egui is no longer in the stack; maintaining two frontends (Svelte for desktop, egui-WASM for web) would be wasteful.
2. **Separate mobile frontend**: Build mobile UI in Swift/Kotlin. Rejected — Tauri v2 runs the same Svelte frontend natively, eliminating duplication.
3. **parish-core compiled to WASM for browser**: Eliminates the server for web play. Rejected — `parish-core` depends on SQLite, filesystem, and Tokio features that don't work well in WASM. Cloud inference requires network access regardless.
4. **Dioxus or Leptos (Rust WASM frontend)**: Replace Svelte with a Rust web framework. Rejected — Svelte is already built and working (ADR-016); rewriting the frontend in Rust would be regression.
5. **WebSocket-only (drop Tauri IPC)**: Make all clients use WebSocket, including desktop. Rejected — Tauri IPC is zero-latency for local play and doesn't require a running server process.

## Related

- [Plan: Phase 7 — Web & Mobile Apps](../plans/phase-7-web-mobile.md)
- [ADR-016: Replace egui with Tauri 2 + Svelte GUI](016-tauri-svelte-gui.md) — the migration that motivates this revision
- [ADR-013: Cloud LLM for Player Dialogue](013-cloud-llm-dialogue.md) — cloud inference for remote players
- [ADR-017: Per-Category Inference Providers](017-per-category-inference-providers.md) — provider routing used by the server
