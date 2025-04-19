use upir_core::ir::*;
use upir_core::types::*;
use upir_to_llvm::lower_upir_to_llvm;
use inkwell::context::Context;

#[test]
fn test_lower_add_function() {
    let tid_i32 = TypeId(1);

    // Build a function: fn add(a: i32, b: i32) -> i32 { return a + b }
    let function = Function {
        name: "add".to_string(),
        signature: FunctionSignature {
            arg_types: vec![tid_i32, tid_i32],
            result_types: vec![tid_i32],
        },
        regions: vec![
            Region {
                blocks: vec![
                    Block {
                        id: BlockId(0),
                        arguments: vec![
                            BlockArgument {
                                value_def: ValueDef { id: ValueId(0), ty: tid_i32 },
                                block_id: BlockId(0),
                                index: 0,
                            },
                            BlockArgument {
                                value_def: ValueDef { id: ValueId(1), ty: tid_i32 },
                                block_id: BlockId(0),
                                index: 1,
                            }
                        ],
                        operations: vec![
                            Operation {
                                name: "core.add".to_string(),
                                operands: vec![ValueId(0), ValueId(1)],
                                results: vec![ValueDef { id: ValueId(2), ty: tid_i32 }],
                                attributes: std::collections::HashMap::new(),
                                regions: vec![],
                            },
                            Operation {
                                name: "func.return".to_string(),
                                operands: vec![ValueId(2)],
                                results: vec![],
                                attributes: std::collections::HashMap::new(),
                                regions: vec![],
                            }
                        ],
                    }
                ]
            }
        ]
    };
    let upir_mod = Module {
        name: "basic".to_string(),
        functions: vec![function],
    };
    let ctx = Context::create();
    let llvm_mod = lower_upir_to_llvm(&upir_mod, &ctx).expect("Should lower");
    let llvm_ir = llvm_mod.print_to_string().to_string();
    println!("{}", llvm_ir);

    assert!(llvm_ir.contains("define"));
    assert!(llvm_ir.contains("add"));
    assert!(llvm_ir.contains("add nsw i32"));
    assert!(llvm_ir.contains("ret i32"));
}