# Plan: Phase 7 — Web & Mobile Apps

> Parent: [Roadmap](../requirements/roadmap.md) | [Docs Index](../index.md)
>
> **Status: Planned**
>
> **Supersedes**: The original version of this plan (pre-Phase 8) assumed an egui/WASM web
> client and a separate Tauri mobile wrapper. Phase 8's migration to Tauri 2 + Svelte
> (ADR-016) fundamentally changes the approach: the existing Svelte frontend is now the
> single UI codebase for desktop, web, and mobile.

## Goal

Deliver Parish as a playable game in web browsers and on mobile devices (iOS/Android) by reusing the existing Svelte 5 frontend. A cloud-hosted game server (axum + WebSocket) replaces the Tauri IPC bridge for web and remote-mobile play. Tauri v2's native mobile targets provide iOS and Android apps. The same `ui/` codebase serves all three platforms with a thin transport abstraction.

## Architecture Overview

```
┌──────────────────────────────────────────────────────────────────┐
│                     Cloud Game Server                            │
│  ┌──────────┐  ┌───────────────┐  ┌───────────────────────────┐ │
│  │  axum    │  │ parish-core   │  │ Cloud LLM (OpenRouter /   │ │
│  │  HTTP +  │←→│ GameSession   │←→│ Anthropic / OpenAI)       │ │
│  │  WS      │  │ per player    │  │                           │ │
│  └────┬─────┘  └───────────────┘  └───────────────────────────┘ │
│       │        ┌───────────────┐                                 │
│       │        │ SQLite        │                                 │
│       │        │ Persistence   │                                 │
│       │        └───────────────┘                                 │
└───────┼──────────────────────────────────────────────────────────┘
        │ WebSocket (JSON — mirrors Tauri IPC contract)
        │
   ┌────┴───────────────────────────────────────────────┐
   │          Svelte 5 Frontend (single codebase)       │
   │          ui/src/ — components, stores, lib          │
   ├─────────────┬─────────────────┬────────────────────┤
   │  Desktop    │  Web            │  Mobile             │
   │  Tauri IPC  │  WebSocket      │  Tauri IPC          │
   │  (local)    │  (remote)       │  (local or remote)  │
   │  src-tauri/ │  Vite static    │  Tauri v2 mobile    │
   │             │  deploy         │  iOS + Android      │
   └─────────────┴─────────────────┴────────────────────┘
```

### Key Design Decisions

1. **One frontend, three transports**: The Svelte `ui/` codebase is shared across desktop, web, and mobile. Only the transport layer differs: Tauri `invoke()`/`listen()` for desktop and mobile-local; WebSocket for web and mobile-remote.
2. **Transport abstraction in `ui/src/lib/ipc.ts`**: Replace direct `@tauri-apps/api` imports with a `GameTransport` interface. At runtime, detect the environment (Tauri webview vs. browser) and inject the correct implementation.
3. **Server mirrors Tauri IPC**: The axum WebSocket server exposes the same commands and events as `src-tauri/src/commands.rs` and `events.rs`. The Svelte frontend is unaware of which backend it talks to.
4. **Mobile: local-first, remote-optional**: Tauri v2 mobile apps embed the Rust game engine (parish-core) for offline play with local inference. An optional "cloud mode" connects to the game server via WebSocket instead.
5. **Cloud-only web**: Browser clients always connect to the game server. No WASM game engine in the browser.

## Prerequisites

- Phase 8 complete: Tauri desktop GUI with Svelte frontend (done)
- Phase 4 complete: persistence layer for server-side session management
- ADR-013 / ADR-017: cloud LLM and per-category provider routing

## Tasks

### Part A: Transport Abstraction Layer

1. **Define `GameTransport` interface in `ui/src/lib/transport.ts`**
   - Interface with methods mirroring current `ipc.ts` exports:
     ```typescript
     interface GameTransport {
       getWorldSnapshot(): Promise<WorldSnapshot>;
       getMap(): Promise<MapData>;
       getNpcsHere(): Promise<NpcInfo[]>;
       getTheme(): Promise<ThemePalette>;
       submitInput(text: string): Promise<void>;
       onStreamToken(cb: (p: StreamTokenPayload) => void): Promise<UnlistenFn>;
       onStreamEnd(cb: (p: StreamEndPayload) => void): Promise<UnlistenFn>;
       onTextLog(cb: (p: TextLogPayload) => void): Promise<UnlistenFn>;
       onWorldUpdate(cb: (p: WorldUpdatePayload) => void): Promise<UnlistenFn>;
       onLoading(cb: (p: LoadingPayload) => void): Promise<UnlistenFn>;
       onThemeUpdate(cb: (p: ThemePalette) => void): Promise<UnlistenFn>;
     }
     ```
   - Export `UnlistenFn` type alias: `() => void`

2. **Implement `TauriTransport` in `ui/src/lib/transport-tauri.ts`**
   - Wraps existing `@tauri-apps/api/core` `invoke()` and `@tauri-apps/api/event` `listen()` calls
   - Identical to current `ipc.ts` logic, just implementing the `GameTransport` interface
   - Used for desktop and mobile-local mode

3. **Implement `WebSocketTransport` in `ui/src/lib/transport-ws.ts`**
   - Connects to game server via `WebSocket` (native browser API)
   - Command requests: send JSON `{ type: "command", name: "get_world_snapshot", args: {} }`, await response with matching request ID
   - Event subscriptions: server pushes events as `{ type: "event", name: "stream-token", payload: {...} }`, transport dispatches to registered callbacks
   - Reconnection: exponential backoff (1s, 2s, 4s, 8s), resend handshake on reconnect
   - Session token: stored in `localStorage`, sent on connect for session resumption

4. **Implement transport auto-detection in `ui/src/lib/transport.ts`**
   - `createTransport(): GameTransport` factory function
   - Detection: check for `window.__TAURI_INTERNALS__` (present in Tauri webview, absent in browser)
   - Tauri detected → return `TauriTransport`
   - Browser detected → read server URL from `import.meta.env.VITE_WS_URL` or default to `ws://${location.host}/ws`, return `WebSocketTransport`
   - Export a Svelte-friendly singleton via a module-level `transport` variable

5. **Migrate all components to use `GameTransport`**
   - Replace all direct imports from `ipc.ts` with the `transport` singleton
   - `App.svelte`: initialize transport on mount, pass to stores
   - `game.ts` store: accept transport as parameter, use it for all commands/events
   - `InputField.svelte`: `transport.submitInput(text)` instead of `submitInput(text)`
   - All event listeners: `transport.onStreamToken(cb)` etc.
   - Verify: desktop Tauri app works identically after migration (no behavioral change)

### Part B: Game Server (Rust)

6. **Create `parish-server` crate in `crates/parish-server/`**
   - New workspace member alongside `parish-core`
   - Dependencies: `parish-core`, `axum`, `axum-extra`, `tower`, `tower-http`, `tokio`, `serde`, `serde_json`, `tracing`
   - Exposes: `async fn run_server(config: ServerConfig) -> Result<()>`

7. **Define server-side protocol in `crates/parish-server/src/protocol.rs`**
   - Reuse the same type names as `src-tauri/src/lib.rs` (`WorldSnapshot`, `MapData`, `NpcInfo`, `ThemePalette`, etc.) by importing from `parish-core` or re-defining with `Serialize + Deserialize`
   - `ClientMessage` enum: `{ type: "command", name: String, args: Value, request_id: String }` | `{ type: "ping" }`
   - `ServerMessage` enum: `{ type: "response", request_id: String, data: Value }` | `{ type: "event", name: String, payload: Value }` | `{ type: "pong" }`
   - This mirrors the Tauri `invoke`/`listen` contract over WebSocket

8. **Implement `GameSession` in `crates/parish-server/src/session.rs`**
   - Wraps `parish-core` types: `WorldState`, `NpcManager`, `GameClock`, `InferenceClients`
   - `async fn process_command(&mut self, name: &str, args: Value) -> Result<Value>` — dispatches to the same logic as `src-tauri/src/commands.rs`
   - `fn subscribe_events(&self) -> broadcast::Receiver<ServerMessage>` — receives game events (world-update, theme-update, stream tokens)
   - Reuses the inference queue, NPC tick loop, and time advance from `parish-core`
   - The command dispatch logic is extracted from `src-tauri/src/commands.rs` into a shared function in `parish-core` so both Tauri and server can use it

9. **Implement the axum server in `crates/parish-server/src/server.rs`**
   - `async fn run_server(config: ServerConfig)` — binds to `0.0.0.0:{port}`
   - Route `GET /ws` — WebSocket upgrade → `handle_ws_connection`
   - Route `GET /` — serves Svelte static files from `ui/build/` (Vite production build)
   - Route `GET /health` — `200 OK` with session count
   - CORS middleware via `tower-http::cors` (allow configured origins)
   - `ServerConfig`: `port: u16`, `static_dir: PathBuf`, `max_sessions: usize`, `cors_origins: Vec<String>`

10. **Implement WebSocket handler in `crates/parish-server/src/ws.rs`**
    - `async fn handle_ws_connection(ws: WebSocket, state: Arc<SessionManager>)`
    - On connect: check for session token in first message, resume or create `GameSession`
    - Read loop: deserialize `ClientMessage`, dispatch commands, send `ServerMessage` responses
    - Write loop: forward game events from `broadcast::Receiver` to WebSocket
    - Heartbeat: `Ping`/`Pong` every 15s, disconnect on 30s timeout
    - Token streaming: `stream-token` events forwarded in real time as server events

11. **Implement session management in `crates/parish-server/src/manager.rs`**
    - `SessionManager`: `HashMap<SessionId, Arc<Mutex<GameSession>>>`
    - `create_session(player_name: &str) -> (SessionId, SessionToken)` — creates game session, returns opaque token
    - `resume_session(token: &str) -> Option<SessionId>` — looks up existing session
    - Idle timeout: background task reaps sessions inactive for 30 minutes (saves to SQLite first)
    - Memory cap: `max_sessions` limit, reject new connections when full

12. **Add `--server` CLI mode in `src/main.rs`**
    - `cargo run -- --server` starts the game server (no Tauri window)
    - `--server-port <PORT>` (default 8080)
    - `--server-static <DIR>` (default `ui/build/`)
    - Imports and calls `parish_server::run_server()`

### Part C: Web Deployment

13. **Configure Vite for standalone web builds**
    - `ui/vite.config.ts`: add `build.outDir` for web target (`ui/build/`)
    - Conditional Tauri plugin: only include `@tauri-apps/vite-plugin` when `TAURI_ENV` is set
    - `npm run build:web` script: builds without Tauri plugin → pure static SPA
    - Output: `index.html` + JS/CSS bundles, no Tauri runtime dependency

14. **Environment-based configuration**
    - `ui/.env.production` (Tauri): no `VITE_WS_URL` → transport auto-detects Tauri
    - `ui/.env.web` (browser): `VITE_WS_URL=wss://parish.example.com/ws`
    - Build scripts: `npm run build:web -- --mode web` uses `.env.web`
    - The `WebSocketTransport` reads `VITE_WS_URL` at runtime

15. **Static hosting and server integration**
    - Game server serves `ui/build/` at `/` via `tower-http::services::ServeDir`
    - Alternative: deploy `ui/build/` to CDN (Cloudflare Pages, Netlify) pointing to game server `/ws`
    - `justfile` recipe: `just build-web` → `cd ui && npm run build:web`
    - Cache headers: immutable hashed assets, `index.html` no-cache

### Part D: Mobile Client (Tauri v2)

16. **Initialize Tauri mobile targets**
    - `cd src-tauri && cargo tauri ios init` → generates Xcode project in `src-tauri/gen/apple/`
    - `cd src-tauri && cargo tauri android init` → generates Gradle project in `src-tauri/gen/android/`
    - Update `src-tauri/tauri.conf.json`: add `bundle.iOS` and `bundle.android` sections
    - Bundle ID: `com.parish.app`

17. **Mobile-specific Svelte layout adaptations**
    - Add CSS media queries in `ui/src/app.css` for narrow screens (`max-width: 768px`):
      - Stack layout vertically: status bar → chat panel → map panel (collapsed) → input
      - Sidebar becomes a slide-out drawer (swipe from right edge)
      - Map panel: collapsed by default, expandable via toggle button
    - `MapPanel.svelte`: increase SVG node tap targets to 44px minimum (Apple HIG)
    - `InputField.svelte`: handle virtual keyboard — scroll chat to bottom on focus, dismiss on send
    - Detect mobile via `navigator.maxTouchPoints > 0` or Tauri platform API

18. **Mobile app lifecycle handling**
    - `src-tauri/src/lib.rs`: register Tauri lifecycle hooks
    - `on_window_event(WindowEvent::CloseRequested)` → save game state
    - Mobile resume: re-emit `world-update` and `theme-update` events to resync frontend
    - Background timer: pause game clock when app is backgrounded, resume on foreground
    - For remote-mode (WebSocket): disconnect on background, reconnect on foreground with session token

19. **Mobile build and distribution**
    - iOS: `cargo tauri ios build` → `.ipa` for TestFlight
    - Android: `cargo tauri android build` → `.apk`/`.aab` for Play Store
    - CI: GitHub Actions workflow with `tauri-action` for cross-platform builds
    - Signing: environment secrets for iOS certificates and Android keystore

### Part E: Server Infrastructure

20. **Authentication and session tokens**
    - Server generates opaque session token (UUID v4) on first connect
    - Client stores token in `localStorage` (web) or Tauri `Store` plugin (mobile)
    - Reconnect sends token → server resumes existing `GameSession`
    - No user accounts in Phase 7 — anonymous play with optional player name
    - Rate limiting: 60 commands/minute per session (tower middleware)

21. **Server deployment**
    - `Dockerfile`: multi-stage build
      - Stage 1: `node:22-alpine` → `cd ui && npm ci && npm run build:web`
      - Stage 2: `rust:1.82-alpine` → `cargo build --release -p parish-server`
      - Stage 3: `alpine:3.20` → copy binary + `ui/build/` + SQLite
    - `docker-compose.yml`: server + named volume for SQLite data
    - Environment: `PARISH_CLOUD_API_KEY`, `PARISH_CLOUD_MODEL`, `PARISH_SERVER_PORT`, `PARISH_MAX_SESSIONS`, `PARISH_CORS_ORIGINS`
    - Health check: `/health` endpoint for container orchestrators

22. **Monitoring and observability**
    - `tracing` JSON logs to stdout (structured, 12-factor compatible)
    - Metrics: active sessions, WebSocket connections, inference requests/sec, p50/p95 latency
    - Optional: `metrics` crate with Prometheus exporter at `/metrics`
    - Per-session rate limiting via `tower::limit::RateLimitLayer`

## New Dependencies

| Crate / Package | Purpose | Used In |
|-----------------|---------|---------|
| `axum` | HTTP/WebSocket server | `parish-server` |
| `axum-extra` | TypedHeader, WebSocket utils | `parish-server` |
| `tower` | Middleware (rate limiting) | `parish-server` |
| `tower-http` | Static file serving, CORS | `parish-server` |
| `uuid` | Session token generation | `parish-server` |

No new frontend npm packages required — `WebSocket` is a native browser API.

## Workspace Structure (After Phase 7)

```
Parish/
├── Cargo.toml                  # Workspace root
├── crates/
│   ├── parish-core/            # Game logic library (exists)
│   └── parish-server/          # axum game server (new)
│       ├── Cargo.toml
│       └── src/
│           ├── lib.rs          # run_server() entry point
│           ├── protocol.rs     # ClientMessage / ServerMessage
│           ├── session.rs      # GameSession wrapper
│           ├── server.rs       # axum routes + WebSocket upgrade
│           ├── ws.rs           # Per-connection WebSocket handler
│           └── manager.rs      # SessionManager (lifecycle, limits)
├── src/                        # CLI binary (TUI, headless, server)
├── src-tauri/                  # Tauri 2 backend (desktop + mobile)
│   └── gen/
│       ├── apple/              # iOS Xcode project (new)
│       └── android/            # Android Gradle project (new)
├── ui/                         # Svelte 5 frontend (shared)
│   └── src/lib/
│       ├── transport.ts        # GameTransport interface + factory (new)
│       ├── transport-tauri.ts  # Tauri IPC implementation (new)
│       ├── transport-ws.ts     # WebSocket implementation (new)
│       ├── ipc.ts              # Deprecated → re-exports from transport
│       └── types.ts            # Shared TypeScript types (unchanged)
└── Dockerfile                  # Server container (new)
```

## Implementation Order

1. **Tasks 1–5** (Transport abstraction): Foundational — unblocks web without breaking desktop
2. **Tasks 6–12** (Game server): Server-side game engine + WebSocket protocol
3. **Tasks 13–15** (Web deployment): Vite web build + static hosting
4. **Tasks 16–19** (Mobile): Tauri v2 mobile targets + responsive UI
5. **Tasks 20–22** (Infrastructure): Auth, Docker, monitoring

## Testing Strategy

- **Transport abstraction**: Unit tests for `WebSocketTransport` with mock WebSocket; verify `TauriTransport` passes existing Vitest suite unchanged
- **Protocol**: Rust unit tests for `ClientMessage`/`ServerMessage` serialization round-trips
- **GameSession**: Reuse `GameTestHarness` patterns with mock inference against the server session
- **Server integration**: `axum::test::TestServer` — connect WebSocket, send commands, verify events
- **Web E2E**: Playwright against `just serve-web` (server + built frontend)
- **Mobile**: Manual device testing + Tauri's `cargo tauri ios dev` / `cargo tauri android dev`
- **Load testing**: `k6` WebSocket script simulating 50 concurrent sessions

## Open Questions

1. **Single-player or multiplayer?** — This plan assumes isolated single-player sessions. Shared-world multiplayer is a future phase.
2. **Mobile: local vs. remote default?** — Local-first (offline) gives the best experience but requires a capable device for LLM inference. Remote-first is simpler. Recommendation: default to remote with cloud inference; local-only as an advanced option.
3. **App store content policy** — AI-generated content may require disclosure for App Store / Play Store review. Need to evaluate Apple's and Google's current policies.
4. **Shared command dispatch** — Task 8 requires extracting command dispatch from `src-tauri/src/commands.rs` into `parish-core` so both Tauri and the server can use it. This is the deepest refactor in the plan.
5. **Mobile bundle size** — Tauri mobile apps embed the full Rust game engine + parish-core. Need to measure `.ipa`/`.apk` sizes and optimize if needed (`lto = true`, `strip = true`).
