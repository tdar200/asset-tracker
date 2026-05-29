# Asset Value Tracker Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** A local all-Rust app that tracks owned assets and their resale value over time, with a WASM frontend and a native server sharing one `core` crate (1:1).

**Architecture:** Cargo workspace with three crates — `tracker-core` (pure serde model + valuation calc, compiles native *and* `wasm32`), `tracker-server` (axum; owns `data.json`, serves the API and the WASM bundle), `tracker-web` (Leptos CSR SPA built with trunk, charts via plotters-canvas). IDs are minted server-side and dates are server/user-supplied so `core`/`web` never touch `uuid`/`getrandom` on wasm.

**Tech Stack:** Rust (stable), Leptos 0.8 (CSR), axum 0.8, tower-http (ServeDir), plotters + plotters-canvas 0.3, web-sys 0.3, gloo-net 0.6, serde/serde_json, uuid 1 (server only), tokio 1, trunk. Tests: `cargo test`, `tempfile`, `axum-test`.

> **Crate-naming note:** the shared crate's *package* name is `tracker-core` (NOT `core` — that collides with the std `core` crate). Its library name is `tracker_core`. Same for `tracker-server` / `tracker-web`.

> **Reference spec:** `docs/superpowers/specs/2026-05-29-asset-value-tracker-design.md`

---

## File Structure

```
asset-tracker/
  Cargo.toml                          # workspace; default-members excludes web
  data.example.json                   # committed sample dataset
  README.md                           # build/run + dev workflow
  RESEARCH.md                         # the valuation playbook Claude follows
  crates/
    core/
      Cargo.toml                      # package: tracker-core
      src/lib.rs                      # model + calc (native + wasm32, no rand/clock)
    server/
      Cargo.toml                      # package: tracker-server
      src/main.rs                     # axum wiring + route handlers
      src/store.rs                    # data.json load/save (atomic) + CRUD
    web/
      Cargo.toml                      # package: tracker-web
      Trunk.toml                      # dist -> ../../dist
      index.html                      # trunk entry
      src/main.rs                     # mount + App shell + view routing signal
      src/api.rs                      # gloo-net fetch wrappers (return core types)
      src/components/
        mod.rs
        dashboard.rs                  # item cards + per-currency totals
        item_form.rs                  # add / edit / delete
        item_detail.rs                # chart + snapshot table
        chart.rs                      # plotters-canvas value-over-time chart
```

Responsibilities are split by feature: each web component file owns one screen; `core` owns all shared types + math so both sides agree by construction.

---

## Task 0: Toolchain setup

**Files:** none (environment only).

- [ ] **Step 1: Install rustup + stable toolchain**

Run:
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
source "$HOME/.cargo/env"
```
Expected: `rustc --version` and `cargo --version` print versions (stable ≥ 1.85).

- [ ] **Step 2: Add the wasm target**

Run: `rustup target add wasm32-unknown-unknown`
Expected: `info: ... installed` (or "already installed").

- [ ] **Step 3: Install trunk + wasm-bindgen-cli**

Run: `cargo install trunk wasm-bindgen-cli`
Expected: both binaries build and land in `~/.cargo/bin`. Verify: `trunk --version`.

- [ ] **Step 4: Verify wasm target present**

Run: `rustup target list --installed | grep wasm32`
Expected: `wasm32-unknown-unknown`.

---

## Task 1: Workspace scaffold

**Files:**
- Create: `Cargo.toml` (workspace root)
- Create: `crates/core/Cargo.toml`, `crates/core/src/lib.rs`
- Create: `crates/server/Cargo.toml`, `crates/server/src/main.rs`
- Create: `crates/web/Cargo.toml`, `crates/web/src/main.rs`
- Create: `data.example.json`

- [ ] **Step 1: Create the workspace root `Cargo.toml`**

```toml
[workspace]
resolver = "2"
members = ["crates/core", "crates/server", "crates/web"]
# `cargo build`/`cargo test` at the root only touch native crates; web is wasm-only and built via trunk.
default-members = ["crates/core", "crates/server"]

[workspace.package]
edition = "2021"
version = "0.1.0"

[workspace.dependencies]
serde = { version = "1", features = ["derive"] }
serde_json = "1"
```

- [ ] **Step 2: Create `crates/core/Cargo.toml`**

```toml
[package]
name = "tracker-core"
version.workspace = true
edition.workspace = true

[lib]
name = "tracker_core"
path = "src/lib.rs"

[dependencies]
serde = { workspace = true }

[dev-dependencies]
serde_json = { workspace = true }
```

- [ ] **Step 3: Stub `crates/core/src/lib.rs`**

```rust
//! Shared data model and valuation calculations. No I/O, no randomness, no clock.
```

- [ ] **Step 4: Create `crates/server/Cargo.toml`**

```toml
[package]
name = "tracker-server"
version.workspace = true
edition.workspace = true

[[bin]]
name = "tracker-server"
path = "src/main.rs"

[dependencies]
tracker-core = { path = "../core" }
serde = { workspace = true }
serde_json = { workspace = true }
tokio = { version = "1", features = ["rt-multi-thread", "macros", "net", "signal"] }
axum = "0.8"
tower-http = { version = "0.6", features = ["fs", "trace"] }
uuid = { version = "1", features = ["v4"] }

[dev-dependencies]
tempfile = "3"
axum-test = "17"
```

- [ ] **Step 5: Stub `crates/server/src/main.rs`**

```rust
fn main() {
    println!("tracker-server placeholder");
}
```

- [ ] **Step 6: Create `crates/web/Cargo.toml`**

```toml
[package]
name = "tracker-web"
version.workspace = true
edition.workspace = true

[[bin]]
name = "tracker-web"
path = "src/main.rs"

[dependencies]
tracker-core = { path = "../core" }
serde = { workspace = true }
serde_json = { workspace = true }
leptos = { version = "0.8", features = ["csr"] }
gloo-net = "0.6"
wasm-bindgen = "0.2"
wasm-bindgen-futures = "0.4"
plotters = { version = "0.3", default-features = false, features = ["line_series", "area_series"] }
plotters-canvas = "0.3"
web-sys = { version = "0.3", features = ["HtmlCanvasElement"] }
console_error_panic_hook = "0.1"
```

- [ ] **Step 7: Stub `crates/web/src/main.rs`**

```rust
fn main() {}
```

- [ ] **Step 8: Create `data.example.json`**

```json
{
  "schemaVersion": 1,
  "items": [
    {
      "id": "example-0001",
      "name": "MacBook Pro 14\" M1",
      "category": "Electronics",
      "details": "A2442, 16GB/512GB, space grey",
      "condition": "Good",
      "currency": "GBP",
      "purchasePrice": 1800.0,
      "purchaseDate": "2022-11-01",
      "notes": "",
      "valuations": [
        {
          "date": "2026-05-29",
          "low": 700.0,
          "high": 950.0,
          "best": 820.0,
          "confidence": "medium",
          "sources": [
            { "name": "eBay sold listings", "url": "https://www.ebay.co.uk/sch/i.html", "value": 800.0, "note": "median of 12 sold" }
          ],
          "comment": "Resale softening with M3 release"
        }
      ]
    }
  ]
}
```

- [ ] **Step 9: Build the native crates**

Run: `cargo build`
Expected: `tracker-core` and `tracker-server` compile (web is excluded by `default-members`).

- [ ] **Step 10: Commit**

```bash
git add -A
git commit -m "feat: scaffold cargo workspace (core/server/web) + sample data"
```

---

## Task 2: Core data model

**Files:**
- Modify: `crates/core/src/lib.rs`
- Test: inline `#[cfg(test)]` module in `crates/core/src/lib.rs`

- [ ] **Step 1: Write the failing round-trip test**

Append to `crates/core/src/lib.rs`:
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dataset_round_trips_through_json() {
        let json = r#"{
            "schemaVersion": 1,
            "items": [{
                "id": "abc",
                "name": "Watch",
                "category": "Luxury",
                "details": "Seamaster",
                "condition": "Good",
                "currency": "GBP",
                "purchasePrice": 1000.0,
                "purchaseDate": "2020-01-01",
                "notes": "",
                "valuations": [{
                    "date": "2026-05-29",
                    "low": 1100.0,
                    "high": 1400.0,
                    "best": 1250.0,
                    "confidence": "high",
                    "sources": [{ "name": "Chrono24", "url": "https://chrono24.com", "value": 1250.0, "note": "" }],
                    "comment": ""
                }]
            }]
        }"#;
        let parsed: Dataset = serde_json::from_str(json).unwrap();
        assert_eq!(parsed.items[0].name, "Watch");
        assert_eq!(parsed.items[0].valuations[0].confidence, Confidence::High);
        // Re-serialize and re-parse must be lossless.
        let again: Dataset = serde_json::from_str(&serde_json::to_string(&parsed).unwrap()).unwrap();
        assert_eq!(parsed, again);
    }
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p tracker-core`
Expected: FAIL — `Dataset`, `Confidence` not found.

- [ ] **Step 3: Define the model in `crates/core/src/lib.rs`** (above the test module)

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Dataset {
    pub schema_version: u32,
    pub items: Vec<Item>,
}

impl Default for Dataset {
    fn default() -> Self {
        Dataset { schema_version: 1, items: Vec::new() }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Item {
    pub id: String,
    pub name: String,
    pub category: String,
    pub details: String,
    pub condition: String,
    pub currency: String,
    pub purchase_price: f64,
    pub purchase_date: String,
    pub notes: String,
    #[serde(default)]
    pub valuations: Vec<Valuation>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Valuation {
    pub date: String,
    pub low: f64,
    pub high: f64,
    pub best: f64,
    pub confidence: Confidence,
    #[serde(default)]
    pub sources: Vec<Source>,
    #[serde(default)]
    pub comment: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Confidence {
    Low,
    Medium,
    High,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Source {
    pub name: String,
    pub url: String,
    pub value: f64,
    #[serde(default)]
    pub note: String,
}

/// Payload for creating a new item (no id, no valuations). Shared by server + web.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NewItem {
    pub name: String,
    pub category: String,
    pub details: String,
    pub condition: String,
    pub currency: String,
    pub purchase_price: f64,
    pub purchase_date: String,
    pub notes: String,
}

impl NewItem {
    /// Build a full `Item` from this payload with a caller-supplied id (minted server-side).
    pub fn into_item(self, id: String) -> Item {
        Item {
            id,
            name: self.name,
            category: self.category,
            details: self.details,
            condition: self.condition,
            currency: self.currency,
            purchase_price: self.purchase_price,
            purchase_date: self.purchase_date,
            notes: self.notes,
            valuations: Vec::new(),
        }
    }
}
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p tracker-core`
Expected: PASS (1 test).

- [ ] **Step 5: Commit**

```bash
git add crates/core/src/lib.rs
git commit -m "feat(core): data model (Dataset/Item/Valuation/Source/NewItem)"
```

---

## Task 3: Core valuation calculations

**Files:**
- Modify: `crates/core/src/lib.rs`
- Test: inline test module in `crates/core/src/lib.rs`

- [ ] **Step 1: Write the failing tests**

Add these tests inside the existing `mod tests`:
```rust
    fn item_with(currency: &str, price: f64, vals: Vec<(&str, f64)>) -> Item {
        Item {
            id: "x".into(), name: "n".into(), category: "c".into(), details: String::new(),
            condition: String::new(), currency: currency.into(), purchase_price: price,
            purchase_date: "2020-01-01".into(), notes: String::new(),
            valuations: vals.into_iter().map(|(date, best)| Valuation {
                date: date.into(), low: best - 10.0, high: best + 10.0, best,
                confidence: Confidence::Medium, sources: vec![], comment: String::new(),
            }).collect(),
        }
    }

    #[test]
    fn latest_valuation_picks_most_recent_date() {
        let item = item_with("GBP", 100.0, vec![("2026-01-01", 90.0), ("2026-05-01", 80.0), ("2026-03-01", 85.0)]);
        assert_eq!(item.latest_valuation().unwrap().date, "2026-05-01");
        assert_eq!(item.current_best(), Some(80.0));
    }

    #[test]
    fn current_best_is_none_without_valuations() {
        let item = item_with("GBP", 100.0, vec![]);
        assert_eq!(item.current_best(), None);
    }

    #[test]
    fn gain_loss_computes_absolute_and_percent() {
        let g = gain_loss(100.0, 80.0);
        assert_eq!(g.absolute, -20.0);
        assert_eq!(g.percent, -20.0);
        let z = gain_loss(0.0, 50.0);
        assert_eq!(z.percent, 0.0); // guard against divide-by-zero
    }

    #[test]
    fn totals_group_by_currency_never_mix() {
        let items = vec![
            item_with("GBP", 100.0, vec![("2026-01-01", 120.0)]),
            item_with("USD", 200.0, vec![("2026-01-01", 150.0)]),
            item_with("GBP", 50.0, vec![]), // no valuation -> current == cost
        ];
        let mut totals = totals_by_currency(&items);
        totals.sort_by(|a, b| a.currency.cmp(&b.currency));
        assert_eq!(totals.len(), 2);
        assert_eq!(totals[0].currency, "GBP");
        assert_eq!(totals[0].total_cost, 150.0);
        assert_eq!(totals[0].total_current, 170.0); // 120 + 50(fallback)
        assert_eq!(totals[0].total_gain, 20.0);
        assert_eq!(totals[1].currency, "USD");
        assert_eq!(totals[1].total_gain, -50.0);
    }

    #[test]
    fn upsert_replaces_same_day_and_keeps_history_sorted() {
        let mut item = item_with("GBP", 100.0, vec![("2026-01-01", 90.0)]);
        upsert_valuation(&mut item, Valuation {
            date: "2026-05-01".into(), low: 70.0, high: 90.0, best: 80.0,
            confidence: Confidence::Low, sources: vec![], comment: String::new(),
        });
        upsert_valuation(&mut item, Valuation {
            date: "2026-01-01".into(), low: 60.0, high: 80.0, best: 70.0, // same day -> replace
            confidence: Confidence::Low, sources: vec![], comment: String::new(),
        });
        assert_eq!(item.valuations.len(), 2);
        assert_eq!(item.valuations[0].date, "2026-01-01");
        assert_eq!(item.valuations[0].best, 70.0); // replaced
        assert_eq!(item.valuations[1].date, "2026-05-01");
    }
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test -p tracker-core`
Expected: FAIL — `latest_valuation`, `gain_loss`, `totals_by_currency`, `upsert_valuation` not found.

- [ ] **Step 3: Implement the calc functions** (add after the model, before `mod tests`)

```rust
use std::collections::BTreeMap;

impl Item {
    /// Most recent snapshot by ISO date (yyyy-mm-dd sorts lexicographically).
    pub fn latest_valuation(&self) -> Option<&Valuation> {
        self.valuations.iter().max_by(|a, b| a.date.cmp(&b.date))
    }

    pub fn current_best(&self) -> Option<f64> {
        self.latest_valuation().map(|v| v.best)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GainLoss {
    pub absolute: f64,
    pub percent: f64,
}

pub fn gain_loss(purchase_price: f64, current_best: f64) -> GainLoss {
    let absolute = current_best - purchase_price;
    let percent = if purchase_price != 0.0 {
        absolute / purchase_price * 100.0
    } else {
        0.0
    };
    GainLoss { absolute, percent }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CurrencyTotal {
    pub currency: String,
    pub total_cost: f64,
    pub total_current: f64,
    pub total_gain: f64,
}

/// Per-currency portfolio totals. Items with no valuation fall back to their
/// purchase price (gain 0). Currencies are never summed together.
pub fn totals_by_currency(items: &[Item]) -> Vec<CurrencyTotal> {
    let mut map: BTreeMap<String, (f64, f64)> = BTreeMap::new();
    for item in items {
        let entry = map.entry(item.currency.clone()).or_insert((0.0, 0.0));
        entry.0 += item.purchase_price;
        entry.1 += item.current_best().unwrap_or(item.purchase_price);
    }
    map.into_iter()
        .map(|(currency, (cost, current))| CurrencyTotal {
            currency,
            total_cost: cost,
            total_current: current,
            total_gain: current - cost,
        })
        .collect()
}

/// Insert a snapshot, replacing any existing snapshot for the same date.
/// History stays sorted ascending by date and is never dropped.
pub fn upsert_valuation(item: &mut Item, valuation: Valuation) {
    if let Some(existing) = item.valuations.iter_mut().find(|v| v.date == valuation.date) {
        *existing = valuation;
    } else {
        item.valuations.push(valuation);
        item.valuations.sort_by(|a, b| a.date.cmp(&b.date));
    }
}
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test -p tracker-core`
Expected: PASS (6 tests total).

- [ ] **Step 5: Confirm core compiles for wasm (no rand/clock leaked in)**

Run: `cargo build -p tracker-core --target wasm32-unknown-unknown`
Expected: compiles cleanly (proves `core` is wasm-safe).

- [ ] **Step 6: Commit**

```bash
git add crates/core/src/lib.rs
git commit -m "feat(core): valuation calc (latest/gain-loss/currency totals/upsert)"
```

---

## Task 4: Server store (data.json persistence + CRUD)

**Files:**
- Create: `crates/server/src/store.rs`
- Modify: `crates/server/src/main.rs` (add `mod store;`)
- Test: inline test module in `crates/server/src/store.rs`

- [ ] **Step 1: Write the failing store tests**

Create `crates/server/src/store.rs`:
```rust
use std::fs;
use std::path::PathBuf;
use tracker_core::{Dataset, Item, NewItem, Valuation, upsert_valuation};

pub struct Store {
    path: PathBuf,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tracker_core::Confidence;

    fn temp_store() -> (tempfile::TempDir, Store) {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("data.json");
        (dir, Store::new(path))
    }

    fn sample_new() -> NewItem {
        NewItem { name: "Phone".into(), category: "Electronics".into(), currency: "GBP".into(), purchase_price: 500.0, purchase_date: "2023-01-01".into(), ..Default::default() }
    }

    #[test]
    fn load_missing_file_returns_empty_dataset() {
        let (_d, store) = temp_store();
        let data = store.load().unwrap();
        assert_eq!(data.schema_version, 1);
        assert!(data.items.is_empty());
    }

    #[test]
    fn add_item_mints_id_and_persists() {
        let (_d, store) = temp_store();
        let created = store.add_item(sample_new()).unwrap();
        assert!(!created.id.is_empty());
        let reloaded = store.load().unwrap();
        assert_eq!(reloaded.items.len(), 1);
        assert_eq!(reloaded.items[0].id, created.id);
    }

    #[test]
    fn save_makes_backup_of_previous_file() {
        let (_d, store) = temp_store();
        store.add_item(sample_new()).unwrap();
        store.add_item(sample_new()).unwrap(); // second save -> .bak of first
        let bak = PathBuf::from(format!("{}.bak", store.path_display()));
        assert!(bak.exists());
    }

    #[test]
    fn update_item_changes_fields_only_for_matching_id() {
        let (_d, store) = temp_store();
        let created = store.add_item(sample_new()).unwrap();
        let mut edited = created.clone();
        edited.name = "Renamed".into();
        let ok = store.update_item(&created.id, edited).unwrap();
        assert!(ok);
        assert_eq!(store.load().unwrap().items[0].name, "Renamed");
        // unknown id -> false, no change
        let mut ghost = created.clone();
        ghost.name = "Nope".into();
        assert!(!store.update_item("missing", ghost).unwrap());
    }

    #[test]
    fn delete_item_removes_by_id() {
        let (_d, store) = temp_store();
        let created = store.add_item(sample_new()).unwrap();
        assert!(store.delete_item(&created.id).unwrap());
        assert!(store.load().unwrap().items.is_empty());
        assert!(!store.delete_item(&created.id).unwrap());
    }

    #[test]
    fn add_valuation_upserts_and_preserves_history() {
        let (_d, store) = temp_store();
        let created = store.add_item(sample_new()).unwrap();
        let v = |date: &str, best: f64| Valuation {
            date: date.into(), low: best - 5.0, high: best + 5.0, best,
            confidence: Confidence::Medium, sources: vec![], comment: String::new(),
        };
        store.add_valuation(&created.id, v("2026-01-01", 400.0)).unwrap();
        store.add_valuation(&created.id, v("2026-05-01", 380.0)).unwrap();
        store.add_valuation(&created.id, v("2026-01-01", 390.0)).unwrap(); // same day replace
        let item = store.load().unwrap().items.into_iter().next().unwrap();
        assert_eq!(item.valuations.len(), 2);
        assert_eq!(item.valuations[0].best, 390.0);
    }
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test -p tracker-server`
Expected: FAIL — `Store::new`, `load`, `add_item`, etc. not found.

- [ ] **Step 3: Implement the store**

Add to `crates/server/src/store.rs` (above the `#[cfg(test)]` module):
```rust
use std::io::{Error, ErrorKind, Result};
use uuid::Uuid;

impl Store {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Store { path: path.into() }
    }

    pub fn path_display(&self) -> String {
        self.path.display().to_string()
    }

    pub fn load(&self) -> Result<Dataset> {
        match fs::read_to_string(&self.path) {
            Ok(s) => serde_json::from_str(&s).map_err(|e| Error::new(ErrorKind::InvalidData, e)),
            Err(e) if e.kind() == ErrorKind::NotFound => Ok(Dataset::default()),
            Err(e) => Err(e),
        }
    }

    /// Atomic save: write `.tmp`, back up the current file to `.bak`, then rename.
    pub fn save(&self, data: &Dataset) -> Result<()> {
        let json = serde_json::to_string_pretty(data).map_err(|e| Error::new(ErrorKind::InvalidData, e))?;
        let tmp = PathBuf::from(format!("{}.tmp", self.path.display()));
        fs::write(&tmp, json)?;
        if self.path.exists() {
            let bak = PathBuf::from(format!("{}.bak", self.path.display()));
            fs::copy(&self.path, &bak)?;
        }
        fs::rename(&tmp, &self.path)
    }

    pub fn add_item(&self, payload: NewItem) -> Result<Item> {
        let mut data = self.load()?;
        let item = payload.into_item(Uuid::new_v4().to_string());
        data.items.push(item.clone());
        self.save(&data)?;
        Ok(item)
    }

    /// Returns true if an item with `id` existed and was updated. `id` is preserved.
    pub fn update_item(&self, id: &str, mut updated: Item) -> Result<bool> {
        let mut data = self.load()?;
        match data.items.iter_mut().find(|i| i.id == id) {
            Some(slot) => {
                updated.id = id.to_string();
                *slot = updated;
                self.save(&data)?;
                Ok(true)
            }
            None => Ok(false),
        }
    }

    pub fn delete_item(&self, id: &str) -> Result<bool> {
        let mut data = self.load()?;
        let before = data.items.len();
        data.items.retain(|i| i.id != id);
        let removed = data.items.len() != before;
        if removed {
            self.save(&data)?;
        }
        Ok(removed)
    }

    /// Upsert a dated snapshot onto an item. Returns false if the id is unknown.
    pub fn add_valuation(&self, id: &str, valuation: Valuation) -> Result<bool> {
        let mut data = self.load()?;
        match data.items.iter_mut().find(|i| i.id == id) {
            Some(item) => {
                upsert_valuation(item, valuation);
                self.save(&data)?;
                Ok(true)
            }
            None => Ok(false),
        }
    }
}
```

- [ ] **Step 4: Wire the module into `main.rs`**

Replace `crates/server/src/main.rs` with:
```rust
mod store;

fn main() {
    println!("tracker-server placeholder");
}
```

- [ ] **Step 5: Run tests to verify they pass**

Run: `cargo test -p tracker-server`
Expected: PASS (6 store tests).

- [ ] **Step 6: Commit**

```bash
git add crates/server/src/store.rs crates/server/src/main.rs
git commit -m "feat(server): atomic data.json store with CRUD + valuation upsert"
```

---

## Task 5: Server HTTP API (axum)

**Files:**
- Modify: `crates/server/src/main.rs`
- Test: `crates/server/tests/api.rs`

> **axum 0.8 note:** path params use brace syntax — `/api/items/{id}`, captured with `Path(id): Path<String>`.

- [ ] **Step 1: Write the failing API integration test**

Create `crates/server/tests/api.rs`:
```rust
use axum_test::TestServer;
use serde_json::json;
use tracker_core::Dataset;

// Build a server bound to a temp data file. `build_router` is defined in main.rs and
// re-exported for tests via a small lib surface; here we call the binary's helper through
// the `tracker_server` path (see Step 3 for the `pub fn build_router`).
fn server() -> (tempfile::TempDir, TestServer) {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("data.json");
    let app = tracker_server::build_router(path, None);
    (dir, TestServer::new(app).unwrap())
}

#[tokio::test]
async fn empty_dataset_is_served() {
    let (_d, server) = server();
    let res = server.get("/api/data").await;
    res.assert_status_ok();
    let data: Dataset = res.json();
    assert!(data.items.is_empty());
}

#[tokio::test]
async fn create_then_list_item() {
    let (_d, server) = server();
    let created = server.post("/api/items").json(&json!({
        "name": "Camera", "category": "Electronics", "details": "", "condition": "Good",
        "currency": "GBP", "purchasePrice": 900.0, "purchaseDate": "2024-02-02", "notes": ""
    })).await;
    created.assert_status_ok();
    let data: Dataset = server.get("/api/data").await.json();
    assert_eq!(data.items.len(), 1);
    assert_eq!(data.items[0].name, "Camera");
    assert!(!data.items[0].id.is_empty());
}

#[tokio::test]
async fn create_rejects_blank_name() {
    let (_d, server) = server();
    let res = server.post("/api/items").json(&json!({
        "name": "", "category": "", "details": "", "condition": "",
        "currency": "GBP", "purchasePrice": 1.0, "purchaseDate": "2024-01-01", "notes": ""
    })).await;
    res.assert_status(axum::http::StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn add_valuation_to_item() {
    let (_d, server) = server();
    let created: tracker_core::Item = server.post("/api/items").json(&json!({
        "name": "Bike", "category": "Vehicles", "details": "", "condition": "Fair",
        "currency": "GBP", "purchasePrice": 1200.0, "purchaseDate": "2021-06-01", "notes": ""
    })).await.json();
    let res = server.post(&format!("/api/items/{}/valuations", created.id)).json(&json!({
        "date": "2026-05-29", "low": 600.0, "high": 800.0, "best": 700.0,
        "confidence": "medium", "sources": [], "comment": ""
    })).await;
    res.assert_status_ok();
    let data: Dataset = server.get("/api/data").await.json();
    assert_eq!(data.items[0].valuations.len(), 1);
    assert_eq!(data.items[0].valuations[0].best, 700.0);
}

#[tokio::test]
async fn delete_item() {
    let (_d, server) = server();
    let created: tracker_core::Item = server.post("/api/items").json(&json!({
        "name": "Ring", "category": "Luxury", "details": "", "condition": "Good",
        "currency": "GBP", "purchasePrice": 300.0, "purchaseDate": "2020-01-01", "notes": ""
    })).await.json();
    server.delete(&format!("/api/items/{}", created.id)).await.assert_status_ok();
    let data: Dataset = server.get("/api/data").await.json();
    assert!(data.items.is_empty());
}
```

- [ ] **Step 2: Expose a lib surface so tests can build the router**

Create `crates/server/src/lib.rs`:
```rust
//! Library surface for tests + the binary. Re-exports the router builder.
mod store;
mod api;

pub use api::build_router;
pub use store::Store;
```
Update `crates/server/Cargo.toml` to add a `[lib]` alongside the bin:
```toml
[lib]
name = "tracker_server"
path = "src/lib.rs"
```
(Keep the existing `[[bin]]` block.)

- [ ] **Step 3: Implement the API in `crates/server/src/api.rs`**

```rust
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use serde_json::json;
use tower_http::services::ServeDir;
use tracker_core::{Dataset, Item, NewItem, Valuation};

use crate::store::Store;

type Shared = Arc<Mutex<Store>>;

/// Build the full router. `dist` optionally serves the WASM bundle as a fallback.
pub fn build_router(data_path: impl Into<PathBuf>, dist: Option<PathBuf>) -> Router {
    let state: Shared = Arc::new(Mutex::new(Store::new(data_path)));
    let api = Router::new()
        .route("/api/data", get(get_data))
        .route("/api/items", post(create_item))
        .route("/api/items/{id}", axum::routing::put(update_item).delete(delete_item))
        .route("/api/items/{id}/valuations", post(add_valuation))
        .with_state(state);

    match dist {
        Some(dir) => api.fallback_service(ServeDir::new(dir).append_index_html_on_directories(true)),
        None => api,
    }
}

struct ApiError(StatusCode, String);

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        (self.0, Json(json!({ "error": self.1 }))).into_response()
    }
}

impl From<std::io::Error> for ApiError {
    fn from(e: std::io::Error) -> Self {
        ApiError(StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
    }
}

fn lock(state: &Shared) -> std::sync::MutexGuard<'_, Store> {
    state.lock().expect("store mutex poisoned")
}

async fn get_data(State(state): State<Shared>) -> Result<Json<Dataset>, ApiError> {
    Ok(Json(lock(&state).load()?))
}

async fn create_item(
    State(state): State<Shared>,
    Json(payload): Json<NewItem>,
) -> Result<Json<Item>, ApiError> {
    validate_new(&payload)?;
    Ok(Json(lock(&state).add_item(payload)?))
}

async fn update_item(
    State(state): State<Shared>,
    Path(id): Path<String>,
    Json(item): Json<Item>,
) -> Result<StatusCode, ApiError> {
    if item.name.trim().is_empty() {
        return Err(ApiError(StatusCode::BAD_REQUEST, "name is required".into()));
    }
    match lock(&state).update_item(&id, item)? {
        true => Ok(StatusCode::OK),
        false => Err(ApiError(StatusCode::NOT_FOUND, "item not found".into())),
    }
}

async fn delete_item(
    State(state): State<Shared>,
    Path(id): Path<String>,
) -> Result<StatusCode, ApiError> {
    match lock(&state).delete_item(&id)? {
        true => Ok(StatusCode::OK),
        false => Err(ApiError(StatusCode::NOT_FOUND, "item not found".into())),
    }
}

async fn add_valuation(
    State(state): State<Shared>,
    Path(id): Path<String>,
    Json(valuation): Json<Valuation>,
) -> Result<StatusCode, ApiError> {
    if valuation.date.trim().is_empty() {
        return Err(ApiError(StatusCode::BAD_REQUEST, "date is required".into()));
    }
    match lock(&state).add_valuation(&id, valuation)? {
        true => Ok(StatusCode::OK),
        false => Err(ApiError(StatusCode::NOT_FOUND, "item not found".into())),
    }
}

fn validate_new(p: &NewItem) -> Result<(), ApiError> {
    if p.name.trim().is_empty() {
        return Err(ApiError(StatusCode::BAD_REQUEST, "name is required".into()));
    }
    if p.currency.trim().is_empty() {
        return Err(ApiError(StatusCode::BAD_REQUEST, "currency is required".into()));
    }
    if p.purchase_price < 0.0 {
        return Err(ApiError(StatusCode::BAD_REQUEST, "purchasePrice must be >= 0".into()));
    }
    if p.purchase_date.trim().is_empty() {
        return Err(ApiError(StatusCode::BAD_REQUEST, "purchaseDate is required".into()));
    }
    Ok(())
}
```

- [ ] **Step 4: Rewrite `crates/server/src/main.rs` to run the server**

```rust
use std::path::PathBuf;

#[tokio::main]
async fn main() {
    let data_path = std::env::var("ASSET_TRACKER_DATA").unwrap_or_else(|_| "data.json".into());
    let dist = std::env::var("ASSET_TRACKER_DIST").unwrap_or_else(|_| "dist".into());
    let port: u16 = std::env::var("ASSET_TRACKER_PORT").ok().and_then(|p| p.parse().ok()).unwrap_or(4000);

    let app = tracker_server::build_router(data_path, Some(PathBuf::from(dist)));
    let listener = tokio::net::TcpListener::bind(("127.0.0.1", port)).await.unwrap();
    println!("asset-tracker listening on http://127.0.0.1:{port}");
    axum::serve(listener, app).await.unwrap();
}
```
(Delete the old `mod store;` line from `main.rs` — modules now live in `lib.rs`.)

- [ ] **Step 5: Run tests to verify they pass**

Run: `cargo test -p tracker-server`
Expected: PASS (6 store unit tests + 5 API tests).

- [ ] **Step 6: Commit**

```bash
git add crates/server
git commit -m "feat(server): axum API (data/items CRUD + valuations) + ServeDir fallback"
```

---

## Task 6: Web app shell (Leptos CSR + trunk)

**Files:**
- Create: `crates/web/index.html`, `crates/web/Trunk.toml`
- Create: `crates/web/src/components/mod.rs`
- Modify: `crates/web/src/main.rs`

- [ ] **Step 1: Create `crates/web/index.html`**

```html
<!DOCTYPE html>
<html lang="en">
  <head>
    <meta charset="utf-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1" />
    <title>Asset Value Tracker</title>
    <link data-trunk rel="rust" data-wasm-opt="z" />
    <style>
      body { font-family: system-ui, sans-serif; margin: 0; background: #f6f7f9; color: #1c2024; }
      header { background: #1c2024; color: #fff; padding: 12px 20px; }
      main { max-width: 1000px; margin: 0 auto; padding: 20px; }
      .card { background: #fff; border-radius: 10px; padding: 16px; box-shadow: 0 1px 3px rgba(0,0,0,.08); }
      .grid { display: grid; grid-template-columns: repeat(auto-fill, minmax(260px, 1fr)); gap: 14px; }
      .gain-pos { color: #16794a; } .gain-neg { color: #b5341a; }
      button { cursor: pointer; border-radius: 7px; border: 1px solid #c3c8cf; padding: 7px 12px; background: #fff; }
      button.primary { background: #2862ff; color: #fff; border-color: #2862ff; }
      input, select, textarea { padding: 7px; border: 1px solid #c3c8cf; border-radius: 7px; width: 100%; box-sizing: border-box; }
      label { display: block; margin: 8px 0 3px; font-size: 13px; color: #54595f; }
      table { width: 100%; border-collapse: collapse; } td, th { text-align: left; padding: 6px 8px; border-bottom: 1px solid #eee; }
    </style>
  </head>
  <body></body>
</html>
```

- [ ] **Step 2: Create `crates/web/Trunk.toml`**

```toml
[build]
target = "index.html"
dist = "../../dist"

[serve]
# Dev server proxies API calls to the running tracker-server.
[[proxy]]
backend = "http://127.0.0.1:4000/api"
```

- [ ] **Step 3: Create `crates/web/src/components/mod.rs`**

```rust
pub mod dashboard;
pub mod item_form;
pub mod item_detail;
pub mod chart;
```

- [ ] **Step 4: Create stub component files so the module tree compiles**

Create `crates/web/src/components/dashboard.rs`:
```rust
use leptos::prelude::*;

#[component]
pub fn Dashboard() -> impl IntoView {
    view! { <p>"dashboard placeholder"</p> }
}
```
Create `crates/web/src/components/item_form.rs`:
```rust
use leptos::prelude::*;

#[component]
pub fn ItemForm() -> impl IntoView {
    view! { <p>"form placeholder"</p> }
}
```
Create `crates/web/src/components/item_detail.rs`:
```rust
use leptos::prelude::*;

#[component]
pub fn ItemDetail() -> impl IntoView {
    view! { <p>"detail placeholder"</p> }
}
```
Create `crates/web/src/components/chart.rs`:
```rust
use leptos::prelude::*;

#[component]
pub fn ValueChart() -> impl IntoView {
    view! { <canvas></canvas> }
}
```

- [ ] **Step 5: Write the app shell in `crates/web/src/main.rs`**

```rust
mod api;
mod components;

use leptos::prelude::*;
use components::dashboard::Dashboard;

#[component]
fn App() -> impl IntoView {
    view! {
        <header><h1 style="margin:0;font-size:18px;">"Asset Value Tracker"</h1></header>
        <main><Dashboard/></main>
    }
}

fn main() {
    console_error_panic_hook::set_once();
    leptos::mount::mount_to_body(App);
}
```

- [ ] **Step 6: Create a minimal `crates/web/src/api.rs` stub** (filled out in Task 7)

```rust
//! HTTP client wrappers (filled in Task 7).
```

- [ ] **Step 7: Build the WASM bundle**

Run (from repo root): `trunk build crates/web/index.html`
Expected: produces `dist/` with `index.html` + `*_bg.wasm` + JS glue, no errors.

- [ ] **Step 8: Commit**

```bash
git add crates/web dist/.gitkeep 2>/dev/null; git add crates/web
git commit -m "feat(web): Leptos CSR shell + trunk build wiring"
```

---

## Task 7: Web API client + shared app state

**Files:**
- Modify: `crates/web/src/api.rs`
- Modify: `crates/web/src/main.rs`

- [ ] **Step 1: Implement the fetch wrappers in `crates/web/src/api.rs`**

```rust
use gloo_net::http::Request;
use tracker_core::{Dataset, Item, NewItem, Valuation};

pub async fn fetch_data() -> Result<Dataset, String> {
    let resp = Request::get("/api/data").send().await.map_err(|e| e.to_string())?;
    resp.json::<Dataset>().await.map_err(|e| e.to_string())
}

pub async fn create_item(payload: &NewItem) -> Result<Item, String> {
    let resp = Request::post("/api/items")
        .json(payload).map_err(|e| e.to_string())?
        .send().await.map_err(|e| e.to_string())?;
    if !resp.ok() { return Err(error_body(resp).await); }
    resp.json::<Item>().await.map_err(|e| e.to_string())
}

pub async fn update_item(item: &Item) -> Result<(), String> {
    let resp = Request::put(&format!("/api/items/{}", item.id))
        .json(item).map_err(|e| e.to_string())?
        .send().await.map_err(|e| e.to_string())?;
    if resp.ok() { Ok(()) } else { Err(error_body(resp).await) }
}

pub async fn delete_item(id: &str) -> Result<(), String> {
    let resp = Request::delete(&format!("/api/items/{}", id))
        .send().await.map_err(|e| e.to_string())?;
    if resp.ok() { Ok(()) } else { Err(error_body(resp).await) }
}

pub async fn add_valuation(id: &str, valuation: &Valuation) -> Result<(), String> {
    let resp = Request::post(&format!("/api/items/{}/valuations", id))
        .json(valuation).map_err(|e| e.to_string())?
        .send().await.map_err(|e| e.to_string())?;
    if resp.ok() { Ok(()) } else { Err(error_body(resp).await) }
}

async fn error_body(resp: gloo_net::http::Response) -> String {
    resp.text().await.unwrap_or_else(|_| "request failed".into())
}
```

- [ ] **Step 2: Add a navigation enum + shared data resource to `crates/web/src/main.rs`**

Replace `main.rs` with:
```rust
mod api;
mod components;

use leptos::prelude::*;
use leptos::task::spawn_local;
use tracker_core::Dataset;

use components::dashboard::Dashboard;
use components::item_detail::ItemDetail;
use components::item_form::ItemForm;

/// Which screen is showing. Kept in a signal instead of a router to stay dependency-light.
#[derive(Clone, PartialEq)]
pub enum View {
    Dashboard,
    NewItem,
    Edit(String),
    Detail(String),
}

#[derive(Clone, Copy)]
pub struct AppCtx {
    pub data: RwSignal<Option<Dataset>>,
    pub view: RwSignal<View>,
}

impl AppCtx {
    /// Re-fetch the dataset from the server into the shared signal.
    pub fn refresh(self) {
        spawn_local(async move {
            match api::fetch_data().await {
                Ok(d) => self.data.set(Some(d)),
                Err(e) => web_sys::console::error_1(&e.into()),
            }
        });
    }
}

#[component]
fn App() -> impl IntoView {
    let ctx = AppCtx { data: RwSignal::new(None), view: RwSignal::new(View::Dashboard) };
    provide_context(ctx);
    ctx.refresh();

    view! {
        <header><h1 style="margin:0;font-size:18px;">"Asset Value Tracker"</h1></header>
        <main>
            {move || match ctx.view.get() {
                View::Dashboard => view! { <Dashboard/> }.into_any(),
                View::NewItem => view! { <ItemForm id=None/> }.into_any(),
                View::Edit(id) => view! { <ItemForm id=Some(id)/> }.into_any(),
                View::Detail(id) => view! { <ItemDetail id=id/> }.into_any(),
            }}
        </main>
    }
}

fn main() {
    console_error_panic_hook::set_once();
    leptos::mount::mount_to_body(App);
}
```

- [ ] **Step 3: Build to verify it compiles**

Run: `trunk build crates/web/index.html`
Expected: compiles (placeholder components still render; `ItemForm`/`ItemDetail` will gain their `id` props next tasks — update their signatures in Steps below if the build flags missing props).

> The placeholder `ItemForm`/`ItemDetail`/`ValueChart` from Task 6 take no props yet. Temporarily update their signatures to match the call sites so this step compiles:
> - `pub fn ItemForm(#[prop(optional)] id: Option<String>) -> impl IntoView`
> - `pub fn ItemDetail(id: String) -> impl IntoView`

- [ ] **Step 4: Commit**

```bash
git add crates/web/src
git commit -m "feat(web): API client + shared app state and view routing"
```

---

## Task 8: Dashboard (cards + per-currency totals)

**Files:**
- Modify: `crates/web/src/components/dashboard.rs`

- [ ] **Step 1: Implement the dashboard**

```rust
use leptos::prelude::*;
use tracker_core::{gain_loss, totals_by_currency, Item};

use crate::{AppCtx, View};

#[component]
pub fn Dashboard() -> impl IntoView {
    let ctx = expect_context::<AppCtx>();

    view! {
        <div style="display:flex;justify-content:space-between;align-items:center;margin-bottom:14px;">
            <h2 style="margin:0;">"Holdings"</h2>
            <button class="primary" on:click=move |_| ctx.view.set(View::NewItem)>"+ Add item"</button>
        </div>
        {move || match ctx.data.get() {
            None => view! { <p>"Loading…"</p> }.into_any(),
            Some(data) if data.items.is_empty() => view! { <p>"No items yet. Add your first asset."</p> }.into_any(),
            Some(data) => {
                let totals = totals_by_currency(&data.items);
                view! {
                    <div class="card" style="margin-bottom:16px;">
                        <h3 style="margin-top:0;">"Portfolio (by currency)"</h3>
                        <table>
                            <thead><tr><th>"Currency"</th><th>"Cost"</th><th>"Current"</th><th>"Gain/Loss"</th></tr></thead>
                            <tbody>
                                {totals.into_iter().map(|t| {
                                    let cls = if t.total_gain >= 0.0 { "gain-pos" } else { "gain-neg" };
                                    view! {
                                        <tr>
                                            <td>{t.currency.clone()}</td>
                                            <td>{format!("{:.2}", t.total_cost)}</td>
                                            <td>{format!("{:.2}", t.total_current)}</td>
                                            <td class=cls>{format!("{:+.2}", t.total_gain)}</td>
                                        </tr>
                                    }
                                }).collect_view()}
                            </tbody>
                        </table>
                    </div>
                    <div class="grid">
                        {data.items.into_iter().map(|item| view! { <ItemCard item=item/> }).collect_view()}
                    </div>
                }.into_any()
            }
        }}
    }
}

#[component]
fn ItemCard(item: Item) -> impl IntoView {
    let ctx = expect_context::<AppCtx>();
    let id = item.id.clone();
    let current = item.current_best();
    let (gl_text, gl_cls) = match current {
        Some(c) => {
            let g = gain_loss(item.purchase_price, c);
            (format!("{:+.2} ({:+.1}%)", g.absolute, g.percent),
             if g.absolute >= 0.0 { "gain-pos" } else { "gain-neg" })
        }
        None => ("no valuation yet".to_string(), ""),
    };
    let cur_text = current.map(|c| format!("{} {:.2}", item.currency, c))
        .unwrap_or_else(|| "—".into());

    view! {
        <div class="card" style="cursor:pointer;" on:click=move |_| ctx.view.set(View::Detail(id.clone()))>
            <strong>{item.name.clone()}</strong>
            <div style="font-size:12px;color:#54595f;">{item.category.clone()}</div>
            <p style="margin:8px 0 2px;">"Current: "{cur_text}</p>
            <p style="margin:2px 0;font-size:13px;">"Paid: "{format!("{} {:.2}", item.currency, item.purchase_price)}</p>
            <p class=gl_cls style="margin:2px 0;">{gl_text}</p>
        </div>
    }
}
```

- [ ] **Step 2: Build and manually verify**

Run two terminals:
```bash
# terminal A — backend (serves dist + API; uses sample data)
ASSET_TRACKER_DATA=data.example.json cargo run -p tracker-server
# terminal B — frontend dev server with API proxy
trunk serve crates/web/index.html --open
```
Expected: the dashboard lists the sample MacBook with a per-currency totals table and a gain/loss figure; clicking a card switches the view (detail placeholder for now).

- [ ] **Step 3: Commit**

```bash
git add crates/web/src/components/dashboard.rs
git commit -m "feat(web): dashboard cards + per-currency portfolio totals"
```

---

## Task 9: Add / Edit / Delete form

**Files:**
- Modify: `crates/web/src/components/item_form.rs`

- [ ] **Step 1: Implement the form**

```rust
use leptos::prelude::*;
use leptos::task::spawn_local;
use tracker_core::{Item, NewItem};

use crate::{api, AppCtx, View};

#[component]
pub fn ItemForm(#[prop(optional)] id: Option<String>) -> impl IntoView {
    let ctx = expect_context::<AppCtx>();

    // Seed fields from the existing item when editing.
    let existing: Option<Item> = id.as_ref().and_then(|id| {
        ctx.data.get().and_then(|d| d.items.into_iter().find(|i| &i.id == id))
    });

    let name = RwSignal::new(existing.as_ref().map(|i| i.name.clone()).unwrap_or_default());
    let category = RwSignal::new(existing.as_ref().map(|i| i.category.clone()).unwrap_or_default());
    let details = RwSignal::new(existing.as_ref().map(|i| i.details.clone()).unwrap_or_default());
    let condition = RwSignal::new(existing.as_ref().map(|i| i.condition.clone()).unwrap_or_default());
    let currency = RwSignal::new(existing.as_ref().map(|i| i.currency.clone()).unwrap_or_else(|| "GBP".into()));
    let price = RwSignal::new(existing.as_ref().map(|i| i.purchase_price).unwrap_or(0.0));
    let date = RwSignal::new(existing.as_ref().map(|i| i.purchase_date.clone()).unwrap_or_default());
    let notes = RwSignal::new(existing.as_ref().map(|i| i.notes.clone()).unwrap_or_default());
    let error = RwSignal::new(String::new());

    let editing_item = existing.clone();
    let on_save = move |_| {
        let ctx = ctx;
        let editing_item = editing_item.clone();
        let payload = NewItem {
            name: name.get(), category: category.get(), details: details.get(),
            condition: condition.get(), currency: currency.get(),
            purchase_price: price.get(), purchase_date: date.get(), notes: notes.get(),
        };
        spawn_local(async move {
            let result = match editing_item {
                Some(mut item) => {
                    item.name = payload.name; item.category = payload.category;
                    item.details = payload.details; item.condition = payload.condition;
                    item.currency = payload.currency; item.purchase_price = payload.purchase_price;
                    item.purchase_date = payload.purchase_date; item.notes = payload.notes;
                    api::update_item(&item).await
                }
                None => api::create_item(&payload).await.map(|_| ()),
            };
            match result {
                Ok(()) => { ctx.refresh(); ctx.view.set(View::Dashboard); }
                Err(e) => error.set(e),
            }
        });
    };

    let delete_id = id.clone();
    let on_delete = move |_| {
        if let Some(id) = delete_id.clone() {
            let ctx = ctx;
            spawn_local(async move {
                match api::delete_item(&id).await {
                    Ok(()) => { ctx.refresh(); ctx.view.set(View::Dashboard); }
                    Err(e) => error.set(e),
                }
            });
        }
    };

    let is_edit = id.is_some();

    view! {
        <div class="card" style="max-width:520px;">
            <h2 style="margin-top:0;">{move || if is_edit { "Edit item" } else { "Add item" }}</h2>
            <label>"Name"</label>
            <input prop:value=move || name.get() on:input=move |e| name.set(event_target_value(&e))/>
            <label>"Category"</label>
            <input prop:value=move || category.get() on:input=move |e| category.set(event_target_value(&e))/>
            <label>"Details (model no., specs — helps valuation accuracy)"</label>
            <input prop:value=move || details.get() on:input=move |e| details.set(event_target_value(&e))/>
            <label>"Condition"</label>
            <input prop:value=move || condition.get() on:input=move |e| condition.set(event_target_value(&e))/>
            <label>"Currency (e.g. GBP, USD)"</label>
            <input prop:value=move || currency.get() on:input=move |e| currency.set(event_target_value(&e))/>
            <label>"Purchase price"</label>
            <input type="number" step="0.01" prop:value=move || price.get()
                on:input=move |e| price.set(event_target_value(&e).parse().unwrap_or(0.0))/>
            <label>"Purchase date (YYYY-MM-DD)"</label>
            <input type="date" prop:value=move || date.get() on:input=move |e| date.set(event_target_value(&e))/>
            <label>"Notes"</label>
            <textarea on:input=move |e| notes.set(event_target_value(&e))>{move || notes.get()}</textarea>
            <p style="color:#b5341a;">{move || error.get()}</p>
            <div style="display:flex;gap:8px;margin-top:12px;">
                <button class="primary" on:click=on_save>"Save"</button>
                <button on:click=move |_| ctx.view.set(View::Dashboard)>"Cancel"</button>
                {is_edit.then(|| view! { <button on:click=on_delete style="margin-left:auto;color:#b5341a;">"Delete"</button> })}
            </div>
        </div>
    }
}
```

- [ ] **Step 2: Build and manually verify**

Run: `trunk serve crates/web/index.html` (with backend running as in Task 8).
Expected: "+ Add item" opens the form; saving a new item returns to the dashboard with the item present; editing changes fields; delete removes it. Confirm `data.example.json` (or your data file) reflects the changes on disk.

- [ ] **Step 3: Commit**

```bash
git add crates/web/src/components/item_form.rs
git commit -m "feat(web): add/edit/delete item form"
```

---

## Task 10: Item detail + plotters chart

**Files:**
- Modify: `crates/web/src/components/chart.rs`
- Modify: `crates/web/src/components/item_detail.rs`

- [ ] **Step 1: Implement the plotters-canvas chart**

`crates/web/src/components/chart.rs`:
```rust
use leptos::html::Canvas;
use leptos::prelude::*;
use plotters::prelude::*;
use plotters_canvas::CanvasBackend;
use tracker_core::Item;
use web_sys::HtmlCanvasElement;

/// Value-over-time chart: shaded low–high band, best-estimate line, purchase-price anchor.
#[component]
pub fn ValueChart(item: Item) -> impl IntoView {
    let canvas_ref: NodeRef<Canvas> = NodeRef::new();

    // Redraw whenever the canvas mounts.
    Effect::new(move |_| {
        if let Some(canvas) = canvas_ref.get() {
            let el: HtmlCanvasElement = canvas.into();
            draw(&el, &item);
        }
    });

    view! { <canvas node_ref=canvas_ref width="680" height="340"></canvas> }
}

fn draw(canvas: &HtmlCanvasElement, item: &Item) {
    let backend = match CanvasBackend::with_canvas_object(canvas.clone()) {
        Some(b) => b,
        None => return,
    };
    let area = backend.into_drawing_area();
    area.fill(&WHITE).ok();

    if item.valuations.is_empty() {
        return;
    }

    let n = item.valuations.len();
    let xs: Vec<f64> = (0..n).map(|i| i as f64).collect();
    let highs: Vec<f64> = item.valuations.iter().map(|v| v.high).collect();
    let lows: Vec<f64> = item.valuations.iter().map(|v| v.low).collect();
    let bests: Vec<f64> = item.valuations.iter().map(|v| v.best).collect();

    let y_min = lows.iter().cloned().fold(item.purchase_price, f64::min) * 0.95;
    let y_max = highs.iter().cloned().fold(item.purchase_price, f64::max) * 1.05;
    let x_max = if n > 1 { (n - 1) as f64 } else { 1.0 };

    let mut chart = ChartBuilder::on(&area)
        .margin(10)
        .x_label_area_size(36)
        .y_label_area_size(52)
        .build_cartesian_2d(0f64..x_max, y_min..y_max)
        .unwrap();

    chart
        .configure_mesh()
        .x_labels(n.min(8))
        .x_label_formatter(&|x| {
            let i = x.round() as usize;
            item.valuations.get(i).map(|v| v.date.clone()).unwrap_or_default()
        })
        .draw()
        .ok();

    // Low–high band as a filled polygon (high forward, low back).
    let mut band: Vec<(f64, f64)> = xs.iter().zip(&highs).map(|(&x, &h)| (x, h)).collect();
    band.extend(xs.iter().zip(&lows).rev().map(|(&x, &l)| (x, l)));
    chart
        .draw_series(std::iter::once(Polygon::new(band, BLUE.mix(0.12))))
        .ok();

    // Best-estimate line.
    chart
        .draw_series(LineSeries::new(xs.iter().zip(&bests).map(|(&x, &b)| (x, b)), BLUE.stroke_width(2)))
        .ok();

    // Purchase-price reference line.
    chart
        .draw_series(LineSeries::new(vec![(0.0, item.purchase_price), (x_max, item.purchase_price)], RED.mix(0.6)))
        .ok();

    area.present().ok();
}
```

- [ ] **Step 2: Implement the item detail screen**

`crates/web/src/components/item_detail.rs`:
```rust
use leptos::prelude::*;
use tracker_core::Item;

use crate::components::chart::ValueChart;
use crate::{AppCtx, View};

#[component]
pub fn ItemDetail(id: String) -> impl IntoView {
    let ctx = expect_context::<AppCtx>();
    let id_for_lookup = id.clone();

    view! {
        <button on:click=move |_| ctx.view.set(View::Dashboard)>"← Back"</button>
        {move || {
            let item: Option<Item> = ctx.data.get()
                .and_then(|d| d.items.into_iter().find(|i| i.id == id_for_lookup));
            match item {
                None => view! { <p>"Item not found."</p> }.into_any(),
                Some(item) => {
                    let edit_id = item.id.clone();
                    view! {
                        <div style="display:flex;justify-content:space-between;align-items:center;">
                            <h2>{item.name.clone()}</h2>
                            <button on:click=move |_| ctx.view.set(View::Edit(edit_id.clone()))>"Edit"</button>
                        </div>
                        <p style="color:#54595f;">{item.details.clone()}" · "{item.condition.clone()}</p>
                        <div class="card" style="margin-bottom:16px;">
                            <ValueChart item=item.clone()/>
                        </div>
                        <div class="card">
                            <h3 style="margin-top:0;">"Valuation history"</h3>
                            <table>
                                <thead><tr><th>"Date"</th><th>"Low"</th><th>"Best"</th><th>"High"</th><th>"Confidence"</th><th>"Sources"</th></tr></thead>
                                <tbody>
                                    {item.valuations.iter().rev().map(|v| {
                                        let conf = format!("{:?}", v.confidence);
                                        view! {
                                            <tr>
                                                <td>{v.date.clone()}</td>
                                                <td>{format!("{:.0}", v.low)}</td>
                                                <td>{format!("{:.0}", v.best)}</td>
                                                <td>{format!("{:.0}", v.high)}</td>
                                                <td>{conf}</td>
                                                <td>{v.sources.iter().map(|s| view! {
                                                    <a href=s.url.clone() target="_blank" style="margin-right:6px;">{s.name.clone()}</a>
                                                }).collect_view()}</td>
                                            </tr>
                                        }
                                    }).collect_view()}
                                </tbody>
                            </table>
                        </div>
                    }.into_any()
                }
            }
        }}
    }
}
```

- [ ] **Step 3: Build and manually verify**

Run: `trunk serve crates/web/index.html` (backend running).
Expected: clicking the sample item opens its detail; the chart renders a best-estimate line with a shaded band and a red purchase-price line; the history table lists the snapshot with a clickable eBay source link.

- [ ] **Step 4: Commit**

```bash
git add crates/web/src/components/chart.rs crates/web/src/components/item_detail.rs
git commit -m "feat(web): item detail + plotters value-over-time chart"
```

---

## Task 11: Docs — README + RESEARCH playbook

**Files:**
- Create: `README.md`
- Create: `RESEARCH.md`

- [ ] **Step 1: Write `README.md`**

```markdown
# Asset Value Tracker

Local, all-Rust app to track owned assets and their resale value over time. WASM
frontend (Leptos) + native server (axum) sharing one `core` crate.

## Prerequisites
- Rust stable, `wasm32-unknown-unknown` target, `trunk`, `wasm-bindgen-cli`
  (see Task 0 of the implementation plan).

## Build & run
```bash
# 1. Build the frontend bundle into ./dist
trunk build crates/web/index.html

# 2. Run the server (serves ./dist + the API on http://127.0.0.1:4000)
cargo run -p tracker-server
# open http://127.0.0.1:4000
```

### Dev mode (hot reload)
```bash
# terminal A
cargo run -p tracker-server
# terminal B (proxies /api to the server)
trunk serve crates/web/index.html --open
```

## Data
`data.json` (gitignored) is the source of truth. Copy `data.example.json` to start.
Override location/port with `ASSET_TRACKER_DATA`, `ASSET_TRACKER_DIST`, `ASSET_TRACKER_PORT`.

## Refreshing valuations
Values are filled in by Claude on demand — see `RESEARCH.md`.
```

- [ ] **Step 2: Write `RESEARCH.md`** (the playbook Claude follows)

```markdown
# Valuation Research Playbook

When the user says "refresh valuations [for X]":

1. Read the dataset (`GET http://127.0.0.1:4000/api/data`, or read `data.json`).
   The server must be running to write back via the API.
2. For each target item, run multiple `research-mcp` searches using `name`,
   `details`, and `condition` (resale/sold listings, marketplaces, price guides).
   Fetch the strongest sources and record each source's figure + URL.
3. Synthesize a snapshot: `low`, `high`, `best`, a `confidence`
   (`low`/`medium`/`high`), and a one-line `comment`.
4. POST it (date = today, ISO `YYYY-MM-DD`):
   `POST /api/items/{id}/valuations` with the Valuation JSON.
   Same-day snapshots are upserted (replaced), so re-running is safe.

Rules:
- Never invent figures — every number traces to a fetched source.
- Currencies are per-item; do not convert. Match the item's currency.
- History is append-forward; never edit or delete past snapshots.
```

- [ ] **Step 3: Commit**

```bash
git add README.md RESEARCH.md
git commit -m "docs: README (build/run) + RESEARCH valuation playbook"
```

---

## Final verification

- [ ] **Step 1: Full native test suite**

Run: `cargo test`
Expected: all `tracker-core` + `tracker-server` tests PASS.

- [ ] **Step 2: Clean wasm build**

Run: `trunk build crates/web/index.html`
Expected: builds with no errors; `dist/` populated.

- [ ] **Step 3: End-to-end smoke**

Run: `cargo run -p tracker-server`, open `http://127.0.0.1:4000`.
Expected: dashboard loads sample data, add/edit/delete works, detail chart renders.

- [ ] **Step 4: Verify clean tree**

Run: `git status`
Expected: clean (or only intended `data.json` which is gitignored).
```
