# Asset Value Tracker

Local, all-Rust app to track owned assets and their resale value over time. WASM
frontend (Leptos) + native server (axum) sharing one `core` crate.

## Prerequisites
- Rust stable, the `wasm32-unknown-unknown` target, `trunk`, and `wasm-bindgen-cli`:
  ```bash
  rustup target add wasm32-unknown-unknown
  cargo install trunk wasm-bindgen-cli
  ```

## Build & run
```bash
# 1. Build the frontend bundle into ./dist (run from the web crate dir)
cd crates/web && trunk build && cd ../..

# 2. Run the server (serves ./dist + the API on http://127.0.0.1:4000)
cargo run -p tracker-server
# open http://127.0.0.1:4000
```

### Dev mode (hot reload)
```bash
# terminal A — backend
cargo run -p tracker-server
# terminal B — frontend dev server (proxies /api to the backend)
cd crates/web && trunk serve --open
```

## Data
`data.json` (gitignored) is the source of truth. Copy `data.example.json` to start:
```bash
cp data.example.json data.json
```
Override location/port with `ASSET_TRACKER_DATA`, `ASSET_TRACKER_DIST`, `ASSET_TRACKER_PORT`.

## Tests
```bash
cargo test            # tracker-core + tracker-server (native)
```

## Refreshing valuations
Values are filled in by Claude on demand — see `RESEARCH.md`.
