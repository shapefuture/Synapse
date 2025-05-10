use upir_core::ir::*;
use upir_core::types::*;
use upir_to_llvm::lower_upir_to_llvm;
use inkwell::context::Context;

#[test]
fn test_lower_add_fn() {
    // UPIR module: fn add(%a: i32, %b: i32) -> i32 { return core.add(%a, %b) }
    let tid = TypeId(1);
    let add_fn = Function {
        name: "add".to_string(),
        signature: FunctionSignature {
            arg_types: vec![tid, tid],
            result_types: vec![tid],
        },
        regions: vec![Region {
            blocks: vec![Block {
                id: BlockId(0),
                arguments: vec![
                    BlockArgument {
                        value_def: ValueDef { id: ValueId(0), ty: tid },
                        block_id: BlockId(0),
                        index: 0,
                    },
                    BlockArgument {
                        value_def: ValueDef { id: ValueId(1), ty: tid },
                        block_id: BlockId(0),
                        index: 1,
                    },
                ],
                operations: vec![
                    Operation {
                        name: "core.add".to_string(),
                        operands: vec![ValueId(0), ValueId(1)],
                        results: vec![ValueDef { id: ValueId(2), ty: tid }],
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
            }]
        }]
    };
    let upir_mod = Module {
        name: "simple".to_string(),
        functions: vec![add_fn],
    };
    let ctx = Context::create();
    let llvm_mod = lower_upir_to_llvm(&upir_mod, &ctx).expect("Should lower");
    let func = llvm_mod.get_function("add");
    assert!(func.is_some());
    let func = func.unwrap();
    assert_eq!(func.count_params(), 2);
    // Optionally, print the IR
    println!("{}", llvm_mod.print_to_string().to_string());
}