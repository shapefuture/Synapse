# Synapse AI API — Phase 3

Fully working AI service for programmatic interaction with the Synapse parse/type/check/diagnostics pipeline.

## gRPC API

- `ParseText`: Accepts Synapse code string, returns ASG as JSON.
- `CheckAsg`: Accepts ASG JSON, runs type+effect checker, returns "OK" or error string.
- `QueryType`: Accepts ASG JSON and node id, returns human-readable type string.
- `GetDiagnostics`: Returns all current diagnostics for an ASG.

See `proto/synapseai.proto` for exact message shapes.

## Usage

```sh
cargo run -p synapse_ai_api
```
Then use a gRPC client (`grpcurl`, any proto-aware tool, or custom Python/Rust client) to interact.

## Example

```sh
grpcurl -plaintext -d '{"text":"(lambda (x : Int) x)"}' localhost:50051 synapseai.Synapseai/ParseText
```

## Integration

This service is used by:
- LSP/IDE (for advanced diagnostics and code actions)
- Synapse Tutor/CLI agents
- AI Syntax Projection Engine
- External research/AI clients