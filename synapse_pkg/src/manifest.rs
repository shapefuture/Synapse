use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Synapse package manifest (synapse.toml)
#[derive(Debug, Serialize, Deserialize)]
pub struct PackageManifest {
    pub package: PackageSection,
    pub dependencies: Option<BTreeMap<String, String>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PackageSection {
    pub name: String,
    pub version: String,
    pub authors: Option<Vec<String>>,
    pub edition: Option<String>,
    pub description: Option<String>,
    pub license: Option<String>,
}

impl PackageManifest {
    pub fn from_file(path: &str) -> anyhow::Result<Self> {
        let contents = std::fs::read_to_string(path)?;
        Ok(toml::from_str(&contents)?)
    }
}
