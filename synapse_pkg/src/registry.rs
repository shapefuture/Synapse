use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Represents a minimal registry index (local or remote)
#[derive(Debug, Serialize, Deserialize)]
pub struct RegistryIndex {
    pub packages: BTreeMap<String, Vec<RegistryPackage>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RegistryPackage {
    pub name: String,
    pub version: String,
    pub checksum: String,
    pub download_url: String,
}

impl RegistryIndex {
    pub fn fetch_remote(url: &str) -> anyhow::Result<Self> {
        let resp = reqwest::blocking::get(url)?;
        let index = resp.json()?;
        Ok(index)
    }
    pub fn from_file(path: &str) -> anyhow::Result<Self> {
        let contents = std::fs::read_to_string(path)?;
        Ok(serde_json::from_str(&contents)?)
    }
}
