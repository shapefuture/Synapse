use crate::{manifest::PackageManifest, registry::RegistryIndex, resolver::resolve_dependencies, install::install_package};
use std::path::Path;
use anyhow::Result;

/// Minimal CLI for Synapse package manager
pub fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    match args.get(1).map(|s| s.as_str()) {
        Some("install") => {
            let manifest = PackageManifest::from_file("synapse.toml")?;
            let registry = RegistryIndex::from_file("registry.json")?;
            let resolved = resolve_dependencies(&manifest, &registry)?;
            for (name, version) in &resolved {
                install_package(name, version, &registry, "deps")?;
                println!("Installed {} v{}", name, version);
            }
            Ok(())
        },
        Some("resolve") => {
            let manifest = PackageManifest::from_file("synapse.toml")?;
            let registry = RegistryIndex::from_file("registry.json")?;
            let resolved = resolve_dependencies(&manifest, &registry)?;
            println!("Resolved dependencies:");
            for (name, version) in &resolved {
                println!("  {} v{}", name, version);
            }
            Ok(())
        },
        Some("help") | _ => {
            println!("Synapse Package Manager CLI");
            println!("Usage:");
            println!("  synapse_pkg install  # Install all dependencies");
            println!("  synapse_pkg resolve  # Show resolved dependency graph");
            println!("  synapse_pkg help     # Show this message");
            Ok(())
        }
    }
}
