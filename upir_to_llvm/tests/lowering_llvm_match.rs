use upir_core::ir::*;
use upir_core::types::*;
use upir_to_llvm::lower_upir_to_llvm;
use inkwell::context::Context;

#[test]
fn test_lower_match_stub() {
    let tid = TypeId(1);
    let arm_blocks = vec![
        AdtMatchArm { ctor: "Left".to_string(), vars: vec![ValueId(1)], body_block: BlockId(11) },
        AdtMatchArm { ctor: "Right".to_string(), vars: vec![ValueId(2)], body_block: BlockId(12) },
    ];
    let match_info = MatchInfo { arms: arm_blocks };
    let match_op = Operation {
        name: "core.match".to_string(),
        operands: vec![ValueId(100)],
        results: vec![],
        attributes: std::collections::HashMap::new(),
        regions: vec![],
        datatype_info: None,
        match_info: Some(match_info),
    };
    let block = Block {
        id: BlockId(10),
        arguments: vec![],
        operations: vec![match_op],
    };
    let fun = Function {
        name: "demomatch".to_string(),
        signature: FunctionSignature { arg_types: vec![tid], result_types: vec![tid] },
        type_params: vec![],
        regions: vec![Region { blocks: vec![block] }],
    };
    let upir_mod = Module {
        name: "adl".to_string(),
        functions: vec![fun],
        datatype_decls: vec![],
        typeparam_decls: vec![],
        effect_decls: vec![],
    };
    let ctx = Context::create();
    let llvm_mod = lower_upir_to_llvm(&upir_mod, &ctx).expect("Should lower");
    let llvm_ir = llvm_mod.print_to_string().to_string();
    println!("{}", llvm_ir);
    assert!(llvm_ir.contains("core.match") || llvm_ir.contains("synapse.match"));
}