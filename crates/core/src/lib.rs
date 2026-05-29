//! Shared data model and valuation calculations. No I/O, no randomness, no clock.

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

#[cfg(test)]
mod tests {
    use super::*;

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
