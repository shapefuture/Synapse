# Synapse Package Manager

Phase 4: Ecosystem & Collaboration

## Features

- `synapse.toml` manifest parsing
- Registry index support (local or remote)
- Dependency resolution
- Package installation and update
- CLI for install, resolve, and help

## Usage

```sh
synapse_pkg install      # Install all dependencies as specified in synapse.toml
synapse_pkg resolve      # Show resolved dependency graph
```

## Extensibility

- Supports custom registries, future integration with collaborative/real-time features.
- Intended to be integrated with build/test toolchains.
