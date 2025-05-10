use crate::registry::RegistryIndex;
use std::io::Write;

/// Downloads and installs a package from the registry to the target directory
pub fn install_package(
    name: &str,
    version: &str,
    registry: &RegistryIndex,
    target_dir: &str,
) -> anyhow::Result<()> {
    let pkgs = registry.packages.get(name)
        .ok_or_else(|| anyhow::anyhow!("Package {} not found in registry", name))?;
    let pkg = pkgs.iter().find(|p| &p.version == version)
        .ok_or_else(|| anyhow::anyhow!("Version {} of {} not found", version, name))?;

    // Download package
    let resp = reqwest::blocking::get(&pkg.download_url)?;
    let mut out = std::fs::File::create(format!("{}/{}-{}.tar.gz", target_dir, name, version))?;
    let bytes = resp.bytes()?;
    out.write_all(&bytes)?;
    Ok(())
}
