# Synapse Verified FFI

Phase 4 implementation of the safe foreign-function interface (FFI) framework for Synapse.

## Features

- Register/import/export foreign functions at compile time and runtime
- Verify type, memory, and effect contract compliance for all FFI calls
- Sandboxing/untrusted code support with effect capability enforcement
- Contract specification for FFI (pre/post/invariants)
- Dynamic library loading with interface validation
- Integration with UART and UPIR

## Example

```rust
// Register a foreign function
let mut reg = FfiRegistry::new();
reg.register(ForeignFunction {
    name: "add_ints".to_string(),
    arg_types: vec![UpirType::builtin("i32"), UpirType::builtin("i32")],
    ret_type: UpirType::builtin("i32"),
    required_effects: vec!["Pure".to_string()],
    contract: Some("forall x y. result == x + y".to_string()),
    abi: "C".to_string(),
})?;

// Call an FFI function
let engine = FfiEngine::new(reg, Arc::new(UartRuntime::new()?));
let result: i32 = engine.call(
    "add_ints",
    "libadd.so",
    vec![serde_json::json!(2), serde_json::json!(3)],
    &[EffectCap::io()],
)?;
assert_eq!(result, 5);
```

## Safety

- All calls are checked statically and dynamically for type/effect contract compliance
- Arg/result types must match UPIR and FFI ABI
- Effect capabilities must be present at call site

## Next Steps

- Integration with Synapse compiler/codegen for seamless verified FFI calls
- Support for async FFI and streaming interfaces
- Post-mortem contract checking for debugging