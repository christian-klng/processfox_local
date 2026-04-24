use serde::Serialize;
use sysinfo::System;

use super::models::ModelCatalog;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HardwareInfo {
    pub ram_gb: u32,
    /// Recommended catalog entry id, chosen based on RAM only.
    /// `None` if no catalog entry fits the host.
    pub recommended_model_id: Option<String>,
}

impl HardwareInfo {
    pub fn detect(catalog: &ModelCatalog) -> Self {
        let mut sys = System::new();
        sys.refresh_memory();
        // sysinfo reports bytes since 0.30; convert to GiB.
        let ram_gb = (sys.total_memory() / 1_073_741_824) as u32;
        let recommended = pick_recommended(ram_gb, catalog);
        Self {
            ram_gb,
            recommended_model_id: recommended,
        }
    }
}

/// Largest catalog model whose `minRamGb` still fits the host. Ties break
/// toward smaller (safer) choices.
fn pick_recommended(ram_gb: u32, catalog: &ModelCatalog) -> Option<String> {
    let mut best: Option<&super::models::CatalogEntry> = None;
    for entry in &catalog.models {
        if entry.min_ram_gb > ram_gb {
            continue;
        }
        match best {
            Some(prev) if prev.min_ram_gb >= entry.min_ram_gb => {}
            _ => best = Some(entry),
        }
    }
    best.map(|m| m.id.clone())
}
