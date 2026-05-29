# Asset Value Tracker — Design

**Date:** 2026-05-29
**Status:** Approved (design + stack)

## 1. Purpose

A local, single-user app for tracking items/assets you own and watching their
resale value evolve over time. You log each item once (purchase price, date,
identifying details). When you want fresh values, you ask Claude, which uses the
`research-mcp` tools to look up each item across multiple sources and appends a
dated, sourced valuation snapshot. The app is the dashboard, the data-entry
surface, and the value-over-time charts.

General-purpose: any asset type (electronics, vehicles, collectibles, luxury,
market-priced goods). Valuation is free-text web research per item rather than
category-specific price APIs, so it works for anything.

## 2. Architecture — all Rust, 1:1 via a shared core

A Cargo workspace with three crates. The shared `core` crate compiles to **both**
native (server) and `wasm32` (browser), so the data model and valuation
calculations are literally the same code on both sides — that is the 1:1.

```
crates/
  core/     # shared: serde data model + valuation calc. Targets native AND wasm32.
  server/   # native binary (axum): owns data.json, serves API + WASM bundle.
  web/      # Leptos SPA, compiled to WASM with trunk. Depends on core.
```

- **core** — `serde` structs for the data model; pure functions for gain/loss,
  latest-snapshot selection, currency grouping, and snapshot upsert-by-date. No
  I/O, no framework deps, so it is safe for `wasm32`. This crate is the single
  source of truth for both server and client.
- **server** — `axum` + `tokio`. Owns `data.json` on disk (the file Claude reads
  and writes during research). Serves the JSON API and the static WASM bundle
  produced by trunk. Atomic writes (temp file + rename) with a `.bak` copy.
- **web** — `leptos` SPA. Fetches `/api/data`, renders the dashboard, item
  detail, and the add/edit/delete forms. Charts drawn with `plotters` on a
  `<canvas>` (pure Rust, no JS charting dependency).

### Why a server is still required

The browser cannot persist server-side state on its own, and Claude needs to
read/write `data.json` on disk during research. The native Rust server owns that
file. "1:1" refers to shared logic via `core`, not to eliminating the server.

### Build / run

- Frontend: `trunk build` (or `trunk serve` in dev) compiles `web` to WASM into
  `dist/`.
- Backend: `cargo run -p server` serves `dist/` + the API on `localhost:4000`.
- Dev convenience documented in README (run trunk + server, or build then run).

## 3. Data model (`data.json`)

`data.json` is the single source of truth on disk. Shape mirrors the `core`
serde structs exactly.

```jsonc
{
  "schemaVersion": 1,
  "items": [
    {
      "id": "uuid",
      "name": "MacBook Pro 14\" M1",
      "category": "Electronics",
      "details": "A2442, 16GB/512GB, space grey",   // feeds research accuracy
      "condition": "Good",
      "currency": "GBP",                              // per-item, ISO-4217 string
      "purchasePrice": 1800.0,
      "purchaseDate": "2022-11-01",                   // ISO date
      "notes": "",
      "valuations": [                                 // append-only, dated snapshots
        {
          "date": "2026-05-29",
          "low": 700.0,
          "high": 950.0,
          "best": 820.0,
          "confidence": "medium",                     // low | medium | high
          "sources": [
            { "name": "eBay sold listings", "url": "https://…", "value": 800.0, "note": "" }
          ],
          "comment": "Synthesis note"
        }
      ]
    }
  ]
}
```

**History model — track forward.** The purchase price/date is the one historical
anchor. Each research pass appends a snapshot dated that day. Nothing is
overwritten; history is never lost. If a snapshot already exists for today, it is
updated in place (upsert-by-date) rather than duplicated.

**Value shape — range + best estimate.** Each snapshot stores every source's
figure with its URL, plus a synthesized `low` / `high` / `best` and a
`confidence` level. The chart plots `best` as a line with a shaded `low`–`high`
band.

**Currency — per item, no FX conversion.** Each item carries its own currency.
The dashboard groups totals by currency and shows them side by side; it never
sums across currencies.

## 4. Valuation workflow (Claude-driven)

Documented in `RESEARCH.md`. When the user says "refresh valuations [for X]",
Claude:

1. Reads `data.json` (via the API or directly from disk).
2. For each target item, runs multiple `research-mcp` searches (resale/sold
   listings, marketplaces, price guides) using the item's `name`, `details`, and
   `condition`; fetches the strongest sources.
3. Records per-source figures + URLs, then synthesizes `low` / `high` / `best`
   and a `confidence` level with a short `comment`.
4. Appends (or upserts for today) a snapshot via `POST /api/items/:id/valuations`
   and the server persists it atomically.

This is a manual, on-demand process — there is no background/autonomous scraping,
because `research-mcp` only runs inside a Claude session.

## 5. Frontend (Leptos)

- **Dashboard:** a card per item — current best estimate, purchase price,
  gain/loss in absolute and percent, and a small sparkline. Portfolio totals
  grouped by currency (total cost, total current value, total gain/loss per
  currency).
- **Item detail:** a plotters line chart of `best` over time with a shaded
  `low`–`high` band and the purchase price marked as the anchor; a table of
  snapshots with clickable source links and confidence.
- **Add / Edit / Delete** item form. Manual valuation entry is also possible via
  the same valuation endpoint.

## 6. HTTP API (`localhost:4000`)

| Method | Path | Purpose |
|--------|------|---------|
| GET    | `/api/data` | Full dataset (all items + valuations) |
| POST   | `/api/items` | Add an item |
| PUT    | `/api/items/:id` | Update item fields |
| DELETE | `/api/items/:id` | Remove an item |
| POST   | `/api/items/:id/valuations` | Append/upsert a dated snapshot |

The server validates required fields and returns clear error responses; the
frontend surfaces them. Static routes serve the trunk `dist/` bundle.

## 7. File structure

```
asset-tracker/
  Cargo.toml                 # workspace
  crates/
    core/
      Cargo.toml
      src/lib.rs             # data model + valuation calc (native + wasm32)
    server/
      Cargo.toml
      src/main.rs            # axum: API + static, atomic data.json store
      src/store.rs           # load/save data.json, CRUD, atomic write + .bak
    web/
      Cargo.toml
      src/main.rs            # Leptos app entry
      src/components/        # dashboard, item_detail, item_form, chart
      index.html            # trunk entry
  data.json                  # gitignored (real holdings)
  data.example.json          # committed sample
  Trunk.toml
  README.md                  # build/run + dev workflow
  RESEARCH.md                # the valuation playbook Claude follows
  docs/superpowers/specs/    # this design
```

## 8. Testing

Focused unit tests in `core` via `cargo test`:

- Valuation calcs: gain/loss (absolute + percent), latest-snapshot selection,
  currency grouping (never sums across currencies).
- Snapshot upsert-by-date: appends new dates, replaces same-day, never loses
  prior history.

Server store tests: atomic write round-trips and CRUD preserve valuation history.
No frontend test harness.

## 9. Decisions / non-goals

- **All Rust**, no JS charting dependency (plotters on canvas).
- **Stack:** Leptos (frontend), plotters (charts), axum (server), trunk (build).
- `data.json` is **gitignored**; `data.example.json` is committed.
- No multi-user, no auth, no FX conversion, no background scraping (out of scope).
