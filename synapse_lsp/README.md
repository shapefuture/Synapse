# Synapse LSP++ (Language Server Protocol)

Phase 3 initial server for Synapse:

- Responds to LSP IDE clients (VSCode, Neovim, etc)
- Provides diagnostics, hover, and future completion/type info

## Features

- Handles `didOpen`, `didChange` notifications (tracks file content)
- Responds to `hover` with placeholder (to be extended with type info/effects)
- Designed for rapid extension to full diagnostics, completion, explainables

## Usage

Build:
```sh
cargo build -p synapse_lsp
```

Run (with LSP client):
```sh
synapse_lsp
```

## Next Steps

- Wire in parser/type checker for real diagnostics
- Implement intelligent hover and completion
- AI-enhanced explainables via ASG/AI API (Phase 3.2+)