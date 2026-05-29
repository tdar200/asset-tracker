use std::fs;
use std::path::PathBuf;
use tracker_core::{Dataset, Item, NewItem, Valuation, upsert_valuation};

pub struct Store {
    path: PathBuf,
}

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
