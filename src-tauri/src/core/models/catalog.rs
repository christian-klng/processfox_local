use serde::{Deserialize, Serialize};

use crate::core::error::{CoreError, CoreResult};

const CATALOG_JSON: &str = include_str!("../../../resources/catalog.json");

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CatalogEntry {
    pub id: String,
    pub title: String,
    pub vendor: String,
    pub quant: String,
    pub size_bytes: u64,
    pub min_ram_gb: u32,
    pub hf_url: String,
    pub filename: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelCatalog {
    pub version: u32,
    pub models: Vec<CatalogEntry>,
}

impl ModelCatalog {
    /// Parse the catalog embedded at compile time from `resources/catalog.json`.
    pub fn embedded() -> CoreResult<Self> {
        serde_json::from_str(CATALOG_JSON).map_err(CoreError::from)
    }

    pub fn find(&self, id: &str) -> Option<&CatalogEntry> {
        self.models.iter().find(|m| m.id == id)
    }
}
