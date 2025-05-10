use crate::manifest::PackageManifest;
use crate::registry::RegistryIndex;
use std::collections::{BTreeMap, HashSet};

/// Computes a resolved dependency graph for a package manifest
pub fn resolve_dependencies(
    manifest: &PackageManifest,
    registry: &RegistryIndex,
) -> anyhow::Result<BTreeMap<String, String>> {
    let mut resolved = BTreeMap::new();
    let mut seen = HashSet::new();
    let mut stack = vec![];

    if let Some(deps) = &manifest.dependencies {
        for (name, req_version) in deps {
            stack.push((name.clone(), req_version.clone()));
        }
    }

    while let Some((name, req_version)) = stack.pop() {
        if seen.contains(&name) {
            continue;
        }
        let pkgs = registry.packages.get(&name)
            .ok_or_else(|| anyhow::anyhow!("Package {} not found in registry", name))?;
        let pkg = pkgs.iter().find(|p| &p.version == &req_version)
            .ok_or_else(|| anyhow::anyhow!("Version {} of {} not found", req_version, name))?;
        resolved.insert(name.clone(), pkg.version.clone());
        seen.insert(name.clone());
        // In future: recursively add transitive deps
    }
    Ok(resolved)
}
