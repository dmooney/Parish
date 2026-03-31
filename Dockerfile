# Stage 1: Build the Svelte frontend
FROM node:22-slim AS frontend
WORKDIR /app/ui
COPY ui/package.json ui/package-lock.json ./
RUN npm ci
COPY ui/ ./
RUN npm run build

# Stage 2: Build the Rust backend
FROM rust:1.87-bookworm AS backend
RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config libssl-dev && \
    rm -rf /var/lib/apt/lists/*
WORKDIR /app

# Copy workspace manifests for dependency caching
COPY Cargo.toml Cargo.lock ./
COPY crates/parish-core/Cargo.toml crates/parish-core/Cargo.toml
COPY crates/parish-server/Cargo.toml crates/parish-server/Cargo.toml
COPY src-tauri/Cargo.toml src-tauri/Cargo.toml

# Create minimal stubs so cargo can resolve the workspace and cache deps
RUN mkdir -p src crates/parish-core/src crates/parish-server/src src-tauri/src \
    && echo "fn main() {}" > src/main.rs \
    && echo "pub fn stub() {}" > src/lib.rs \
    && echo "pub fn stub() {}" > crates/parish-core/src/lib.rs \
    && echo "pub fn stub() {}" > crates/parish-server/src/lib.rs \
    && echo "pub fn stub() {}" > src-tauri/src/lib.rs \
    && echo "fn main() {}" > src-tauri/src/main.rs \
    && mkdir -p src/bin/geo_tool && echo "fn main() {}" > src/bin/geo_tool/main.rs \
    && cargo build --release -p parish 2>/dev/null || true

# Copy real source code and build
COPY src/ src/
COPY crates/ crates/
# Touch files so cargo detects changes vs the stubs
RUN touch src/main.rs crates/parish-core/src/lib.rs crates/parish-server/src/lib.rs
RUN cargo build --release --bin parish

# Stage 3: Minimal runtime image
FROM debian:bookworm-slim AS runtime
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates libssl3 && \
    rm -rf /var/lib/apt/lists/*
WORKDIR /app

COPY --from=backend /app/target/release/parish ./parish
COPY --from=frontend /app/ui/dist ./ui/dist
COPY data/ ./data/

EXPOSE 8080

ENV PARISH_PROVIDER=openrouter
ENV RUST_LOG=parish=info

CMD ["./parish", "--web", "8080"]
