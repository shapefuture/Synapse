//! Synapse Package Manager Core
//!
//! - Manifest and lockfile handling
//! - Dependency resolution
//! - Registry fetch and install
//! - Local/remote registry support

mod manifest;
mod registry;
mod resolver;
mod install;
mod cli;

pub use manifest::*;
pub use registry::*;
pub use resolver::*;
pub use install::*;
