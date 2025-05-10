//! Implements the core 'compile' pipeline from source to executable.

use std::path::Path;
use std::process::Command;

use asg_core::AsgGraph;
use parser_core::parse_file;
use type_checker_l1::check_and_annotate_graph;
use asg_to_upir::lower_graph_to_upir;
use upir_core::ir::print_module;
use upir_to_llvm::lower_upir_to_llvm;
use inkwell::context::Context;

pub fn compile_to_executable(input: &Path, output: &Path, runtime_lib: &Path) -> anyhow::Result<()> {
    // 1. Parse
    let mut asg = parse_file(input)?;
    // 2. Type check
    let type_map = check_and_annotate_graph(&mut asg)?;
    // 3. Lower to UPIR
    let upir = lower_graph_to_upir(&asg, &type_map)?;
    // 4. Lower to LLVM
    let context = Context::create();
    let llvm = lower_upir_to_llvm(&upir, &context)?;
    // 5. Emit LLVM IR to temp file
    let ll_path = output.with_extension("ll");
    llvm.print_to_file(&ll_path)?;
    // 6. Compile LLVM IR to object (invoke clang)
    let obj_path = output.with_extension("o");
    Command::new("clang")
        .args(&["-c", ll_path.to_str().unwrap(), "-o", obj_path.to_str().unwrap()])
        .status()?;
    // 7. Link object with runtime to create executable
    Command::new("clang")
        .args(&[
            obj_path.to_str().unwrap(),
            runtime_lib.to_str().unwrap(),
            "-o",
            output.to_str().unwrap(),
        ])
        .status()?;
    Ok(())
}