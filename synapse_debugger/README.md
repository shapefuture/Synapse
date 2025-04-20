# Synapse Holographic Debugger

Phase 4.1 implementation of the deep program tracing and omniscient debugging infrastructure for Synapse.

## Features

- **Event Tracing**: Capture function calls, returns, variable assignments, memory operations, and effect usage in a structured trace format.
- **Causal Analysis**: Track causal relationships between events for dynamic analysis.
- **State Reconstruction**: Recreate program state at any point in time for post-mortem debugging.
- **Extensible API**: Simple API points for runtime instrumentation.
- **Visualization**: (Future) Holographic visualization for program execution.

## Implementation

The debugger consists of:

1. **Trace Capture**: Efficient event recording with minimal overhead
2. **Storage Engine**: In-memory and file-based trace storage
3. **Querying Layer**: Filter and retrieve events by various criteria
4. **State Reconstruction**: Rebuild program state from trace entries
5. **API Layer**: Integration with runtime and UART features

## Usage 

```rust
// Start a debug session
let trace_session = synapse_debugger::TraceManager::instance().start_trace()?;

// Get a thread context 
let mut ctx = TraceManager::instance().current_thread_context()?;

// Record events (from instrumented code)
ctx.record_function_call("example", args, Some(node_id))?;
// ... code execution ...
ctx.record_function_return(Some(json!(result)), None, Some(node_id))?;

// Query the trace
let query = TraceQuery { 
    categories: Some(vec![EventCategory::FunctionCall]),
    ..Default::default()
};
let function_calls = trace.filter_events(|e| query.matches(e));

// Reconstruct state at a point in time
let reconstructor = StateReconstructor::new(trace);
let state = reconstructor.reconstruct_at(logical_time)?;
```

## Integration

- Adds instrumentation hooks to the runtime (UART)
- Provides gRPC API for trace query and visualization tools
- Will be integrated with synapse_ai_api for AI-powered debugging assistance