use upir_core::ir::*;
use upir_core::types::*;
use upir_to_llvm::lower_upir_to_llvm;
use inkwell::context::Context;

#[test]
fn test_lower_empty_module() {
    let upir_mod = Module {
        name: "empty".to_string(),
        functions: vec![],
    };
    let ctx = Context::create();
    let llvm_mod = lower_upir_to_llvm(&upir_mod, &ctx).expect("Should lower");
    assert!(llvm_mod.get_functions().count() == 0);
}