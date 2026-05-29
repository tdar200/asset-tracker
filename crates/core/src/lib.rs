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
