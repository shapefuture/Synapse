use upir_core::ir::*;
use upir_core::types::*;

#[test]
fn test_print_minimal_module() {
    let mut builder = IRBuilder::new();
    let mut module = builder.module("simple");

    // Minimal types
    let tid_i32 = TypeId(1);
    // Add a function: add(i32, i32) -> i32
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
    module.functions.push(function);
    let output = print_module(&module);
    println!("{}", output);

    assert!(output.contains("func @add("));
    assert!(output.contains("core.add"));
    assert!(output.contains("func.return"));
}