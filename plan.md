# Synapse Implementation Plan (v3) - Detailed Steps

Objective: To implement the Synapse language, compiler, runtime, and tooling according to the "Revised Ultimate Proposal: Synapse (v3)".
Methodology: Iterative, bootstrap, formal-methods-driven, AI-assisted, community-focused.
Bootstrap Language Recommendation: Rust (due to performance, safety, strong ecosystem, and good tooling for parsing, LLVM, and potentially formal methods integration). Adapt commands if using OCaml/Haskell.
Tracking: Mark steps as completed by changing [ ] to [v] upon successful implementation and verification/testing. Use Git commit messages referencing the Task ID (e.g., "feat(P0T1): Define core operational semantics v0.1"). Log significant design decisions and challenges encountered in a DESIGN_LOG.md file.

Phase 0: Foundational Formalism & Core ASG
Goal: Establish the mathematical and structural bedrock of Synapse. This phase focuses on defining the precise meaning of the core language fragment and creating the fundamental data structures (ASG) and basic tools (parser, formatter, CLI) needed to represent and manipulate code according to this definition. Formalism here prevents ambiguity later.

Task P0T1: Formal Semantics (Core) Definition
Goal: Precisely define the syntax, typing rules, and evaluation rules for a minimal core subset of Synapse. This serves as the ultimate reference for correctness.
Input: Programming Language Theory Principles (specifically operational semantics, type theory).
Output:
Formal Semantics Specification Document (docs/semantics/core_v0.1.pdf or .tex).
Machine-checkable Proof Assistant files (proofs/semantics/core_v0.1.v or .thy or .k).
Instructions:
[v] Choose Formalism: Decide on the primary semantic style. Recommendation: Small-step Structural Operational Semantics (SOS). It's often easier to model fine-grained evaluation and concurrency later. Document this choice in DESIGN_LOG.md.
[v] Define Core Syntax (Abstract): Define the abstract syntax terms (metavariables: t for terms, v for values, τ for types, Γ for typing contexts, σ for stores/memory). Include:
t ::= x | λx:τ. t | t₁ t₂ | c | op(t₁, ..., tₙ) | ref t | !t | t₁ := t₂ | perform E t (variable, abstraction, application, constants c, primitive ops op, reference creation, dereference, assignment, basic effect perform)
v ::= λx:τ. t | c | loc (values: closures, constants, memory locations loc)
τ ::= Int | Bool | τ₁ -> τ₂ | Ref τ | ... (basic types: integers, booleans, function types, reference types)
E ::= 'EffectName (Effect identifiers)
[v] Define Judgments: Specify the key relations to be defined by rules:
Typing: Γ ⊢ t : τ (Term t has type τ in context Γ)
Evaluation (SOS): ⟨t, σ⟩ ⟶ ⟨t', σ'⟩ (Term t with store σ steps to term t' and store σ')
[v] Write Inference Rules:
[v] Define typing rules for each term construct (variable lookup, lambda abstraction, application, constants, primitive ops, ref/deref/assign, perform). Focus: Standard simply-typed lambda calculus rules initially, plus rules for state.
[v] Define evaluation rules (SOS). Include rules for function application (beta-reduction, potentially call-by-value), primitive operations, state operations (ref allocates new location, ! reads from store, := updates store), and context rules to allow evaluation within subterms.
[v] Best Practice (Formal Spec): Create the docs/semantics/ directory. Write the specification document using clear mathematical notation (LaTeX recommended). Define every symbol and judgment used. Explain the intuition behind the rules. Version the document (core_v0.1).
[v] Best Practice (Verification): Create the proofs/semantics/ directory. Begin formalizing the syntax, typing rules, and evaluation rules in your chosen proof assistant (Coq recommended).
[v] Define the inductive types for terms and types.
[v] Define the typing judgment (Inductive typing : context -> term -> type -> Prop := ...).
[v] Define the evaluation judgment (Inductive step : state -> term -> state -> term -> Prop := ...).
[v] Goal: State and attempt (even if manually) proofs for Type Preservation (if Γ ⊢ t : τ and ⟨t, σ⟩ ⟶ ⟨t', σ'⟩, then Γ ⊢ t' : τ) and Progress (if Γ ⊢ t : τ, then t is a value or there exists t', σ' such that ⟨t, σ⟩ ⟶ ⟨t', σ'⟩). These proofs are crucial for validating the semantics.
[v] Review Point: Have a peer (or use the AI agent's analytical capabilities) review the rules for internal consistency, completeness (covering all defined syntax), and correctness according to PL theory standards. Ensure the proof assistant formalization matches the document.
Task P0T2: ASG Schema Definition (v1)
Goal: Define the concrete data structure (Abstract Semantic Graph) that will represent Synapse code in memory and on disk, ensuring it can capture the essence of the formal semantics.
Input: Formal Semantics v0.1 (Task P0T1).
Output:
Formal ASG schema definition file (schemas/asg_schema_v1.proto).
Documentation explaining the schema (docs/asg_schema_v1_rationale.md).
Instructions:
[v] Choose Schema Language: Confirm Protocol Buffers v3. Create the schemas/ directory if it doesn't exist.
[v] Define Core Structure: In asg_schema_v1.proto (with syntax = "proto3"; and a package declaration like package synapse.asg.v1;):
[v] message AsgGraph { repeated AsgNode nodes = 1; uint64 root_node_id = 2; // Optional entry point }
[v] message AsgNode { uint64 node_id = 1; NodeType type = 2; oneof content { TermVariable term_variable = 3; TermLambda term_lambda = 4; ... ; Metadata metadata = 50; } } (Use unique field numbers).
[v] Define enum NodeType { NODE_TYPE_UNSPECIFIED = 0; TERM_VARIABLE = 1; ... ; METADATA = 50; }
[v] Map Semantics to Schema: Define messages corresponding to each construct in the formal semantics (P0T1):
[v] TermVariable { string name = 1; uint64 definition_node_id = 2; /* Link to lambda binder or let binding */ }
[v] TermLambda { uint64 binder_variable_node_id = 1; uint64 body_node_id = 2; uint64 type_annotation_id = 3; /* Optional link to TypeNode */ }
[v] TermApplication { uint64 function_node_id = 1; uint64 argument_node_id = 2; }
[v] LiteralInt { int64 value = 1; }, LiteralBool { bool value = 1; }
[v] PrimitiveOp { string op_name = 1; repeated uint64 argument_node_ids = 2; }
[v] TypeNode { uint64 node_id = 1; TypeKind type_kind = 2; oneof content { TypeInt type_int = 3; ... } } (Mirror type structure from P0T1)
[v] TermRef { uint64 init_value_node_id = 1; }, TermDeref { uint64 ref_node_id = 1; }, TermAssign { uint64 ref_node_id = 1; uint64 value_node_id = 2; }
[v] EffectPerform { string effect_name = 1; uint64 value_node_id = 2; /* Node ID of the value passed to the effect */ }
[v] ProofObligation { uint64 node_id = 1; string description = 2; uint64 related_code_node_id = 3; enum Status { STATUS_PENDING = 0; STATUS_DISCHARGED = 1; STATUS_FAILED = 2; } status = 4; } // Placeholder for linking proofs later
[v] Metadata { uint64 node_id = 1; SourceLocation source_location = 2; repeated uint64 annotation_ids = 3; }
[v] SourceLocation { string filename = 1; uint32 start_line = 2; uint32 start_col = 3; uint32 end_line = 4; uint32 end_col = 5; }
[v] Design Decisions: Use uint64 for node IDs consistently. Use direct node ID references to represent graph edges. Store nodes in a flat list within AsgGraph.
[v] Best Practice (Formal Spec): Add comments directly in the .proto file explaining the purpose of each message and field, and how it relates to the formal semantics.
[v] Best Practice (Documentation): Create docs/asg_schema_v1_rationale.md. Explain why the ASG is structured this way (e.g., flat structure for easier querying/mutation, explicit node IDs for graph representation). Discuss alternatives considered.
[v] Review Point: Cross-reference every construct in the formal semantics with its representation in the ASG schema. Ensure all necessary connections (like variable definitions) can be represented. Check for potential ambiguities.
Task P0T3: Core ASG Libraries Implementation
Goal: Create the foundational Rust code (the asg_core library) to build, manipulate, serialize, deserialize, and hash ASG instances based on the defined schema.
Input: ASG Schema v1 (asg_schema_v1.proto), Bootstrap Language (Rust).
Output: Rust library (asg_core) published locally (initially).
Instructions:
[v] Setup Project:
[v] If not done: git init synapse && cd synapse
[v] cargo new asg_core --lib
[v] Configure Dependencies (asg_core/Cargo.toml):
[v] Add protobuf crate (or prost and prost-build if using Prost).
[v] Add serde (with derive feature) for potential JSON debugging/alternative serialization.
[v] Add a hashing crate: blake3 (recommended for speed) or sha2.
[v] If using prost-build, add it to [build-dependencies].
[v] Generate Rust Code from Schema:
[v] If using protobuf crate: Use protoc --rust_out asg_core/src/ schemas/asg_schema_v1.proto (requires protoc compiler installed).
[v] If using prost: Create a build.rs file in asg_core/ and use prost_build::compile_protos(&["../schemas/asg_schema_v1.proto"], &["../schemas/"])?;. Configure OUT_DIR mapping in src/lib.rs.
[v] Verify generated Rust structs/enums in asg_core/src/.
[v] Implement Core Graph Structure (asg_core/src/graph.rs):
[v] pub struct AsgGraph { nodes: std::collections::HashMap<u64, AsgNode>, next_id: std::sync::atomic::AtomicU64, // For generating unique IDs }
[v] Implement impl AsgGraph { ... } with methods:
[v] pub fn new() -> Self
[v] fn generate_id(&self) -> u64 (handles incrementing next_id safely).
[v] pub fn add_node(&mut self, content: AsgNodeContent) -> u64 (Generates ID, creates AsgNode, inserts into map).
[v] pub fn get_node(&self, node_id: u64) -> Option<&AsgNode>
[v] pub fn get_node_mut(&mut self, node_id: u64) -> Option<&mut AsgNode>
[v] Consider helper methods like get_lambda(&self, node_id: u64) -> Option<&TermLambda>, etc.
[v] Implement Serialization/Deserialization (asg_core/src/serde.rs):
[v] pub fn save_asg_binary<P: AsRef<std::path::Path>>(graph: &AsgGraph, path: P) -> Result<(), Error> (Use protobuf::Message::write_to_bytes or prost::Message::encode).
[v] pub fn load_asg_binary<P: AsRef<std::path::Path>>(path: P) -> Result<AsgGraph, Error> (Use protobuf::Message::parse_from_bytes or prost::Message::decode).
[v] (Optional) Implement JSON serialization/deserialization using serde and serde_json for debugging purposes. Derive Serialize, Deserialize on generated structs if possible (may require manual impls or prost features).
[v] Implement Content Addressing (asg_core/src/hash.rs):
[v] Define a HashDigest type alias (e.g., pub type HashDigest = [u8; 32]; for BLAKE3/SHA256).
[v] fn canonicalize_node(node: &AsgNode) -> Vec<u8>: Define a stable binary serialization specifically for hashing. Crucial: Must ignore volatile fields like node_id itself if not part of content, and handle map iteration order if hashing the whole graph. Hashing individual nodes based on their content and direct children's hashes is often more robust. Decide on a strategy and document it.
[v] pub fn hash_node(node: &AsgNode) -> HashDigest: Hash the output of canonicalize_node.
[v] Consider how to hash the entire graph consistently.
[v] Best Practice (Modularity): Create modules: graph.rs, nodes.rs (or use generated file), serde.rs, hash.rs, error.rs. Define custom error types.
[v] Best Practice (TDD/Testing): Create asg_core/tests/integration_tests.rs:
[v] Test node creation, retrieval, modification.
[v] Test serialization/deserialization round trip (binary and optional JSON). Check that loading a saved graph results in an identical structure.
[v] Test hashing: ensure hashing the same node content yields the same hash; ensure minor changes result in different hashes.
[v] Best Practice (Documentation): Add Rustdoc comments (///) explaining the purpose of structs, functions, and modules. Explain the hashing strategy clearly.
[v] Review Point: Check API design, serialization correctness, hashing stability and strategy. Ensure tests provide good coverage.
Task P0T4: Minimal Parser & Pretty Printer Implementation
Goal: Create tools to translate between a human-readable minimal text format and the ASG, enabling basic code input and inspection.
Input: Core Semantics (P0T1), asg_core library (P0T3).
Output: Rust libraries: parser_core, formatter_core.
Instructions:
[v] Setup Projects:
[v] cargo new parser_core --lib
[v] cargo new formatter_core --lib
[v] Configure Dependencies: Add asg_core as a path dependency in both Cargo.toml files (e.g., asg_core = { path = "../asg_core" }).
Parser (parser_core):
[v] Choose Parsing Library: Recommendation: lalrpop. It requires a separate grammar file and integrates well with build scripts. Add lalrpop to [build-dependencies] and lalrpop-util to [dependencies].
[v] Define Concrete Syntax: Create parser_core/src/core_syntax.lalrpop. Define terminals (keywords like lambda, ref, operators) and non-terminals corresponding to semantic constructs (e.g., Term, Expr, Type). Use an S-expression like syntax for simplicity:
[v] Define Intermediate AST: Create parser_core/src/ast.rs. Define simple structs/enums that the parser actions in the .lalrpop file can easily build before constructing the complex AsgGraph. This decouples parsing logic from AsgGraph details.
[v] Implement LALRPOP Actions: Write Rust code within the .lalrpop actions (=> { ... }) to build the intermediate AST defined above.
[v] Implement AST -> ASG Conversion: Write a function pub fn build_asg(input_ast: ast::Root) -> Result<asg_core::AsgGraph, BuildError> in parser_core/src/lib.rs. This function traverses the intermediate AST and uses the asg_core library functions (graph.add_node(...)) to construct the final AsgGraph. Handle scoping and variable definition linking here.
[v] Configure Build Script: Create parser_core/build.rs to process the .lalrpop file.
[v] Error Handling: lalrpop provides error types. Map these to your custom ParseError type, including location information.
[v] Best Practice (Security): LALRPOP helps prevent some ambiguities. Be mindful of potential resource exhaustion (e.g., stack overflow) on deeply nested input during AST -> ASG conversion.
[v] Best Practice (TDD/Testing): In parser_core/tests/, write tests that:
Parse valid code snippets and check the resulting AsgGraph structure (or intermediate AST).
Parse invalid code snippets and verify that the correct ParseError is returned with accurate location info.
Formatter (formatter_core):
[v] Implement Core Logic (formatter_core/src/lib.rs):
[v] pub fn format_asg(graph: &asg_core::AsgGraph, root_id: u64) -> Result<String, FormatError>
[v] Implement a recursive function that traverses the ASG starting from root_id.
[v] Use a PrettyPrinter helper struct to manage indentation levels and the output string buffer.
[v] Pattern match on AsgNode content and recursively call formatting for child nodes, printing parentheses, keywords, and literals according to the defined minimal concrete syntax.
[v] Handle potential cycles if the graph allows them (though the core language shouldn't have term cycles).
[v] Best Practice (TDD/Testing): In formatter_core/tests/:
Create AsgGraph instances programmatically.
Format them and assert the output matches the expected string exactly.
Round-trip Test: Combine with parser_core. Parse a string -> Format the resulting ASG -> Assert the output string matches the original input string (or a canonical formatted version).
[v] Best Practice (Documentation): Document the concrete syntax expected by the parser and generated by the formatter. Explain the AST -> ASG conversion logic.
[v] Review Point: Check parser correctness and error reporting. Verify formatter output matches the defined syntax. Ensure round-trip tests pass.
Task P0T5: Basic CLI & Linter (Level 0) Implementation
Goal: Create the main user interface tool (synapse_cli) integrating the parser, formatter, and adding basic (Level 0) structural validation of the ASG.
Input: asg_core, parser_core, formatter_core libraries.
Output: Executable binary (target/debug/synapse_cli).
Instructions:
[ ] Setup Project:
[ ] cargo new synapse_cli
[ ] Configure Dependencies (synapse_cli/Cargo.toml):
[ ] Add asg_core, parser_core, formatter_core as path dependencies.
[ ] Add clap (with derive feature) for CLI argument parsing.
[ ] Add thiserror or anyhow for easier error handling in the application.
[ ] Define CLI Interface (synapse_cli/src/main.rs):
[ ] Use clap::Parser derive macro to define the main Cli struct and subcommands (Commands enum).
[ ] Define subcommands:
Parse { input_file: std::path::PathBuf }
Format { input_file: std::path::PathBuf, #[clap(short, long)] output_file: Option<std::path::PathBuf> }
Lint { input_file: std::path::PathBuf }
DumpAsg { input_file: std::path::PathBuf, #[clap(long, arg_enum, default_value = "binary")] format: DumpFormat } (Define DumpFormat enum: Binary, Json).
[ ] Implement Subcommand Logic:
[ ] Parse: Call parser_core::parse_file(input_file), report success or display formatted error.
[ ] Format: Parse file, call formatter_core::format_asg, print to stdout or save to output file. Handle errors.
[ ] DumpAsg: Parse file, call asg_core::save_asg_binary or the JSON equivalent, write to stdout.
[ ] Lint:
Parse file into an AsgGraph.
Implement the lint_graph(graph: &AsgGraph) -> Vec<LintError> function in a new synapse_cli/src/linter.rs module.
Level 0 Checks:
[ ] Traverse the graph.
[ ] Check for graph structural integrity (e.g., node IDs referenced actually exist in the graph map).
[ ] Perform basic scope checking: Ensure TermVariable nodes link to a valid binder (e.g., TermLambda binder) within an enclosing scope. This requires building a symbol table during traversal.
[ ] Check for simple type mismatches detectable without full inference (e.g., applying a non-lambda value, assigning to a non-ref).
Define LintError struct including error code (e.g., L001), message, relevant node_id, and potentially SourceLocation (requires mapping node IDs back to source - store location in ASG Metadata).
Report lint errors found.
[ ] Implement Main Function: Parse CLI args using Cli::parse(), match on the subcommand, call the corresponding logic, handle errors gracefully (print user-friendly messages).
[ ] Best Practice (TDD/Testing): Create integration tests (e.g., in synapse_cli/tests/cli_tests.rs) using test files:
[ ] Test each subcommand with valid inputs.
[ ] Test error handling for non-existent files, parse errors, lint errors.
[ ] Test output formats (format, dump-asg).
[ ] Best Practice (CI): Create a GitHub Actions workflow (.github/workflows/ci.yml):
[ ] Trigger on push/pull_request.
[ ] Setup Rust environment.
[ ] Run cargo fmt --check --all.
[ ] Run cargo clippy --all-targets -- -D warnings.
[ ] Run cargo build --all-targets --release.
[ ] Run cargo test --all-targets.
[ ] Best Practice (Documentation): Create a README.md for synapse_cli explaining usage and commands.
[ ] Review Point: Verify CLI commands work as expected. Check linter logic for correctness on basic cases. Confirm CI pipeline passes. Ensure user-facing errors are helpful.
(End of Phase 0)

Proceed to Phase 1 once all Phase 0 tasks are reviewed and marked complete ([v]).
Okay, here is the detailed plan for Phase 1, continuing from the end of Phase 0.

Synapse Implementation Plan (v3) - Detailed Steps (Continued)
Previous Phase: Phase 0: Foundational Formalism & Core ASG
Current Phase: Phase 1: Basic Type Checking & Compilation

Goal: Implement a minimal but functional end-to-end compiler pipeline. This involves adding type checking (Level 1) to the ASG, defining a suitable Intermediate Representation (UPIR), translating ASG to UPIR, translating UPIR to executable code (via LLVM), and creating the necessary minimal runtime support.

Phase 1: Basic Type Checking & Compilation
Task P1T1: Type System Implementation (Level 1 - Basic)
Goal: Implement a Hindley-Milner style type checker (Algorithm W or similar) operating on the ASG, inferring types for expressions and functions, and annotating the ASG with this type information.
Input: Formal Semantics v0.1 (P0T1), asg_core library.
Output: Rust library (type_checker_l1), Updated ASG Schema, ASG instances annotated with type information.
Instructions:
[ ] Setup Project:
[ ] cargo new type_checker_l1 --lib
[ ] Add asg_core = { path = "../asg_core" } to dependencies. Add crates for managing unification if needed (e.g., unification).
[ ] Update ASG Schema (schemas/asg_schema_v1.proto -> v1.1):
[ ] Increment package version if desired (e.g., package synapse.asg.v1_1;).
[ ] Add a dedicated TypeId message or alias (uint64) if not already cleanly defined.
[ ] Add TypeId inferred_type_id = NN; field to AsgNode content messages where a type is meaningful (e.g., TermApplication, TermVariable, TermLambda binder, LiteralInt, etc.). Ensure NN is a new, unused field number.
[ ] Define TypeNode structure within the schema if not done in P0T2, mirroring the formal semantics types:
[ ] Regenerate Rust code for asg_core from the updated schema. Fix any resulting compile errors in asg_core.
[ ] Implement Core Type Structures (type_checker_l1/src/types.rs):
[ ] Define internal Rust representations for types (enum Type { Int, Bool, Function(Box<Type>, Box<Type>), Var(u64), Ref(Box<Type>) }). These are used during inference.
[ ] Define type schemes (struct TypeScheme { vars: Vec<u64>, body: Type }) for polymorphism (let-generalization).
[ ] Implement Unification Algorithm (type_checker_l1/src/unification.rs):
[ ] Implement the unify(type1: &Type, type2: &Type, substitutions: &mut SubstitutionMap) -> Result<(), UnificationError>.
[ ] Handle recursive unification for function types, ref types, etc.
[ ] Implement the occurs check to prevent infinite types (Var(a) unifying with Function(Var(a), ...)).
[ ] Use a substitution map (type SubstitutionMap = HashMap<u64, Type>) to track solved unification variables.
[ ] Implement Type Inference Algorithm (type_checker_l1/src/inference.rs):
[ ] Implement Algorithm W or a similar bottom-up/constraint-based approach.
[ ] Define a TypingContext struct to map ASG variable node IDs (uint64) to TypeSchemes.
[ ] Implement infer(context: &TypingContext, node_id: u64, graph: &AsgGraph, substitutions: &mut SubstitutionMap) -> Result<Type, TypeError>.
[ ] Handle generalization (let bindings or function definitions) and instantiation (variable usage, function application).
[ ] Generate fresh unification variables (Type::Var(fresh_id)) as needed.
[ ] Implement Main Checker Function (type_checker_l1/src/lib.rs):
[ ] pub fn check_and_annotate_graph(graph: &mut AsgGraph) -> Result<(), Vec<TypeError>>.
[ ] Initialize an empty TypingContext and SubstitutionMap.
[ ] Perform a topological sort or suitable traversal of the ASG (handle function definitions before calls if applicable, manage scopes).
[ ] Call infer for expression nodes.
[ ] After successful inference for a node, apply the current substitutions to the inferred type to get the final concrete type (or type with generalized variables).
[ ] ASG Annotation: Create corresponding TypeNodes in the AsgGraph for the final types and link them to the expression nodes using the inferred_type_id field added to the schema. Add the TypeNodes to the graph's node map.
[ ] Error Handling: Define TypeError enum/struct with variants like UnificationFail, OccursCheck, UndefinedVariable, ApplicationMismatch, etc. Include relevant ASG node IDs and source locations (if available from metadata).
[ ] Best Practice (TDD/Testing): Create extensive unit tests in type_checker_l1/tests/:
[ ] Test basic type inference (literals, primitives).
[ ] Test function types (lambda, application).
[ ] Test basic let-polymorphism.
[ ] Test inference involving ref, !, :=.
[ ] Test all expected TypeError conditions.
[ ] Test that the ASG is correctly annotated with TypeNode IDs after checking.
[ ] Best Practice (Formal Spec Alignment): Ensure the implemented unification and inference rules align with the formal typing rules from P0T1. Document any necessary deviations or extensions.
[ ] Integrate with CLI: Add a check subcommand to synapse_cli that calls check_and_annotate_graph. The compile command (to be added later) should also run this check first.
[ ] Review Point: Verify correctness against tests. Check the quality and accuracy of type error messages. Ensure ASG annotation works correctly. Assess alignment with formal semantics.
Task P1T2: UPIR Definition (Core Dialects)
Goal: Define the structure and core dialects of the Universal Polymorphic Intermediate Representation (UPIR), inspired by MLIR, to serve as the interface between the language frontend and various backends.
Input: Compiler Design Principles (MLIR), Core Language Semantics (P0T1).
Output:
UPIR Core Specification Document (docs/upir/core_spec_v1.md).
Rust library (upir_core) providing UPIR data structures and manipulation functions.
Instructions:
[ ] Setup Project:
[ ] cargo new upir_core --lib
[ ] Define Core IR Concepts (upir_core/src/ir.rs):
[ ] struct Module { name: String, functions: Vec<Function>, ... }
[ ] struct Function { name: String, signature: FunctionSignature, regions: Vec<Region>, ... }
[ ] struct Region { blocks: Vec<Block>, ... } (MLIR concept, often a function body is one region)
[ ] struct Block { id: BlockId, arguments: Vec<BlockArgument>, operations: Vec<Operation>, ... }
[ ] struct Operation { name: String, /* e.g., "core.add" */ operands: Vec<ValueId>, results: Vec<ValueDef>, attributes: HashMap<String, Attribute>, regions: Vec<Region>, ... }
[ ] struct ValueDef { id: ValueId, ty: TypeId } (Represents SSA value definitions)
[ ] struct BlockArgument { value_def: ValueDef, block_id: BlockId, index: usize }
[ ] Define ValueId, BlockId, TypeId etc. as distinct types (e.g., tuple structs wrapping u64).
[ ] Define Core Types (upir_core/src/types.rs):
[ ] struct Type { id: TypeId, kind: TypeKind }
[ ] enum TypeKind { Builtin(BuiltinType), Ptr { element_type: TypeId }, Struct { field_types: Vec<TypeId> }, ... }
[ ] enum BuiltinType { I32, I64, F32, F64, Bool, Void, ... }
[ ] Define Attributes (upir_core/src/attributes.rs):
[ ] enum Attribute { String(String), Integer(i64), Bool(bool), Type(TypeId), ... } (Used for constants, annotations on ops)
[ ] Specify Core Dialects (docs/upir/core_spec_v1.md and upir_core/src/dialects/):
[ ] builtin Dialect:
Ops: module, func (defines structure).
Types: Core types like i32, f64, etc.
[ ] core Dialect: (Focus on pure computation)
Ops: core.constant value: Attribute -> result: Type
Ops: core.add lhs: ValueId, rhs: ValueId -> result: Type
Ops: core.sub, core.mul, core.div_s, core.div_u, core.rem_s, core.rem_u
Ops: core.and, core.or, core.xor
Ops: core.cmp predicate: CmpPredicate, lhs: ValueId, rhs: ValueId -> result: Bool (Define CmpPredicate enum: EQ, NE, SLT, SGT, etc.)
[ ] mem Dialect: (Memory access)
Ops: mem.alloc type: Type -> result: Ptr (Stack allocation typically)
Ops: mem.load ptr: ValueId -> result: Type
Ops: mem.store ptr: ValueId, value: ValueId
[ ] cf Dialect: (Control Flow)
Ops: cf.br dest: BlockId (operands: Vec<ValueId>) (Unconditional branch, pass args to block)
Ops: cf.cond_br cond: ValueId, true_dest: BlockId (true_operands: Vec<ValueId>), false_dest: BlockId (false_operands: Vec<ValueId>) (Conditional branch)
[ ] func Dialect: (Reusing builtin.func for definition)
Ops: func.call callee: SymbolRefAttr, args: Vec<ValueId> -> results: Vec
Ops: func.return operands: Vec<ValueId>
[ ] Best Practice (Formal Spec): For each operation, formally define its name, arguments (name, type constraint), results (name, type constraint), attributes, and a clear description of its semantics in core_spec_v1.md. Use MLIR syntax for textual representation examples.
[ ] Implement Rust Representation: Create Rust structs/enums mirroring the spec in upir_core/src/ir.rs, types.rs, attributes.rs, and potentially dialect-specific files in src/dialects/.
[ ] Implement UPIR Builder API: Provide functions/methods to programmatically construct UPIR Modules, Functions, Blocks, and Operations in a safe way (e.g., ensuring correct block termination, SSA value usage). An IRBuilder struct similar to LLVM's can be helpful.
[ ] Implement UPIR Pretty-Printer: Create fn print_module(module: &Module) -> String that generates the defined textual representation. This is critical for debugging.
[ ] Best Practice (Modularity): Keep core IR structures separate from dialect definitions where possible.
[ ] Best Practice (TDD/Testing): Write unit tests for:
[ ] Creating and manipulating valid UPIR structures using the builder API.
[ ] Ensuring the pretty-printer produces correct textual output matching the spec examples.
[ ] Basic validation checks (e.g., ops belong to loaded dialects, block terminators are correct).
[ ] Review Point: Check the UPIR specification for clarity, completeness, and suitability for representing the core language and targeting LLVM. Review the Rust implementation for correctness and API usability. Verify the pretty-printer output.
Task P1T3: ASG-to-UPIR Lowering (Core)
Goal: Implement the compiler stage that translates the type-checked ASG into the newly defined UPIR Core dialects.
Input: Type-checked ASG (from P1T1), UPIR Core Spec & library (P1T2).
Output: Rust library/compiler stage (asg_to_upir).
Instructions:
[ ] Setup Project:
[ ] cargo new asg_to_upir --lib
[ ] Add dependencies: asg_core = { path = "../asg_core" }, type_checker_l1 = { path = "../type_checker_l1" }, upir_core = { path = "../upir_core" }. Use thiserror for custom errors.
[ ] Implement Lowering Context:
[ ] struct LoweringContext<'a> { graph: &'a AsgGraph, upir_module: upir_core::Module, builder: upir_core::IRBuilder, // If using a builder pattern scope_stack: Vec<HashMap<u64, upir_core::ValueId>>, // Maps ASG var node IDs to UPIR SSA values type_map: HashMap<u64, upir_core::TypeId>, // Maps ASG type node IDs to UPIR type IDs current_function: Option<upir_core::FunctionBuilder>, // Or similar current_block: Option<upir_core::BlockId>, ... }
[ ] Implement Main Lowering Function:
[ ] pub fn lower_graph_to_upir(graph: &AsgGraph) -> Result<upir_core::Module, LoweringError>
[ ] Initialize LoweringContext.
[ ] Iterate through the ASG (likely starting with function definitions).
[ ] Call helper functions (e.g., lower_function, lower_expression_node).
[ ] Implement Expression Lowering:
[ ] fn lower_expression_node(&mut self, node_id: u64) -> Result<upir_core::ValueId, LoweringError> (returns the UPIR SSA value representing the result of the expression).
[ ] Use pattern matching on the AsgNode content:
TermVariable: Look up in scope_stack.
LiteralInt/Bool: Create core.constant op.
TermApplication: Recursively lower function and argument. Generate func.call op. Map argument/return types using type_map.
PrimitiveOp: Recursively lower arguments. Generate corresponding core.* op (e.g., core.add).
TermRef: Lower initial value. Generate mem.alloc. Generate mem.store. Return the pointer ValueId.
TermDeref: Lower pointer expression. Generate mem.load.
TermAssign: Lower pointer and value expressions. Generate mem.store. Assignment typically doesn't return a meaningful value in functional style (or returns void/unit).
TermLambda: This defines a function. Call lower_function. Represent the closure (if needed later) potentially as a struct containing code pointer and environment pointer. For now, assume top-level functions.
[ ] Implement Function Lowering:
[ ] fn lower_function(&mut self, lambda_node_id: u64) -> Result<(), LoweringError>
[ ] Create a new builtin.func in the UPIR module.
[ ] Define function signature based on ASG type annotations (use type_map).
[ ] Create the entry Block. Add block arguments corresponding to lambda parameters. Update scope_stack.
[ ] Recursively lower the function body expression (TermLambda.body_node_id).
[ ] Ensure the function's blocks are properly terminated (e.g., with func.return using the result of lowering the body).
[ ] Handle Control Flow (Basic If/Then/Else):
[ ] Represent if cond then true_expr else false_expr in ASG (needs a specific TermIf node type added in Phase 0/1).
[ ] Lower cond expression to a boolean ValueId.
[ ] Create three new blocks: then_block, else_block, merge_block.
[ ] End current block with cf.cond_br cond, then_block, else_block.
[ ] Lower true_expr within then_block, end with cf.br merge_block (result_true).
[ ] Lower false_expr within else_block, end with cf.br merge_block (result_false).
[ ] Define merge_block with a block argument to receive the result. The type comes from the TermIf node's inferred type. The SSA value for the 'if' expression result is this block argument.
[ ] SSA Form: Ensure that every UPIR value (ValueId) has exactly one definition (result of an op or a block argument) and is defined before use. The builder pattern often helps manage this.
[ ] Error Handling: Define LoweringError enum.
[ ] Best Practice (TDD/Testing): In asg_to_upir/tests/:
[ ] Create sample ASGs (programmatically or parsed from minimal syntax). Include functions, variables, refs, basic arithmetic, and if/then/else.
[ ] Run the lowering pass.
[ ] Pretty-print the resulting UPIR module.
[ ] Assert that the printed UPIR matches the expected output string exactly. This validates the translation logic.
[ ] Test error conditions (e.g., lowering ill-typed ASG, though type checker should prevent this).
[ ] Integrate with CLI: Add this pass to the synapse_cli compile flow, after type checking. Add a --emit=upir flag to stop compilation here and print the UPIR text.
[ ] Review Point: Check the translation logic for all core ASG constructs. Verify SSA form is correctly generated. Examine handling of variables, scopes, and control flow. Ensure tests cover main cases.
Task P1T4: UPIR-to-LLVM Lowering (Core)
Goal: Implement the compiler backend stage that translates the core UPIR dialects into LLVM Intermediate Representation (IR).
Input: UPIR Core representation (P1T2/P1T3), LLVM API Bindings.
Output: Rust library/compiler stage (upir_to_llvm), LLVM IR Module.
Instructions:
[ ] Setup Project:
[ ] cargo new upir_to_llvm --lib
[ ] Add upir_core = { path = "../upir_core" } dependency.
[ ] Add LLVM bindings: inkwell crate is recommended for a safer API over llvm-sys. Configure inkwell features if needed (e.g., llvm14-0). Ensure LLVM development libraries are installed on the system.
[ ] Implement LLVM Lowering Context:
[ ] struct LlvmLoweringContext<'ctx> { llvm_context: &'ctx inkwell::context::Context, llvm_module: inkwell::module::Module<'ctx>, builder: inkwell::builder::Builder<'ctx>, upir_module: &'a upir_core::Module, // Reference to the input UPIR value_map: HashMap<upir_core::ValueId, inkwell::values::BasicValueEnum<'ctx>>, // Maps UPIR SSA values to LLVM values type_map: HashMap<upir_core::TypeId, inkwell::types::BasicTypeEnum<'ctx>>, block_map: HashMap<upir_core::BlockId, inkwell::basic_block::BasicBlock<'ctx>>, function_map: HashMap<String, inkwell::values::FunctionValue<'ctx>>, ... }
[ ] Implement Main Lowering Function:
[ ] pub fn lower_upir_to_llvm<'ctx>(upir_module: &upir_core::Module, llvm_context: &'ctx inkwell::context::Context) -> Result<inkwell::module::Module<'ctx>, LlvmLoweringError>
[ ] Create llvm_context, llvm_module, builder.
[ ] Initialize LlvmLoweringContext.
[ ] Iterate through UPIR functions first to declare them in LLVM (handling signatures).
[ ] Then iterate again to lower function bodies (blocks and operations).
[ ] Implement Type Lowering:
[ ] fn lower_type(&mut self, upir_type_id: upir_core::TypeId) -> inkwell::types::BasicTypeEnum<'ctx> (Map UPIR builtins, pointers, etc., to corresponding llvm_context.i32_type(), ptr_type(), etc.) Cache results in type_map.
[ ] Implement Function Lowering:
[ ] Declare LLVM function using llvm_module.add_function. Lower signature types. Store in function_map.
[ ] For each function body:
[ ] Iterate through UPIR blocks, create corresponding LLVM BasicBlocks using llvm_context.append_basic_block. Store in block_map.
[ ] Iterate through UPIR blocks again to lower operations.
[ ] Position the builder at the end of the corresponding LLVM block using builder.position_at_end().
[ ] Lower block arguments (map UPIR args to LLVM function params for entry block, or PHI nodes for other blocks - initially only handle entry block args).
[ ] Lower each operation within the block.
[ ] Implement Operation Lowering:
[ ] fn lower_operation(&mut self, op: &upir_core::Operation)
[ ] Match on op.name (e.g., "core.add", "mem.load", "cf.cond_br"):
Look up operand ValueIds in value_map to get LLVM values.
Use the builder to create corresponding LLVM instructions (e.g., builder.build_int_add, builder.build_load, builder.build_conditional_branch).
Map result ValueIds to the resulting LLVM values in value_map.
Handle terminators (cf.br, cf.cond_br, func.return) correctly using builder.build_.... Ensure blocks are properly terminated.
For func.call, look up the function in function_map and use builder.build_call.
[ ] Error Handling: Define LlvmLoweringError.
[ ] Best Practice (TDD/Testing): In upir_to_llvm/tests/:
[ ] Create sample UPIR modules programmatically (representing simple functions, arithmetic, memory access, branches).
[ ] Run the UPIR-to-LLVM lowering pass.
[ ] Get the textual representation of the generated LLVM IR using llvm_module.print_to_string().
[ ] Assert that the LLVM IR string matches the expected output.
[ ] Advanced: Use llvm_module.verify() to check LLVM's own validation rules. Optionally, use llc to compile the IR to assembly and lli to execute it for simple test cases.
[ ] Integrate with CLI: Add this pass to the synapse_cli compile flow after ASG-to-UPIR. Add an --emit=llvm-ir flag.
[ ] Review Point: Check the mapping from UPIR types and ops to LLVM types and instructions. Verify handling of SSA values and control flow. Check test coverage and LLVM IR validity.
Task P1T5: Minimal Runtime Implementation (Core)
Goal: Create a tiny runtime library providing essential services needed by the compiled code, like memory allocation and basic I/O for testing.
Input: Compiler Design, Target Platform ABI.
Output: Static library (target/debug/libsynapse_runtime.a).
Instructions:
[ ] Setup Project:
[ ] cargo new synapse_runtime --lib
[ ] Modify synapse_runtime/Cargo.toml to build a static library: [lib]\ncrate-type = ["staticlib"]
[ ] Implement Memory Allocation (synapse_runtime/src/alloc.rs):
[ ] #[no_mangle] pub extern "C" fn synapse_alloc(size: usize) -> *mut u8 { ... } (Use libc::malloc initially).
[ ] #[no_mangle] pub extern "C" fn synapse_free(ptr: *mut u8) { ... } (Use libc::free).
[ ] Note: This is very basic. Phase 2 (Quantitative Types) will aim to eliminate the need for explicit runtime alloc/free in many cases.
[ ] Implement Basic I/O (synapse_runtime/src/io.rs):
[ ] #[no_mangle] pub extern "C" fn synapse_print_int(value: i64) { println!("{}", value); } (Simple print for testing).
[ ] Define Entry Point (synapse_runtime/src/main_glue.rs - Optional but helpful):
[ ] Define the actual C main function if needed, which might perform runtime setup and then call the main function compiled from Synapse code.
[ ] Declare the expected signature of the Synapse main function (e.g., extern "C" { fn synapse_main() -> i32; }).
[ ] #[no_mangle] pub extern "C" fn main() -> i32 { // Setup if needed... unsafe { synapse_main() } }
[ ] Expose Runtime Functions in synapse_runtime/src/lib.rs: Ensure necessary modules are public.
[ ] Best Practice (Build): Ensure the static library libsynapse_runtime.a is built correctly.
[ ] Best Practice (Testing): Basic unit tests within the runtime crate might be possible, but integration testing via compiled Synapse code (P1T6) is more crucial.
[ ] Review Point: Check function signatures (extern "C", #[no_mangle]) for C ABI compatibility. Verify basic implementations.
Task P1T6: End-to-End Pipeline Integration & Testing
Goal: Connect all the compiler stages (Parse -> Lint -> TypeCheck -> ASG->UPIR -> UPIR->LLVM) within the synapse_cli, link the generated code with the runtime, and create end-to-end tests.
Input: All previously implemented stages (parser_core, type_checker_l1, asg_to_upir, upir_to_llvm, synapse_runtime).
Output: Updated synapse_cli capable of compiling simple programs to executables. End-to-end test suite.
Instructions:
[ ] Update synapse_cli Dependencies: Ensure Cargo.toml includes all stage crates as path dependencies.
[ ] Implement compile Command Logic:
[ ] Add a Compile { input_file: PathBuf, #[clap(short, long)] output_file: Option<PathBuf> } subcommand.
[ ] Define the compilation pipeline function:
Parse file -> AsgGraph (parser_core).
Run Linter (optional, maybe integrated with type check).
Type Check & Annotate -> AsgGraph (type_checker_l1). Handle errors.
Lower ASG -> UPIR -> UpirModule (asg_to_upir). Handle errors.
Lower UPIR -> LLVM -> LlvmModule (upir_to_llvm). Handle errors.
Emit LLVM IR/Bitcode: Save LLVM IR text (--emit=llvm-ir) or bitcode (--emit=llvm-bc) if requested. Use llvm_module.print_to_file or llvm_module.write_bitcode_to_path.
Generate Object File: Use LLVM tools/APIs to compile LLVM IR/BC to an object file (.o). Need to determine target triple. Use inkwell::targets::{InitializationConfig, Target, TargetMachine}. target_machine.write_to_file(&llvm_module, FileType::Object, &obj_path)?;
Link Object File with Runtime: Invoke a linker (like cc or clang) programmatically (using std::process::Command). Link the generated object file (.o) with the runtime static library (libsynapse_runtime.a). Specify the output executable name. Handle linker errors.
[ ] Error Handling: Ensure errors from any stage are propagated and reported clearly to the user.
[ ] Create End-to-End Test Suite (synapse_cli/tests/e2e/):
[ ] Create simple Synapse source files (.syn) in the minimal syntax (e.g., add.syn, ref_test.syn, if_test.syn). These should use features like printing integers from the runtime.
[ ] Write a test runner script (or Rust test function) that:
Calls synapse_cli compile <test_file.syn> -o <test_executable>.
Asserts that compilation succeeds.
Runs the compiled <test_executable>.
Captures the standard output of the executable.
Asserts that the output matches the expected output for that test case.
[ ] Best Practice (CI): Add a job to the ci.yml workflow to run these end-to-end tests after the build and unit tests pass. This requires the runtime and compiler to be built first.
[ ] Best Practice (Documentation): Update synapse_cli/README.md to explain the compile command and emission flags.
[ ] Review Point: Test the compile command thoroughly. Verify generated executables run correctly for simple cases. Check error propagation through the pipeline. Ensure E2E tests pass in CI.
Task P1T7: Verification of Type Checker (Core Soundness)
Goal: Formally verify the soundness of the core Level 1 type checker logic against the core formal semantics using a proof assistant. This is a foundational step for the "Verifiability First" principle.
Input: Formal Semantics v0.1 + Proofs (P0T1), type_checker_l1 implementation (specifically the inference/unification logic).
Output: Formal proof of soundness for the core type checking algorithm (proofs/semantics/type_checker_l1_soundness.v or similar).
Instructions:
[ ] Model Checker Logic: Abstract the core logic of the implemented type checker (unification, constraint generation, context lookup) within the proof assistant used in P0T1 (e.g., Coq). This might not be a line-by-line translation but should capture the algorithmic essence.
[ ] Formalize Soundness Statement: State the Type Preservation theorem formally within the proof assistant, relating the modeled type checking judgment to the semantic evaluation judgment defined in P0T1. (e.g., forall t ctx sigma sigma' t', WFContext(ctx) -> type_check(ctx, t, T) -> step(sigma, t, sigma', t') -> exists T', type_check(ctx, t', T') /\ Relation(T, T')). The exact statement depends on the chosen formalism.
[ ] Develop Proof: Interactively develop the proof using tactics within the proof assistant. This typically involves induction on the typing derivation or the evaluation steps.
[ ] Document Proof: Add comments within the proof script explaining the high-level steps and strategy. Reference the relevant parts of the implementation being modeled.
[ ] Best Practice (Verification): Focus initially on proving soundness for the core calculus without complex polymorphism or effects, mirroring the minimal formal semantics. Extend the proof as the semantics and checker evolve.
[ ] Best Practice (CI): Add a step to the CI workflow to compile/check the proof assistant files (*.v, *.thy, etc.) to ensure the proofs remain valid as the semantics/checker model evolves.
[ ] Review Point: Check the formal statement of soundness for correctness. Review the proof structure and key arguments. Ensure the proof assistant accepts the proof.
(End of Phase 1)

Proceed to Phase 2 once all Phase 1 tasks are reviewed and marked complete ([v]). This phase delivers the first minimally viable compiler.

Okay, here is the detailed plan for Phase 2, focusing on adding advanced type system features and the corresponding verification infrastructure.

Synapse Implementation Plan (v3) - Detailed Steps (Continued)
Previous Phase: Phase 1: Basic Type Checking & Compilation
Current Phase: Phase 2: Advanced Types & Verification Infrastructure

Goal: Significantly enhance the Synapse type system by incorporating Quantitative Types (for resource management), Effect Types (for tracking side effects), and basic Dependent Types (for expressing richer invariants). This phase also involves integrating an SMT solver to support verification and managing the proof objects associated with dependent types.

Phase 2: Advanced Types & Verification Infrastructure
Task P2T1: Formal Semantics Extension (Quantitative, Effect, Dependent)
Goal: Extend the formal semantics defined in P0T1 to accurately model the behavior and rules for quantitative types (linearity/affinity), effect tracking, and basic dependent types.
Input: Phase 0 Formal Semantics (P0T1), Programming Language Theory (Linear Logic, Effect Systems, Dependent Type Theory).
Output:
Updated Formal Semantics Specification Document (docs/semantics/core_v1.0.pdf or .tex).
Updated/Extended Machine-checkable Proof Assistant files (proofs/semantics/core_v1.0.v or .thy or .k).
Instructions:
[ ] Quantitative Types (Linearity/Affinity):
[ ] Introduce type contexts that track resource usage (e.g., Γ; Δ where Γ is unrestricted, Δ is linear).
[ ] Add linear function types (τ₁ ⊸ τ₂) and potentially affine types.
[ ] Modify typing rules (esp. for variables, lambda, application) to enforce resource usage constraints (e.g., linear variables used exactly once). Reference linear logic principles.
[ ] Extend evaluation semantics if needed (e.g., to model resource consumption explicitly, though often enforced statically).
[ ] Effect Types:
[ ] Introduce effect annotations/kinds (E) and effect variables (ε).
[ ] Add computation types like Comp<E, τ> (computation producing type τ with effects E).
[ ] Modify function types to include effects: τ₁ -> Comp<E, τ₂>.
[ ] Define an effect algebra/lattice (e.g., E₁ ∪ E₂, empty effect ∅).
[ ] Update typing rules to calculate and propagate effect sets (e.g., effect of application is union of function effect, argument effect, and effect annotation on the function type).
[ ] Add rules for effect handling constructs (e.g., handle t with handlers). Initially focus on tracking, handling comes later.
[ ] Dependent Types (Basic):
[ ] Introduce dependent function types (Pi-types): Πx:τ₁. τ₂(x). Start with simple dependencies (e.g., types indexed by static integers like Vector<n: Nat>).
[ ] Add basic value indices to types (e.g., Int where i > 0).
[ ] Add equality types (t₁ == t₂) or propositions.
[ ] Update typing rules for dependent functions (application requires checking dependency).
[ ] Introduce a type conversion rule based on computation (if t₁ computes to t₂, then T(t₁) converts to T(t₂) under certain conditions).
[ ] Add rules involving an external SMT solver judgment (e.g., SMTValid(φ)) for checking constraints derived from dependent types.
[ ] Best Practice (Formal Spec): Update the LaTeX/PDF document meticulously. Clearly define new syntax, judgments, and rules. Explain interactions between the systems (e.g., how effects interact with linearity). Version as v1.0.
[ ] Best Practice (Verification): Extend the Coq/Isabelle/K formalization:
[ ] Update syntax definitions.
[ ] Update typing and evaluation judgments and rules.
[ ] Goal: Re-prove (or adapt proofs of) Type Preservation and Progress for the extended system. This will be significantly more complex. Focus on key interactions. Formalize the SMT solver interaction as an oracle/axiom initially.
[ ] Review Point: Carefully review the consistency and interaction between the different type system features. Check formalization accuracy in the proof assistant. Ensure the specification document is clear and unambiguous.
Task P2T2: Type System Implementation (Level 2 - Quantitative/Effect/Capability)
Goal: Implement the type checking logic for quantitative types (resource tracking) and effect/capability tracking within the compiler.
Input: Extended Formal Semantics v1.0 (P2T1), type_checker_l1 library, asg_core library.
Output: Rust library (type_checker_l2), Updated ASG Schema, ASG instances annotated with resource/effect info.
Instructions:
[ ] Setup Project:
[ ] cargo new type_checker_l2 --lib
[ ] Add dependencies: asg_core, type_checker_l1 (or integrate directly if preferred), crates for graph analysis if needed.
[ ] Update ASG Schema (schemas/asg_schema_v1.1.proto -> v1.2):
[ ] Add representations for quantitative types (e.g., TypeLinearFunction, TypeAffineVariable), possibly using annotations on existing types/variables.
[ ] Add representations for effect types/sets (e.g., TypeComputation { effect_set_id: uint64, result_type_id: uint64 }, EffectSetNode { effects: repeated string /* or enum */ }).
[ ] Add representations for capabilities (e.g., CapabilityNode { name: string, arguments: repeated uint64 /* links to value/type nodes */ }, add capability sets to contexts/functions).
[ ] Add fields to relevant ASG nodes to store inferred effect sets or required capabilities.
[ ] Regenerate Rust code for asg_core.
[ ] Implement Quantitative Type Checker Logic:
[ ] Augment the TypingContext to track linear/affine assumptions separately (e.g., context: Map<VarId, (TypeScheme, UsageCount)>).
[ ] Modify the infer/check functions to enforce usage rules: consume linear variables exactly once, affine variables at most once. This often requires careful analysis of control flow (e.g., ensuring a variable is used on all paths).
[ ] Implement splitting/merging of linear contexts at control flow joins/branches.
[ ] Annotate ASG nodes or context with usage information for error reporting.
[ ] Focus initially on memory resources (ensuring unique pointers are not duplicated, memory is deallocated/passed on correctly).
[ ] Implement Effect Type Checker Logic:
[ ] Augment inferred types to include effect sets (e.g., (Type, EffectSet)).
[ ] Modify infer/check to compute and propagate effect sets according to the rules in P2T1. Use an efficient representation for effect sets.
[ ] Implement checking against declared effect signatures on functions.
[ ] Annotate ASG function nodes/call sites with inferred/checked effect sets.
[ ] Implement Capability Checker Logic:
[ ] Define capability representations (e.g., Capability { resource: String, permission: PermissionLevel }).
[ ] Augment TypingContext to include available capabilities.
[ ] Add checks at specific ASG nodes (e.g., primitive ops for file I/O, network access) ensuring the required capability is present in the context.
[ ] Implement logic for capability delegation/attenuation if designed.
[ ] Integration: Combine L1, Quantitative, Effect, and Capability checking into a unified pass or sequence of passes. Ensure interactions are handled correctly. The check_and_annotate_graph function should now perform all these checks.
[ ] Error Handling: Add specific TypeError variants for linearity violations, effect mismatches, missing capabilities, etc., providing detailed context.
[ ] Best Practice (TDD/Testing): Add new test cases in type_checker_l2/tests/ specifically targeting:
[ ] Correct detection of linear/affine resource misuse (double-drops, leaks).
[ ] Correct effect propagation and mismatch errors.
[ ] Correct capability checking (permission required/denied).
[ ] Interactions between the systems.
[ ] Test ASG annotations for effects/capabilities.
[ ] Integrate with CLI: Update the synapse_cli check and compile commands to run the Level 2 checks. Add flags (--verification-level=1|2) if desired.
[ ] Review Point: Verify implementation against formal semantics rules. Check correctness on test cases. Review error message clarity. Ensure ASG annotations are useful for later stages.
Task P2T3: SMT Integration & Type System Implementation (Level 3 - Dependent Core)
Goal: Integrate an SMT solver (like Z3) and implement the type checking logic for basic dependent types, leveraging the SMT solver to discharge proof obligations arising from value dependencies.
Input: Extended Formal Semantics v1.0 (P2T1), type_checker_l2 library, SMT Solver (Z3 recommended).
Output: Rust library (type_checker_l3_core), Updated ASG Schema, Ability to verify simple value-dependent properties.
Instructions:
[ ] Setup Project:
[ ] cargo new type_checker_l3_core --lib (or integrate into existing checker).
[ ] Add dependency on an SMT solver binding crate (e.g., z3 for Rust). Ensure Z3 library is installed.
[ ] Add asg_core, type_checker_l2 dependencies.
[ ] Update ASG Schema (schemas/asg_schema_v1.2.proto -> v1.3):
[ ] Add representations for dependent types:
[ ] TypePi { binder_variable_node_id: uint64, domain_type_id: uint64, codomain_type_id: uint64 } (Codomain type node can reference the binder).
[ ] TypeRefinement { base_type_id: uint64, predicate_node_id: uint64 } (Link to a predicate expression node).
[ ] Add TermPredicate, TermEquality, etc. node types for representing predicates/propositions within the ASG.
[ ] Add ProofObligationNode to ASG if not already detailed, including fields for the logical formula (as an ASG expression node ID) and status.
[ ] Regenerate Rust code for asg_core.
[ ] Implement SMT Interaction Layer (type_checker_l3_core/src/smt.rs):
[ ] Create functions to initialize the Z3 context and solver.
[ ] Implement translate_asg_predicate_to_smt(predicate_node_id: u64, graph: &AsgGraph, context: &Z3Context) -> Result<z3::Ast, TranslateError>. Translate ASG boolean/integer expressions and comparisons into Z3 ASTs. Handle variables by declaring them as SMT constants/functions based on the typing context.
[ ] Implement check_smt_validity(formula: &z3::Ast, solver: &z3::Solver) -> SmtResult /* Sat, Unsat, Unknown */. Query the solver to check if Not(formula) is unsatisfiable. Handle solver timeouts.
[ ] Implement Dependent Type Checking Logic:
[ ] Extend the infer/check functions or create a new pass.
[ ] Pi-Types (Dependent Functions): When checking an application f(a), infer the type of f as Πx:τ₁. τ₂(x). Check a has type τ₁. Substitute a for x in τ₂(x) to get the expected result type. This requires implementing substitution on types/ASG type nodes.
[ ] Refinement Types: When checking if expression e has type { b: BaseT | P(b) }, first check e has type BaseT. Then, generate the proof obligation P(e). Translate P(e) to an SMT formula using the SMT interaction layer. Add this obligation to the ASG.
[ ] Type Conversion: Implement the type conversion rule (subsumption_check(type1, type2)). If checking type1 <: type2, generate SMT constraints based on their definitions (esp. refinements) and query the solver.
[ ] Proof Obligation Management:
[ ] When a proof obligation (SMT query) is generated during checking, create a ProofObligationNode in the ASG, linking it to the relevant code node and storing the predicate.
[ ] Invoke the SMT solver via the interaction layer.
[ ] Update the ProofObligationNode status based on the SmtResult.
[ ] Error Handling: Add TypeError variants for failed SMT checks (SMTCheckFailed), solver timeouts (SMTTimeout), translation errors (SMTTranslationError), failed substitutions, etc.
[ ] Best Practice (TDD/Testing): Add tests specifically for:
[ ] Basic refinement types (e.g., x: Int where x > 0). Check acceptance of valid values and rejection of invalid ones.
[ ] Simple dependent function applications.
[ ] Correct generation and checking of SMT formulas.
[ ] Test ASG annotation with ProofObligationNodes and their status.
[ ] Integrate with CLI: Update synapse_cli check/compile. Add a Level 3 verification option (--verification-level=3). Ensure Level 3 implicitly runs Level 1 and 2 checks first. The command should fail if SMT checks fail (or report obligations if designed differently).
[ ] Review Point: Verify the logic for generating SMT queries from refinements/dependencies. Check the SMT solver integration. Ensure proof obligations are correctly generated and tracked in the ASG. Review test cases for dependent types.
Task P2T4: Proof Object Management Infrastructure
Goal: Define how proof objects (evidence for dependent type checks, potentially generated by tactics or users later) are represented and linked within the ASG.
Input: ASG Schema v1.3 (P2T3), Type Theory (Propositions as Types).
Output: Updated ASG Schema (v1.4), Basic infrastructure in asg_core.
Instructions:
[ ] Design Proof Representation: Decide how explicit proofs will be stored. Options:
Embed proof terms directly in the ASG (using dedicated ProofTerm* node types mirroring a proof calculus like lambda calculus or specific tactics).
Store proofs externally and link via hash/ID.
Recommendation: Start by focusing on tracking obligations (done in P2T3) and their status (discharged by SMT). Defer complex proof term representation unless targeting a full theorem prover integration immediately.
[ ] Update ASG Schema (schemas/asg_schema_v1.3.proto -> v1.4):
[ ] Refine the ProofObligationNode if necessary. Add fields for:
solver_used: string (e.g., "SMT:Z3", "UserProvided", "Tactic:XYZ")
evidence_link: string (Optional: hash or URI to external proof object/tactic script).
[ ] Add AnnotationNode for potentially attaching user-provided proof hints or tactic suggestions to code nodes.
[ ] Regenerate Rust code for asg_core.
[ ] Implement Basic Management (asg_core):
[ ] Add helper functions to AsgGraph to find proof obligations related to a specific code node (find_obligations_for(node_id)).
[ ] Add functions to update the status and evidence link of a ProofObligationNode.
[ ] Integrate with Type Checker (P2T3): Ensure the Level 3 checker correctly uses the refined ProofObligationNode structure when interacting with the SMT solver.
[ ] Best Practice (TDD/Testing): Add tests in asg_core for manipulating proof obligation nodes and their statuses.
[ ] Best Practice (Documentation): Document the chosen strategy for proof representation and management in docs/proof_management.md.
[ ] Review Point: Evaluate if the chosen proof obligation tracking mechanism is sufficient for current needs and extensible for future proof synthesis/checking features.
Task P2T5: UPIR Dialects Extension (Resources, Effects, Verification)
Goal: Extend UPIR with dialects to represent information derived from the advanced type checks (resources, effects) and verification metadata needed by the backend or runtime.
Input: UPIR Core Spec & library (P1T2), Type System Implementation (P2T2, P2T3).
Output: Updated UPIR Specification (docs/upir/spec_v1.1.md), Extended upir_core library.
Instructions:
[ ] Update UPIR Specification (docs/upir/core_spec_v1.md -> spec_v1.1.md):
[ ] resource Dialect:
Define operations or attributes to annotate values with resource semantics (e.g., resource.is_linear, resource.affine_use).
Define operations for explicit resource management if needed (e.g., resource.drop, resource.acquire), although the type system aims to make these static.
[ ] effect Dialect:
Define operations for effect handling (effect.handle, effect.resume_with).
Define attributes on func.call or func.func to represent allowed/produced effects.
[ ] verification Dialect:
Define attributes or metadata ops (verification.checked_property) to attach proof obligation status or verified invariants to UPIR ops/functions. This allows backends to potentially use this info (e.g., skip runtime checks).
[ ] Update upir_core Library:
[ ] Implement Rust structs/enums for the new dialect operations and attributes in src/dialects/.
[ ] Extend the UPIR builder API and pretty-printer to support the new dialects.
[ ] Best Practice (TDD/Testing): Add unit tests for creating, manipulating, and printing UPIR constructs involving the new dialects.
[ ] Review Point: Check if the new dialects adequately represent the necessary information from the type system. Ensure consistency with the overall UPIR design.
Task P2T6: Advanced Lowering Pass Implementation (ASG -> UPIR)
Goal: Update the ASG-to-UPIR lowering pass (from P1T3) to handle the new type system features and generate UPIR using the extended dialects.
Input: Type-checked/Annotated ASG (from P2T2/P2T3), Extended UPIR library (upir_core from P2T5).
Output: Updated asg_to_upir library.
Instructions:
[ ] Update asg_to_upir Project: Modify dependencies to use the latest asg_core and upir_core.
[ ] Enhance Lowering Logic:
[ ] Access resource/effect/capability annotations added to the ASG during type checking.
[ ] Generate UPIR using the new dialects:
Add resource.* attributes to UPIR values based on quantitative type info.
Add effect.* attributes/operations based on effect types and handler constructs (if any).
Add verification.* metadata based on proof obligation status attached to ASG nodes.
[ ] Ensure the lowering respects linearity/resource constraints when generating code (e.g., avoid implicitly duplicating linear values).
[ ] Best Practice (TDD/Testing): Create new ASG test cases involving linear types, effects, and dependent types. Lower them and verify that the resulting UPIR includes the expected resource/effect/verification dialect ops/attributes by comparing pretty-printed output.
[ ] Integrate with CLI: Ensure the synapse_cli compile command uses the updated lowering pass.
[ ] Review Point: Verify that the lowering pass correctly translates the advanced type system information into the corresponding UPIR dialect representations.
Task P2T7: Verification of Advanced Checkers (Soundness Sketch/Approach)
Goal: Extend the formal soundness proof efforts (from P1T7) to cover the core logic of the Level 2 (Quantitative/Effect) and Level 3 (Dependent Core) checkers against the extended formal semantics.
Input: Extended Formal Semantics v1.0 + Proofs (P2T1+), type_checker_l2/l3 implementation logic.
Output: Extended formal proofs or at least detailed proof sketches (proofs/semantics/type_checker_l2_soundness.v, ..._l3_...).
Instructions:
[ ] Model Checker Logic: Extend the proof assistant model to include linearity/affinity tracking, effect calculation, and basic dependent type checking logic (including the SMT oracle).
[ ] Formalize Soundness Statements: Update the Type Preservation and Progress theorems (or state new relevant theorems) to account for resources (e.g., evaluation preserves resource invariants) and effects (e.g., evaluation produces effects consistent with the inferred type). For dependent types, soundness relates to the consistency of the constraints with evaluation.
[ ] Develop Proofs/Sketches: Extend the proofs from P1T7. This will be complex.
Focus: Prove soundness for key interaction rules (e.g., how function application handles linearity and effects simultaneously, how SMT checks interact with evaluation).
Strategy: If full proofs are too time-consuming, create detailed proof sketches outlining the main inductive arguments and problematic cases. Identify necessary lemmas.
[ ] Document: Update proof scripts and the semantics document with the new soundness statements and proof status (fully proven, sketched, assumed).
[ ] Best Practice (CI): Ensure the CI checks the extended proof assistant files.
[ ] Review Point: Assess the correctness of the extended soundness statements. Evaluate the progress and feasibility of the formal proofs or the clarity of the sketches. Identify key lemmas or challenges remaining.
(End of Phase 2)

Proceed to Phase 3 once all Phase 2 tasks are reviewed and marked complete ([v]). This phase significantly increases the expressive power and verification capabilities of Synapse.

Okay, here is the detailed plan for Phase 3, focusing on AI integration and creating a richer developer experience.

Synapse Implementation Plan (v3) - Detailed Steps (Continued)
Previous Phase: Phase 2: Advanced Types & Verification Infrastructure
Current Phase: Phase 3: AI Integration & Developer Experience

Goal: Integrate AI capabilities deeply into the Synapse ecosystem to assist developers (human and AI agents). This involves creating language server tooling (LSP++), dedicated AI interaction APIs, the AI Syntax Projection Engine (ASPE) prototype, and leveraging AI for more explainable errors and basic code assistance.

Phase 3: AI Integration & Developer Experience
Task P3T1: LSP++ Server Implementation (Core Features)
Goal: Implement a Language Server Protocol (LSP) server for Synapse that operates on the ASG, providing core IDE features like diagnostics, type information, and basic completion.
Input: asg_core, parser_core, type_checker_l1, type_checker_l2, type_checker_l3_core libraries, LSP Specifications (vscode-languageserver-node, lsp-types crates helpful).
Output: Executable binary (synapse_lsp) implementing core LSP features.
Instructions:
[ ] Setup Project:
[ ] cargo new synapse_lsp
[ ] Add dependencies: asg_core, parser_core, type_checker_l1/l2/l3, tokio (for async), serde_json, lsp-types, lsp-server (or similar crates for handling LSP communication).
[ ] Implement LSP Communication Channel:
[ ] Set up stdin/stdout communication channel according to LSP specification.
[ ] Implement message parsing (JSON-RPC) and dispatching loop (likely using tokio for async handling of requests).
[ ] Implement Core LSP Handlers:
[ ] initialize: Handle client capabilities negotiation.
[ ] textDocument/didOpen, textDocument/didChange, textDocument/didSave:
Maintain the document content in memory.
On change/open/save, trigger parsing (parser_core) to get/update the ASG for the document. Store the ASG associated with the document URI.
Trigger background linting (Level 0) and type checking (Level 1/2 initially, maybe configurable Level 3 later) using type_checker* libraries on the updated ASG.
Store the diagnostics (parse errors, lint errors, type errors) reported by the checkers.
[ ] textDocument/publishDiagnostics: Send the stored diagnostics to the client after checks complete. Clear previous diagnostics.
[ ] textDocument/hover:
Identify the ASG node corresponding to the hover position (requires mapping source locations stored in ASG metadata).
Retrieve the inferred type (inferred_type_id) and potentially effect/resource info annotated on that ASG node (from P1T1, P2T2).
Format the type/info into a user-readable string.
Return the formatted string in the Hover response.
[ ] textDocument/completion:
(Basic) Based on cursor position, suggest keywords relevant to the minimal syntax.
(Slightly Advanced) Identify the current scope (requires scope analysis on ASG). Suggest local variables available in that scope by looking up variable names in the ASG context.
[ ] ASG Management: Implement efficient storage and retrieval of ASGs per open document URI. Handle incremental updates if possible, otherwise re-parse and re-check on change.
[ ] Error Handling: Handle errors gracefully within handlers (e.g., parse errors, check errors) and log them without crashing the server.
[ ] Best Practice (Concurrency): Use asynchronous processing (tokio) for potentially long-running tasks like type checking to avoid blocking LSP requests. Use mechanisms like debouncing for didChange events to avoid excessive re-checking.
[ ] Best Practice (TDD/Testing): Write integration tests for the LSP server:
Simulate client messages (initialize, didOpen, didChange, hover, completion) as JSON-RPC.
Send them to the server process's stdin.
Read the server's stdout and parse the JSON-RPC responses.
Assert that the responses (diagnostics, hover info, completion items) match the expected output for given inputs. Use tools like jest-language-server or write custom test harnesses.
[ ] Best Practice (Documentation): Document the supported LSP features and any custom extensions. Create instructions for integrating synapse_lsp with common editors (VS Code, Neovim).
[ ] Review Point: Verify implementation against LSP specification for core features. Test responsiveness and correctness of diagnostics, hover, and completion in a real editor environment. Check error handling and concurrency behavior.
Task P3T2: AI Interaction APIs Implementation (v1)
Goal: Define and implement the initial set of APIs specifically designed for AI agents to programmatically interact with the Synapse compiler infrastructure.
Input: Language design needs, Tooling requirements, asg_core, type_checker* libraries.
Output: API Specification (docs/ai_api_v1.md), Rust library/service (synapse_ai_api).
Instructions:
[ ] Choose Protocol: Select a robust RPC protocol suitable for potentially complex data structures and cross-language interaction. Recommendation: gRPC (using Protocol Buffers for schema definition). JSON-RPC over HTTP/WebSockets is an alternative.
[ ] Setup Project:
[ ] cargo new synapse_ai_api --lib (or potentially a binary if running as a separate server).
[ ] Add dependencies: asg_core, type_checker*, tonic (for gRPC server/client), prost (if using gRPC), tokio.
[ ] Define API Schema (protos/ai_api_v1.proto if using gRPC):
[ ] Define service SynapseAIService { ... }.
[ ] Define core messages, likely reusing or referencing the ASG schema: AsgGraph, AsgNode, TypeError, Diagnostic, SourceLocation, TypeNode.
[ ] Define RPC methods:
rpc ParseText (ParseRequest) returns (ParseResponse); (ParseRequest { text: string }, ParseResponse { asg_graph: AsgGraph, diagnostics: repeated Diagnostic })
rpc GetAsgNode (NodeRequest) returns (NodeResponse); (NodeRequest { graph_id: string, node_id: uint64 }, NodeResponse { node: AsgNode }) // Requires managing graph instances
rpc CheckAsg (CheckRequest) returns (CheckResponse); (CheckRequest { asg_graph: AsgGraph, level: VerificationLevel /* Enum: L0, L1, L2, L3 */ }, Response { annotated_asg_graph: AsgGraph, diagnostics: repeated Diagnostic })
rpc QueryType (TypeQueryRequest) returns (TypeQueryResponse); (Request { graph_id: string, node_id: uint64 }, Response { type_node: TypeNode, type_string: string }) // Returns inferred/checked type
rpc GetDiagnostics (DiagnosticsRequest) returns (DiagnosticsResponse); (Request { graph_id: string }, Response { diagnostics: repeated Diagnostic })
rpc GenerateAsgFragment (GenerateRequest) returns (GenerateResponse); // Placeholder for later - complex
rpc ApplyAsgPatch (PatchRequest) returns (PatchResponse); // Placeholder for later - complex
[ ] Implement API Server Logic:
[ ] Implement the gRPC service (SynapseAIService).
[ ] Handlers should call the corresponding functions from parser_core, asg_core, type_checker* libraries.
[ ] Need a mechanism to manage AsgGraph instances associated with different AI sessions or requests (e.g., a HashMap<String, AsgGraph> where the key is a graph_id).
[ ] Ensure thread safety if the server handles concurrent requests modifying the same graph (use Mutex, RwLock).
[ ] Implement API Client (Optional but helpful):
[ ] Create a basic Rust client library using tonic to interact with the server for testing purposes.
[ ] Best Practice (API Design): Design the API to be stateless where possible, or manage state carefully with explicit IDs. Ensure request/response structures are well-defined and versioned. Use standard status codes/error reporting mechanisms for the chosen protocol (e.g., gRPC status codes).
[ ] Best Practice (Formal Spec): Document the API methods, request/response messages, error codes, and usage protocols clearly in docs/ai_api_v1.md.
[ ] Best Practice (TDD/Testing): Write integration tests that:
[ ] Start the API server.
[ ] Use the client library (or send raw RPC calls) to invoke API methods.
[ ] Verify the responses (returned ASG, diagnostics, types) are correct based on the inputs.
[ ] Test error conditions (e.g., invalid graph ID, check failure).
[ ] Review Point: Evaluate API design for clarity, usability by AI agents, and completeness for core tasks (parsing, checking, querying). Check implementation correctness and error handling. Review documentation.
Task P3T3: ASPE Prototype (AI Syntax Projection Engine)
Goal: Build the initial prototype of the AI-driven engine that translates between a chosen familiar syntax (e.g., Pythonic) and the canonical Synapse ASG, including basic AI-powered error correction/suggestions.
Input: asg_core, parser_core (for potential fallback/structure), Machine Learning framework (e.g., Python with Hugging Face Transformers, PyTorch/TensorFlow), Pre-trained Large Language Models (LLMs).
Output: Service or library (aspe_pythonic_v1) capable of basic bidirectional translation for core language constructs.
Instructions:
[ ] Choose Target Projection: Select one initial projection. Recommendation: Pythonic syntax due to its popularity and relative simplicity.
[ ] Setup ML Environment: Set up Python environment with necessary libraries (transformers, torch, protobuf for interacting with ASG schema).
[ ] Data Preparation (Crucial & Hard):
[ ] Goal: Create a parallel corpus of Pythonic code snippets and their corresponding Synapse ASG representations (e.g., serialized ASG JSON or canonical S-expression format from P0T4).
[ ] Strategy 1 (Manual): Write simple Pythonic examples and manually construct the equivalent ASG using asg_core (via Python bindings or a helper tool). Labor-intensive.
[ ] Strategy 2 (Semi-Automated): Use existing Python codebases. Attempt to parse them with standard Python parsers, then write complex transformation logic to convert Python AST -> Synapse ASG for a subset of features. Error-prone.
[ ] Strategy 3 (Synthetic): Generate Synapse ASG programmatically, then use an LLM prompted to translate the ASG (or its canonical S-expression form) into Pythonic code ("Prompt Engineering"). Review and filter results.
[ ] Focus: Start with core features: variable declaration, basic arithmetic, function definition/calls, if/else, basic loops, ref/!/:= equivalents.
[ ] Model Selection & Fine-tuning:
[ ] Choose a suitable pre-trained Seq2Seq model (like T5, BART, or potentially a decoder-only model like GPT variants configured for translation).
[ ] Fine-tune the selected model on the prepared parallel corpus (Pythonic <-> ASG representation).
[ ] Implement Translation Service/API:
[ ] Create a Python service (e.g., using Flask/FastAPI) or library.
[ ] Projection -> ASG: Takes Pythonic code string as input. Uses the fine-tuned model to generate the target ASG representation (e.g., canonical S-expression). Calls parser_core (via FFI or API call) to parse this S-expression into a validated AsgGraph object.
[ ] ASG -> Projection: Takes an AsgGraph (e.g., as serialized proto). Generates its canonical S-expression form using formatter_core. Uses the fine-tuned model to translate the S-expression into Pythonic code string.
[ ] Implement Basic AI Error Correction/Suggestions:
[ ] If Projection -> ASG translation results in an S-expression that parser_core rejects, capture the parse error.
[ ] Feed the original Pythonic code and the parse error back to the LLM with a prompt asking it to correct the Pythonic code based on the error (e.g., "Correct this Python code snippet based on the following Synapse parse error...").
[ ] Provide the LLM's suggestion back to the user/calling agent. This is a basic heuristic.
[ ] Interface with Synapse Tools: The ASPE needs to interact with parser_core and formatter_core (potentially via the AI API created in P3T2, FFI, or by embedding the Rust libraries using PyO3). Choose an integration strategy.
[ ] Best Practice (TDD/Testing): Create a test suite with Pythonic code snippets and their expected ASG translations (and vice-versa). Measure translation accuracy (e.g., BLEU score, exact match percentage). Test the basic error correction mechanism.
[ ] Best Practice (MLOps): Track datasets, model versions, and training parameters. Set up infrastructure for retraining/evaluation.
[ ] Review Point: Evaluate the accuracy and limitations of the prototype translation for core features. Assess the effectiveness of the basic error correction. Check the integration strategy with core Synapse tools.
Task P3T4: Explainable Errors Implementation (Basic AI Assistance)
Goal: Enhance the error reporting framework by integrating AI to provide basic natural language explanations or suggestions based on the structured error data.
Input: Structured error types (TypeError, LintError, etc.), AI Interaction APIs (P3T2), Pre-trained LLMs.
Output: Enhanced diagnostic messages in LSP/CLI containing AI-generated explanations/suggestions.
Instructions:
[ ] Identify Target Errors: Choose a subset of common or complex errors from type_checker* and linter to target initially (e.g., unification failures, linearity errors, complex inference failures).
[ ] Develop Prompting Strategy: For each targeted error type:
[ ] Design prompts for an LLM that include:
The structured error information (error code, message, relevant node IDs).
Context from the ASG (code snippets around the error node ID, inferred types of relevant variables/nodes retrieved via AI API).
A request for a natural language explanation of why the error occurred and/or suggestions for fixing it.
[ ] Example Prompt Fragment: "Explain this Synapse type error in simple terms and suggest a fix. Error: UnificationFail between type 'Int -> Bool' and type 'Int -> Int' at node 123. Code context: let f = lambda (x: Int) -> x > 0; let y = f(5) + 1; "
[ ] Integrate AI Call:
[ ] In synapse_lsp (diagnostics publishing) or synapse_cli (error reporting):
[ ] When a targeted structured error is generated:
Gather necessary context from the ASG using asg_core or the AI API.
Format the prompt.
Make an API call to an LLM service (e.g., OpenAI API, local LLM via API) with the prompt.
Receive the AI-generated explanation/suggestion text.
[ ] Enhance Diagnostics/Output:
[ ] Append the AI-generated text to the original structured error message presented to the user (e.g., in the IDE's diagnostic panel or CLI output). Clearly label it as AI-generated assistance.
[ ] Best Practice (User Experience): Make AI assistance optional or clearly marked. Ensure latency of the AI call doesn't significantly degrade user experience (consider background requests). Provide feedback mechanisms if suggestions are unhelpful.
[ ] Best Practice (Testing): Test the explanation generation for the targeted errors. Evaluate the quality, relevance, and correctness of the AI suggestions. Check different code contexts leading to the same error type.
[ ] Review Point: Assess the quality and usefulness of the AI-generated explanations/suggestions for the targeted errors. Check the integration points and potential performance impact.
Task P3T5: AI Tutor Prototype Implementation
Goal: Develop a basic AI agent that can interact with a user (or another AI agent) to explain Synapse concepts or specific errors, using the AI Interaction APIs.
Input: synapse_ai_api (P3T2), Language Documentation/Specifications, Pre-trained LLMs (potentially conversational models).
Output: Simple conversational AI agent (synapse_tutor) accessible via CLI or basic interface.
Instructions:
[ ] Setup Project:
[ ] Likely a Python project using libraries for LLM interaction (e.g., openai, langchain) and potentially gRPC client libraries (grpcio-tools) to call the synapse_ai_api.
[ ] Define Core Capabilities: Target initial capabilities:
Explain a specific Synapse keyword or concept (based on documentation).
Explain a specific error code or diagnostic message provided by the user.
Show the inferred type of a variable/expression in a given code snippet.
[ ] Implement Interaction Loop:
[ ] Create a simple CLI interface or chat interface.
[ ] Parse user requests (e.g., "Explain linear types", "What does error T042 mean?", "What's the type of x in this code: ...?").
[ ] Integrate with synapse_ai_api:
[ ] For type queries: Parse code snippet using ParseText API, find relevant node ID, call QueryType API.
[ ] For explaining errors in context: Parse code snippet, potentially call CheckAsg to get diagnostics, extract relevant info.
[ ] Integrate with LLM:
[ ] Use an LLM for natural language understanding of the user request.
[ ] Use the LLM to generate explanations, combining information retrieved from the AI API (types, errors) and potentially pre-loaded Synapse documentation/knowledge base.
[ ] Prompt Engineering: Design prompts to guide the LLM to use the retrieved technical information correctly in its explanation. (e.g., "Explain linear types in Synapse. Key concepts: used exactly once, context splitting...").
[ ] Knowledge Base (Optional but helpful): Pre-process Synapse documentation (semantics specs, user guides) into a format the LLM can easily access (e.g., vector database for Retrieval-Augmented Generation - RAG).
[ ] Best Practice (Conversational AI): Maintain basic conversation history if needed. Handle ambiguous requests.
[ ] Best Practice (Testing): Test the tutor with various questions about core concepts and error codes. Verify its ability to retrieve and use information from the synapse_ai_api correctly in its explanations.
[ ] Review Point: Evaluate the accuracy and helpfulness of the tutor's explanations for the targeted capabilities. Assess the integration with the AI API.
Task P3T6: Metaprogramming Framework (Core - Staging/Macros)
Goal: Implement the basic infrastructure for compile-time metaprogramming, allowing code to generate or manipulate ASG structures before subsequent compiler stages.
Input: Language Design (Staging vs Macros), asg_core library.
Output: Enhanced compiler pipeline incorporating a metaprogramming execution stage.
Instructions:
[ ] Choose Metaprogramming Approach:
Recommendation: Start with type-safe, hygienic macros operating directly on ASG representations (similar to Rust's procedural macros or Scheme's syntax-rules but adapted for ASG). Multi-stage computation (like MetaOCaml) is powerful but more complex to implement initially. Document the choice in DESIGN_LOG.md.
[ ] Update ASG Schema (schemas/asg_schema_v1.4.proto -> v1.5):
[ ] Add TermMacroDefinition { name: string, implementation_node_id: uint64 /* Points to Synapse code implementing the macro */ }.
[ ] Add TermMacroInvocation { macro_name: string, argument_nodes: repeated uint64 /* ASG fragments passed to macro */ }.
[ ] Regenerate Rust code for asg_core.
[ ] Implement Macro Expansion Engine:
[ ] Create a new compiler stage/library (macro_expander).
[ ] Input: ASG potentially containing TermMacroInvocation nodes.
[ ] Process:
Identify macro definitions (TermMacroDefinition). Potentially compile the macro implementation code itself using a recursive call to the compiler or an interpreter.
Identify macro invocations (TermMacroInvocation).
Execute the corresponding macro implementation code, passing the argument ASG fragments as input (e.g., serialized ASG data or via dedicated API). The macro implementation code runs at compile time.
The macro implementation returns a new ASG fragment.
Replace the TermMacroInvocation node in the original ASG with the returned ASG fragment. Handle hygiene (avoiding unintended variable capture) carefully.
[ ] Output: Expanded ASG with macro invocations replaced.
[ ] Integrate into Compiler Pipeline: Insert the macro expansion stage into synapse_cli's compile flow after parsing but before type checking (or potentially allow multiple expansion phases).
[ ] API for Macros: Define how macro implementation code (written in Synapse) accesses and constructs ASG fragments (likely using functions exposed from asg_core available at compile time).
[ ] Best Practice (Hygiene): Implement hygiene mechanisms to prevent accidental variable capture during macro expansion. Track lexical scope carefully.
[ ] Best Practice (TDD/Testing):
[ ] Write simple macro definitions (e.g., a macro for assert, a macro for defining simple DSL constructs).
[ ] Write code that invokes these macros.
[ ] Run the macro expansion stage.
[ ] Verify that the resulting expanded ASG is correct (e.g., by pretty-printing it or comparing against an expected structure).
[ ] Test hygiene properties.
[ ] Review Point: Check the macro expansion logic, particularly hygiene and the API provided to macro implementations. Verify the integration point in the compiler pipeline. Test simple macro examples end-to-end.
(End of Phase 3)

Proceed to Phase 4 once all Phase 3 tasks are reviewed and marked complete ([v]). This phase brings AI into the core developer workflow and adds powerful metaprogramming capabilities.

Okay, here is the detailed plan for Phase 4, focusing on implementing advanced runtime features, debugger capabilities, hardware backends, and the ecosystem foundation.

Synapse Implementation Plan (v3) - Detailed Steps (Continued)
Previous Phase: Phase 3: AI Integration & Developer Experience
Current Phase: Phase 4: Advanced Runtime, Backends & Ecosystem Foundation

Goal: Enhance the runtime system (UART) with adaptive capabilities and fault tolerance support, build the Holographic Debugger infrastructure, extend compilation to target additional hardware (GPU, Quantum Simulators), implement the verifiable package manager, and formalize the ethical governance checks.

Phase 4: Advanced Runtime, Backends & Ecosystem Foundation
Task P4T1: Holographic Debugger & Deep Explainability Infrastructure
Goal: Implement the backend infrastructure for omniscient debugging (state capture, causal tracing) and provide APIs for AI agents to query and analyze execution history for deep explanation.
Input: synapse_runtime (P1T5), asg_core, synapse_ai_api (P3T2).
Output: Enhanced synapse_runtime with tracing capabilities, synapse_debugger library/API.
Instructions:
[ ] Design Trace Format:
[ ] Define the structure for trace events (e.g., function call, return, variable mutation, effect performed, actor message send/receive).
[ ] Each event should include: timestamp/logical clock, location (ASG node ID), relevant data (arguments, return value, new variable value), causality link (ID of the event that caused this one, if applicable).
[ ] Use an efficient serialization format (e.g., Cap'n Proto, FlatBuffers, or custom binary format).
[ ] Instrument Runtime/Compiler:
[ ] Runtime: Modify the core runtime (synapse_runtime) to optionally generate trace events during execution.
Hook into function entries/exits.
Hook into memory allocation/mutation (may require compiler support).
Hook into effect handling and concurrency primitives.
[ ] Compiler (UPIR->LLVM): Modify the upir_to_llvm stage (P1T4) to optionally insert calls to runtime tracing functions at relevant points (e.g., before/after calls, before stores). Control via compiler flag (--enable-debugging).
[ ] Implement Trace Storage & Querying:
[ ] cargo new synapse_debugger --lib
[ ] Implement mechanisms to capture trace events generated by the runtime (e.g., write to memory buffer, log file, or dedicated database). Handle potentially large trace volumes.
[ ] Implement a query engine within synapse_debugger that can efficiently search and filter traces based on time, location (ASG ID), data values, causality links.
[ ] Develop Debugger API (Extend AI API):
[ ] Update synapse_ai_api (schema ai_api_v1.proto -> v1.1):
Add service SynapseDebuggerService { ... }.
rpc StartTrace (StartTraceRequest) returns (StartTraceResponse);
rpc StopTrace (StopTraceRequest) returns (StopTraceResponse);
rpc QueryTrace (TraceQuery) returns (stream TraceEvent); (Define TraceQuery message with filters).
rpc GetStateAt (StateQuery) returns (StateSnapshot); (Query program state at a specific trace event/time). This requires reconstructing state from the trace.
rpc ExplainAnomaly (ExplainQuery) returns (ExplanationResponse); (AI uses trace + ASG to explain errors/deviations - placeholder logic initially).
[ ] Implement the server logic for these new RPCs, interacting with the trace storage/query engine.
[ ] Implement State Reconstruction: Develop logic (likely within synapse_debugger) to reconstruct the program state (variable values, heap) at a specific point in the trace by replaying events from the beginning or using snapshotting techniques.
[ ] Integrate AI for Explanation (Basic):
[ ] For the ExplainAnomaly API: Retrieve relevant trace segment around the anomaly, get related code from ASG, format into a prompt for an LLM (similar to P3T4) asking for a causal explanation based on the trace events.
[ ] Best Practice (Performance): Tracing adds overhead. Ensure it can be disabled for release builds. Optimize trace generation and storage. Consider sampling techniques for long runs.
[ ] Best Practice (TDD/Testing):
[ ] Write tests for trace event generation in instrumented code.
[ ] Write tests for the trace query engine (querying by time, node, causality).
[ ] Write tests for state reconstruction accuracy.
[ ] Test the Debugger API endpoints.
[ ] Integrate with CLI: Add flags to synapse_cli compile (--enable-debugging) and potentially a synapse_cli debug <executable> command to manage tracing/querying.
[ ] Review Point: Evaluate trace format suitability. Check instrumentation overhead. Verify query engine correctness and performance. Assess the feasibility of state reconstruction. Review API design.
Task P4T2: Full Dependent Types & Proof Synthesis Assistance
Goal: Implement more advanced dependent type features (beyond basic refinements) and integrate AI to assist users/agents in discharging proof obligations.
Input: type_checker_l3_core (P2T3), proof_manager (P2T4), AI APIs (P3T2), Theorem Proving concepts, AI/ML research on proof synthesis.
Output: Enhanced type checker (type_checker_l3_full), Enhanced AI capabilities.
Instructions:
[ ] Implement Advanced Dependent Features (in type_checker_l3_full):
[ ] Support full Pi-types where types depend on values of previous arguments.
[ ] Support Sigma-types (dependent pairs).
[ ] Implement a more robust type equality/conversion checker, potentially involving term normalization (beta-reduction within the type checker).
[ ] Extend SMT translation to handle more complex predicates and potentially basic quantifiers.
[ ] Develop Proof Tactic Suggester:
[ ] When a proof obligation (from P2T3/P2T4) fails SMT check or requires interactive proof:
[ ] Analyze the structure of the proof goal (the predicate).
[ ] Rule-Based: Implement simple heuristics (e.g., "Goal is A -> B, suggest intro tactic", "Goal involves equality, suggest reflexivity or rewrite").
[ ] AI-Based: Format the proof goal and local context (available hypotheses/variables from ASG/typing context) into a prompt for an LLM trained/fine-tuned on theorem proving data (e.g., LeanStep dataset, ProofWiki). Ask the LLM to suggest applicable tactics.
[ ] Implement Basic Proof Synthesis (Heuristic/AI):
[ ] For common proof patterns (e.g., proving properties about lists by induction, simple arithmetic):
[ ] Heuristic: Implement specific algorithms (proof methods) that attempt to automatically discharge obligations matching these patterns.
[ ] AI: Prompt an LLM with the goal and context, asking it to generate a likely proof script (sequence of tactics) or a proof term. This is highly experimental. Requires careful validation of the generated proof.
[ ] Integrate with Proof Management:
[ ] Store suggested tactics or generated proof sketches/terms potentially linked to the ProofObligationNode in the ASG (using the evidence_link or new fields).
[ ] Update APIs:
[ ] Extend AI API (ai_api_v1.1 -> v1.2) or LSP++:
rpc SuggestProofTactics (ObligationRequest) returns (TacticSuggestions);
rpc AttemptProofSynthesis (ObligationRequest) returns (ProofAttemptResult);
[ ] Extend LSP++ to display proof obligations, allow users to invoke tactic suggestions or synthesis attempts.
[ ] Best Practice (TDD/Testing):
[ ] Test advanced dependent type checking examples.
[ ] Test tactic suggestion mechanism (rule-based and AI-based) for various goals. Evaluate suggestion relevance.
[ ] Test basic proof synthesis attempts. Verify correctness of synthesized proofs (requires a proof checker).
[ ] Review Point: Check implementation of advanced dependent type features. Evaluate the usefulness and correctness of tactic suggestions and synthesis attempts. Review API extensions for proof interaction.
Task P4T3: Advanced Runtime Implementation (UART Core)
Goal: Evolve the minimal runtime into the Universal Adaptive Runtime (UART) core, adding support for adaptive scheduling, dynamic optimization hooks, fault tolerance patterns, and effect handling.
Input: synapse_runtime v0.1 (P1T5), Extended UPIR (P2T5), Concurrency/Effect System Design.
Output: Enhanced runtime library (synapse_uart v1.0).
Instructions:
[ ] Setup Project: Rename/refactor synapse_runtime to synapse_uart.
[ ] Implement Concurrency Model (Actors):
[ ] Implement lightweight actor primitives (spawn actor, send message).
[ ] Implement message queues (potentially lock-free) for actors.
[ ] Implement a scheduler (e.g., work-stealing scheduler using tokio or similar async framework) to run actors on a thread pool.
[ ] Implement Effect Handling Mechanism:
[ ] Implement runtime support for algebraic effect handlers specified in UPIR (effect.handle, effect.resume_with).
[ ] This likely involves stack manipulation (e.g., delimited continuations) or code generation strategies (CPS transformation) supported by the compiler backend. Design the runtime side of the chosen mechanism.
[ ] Dynamic Optimization Hooks (PGO):
[ ] Add runtime components for Profile-Guided Optimization (PGO):
Instrumentation hooks (called by compiled code) to collect execution counts (branches, calls).
Mechanism to dump profile data.
[ ] Add hooks for potential dynamic recompilation/specialization based on runtime conditions (JIT capabilities - advanced, may defer).
[ ] Fault Tolerance Patterns:
[ ] Provide runtime support for basic patterns like supervisor hierarchies (linked actors where failure of one notifies another).
[ ] Implement transactional effect handling (allowing effects to be rolled back if part of a computation fails). Requires careful design with the effect system.
[ ] Resource Management Integration:
[ ] If Level 2 quantitative types require runtime support (e.g., for complex region management, finalizers), implement those hooks. Aim for static enforcement where possible.
[ ] Best Practice (Modularity): Structure the UART into components (scheduler, memory manager, effect handler, actor system, PGO hooks).
[ ] Best Practice (TDD/Testing): Write unit tests for:
[ ] Actor creation, message passing, scheduling.
[ ] Effect handler mechanism (if testable at runtime level).
[ ] PGO counter collection.
[ ] Basic fault tolerance patterns (e.g., test supervisor notification on actor crash).
[ ] Update Compiler Backend: Modify upir_to_llvm (or other backends) to generate code that interacts with the new UART features (actor spawning, effect operations, PGO counters).
[ ] Review Point: Check UART architecture and component design. Verify implementation of concurrency, effects, and PGO hooks. Review testing strategy.
Task P4T4: Verified FFI Implementation
Goal: Implement a Foreign Function Interface (FFI) mechanism that allows calling external code (e.g., C libraries) while preserving Synapse's safety and verification guarantees as much as possible.
Input: Runtime (synapse_uart), Type System (type_checker*), UPIR (upir_core), Target ABI knowledge.
Output: Safe FFI mechanism integrated into the compiler and runtime.
Instructions:
[ ] Design FFI Specification Syntax:
[ ] Define syntax within Synapse (likely using ASG annotations or dedicated nodes) to declare external functions.
[ ] Declaration must include:
External function name (symbol).
Synapse-level type signature, including parameter and return types.
Crucially: Annotations specifying assumed preconditions, postconditions, effects (I/O, mutation), resource usage (linearity/ownership transfer), and necessary capabilities required by the foreign function. These annotations use Synapse's own specification/type language.
[ ] Update ASG Schema (v1.5 -> v1.6):
[ ] Add TermForeignFunctionDecl { external_name: string, synapse_type_id: uint64, precondition_id: uint64, postcondition_id: uint64, effect_set_id: uint64, capability_set_id: uint64, ... }.
[ ] Regenerate Rust code for asg_core.
[ ] Implement Compiler FFI Checks:
[ ] During type checking (type_checker_l2/l3): When encountering a call to a foreign function, treat it like a Synapse function with the declared type signature, effects, capabilities, etc.
[ ] Verification: Generate proof obligations to ensure the call site satisfies the declared precondition of the foreign function. Assume the postcondition holds after the call.
[ ] Implement Code Generation (Backend):
[ ] In upir_to_llvm (or other backends): Generate the actual FFI call instruction according to the target platform's C ABI. Handle data type marshalling/conversion between Synapse representations and C representations carefully.
[ ] Implement AI Assistance for Wrappers:
[ ] Develop a tool/feature (potentially using AI API / LLM) that takes a C header file (.h) as input.
[ ] Attempt to parse the C function signatures.
[ ] Prompt the user (or AI) to provide the necessary Synapse-level specifications (pre/post conditions, effects, etc.) for those functions.
[ ] Automatically generate the Synapse TermForeignFunctionDecl ASG nodes based on this information.
[ ] Best Practice (Security): The core principle is TRUST BUT VERIFY. Verify preconditions before the call. Trust the declared postconditions/effects (this is the boundary). Minimize the surface area of FFI. Provide "unsafe" escape hatches only when absolutely necessary and clearly marked.
[ ] Best Practice (TDD/Testing):
[ ] Create simple C libraries with functions to call.
[ ] Write Synapse code declaring and calling these functions using the FFI mechanism. Include correct and incorrect specifications.
[ ] Test that the compiler correctly accepts calls satisfying preconditions and rejects those that don't.
[ ] Test that the generated executable correctly calls the C function and handles data marshalling.
[ ] Test the AI wrapper generation tool.
[ ] Review Point: Evaluate the design of FFI specification syntax. Check the verification logic for preconditions. Verify code generation and data marshalling. Assess the security implications and the balance between safety and usability.
Task P4T5: Additional Backends Implementation (GPU/Quantum Sim)
Goal: Extend the compiler backend infrastructure to target additional computational paradigms, demonstrating the universality of UPIR. Target GPU (via SPIR-V or CUDA) and a quantum circuit simulator initially.
Input: Extended UPIR (upir_core P2T5), Target specifications (SPIR-V, CUDA PTX, QASM/QIR).
Output: New compiler backend stages (upir_to_spirv, upir_to_qsim).
Instructions:
[ ] Extend UPIR Dialects (if needed):
[ ] Define gpu dialect: Ops for kernel launch configuration, thread/block IDs, shared memory access, GPU builtins (e.g., gpu.barrier, gpu.global_id_x).
[ ] Define quantum dialect: Ops for qubit allocation (qalloc), quantum gates (Hadamard H, CNOT CX, measurement measure), classical control based on measurement.
[ ] Update upir_core and UPIR specification.
[ ] Update ASG->UPIR Lowering (P2T6): Modify the lowering pass to generate the new gpu/quantum dialect operations when encountering corresponding high-level Synapse constructs (e.g., parallel loops mapped to GPU kernels, quantum gate applications). This requires defining those high-level constructs first.
[ ] Implement UPIR-to-SPIRV Backend:
[ ] cargo new upir_to_spirv --lib
[ ] Add dependency on a SPIR-V generation library (e.g., rspirv).
[ ] Implement lower_upir_to_spirv(module: &UpirModule) -> Result<Vec<u32>, SpirvLoweringError>.
[ ] Translate relevant UPIR dialects (core, mem, cf, gpu) into SPIR-V instructions and capabilities. Handle SPIR-V specific requirements (entry points, execution models, decorations).
[ ] Implement UPIR-to-Quantum Simulator Backend:
[ ] cargo new upir_to_qsim --lib
[ ] Choose a target quantum assembly format (e.g., OpenQASM 3 or QIR - LLVM-based).
[ ] Implement lower_upir_to_qasm(module: &UpirModule) -> Result<String, QasmLoweringError> (or QIR equivalent).
[ ] Translate quantum dialect operations into the corresponding QASM/QIR instructions. Translate classical control flow (cf) appropriately.
[ ] Best Practice (TDD/Testing):
[ ] Create UPIR examples using the gpu and quantum dialects.
[ ] Test that lowering produces syntactically valid SPIR-V / QASM / QIR.
[ ] Use external tools (SPIR-V validators, QASM simulators like Qiskit Aer, QIR runners) to validate and potentially execute the generated output for simple test cases.
[ ] Integrate with CLI: Add target flags to synapse_cli compile (e.g., --target=gpu-spirv, --target=quantum-qasm) to invoke the appropriate backend.
[ ] Review Point: Check the design of new UPIR dialects. Verify the correctness of lowering to SPIR-V and QASM/QIR. Assess test coverage using external validators/simulators.
Task P4T6: Ethical Governance Framework Implementation
Goal: Implement the compiler checks for verifying ethical constraints and safety properties formally specified within Synapse code.
Input: Formalized ethical constraints/rules (design needed), type_checker*, Metaprogramming framework (P3T6), ASG.
Output: ethics_checker compiler stage/library.
Instructions:
[ ] Formalize Ethical Constraint Language:
[ ] Design how ethical constraints (fairness, bias limits, data usage policies, transparency rules) are represented. Recommendation: Use the existing specification/metaprogramming language (P3T6) or refinement types (P2T3).
[ ] Examples: @constraint(FairnessMetric(output) > 0.9), type InputData where !ContainsPII(self), annotations requiring specific logging (@requires_audit_log(input, decision)).
[ ] Update ASG Schema (v1.6 -> v1.7): Add nodes/attributes for ethical constraints if not covered by existing spec/annotation mechanisms. Regenerate code.
[ ] Implement Ethics Checking Logic (ethics_checker):
[ ] Create a new compiler stage/library.
[ ] Traverse the ASG, looking for ethical constraint annotations.
[ ] Leverage Existing Mechanisms:
For constraints expressible as type refinements (!ContainsPII), use the Level 3 type checker (P2T3) and SMT solver.
For statistical properties (Fairness), this is hard at compile time. Check might involve verifying calls to specific auditing libraries or analyzing data flow statically. Define the scope precisely.
For procedural constraints (@requires_audit_log), check if the required operations (e.g., logging calls) exist in the code structure using ASG analysis.
[ ] Integrate with metaprogramming: Allow constraints to be defined and checked by user-defined metaprograms/macros.
[ ] Error Reporting: Generate specific EthicsViolation errors if checks fail, pointing to the violated constraint and relevant code.
[ ] Best Practice (TDD/Testing):
[ ] Write Synapse code examples with various ethical annotations (both satisfied and violated).
[ ] Test that the ethics_checker correctly identifies violations and passes compliant code.
[ ] Test integration with type checking and SMT solver for relevant constraints.
[ ] Integrate with CLI: Add the ethics_checker stage to the compile pipeline (likely running alongside or after Level 2/3 verification). Add flags to enable/disable or configure specific checks.
[ ] Review Point: Evaluate the expressiveness of the ethical constraint language. Check the correctness and feasibility of the compile-time checking mechanisms. Review test cases for ethical rules.
Task P4T7: Package Manager Implementation (Content Addressed & Verified)
Goal: Implement the foundational package manager (synapse_pkg) based on content addressing (ASG hashing) and incorporating verified semantic versioning.
Input: asg_core (esp. hashing P0T3), Verification system (type_checker*, P2T2/P2T3).
Output: synapse_pkg CLI tool.
Instructions:
[ ] Setup Project:
[ ] cargo new synapse_pkg
[ ] Add dependencies: asg_core, clap, potentially networking libraries (reqwest), compression libraries (zstd), file system utilities.
[ ] Define Package Structure:
[ ] A Synapse package consists of: ASG source files (*.asg, potentially text projection *.syn + compiled ASG), metadata file (synapse.toml?), potentially compiled artifacts (UPIR, object code).
[ ] Define synapse.toml format: package name, version, dependencies (specified by content hash or name+version range), authors, license, etc.
[ ] Implement Content Addressing:
[ ] Use the ASG hashing function from asg_core (P0T3). The hash of a package or file is its unique identifier.
[ ] Implement commands for hashing files/directories.
[ ] Implement Local Cache:
[ ] Design a local cache structure (e.g., ~/.synapse/cache/) to store fetched/built packages, indexed by their content hash.
[ ] Implement Core Commands:
[ ] synapse_pkg build: Compiles the current package (using synapse_cli compile logic), stores result in cache.
[ ] synapse_pkg publish <path_to_package>: Bundles the package, uploads it to a central (or decentralized) repository (design repository interaction protocol - simple HTTP PUT/GET initially).
[ ] synapse_pkg fetch <package_hash>: Downloads package from repository into local cache.
[ ] synapse_pkg add <package_spec>: Adds a dependency to synapse.toml and fetches it.
[ ] synapse_pkg install: Fetches all dependencies listed in synapse.toml into the cache.
[ ] Implement Verified Semantic Versioning:
[ ] When resolving version ranges for dependencies:
[ ] Fetch candidate versions.
[ ] For each candidate new_version compared to the currently used old_version:
Use the Synapse type checkers (type_checker*) and potentially formal verification tools to prove behavioral compatibility (e.g., new_version is a subtype of old_version, or satisfies the same specified interface properties). This requires analyzing the public API (exported functions/types) of the packages.
Allow fetching only if compatibility is proven (for automated updates) or with explicit user confirmation.
[ ] This is complex; start with basic interface compatibility checks (function signatures, type definitions).
[ ] Repository Interaction: Define a simple API for the package repository (uploading by hash, downloading by hash). Implement basic client logic using reqwest. A simple file server can act as the initial repository.
[ ] Best Practice (TDD/Testing):
[ ] Test core commands (build, fetch, add, install) using local cache operations.
[ ] Test package publishing/fetching against a mock repository.
[ ] Test verified version resolution with simple compatible and incompatible package updates (requires creating test packages).
[ ] Best Practice (Documentation): Document package structure, synapse.toml format, and CLI commands.
[ ] Review Point: Evaluate package structure and metadata format. Check implementation of core package management commands and content addressing. Review the feasibility and correctness of the initial verified semantic versioning approach.
Task P4T8: Real-time Collaboration Backend (Basic Prototype)
Goal: Implement a basic server and protocol prototype demonstrating real-time collaborative editing of Synapse ASG structures by multiple clients (LSP++/AI Agents).
Input: asg_core, Networking libraries (tokio, warp, tungstenite), Conflict Resolution Algorithms (Operational Transformation - OT, or Conflict-free Replicated Data Types - CRDTs).
Output: synapse_collab_server prototype, Basic client logic example.
Instructions:
[ ] Choose Synchronization Strategy:
Recommendation: CRDTs (specifically Log-structured or State-based CRDTs applied to graph edits) are often simpler to implement correctly than OT for decentralized/concurrent editing. Choose a specific CRDT approach suitable for graph structures. Document choice in DESIGN_LOG.md.
[ ] Define ASG Edit Operations: Define a set of primitive operations for modifying the ASG (e.g., AddNode, UpdateNodeContent, AddEdge, RemoveEdge). These operations must be designed to work with the chosen CRDT strategy.
[ ] Setup Server Project:
[ ] cargo new synapse_collab_server
[ ] Add dependencies: asg_core, tokio, a WebSocket library (tungstenite, tokio-tungstenite), a web framework (warp, axum), CRDT library (or implement core logic), serde, serde_json.
[ ] Implement Collaboration Server:
[ ] Use WebSockets for real-time communication.
[ ] Maintain server-side state for active collaboration sessions (e.g., mapping session ID to the current ASG state represented as a CRDT).
[ ] When a client connects, send the current CRDT state.
[ ] When a client sends an ASG edit operation:
Validate the operation.
Apply it locally to the server's CRDT representation of the ASG.
Broadcast the operation (or the resulting state update, depending on CRDT type) to all other connected clients in the same session.
[ ] Implement Client-Side CRDT Logic (Prototype):
[ ] Create a minimal client example (can be Rust or potentially JS).
[ ] Implement the client-side CRDT logic to receive the initial state, apply local user edits (as ASG operations), send local operations to the server, and receive/apply remote operations from the server. Ensure convergence.
[ ] Integrate with ASG: Ensure the CRDT operations correctly correspond to modifications of the asg_core::AsgGraph structure.
[ ] Best Practice (TDD/Testing):
[ ] Test the server's handling of multiple clients connecting/disconnecting.
[ ] Test broadcasting of operations.
[ ] Test CRDT convergence: have multiple simulated clients make concurrent conflicting/non-conflicting edits and verify they all reach the same final ASG state.
[ ] Best Practice (API Design): Define the WebSocket message protocol clearly (e.g., JSON messages for specific operation types).
[ ] Review Point: Evaluate the chosen CRDT strategy and its application to ASG edits. Check server logic for session management and broadcasting. Verify client-side convergence in tests.
(End of Phase 4)

Proceed to Phase 5 once all Phase 4 tasks are reviewed and marked complete ([v]). This phase delivers advanced runtime capabilities, broader hardware support, and the foundational tools for a collaborative and verifiable ecosystem.

Okay, here is the detailed plan for Phase 5, the final phase focusing on self-hosting, self-improvement, maturation, and long-term evolution.

Synapse Implementation Plan (v3) - Detailed Steps (Continued)
Previous Phase: Phase 4: Advanced Runtime, Backends & Ecosystem Foundation
Current Phase: Phase 5: Self-Hosting, Self-Improvement & Maturation

Goal: Ensure the long-term viability, trustworthiness, and adaptability of Synapse by rewriting parts of the compiler in Synapse itself (self-hosting), formally verifying critical compiler components, integrating AI for compiler optimization, establishing community governance, and continuously incorporating new research and features.

Phase 5: Self-Hosting, Self-Improvement & Maturation (Ongoing)
Task P5T1: Compiler Self-Hosting (Incremental)
Goal: Incrementally rewrite significant parts of the Synapse compiler (initially written in the bootstrap language like Rust) in Synapse itself, using the existing Synapse compiler to compile the new Synapse implementation of the compiler.
Input: Synapse Compiler vX (Phases 1-4), Synapse Language Specification.
Output: Synapse Compiler vX+1 (partially self-hosted).
Instructions:
[ ] Identify Target Components: Select initial compiler components for rewriting in Synapse. Good candidates are often:
The parser (parser_core) - Requires good string manipulation support in Synapse.
Specific analysis passes (e.g., parts of the linter, specific type checker rules if feasible).
ASG utility functions (asg_core helpers).
Avoid core type checker, backend code generators initially due to complexity.
[ ] Develop Synapse Implementation: Rewrite the chosen component's logic purely in Synapse, using the language features developed in previous phases. This requires the Synapse language to be expressive enough for compiler tasks.
[ ] Bootstrapping Process:
Use the existing compiler (vX, written in Rust) to compile the new Synapse implementation of the component (e.g., parser_core.syn).
This produces an object file/library for the component, now compiled from Synapse.
Modify the build system of the overall compiler project to link this newly compiled Synapse component instead of the original Rust implementation for that component.
Build the next version of the compiler (vX+1) using the existing Rust components plus the newly compiled Synapse component.
[ ] Testing: Rigorously test the new compiler (vX+1) to ensure the self-hosted component functions identically to the original Rust version. Reuse existing unit and integration tests. Pay close attention to performance.
[ ] Iterate: Gradually repeat the process, rewriting more components in Synapse. The ultimate goal is a compiler that can compile its own source code entirely.
[ ] Best Practice (Build System): Maintain a complex build system capable of handling the multi-stage bootstrapping process (Stage 0: Rust compiler -> Stage 1: Synapse compiler using Stage 0 -> Stage 2: Synapse compiler using Stage 1).
[ ] Best Practice (Performance): Continuously monitor the performance (compilation speed, runtime speed of compiled code) of the self-hosted compiler compared to the bootstrap version. Optimize the Synapse implementation if regressions occur.
[ ] Review Point: Verify the correctness of the self-hosted components against tests. Check the bootstrapping build process. Analyze performance impact.
Task P5T2: Full Compiler Verification (Critical Path)
Goal: Formally verify the correctness of critical compiler passes (especially the type checker and core ASG-to-UPIR transformations) against the language's formal semantics using proof assistants.
Input: Formal Semantics v1.0+ (P2T1+), Compiler implementation (especially type_checker*, asg_to_upir), Proof Assistant files (P0T1, P1T7, P2T7).
Output: Extended formal proofs covering critical compiler stages (proofs/compiler_verification/).
Instructions:
[ ] Identify Critical Path: Define the most security/correctness-critical stages of the compilation pipeline. Typically:
Type Checker (all levels implemented).
Core ASG transformations (ensuring semantics are preserved).
Potentially parts of the resource/effect analysis if safety-critical.
[ ] Model Implementation in Proof Assistant: Refine the proof assistant models (from P1T7, P2T7) to more closely match the actual implementation details of the critical path components (data structures, algorithms). This is easier if the components were written with verification in mind. Self-hosted components written in Synapse (with its formal semantics) can be easier to reason about formally.
[ ] Formalize Correctness Statements: State theorems asserting that the modeled compiler pass correctly implements the specification defined by the formal semantics (e.g., "If type_check(asg) succeeds and returns asg', then asg' is well-typed according to the formal semantics rules", "If lower_asg_to_upir(asg) produces upir, then upir preserves the semantics of asg"). CompCert project provides examples of such statements.
[ ] Develop Proofs: Extend existing proofs or develop new ones to establish the correctness theorems. This requires deep expertise in the proof assistant and compiler internals. Break down proofs into manageable lemmas.
[ ] Best Practice (Verification Strategy): Focus on proving semantics preservation for transformation passes and soundness for analysis/checking passes relative to the formal language semantics.
[ ] Best Practice (CI): Integrate proof checking into the CI pipeline to ensure proofs remain valid as the compiler implementation evolves. Flag proof failures as build errors.
[ ] Best Practice (Documentation): Document the verified components, the correctness theorems proven, any assumptions made, and the parts of the compiler that remain unverified.
[ ] Review Point: Requires expert review of the formal models and proofs. Check the accuracy of correctness statements and the coverage of critical components.
Task P5T3: AI-Driven Optimization (Compiler & Code)
Goal: Leverage AI techniques to optimize the Synapse compiler itself and the code it generates.
Input: Synapse Compiler, UPIR, AI Interaction APIs (P3T2+), ML Frameworks, Performance Benchmarks.
Output: Optimized compiler, Improved code generation strategies.
Instructions:
[ ] Optimize UPIR Passes:
[ ] Use Reinforcement Learning (RL) or evolutionary algorithms to find the optimal sequence of existing UPIR optimization passes (e.g., inlining, dead code elimination, loop unrolling) for a given function or module to minimize code size or maximize speed. Train the AI on a benchmark suite.
[ ] Use ML models to predict the performance impact of applying a specific optimization pass, guiding the optimization selection process.
[ ] Improve Backend Code Generation:
[ ] Use ML (e.g., graph neural networks on UPIR) to predict optimal instruction scheduling or register allocation strategies for specific target architectures, potentially outperforming heuristics in the LLVM backend (or custom backends).
[ ] Fine-tune LLMs to suggest target-specific code improvements or vectorization strategies based on UPIR input.
[ ] Optimize the Compiler Itself:
[ ] Profile the Synapse compiler (especially if partially self-hosted).
[ ] Use AI code assistance tools (like GitHub Copilot adapted for Synapse, or custom models) trained on compiler codebases to suggest performance improvements for identified bottlenecks within the compiler's Synapse code.
[ ] Use RL to tune heuristics within compiler algorithms (e.g., parameters in the type inference engine, cost models for optimization passes).
[ ] Infrastructure:
[ ] Develop robust benchmarking infrastructure to measure the impact of AI-driven optimizations on compile time and runtime performance.
[ ] Integrate AI model training and inference into the compiler development workflow or as separate analysis tools.
[ ] Best Practice (Testing & Validation): Always validate the correctness of code generated using AI-optimized strategies. Ensure performance improvements on benchmarks translate to real-world code. Compare against traditional optimization techniques.
[ ] Review Point: Evaluate the effectiveness of AI-driven optimization techniques. Check impact on performance and code correctness. Assess the integration complexity and infrastructure requirements.
Task P5T4: Community Building & Governance
Goal: Foster a community around Synapse and establish clear, transparent processes for language evolution, contribution, and ethical oversight.
Input: Community interest, Existing open-source governance models.
Output: Governance structure, Contribution guidelines, Communication channels.
Instructions:
[ ] Establish Communication Channels: Create public forums, mailing lists, chat servers (e.g., Discord, Zulip) for discussion and support.
[ ] Develop Contribution Guidelines: Create CONTRIBUTING.md outlining:
Code style guide.
Commit message conventions.
Testing requirements.
Code review process (human and potentially AI-assisted).
Process for reporting bugs and submitting feature requests.
[ ] Define Language Evolution Process:
[ ] Implement a formal proposal process (e.g., SLEP - Synapse Language Enhancement Proposal, similar to Python PEPs or Rust RFCs).
[ ] Define stages for proposals (Draft -> Review -> Accepted/Rejected -> Implemented).
[ ] Establish a core team or BDFL (Benevolent Dictator For Life) model initially for decision-making on proposals, with plans for broader community representation later. Document the process clearly.
[ ] Create Code of Conduct: Adopt and enforce a standard Code of Conduct (e.g., Contributor Covenant) for all community spaces.
[ ] Documentation & Website: Create comprehensive documentation (tutorials, language reference, API docs, design rationale) and a central project website.
[ ] Ethical Oversight Board (Optional but recommended): Consider establishing a separate board or process specifically for reviewing the ethical implications of language features, standard libraries, or ecosystem tools, potentially informed by the Ethical Governance Framework (P4T6).
[ ] Review Point: Assess clarity and completeness of contribution guidelines and the language evolution process. Check if communication channels are active. Evaluate the suitability of the chosen governance model.
Task P5T5: Advanced Security Features Integration
Goal: Integrate advanced security analysis and verification features beyond basic capabilities and type safety.
Input: Security research (Information Flow Control - IFC, Cryptographic Verification, Side-channel analysis), type_checker*, upir_core.
Output: Enhanced security verification capabilities in the compiler.
Instructions:
[ ] Information Flow Control (IFC):
[ ] Design and implement IFC types/lattices (e.g., security labels like High, Low) integrated into the Synapse type system.
[ ] Extend the type checker (type_checker_l2/l3) to enforce IFC policies (e.g., prevent implicit flows from High to Low variables).
[ ] Annotate FFI boundaries (P4T4) and core libraries with IFC labels.
[ ] Cryptographic Verification Integration:
[ ] Integrate with specialized verification tools for cryptographic protocols (e.g., ProVerif, CryptoVerif) or libraries for verifying cryptographic implementations (e.g., Fiat Crypto, EasyCrypt interfaces).
[ ] Allow Synapse code to specify cryptographic properties using the specification language (P3T6) and invoke these external verifiers as part of the build/verification process.
[ ] Side-Channel Analysis Hooks:
[ ] Extend UPIR or backend annotations to mark security-sensitive computations.
[ ] Integrate static analysis tools (or develop specific ones) that operate on LLVM IR or target code to detect potential timing or cache side-channels based on these annotations or code patterns. Report potential vulnerabilities as warnings/errors.
[ ] Best Practice (TDD/Testing): Create test cases specifically demonstrating IFC violations, correct cryptographic specifications, and potential side-channel patterns. Test that the tools correctly identify these issues.
[ ] Integrate with CLI: Add flags to synapse_cli check/compile to enable/configure these advanced security checks (e.g., --ifc-policy=..., --crypto-verify).
[ ] Review Point: Evaluate the effectiveness and usability of the integrated security features. Assess the performance overhead. Check integration with the type system and build process.
Task P5T6: Continuous Research Integration
Goal: Establish a process for actively monitoring and integrating relevant cutting-edge research from Programming Languages, AI, Formal Methods, Security, and Hardware communities into Synapse.
Input: Academic publications (PLDI, POPL, ICFP, CAV, CCS, NeurIPS, ICML, ISCA, etc.), Research blogs, Community discussions.
Output: Ongoing evolution of Synapse features and implementation techniques.
Instructions:
[ ] Monitor Research Venues: Regularly review proceedings and publications from top-tier conferences and journals in relevant fields.
[ ] Identify Relevant Advances: Look for new type systems, verification techniques, AI algorithms for code/proofs, compiler optimizations, runtime techniques, security analyses, or hardware paradigms that align with Synapse's vision.
[ ] Prototyping & Evaluation: Implement prototypes of promising research ideas within the Synapse framework or as separate experimental branches. Evaluate their potential benefits, integration complexity, and performance impact.
[ ] Formal Proposal: If a research integration proves valuable, submit it through the established language evolution process (SLEP - P5T4) for community review and potential adoption.
[ ] Dedicated Research Liaison (Optional): Assign specific team members or encourage community members to focus on tracking specific research areas.
[ ] Best Practice (Documentation): Maintain a RESEARCH_IDEAS.md file tracking potentially relevant research papers and integration ideas. Document the results of prototype evaluations.
[ ] Review Point: Periodically review the process for research monitoring and integration. Assess whether Synapse is effectively incorporating valuable new ideas.
(End of Phase 5)

This phase transitions Synapse from a developed project to a living, evolving, and community-driven ecosystem with a strong focus on trust, verification, and continuous improvement.

Extension: Additional Best Practices & Considerations for AI Implementation
Context: This section extends the core plan with practices crucial for an AI agent executing the implementation. The goal is to maximize the quality, robustness, and maintainability of the outcome, while ensuring alignment with the project's complex vision.

A. AI Self-Correction and Validation
Goal: Leverage the AI's capabilities to continuously validate its own work beyond standard testing.
Practices:
[ ] Implement Automated Property-Based Testing: For core libraries (asg_core, upir_core, type checker logic), automatically generate and run property-based tests (using crates like proptest) in addition to unit/integration tests. The AI should define relevant properties based on specifications (e.g., "serialization then deserialization yields the original graph", "type checking a well-typed term should succeed").
[ ] Perform Cross-Representation Consistency Checks: After implementing multiple representations (Text -> AST -> ASG -> UPIR -> LLVM IR), implement checks that attempt round-trips or compare semantics across stages automatically for test cases. The AI should identify inconsistencies (e.g., "Formatting the parsed ASG doesn't produce canonical text", "UPIR semantics differ from ASG semantics for this construct").
[ ] Enable AI Code Review Simulation: Before committing significant code blocks, configure the AI agent to perform a "self-review". Prompt it to analyze its generated code specifically for:
Adherence to established coding style and best practices.
Potential edge cases missed.
Comparison against documented requirements for the specific task.
Clarity and maintainability.
Potential security vulnerabilities (using its pattern recognition).
Log the self-review output.
[ ] Implement Semantic Commit Validation: Before finalizing a commit, especially for changes affecting core semantics or interfaces (ASG schema, UPIR dialects, APIs), prompt the AI to explicitly verify that the changes are consistent with the documented formal specifications and design rationale (DESIGN_LOG.md).
B. Explicit Design Rationale & Context Maintenance
Goal: Ensure the AI maintains and utilizes a deep understanding of the why behind design decisions, not just the what.
Practices:
[ ] Maintain DESIGN_LOG.md Rigorously: For every significant design choice (formalism selection, schema structure, API protocol, algorithm choice), the AI must log the decision, the rationale, alternatives considered, and links to relevant specifications or requirements before implementing. Reference log entries in commit messages.
[ ] Link Code to Specifications: Implement tooling or conventions (e.g., specific comment formats like // Implements SemanticsRule: EVAL_APP) that explicitly link code sections (functions, modules) to the specific rules or requirements in the formal semantics documents or task descriptions. The AI should maintain these links.
[ ] Contextual Prompts: When prompting the AI for implementation tasks, always include relevant context: links to related specifications, design log entries, related task IDs, and existing relevant code snippets. Avoid overly generic prompts.
[ ] Persistent Session State: Ensure the AI development environment preserves session state effectively, including loaded context, design logs, and current understanding of the project, to avoid loss of context between work sessions.
C. Enhanced Verification & Formal Methods Integration
Goal: Push the "Verifiability First" principle deeper into the development workflow.
Practices:
[ ] Version Formal Specifications: Treat formal specification documents (.tex, .pdf) and proof assistant files (.v, .thy) as first-class versioned artifacts in the Git repository. Changes to semantics require version bumps and updates to dependent proofs.
[ ] Automate Proof Checking in CI: Ensure the CI pipeline (P0T5, P1T7, P2T7, P5T2) strictly enforces that all formal proofs (.v, .thy files) compile/check successfully. A failing proof should fail the build.
[ ] Generate Verification Conditions: Implement tooling (or configure the AI) to automatically generate verification conditions (predicates that should be provable) from code annotations or specifications, feeding them into the SMT solver or proof assistant workflow.
[ ] Track Proof Coverage: Maintain a status report (potentially auto-generated by analyzing proof files and specs) indicating which parts of the formal semantics and which critical compiler components have formal proofs associated with them and the status of those proofs.
D. Robustness, Security & Performance Awareness
Goal: Build in resilience, security, and performance considerations throughout the development process, not just as specific features.
Practices:
[ ] Implement Input Validation Rigorously: Apply strict input validation not only to user inputs but also to data crossing internal boundaries (e.g., ASG from parser, data from AI APIs, FFI calls, RPC data). Define expected formats/constraints clearly.
[ ] Security Static Analysis: Integrate security-focused static analysis tools (like cargo-audit for dependencies, clippy with security lints enabled, potentially Semgrep with custom rules) into the CI pipeline. The AI should be tasked with addressing reported vulnerabilities.
[ ] Fuzz Testing: Implement fuzz testing (using cargo-fuzz or similar) for critical components, especially parsers, serializers/deserializers, and potentially the type checker, to uncover unexpected crashes or vulnerabilities.
[ ] Continuous Benchmarking: Set up benchmarks (using criterion or similar) for critical functions/passes early in the development cycle (not just in Phase 5). Integrate benchmark runs into CI and configure it to detect performance regressions automatically. The AI should be prompted to investigate regressions.
[ ] Resource Usage Monitoring: Monitor memory and CPU usage during CI runs and testing to catch unexpected resource exhaustion issues early.
E. Human-AI Collaboration & Oversight
Goal: Facilitate effective oversight and collaboration between the AI agent and human supervisors/developers.
Practices:
[ ] Explicit Review Checkpoints: Designate specific tasks or commit points in the plan ([R] marker could be added) that require explicit review and approval from a human supervisor before the AI proceeds. The AI should pause and request review at these points.
[ ] Structured Status Reporting: Configure the AI to provide regular, structured progress reports summarizing completed tasks ([v]), current task, any blockers encountered, results of self-validation checks, and upcoming steps.
[ ] Query Clarification Protocol: Define a protocol for how the AI should ask clarifying questions when encountering ambiguity in tasks or specifications. It should present the ambiguity, list potential interpretations, and suggest a preferred interpretation based on context, asking for confirmation.
[ ] Traceability of AI Actions: Log the high-level prompts given to the AI and the significant actions/code blocks it generated in response. This helps debugging and understanding the AI's "thought process".
F. Maintainability & Long-Term Health
Goal: Ensure the codebase, largely generated by AI, remains understandable, maintainable, and adaptable in the long run.
Practices:
[ ] Enforce Code Quality Metrics: Integrate tools to measure code complexity (cyclomatic complexity), code duplication, and adherence to style guides. Configure CI to warn or fail if metrics exceed defined thresholds. Task the AI with refactoring to improve metrics.
[ ] Automated Documentation Generation: Leverage the AI to generate documentation (Rustdoc, API docs) automatically from code structure, type signatures, annotations, and linked specifications. Ensure this documentation is kept up-to-date via CI checks.
[ ] Refactoring Support: Ensure the AI agent has capabilities (either built-in or via tools) to perform common refactoring tasks safely across the codebase when requested or when code quality metrics degrade.
[ ] Dependency Management Hygiene: Regularly run dependency updates (cargo update) and security audits (cargo audit). Configure tools like Dependabot/Renovate. Task the AI with addressing compatibility issues arising from updates.


## Extension: Additional Best Practices & Considerations for AI Implementation (Based on Mojo Analysis & Project Goals)

**Context:** This section incorporates lessons learned from observing related projects like Mojo and reinforces best practices critical for Synapse's success, especially when implemented by an AI agent.

### G. UPIR & Hardware Abstraction Excellence

*   **Goal:** Ensure UPIR effectively serves its role as a universal hardware abstraction layer, learning from MLIR's success in Mojo's target domain.
*   **Practices:**
    *   `[ ]` **Validate Dialect Design:** When defining UPIR dialects (P1T2, P2T5, P4T5), explicitly compare designs against corresponding MLIR dialects (if they exist) for completeness and compatibility where appropriate. Document rationale for deviations (`DESIGN_LOG.md`).
    *   `[ ]` **Prioritize AI Hardware Dialects:** Given the AI-native vision, ensure dialects for GPUs (`gpu`), TPUs/Accelerators, and potentially Neuromorphic hardware (`neuro`) are well-developed and tested early (P4T5).
    *   `[ ]` **Test Cross-Dialect Interaction:** Implement tests that involve lowering code which uses operations from multiple dialects (e.g., core logic interacting with GPU kernels via memory) to ensure seamless integration.

### H. Python Projection & Interoperability Focus

*   **Goal:** Maximize potential adoption by Python users through a high-quality Pythonic projection and robust Python ecosystem interoperability.
*   **Practices:**
    *   `[ ]` **Prioritize ASPE Quality (Pythonic):** Dedicate significant effort to the Pythonic ASPE prototype (P3T3). Focus on achieving high fidelity in *bidirectional* translation for a large subset of idiomatic Python 3. Test edge cases and common libraries.
    *   `[ ]` **Measure ASPE Fidelity:** Implement quantitative metrics (beyond basic tests) to track the accuracy and completeness of the Pythonic projection <-> ASG translation during development.
    *   `[ ]` **Prioritize Verified FFI for Python:** Make the Verified FFI (P4T4) robust specifically for calling major Python C libraries (like NumPy, Pandas core). The AI Assistant for wrappers should be explicitly tested on generating safe wrappers for functions in these libraries.
    *   `[ ]` **Consider Projection Extensibility:** Design the ASPE framework (P3T3) with extensibility in mind, allowing future addition or refinement of projections beyond Pythonic (e.g., C-like, mathematical).

### I. Explicit Performance Features & Optimization Path

*   **Goal:** Ensure Synapse provides clear paths to achieving high performance comparable to systems languages, incorporating explicit performance primitives inspired by Mojo and others.
*   **Practices:**
    *   `[ ]` **Define Performance Primitives:** Explicitly design and add UPIR operations/dialects (P2T5, P4T5) for:
        *   Vector operations / SIMD types.
        *   Parallel execution primitives (e.g., `parallel_for`, task spawning mapped to UART).
        *   Memory layout control and tiling hints.
    *   `[ ]` **Integrate Auto-Tuning Hooks:** Design hooks into the UART (P4T3) and potentially the compiler (P5T3) to allow for auto-tuning of parameters (e.g., tile sizes, parallel chunk sizes) based on target hardware, similar to Mojo's approach.
    *   `[ ]` **Benchmark Performance Primitives:** Create specific benchmarks (see Sec D) to measure the effectiveness of SIMD, parallelism, and other performance features as they are implemented.

### J. Pragmatic Safety & `unsafe` Boundaries

*   **Goal:** Balance the strong verification goals with the practical need for low-level control or FFI, providing clearly defined and controlled `unsafe` mechanisms.
*   **Practices:**
    *   `[ ]` **Design `unsafe` Construct:** Define a specific language construct (e.g., `unsafe { ... }` block or annotation) for code segments that bypass certain static checks (e.g., quantitative type rules for raw pointer manipulation, specific FFI calls).
    *   `[ ]` **Require Justification:** Consider requiring justifications or specific capability grants (`Capability<UnsafeMemoryAccess>`) to use `unsafe` blocks, trackable via the ASG/verification system.
    *   `[ ]` **Minimize Unsafe Kernels:** Encourage designs where `unsafe` code is localized within small, well-tested library functions, exposing safe higher-level APIs built upon them.
    *   `[ ]` **Verify `unsafe` Boundaries:** Where possible, use verification (SMT, proof tactics) to check invariants *at the boundary* of `unsafe` blocks, even if the internal logic isn't fully verified.

### K. Ecosystem Enablement & Standard Library Strategy

*   **Goal:** Proactively plan for ecosystem growth and developer productivity by supporting library creation and potentially providing useful standard libraries early.
*   **Practices:**
    *   `[ ]` **Empower Metaprogramming:** Ensure the Metaprogramming Framework (P3T6) is sufficiently powerful and ergonomic for users to define high-level DSLs and libraries effectively within Synapse. Provide good documentation and examples.
    *   `[ ]` **Identify Core Library Needs:** Analyze common tasks in target domains (AI/ML, systems programming, web) and identify foundational *native* Synapse libraries that would significantly boost productivity beyond the absolute runtime necessities (e.g., enhanced collections, basic data processing, async utilities, matrix operations if targeting ML).
    *   `[ ]` **Develop Seed Standard Libraries:** Allocate specific tasks (potentially for the AI agent or early community contributors) to implement a small set of high-priority standard libraries using Synapse itself, applying Synapse's own verification features to them.
    *   `[ ]` **Test Verified SemVer Rigorously:** Place extra testing focus on the *verified* semantic versioning component of `synapse_pkg` (P4T7). Ensure the compatibility checks (subtyping, property satisfaction) are robust and practical for real-world library evolution.

### L. Enhance AI Assistance Based on Ecosystem Gaps

*   **Goal:** Leverage Synapse's AI-native features to mitigate challenges arising from a nascent ecosystem or complex language features.
*   **Practices:**
    *   `[ ]` **Prioritize AI Tutor (P3T5):** Ensure the AI Tutor is effective at explaining Synapse's unique/advanced features (quantitative types, dependent types, effects, metaprogramming) and guiding users through common patterns, referencing formal specs where helpful.
    *   `[ ]` **Contextual Code Generation:** Enhance AI code generation capabilities (beyond current plan) to suggest idiomatic Synapse code, potentially leveraging knowledge of available (even if limited) native libraries or FFI wrappers.
    *   `[ ]` **Ecosystem-Aware Assistance:** Explore having AI tools (LSP++, AI Tutor) be aware of packages available via `synapse_pkg` to suggest relevant library usages.

### M. Learn from Peer Language Development

*   **Goal:** Continuously learn from the trajectory, successes, and challenges of related language projects like Mojo, Rust, Zig, etc.
*   **Practices:**
    *   `[ ]` **Track Ecosystem Growth:** Monitor how the Mojo ecosystem develops (library availability, community adoption, tooling evolution) and adapt Synapse's community/ecosystem strategy (P5T4, Sec K) accordingly.
    *   `[ ]` **Analyze Adoption Factors:** Study the factors driving adoption (or lack thereof) for related new languages, paying attention to performance claims vs reality, tooling quality, learning curve, and killer applications. Apply lessons to Synapse positioning and development priorities.

------

## Extension: Additional Best Practices & Considerations for AI Implementation (Based on Multi-Language Comparison Analysis)

**Context:** Insights from comparing practical experiences across 10 modern languages highlight critical factors for Synapse's usability and adoption. This section adds practices to address these factors during AI-driven implementation.

### N. Prioritize Tooling Robustness & Performance

*   **Goal:** Ensure the core developer tooling (LSP, AI APIs) provides a seamless, reliable, and performant experience, avoiding the pitfalls observed in several compared languages.
*   **Practices:**
    *   `[ ]` **Rigorous LSP++ Testing (P3T1):** Implement extensive automated tests simulating real-world editor interactions (multiple files, rapid changes, complex queries) for `synapse_lsp`. Measure response times for diagnostics, hover, completion under load and set performance targets. Ensure stability (no crashes).
    *   `[ ]` **Accurate Semantic Information:** Validate that information provided by LSP++ (types, errors, definitions) precisely matches the results from the core type checkers (`type_checker*`) operating on the canonical ASG.
    *   `[ ]` **AI API Reliability (P3T2):** Implement health checks, monitoring, and robust error handling for the `synapse_ai_api` service. Ensure it can handle concurrent requests reliably.

### O. Design Ergonomic & Verifiable Error Handling

*   **Goal:** Implement an error handling mechanism that is both formally sound (verifiable) and developer-friendly, avoiding excessive boilerplate or the unsafety of unchecked exceptions.
*   **Practices:**
    *   `[ ]` **Integrate Errors with Effects (P2T2):** Design the primary error handling mechanism using the Effect System. Errors should be tracked effects that must be handled (or explicitly propagated), ensuring totality.
    *   `[ ]` **Provide Ergonomic Handling Syntax:** Design syntax for handling errors (e.g., `try`/`catch`-like constructs for effects, or monadic `?` operator if using `Result`-like types within the effect system) that is less verbose than `if err != nil` but still explicit.
    *   `[ ]` **Avoid Unchecked Exceptions:** Explicitly disallow unchecked exceptions as the primary error mechanism in the language design and standard library.

### P. Ensure Core Safety Features are Idiomatic

*   **Goal:** Make key safety features feel natural and integrated, not bolted-on.
*   **Practices:**
    *   `[ ]` **Implement Exhaustive Matching:** Ensure Synapse's pattern matching construct (on variants, tuples, etc.) requires exhaustive checking by the compiler, similar to Rust `match` or Kotlin `when`.
    *   `[ ]` **Promote Immutability:** Design standard library APIs and common patterns to favour immutability, leveraging Quantitative Types (P2T2) for efficient mutation where necessary, rather than making mutability the easy default.
    *   `[ ]` **Built-in Null Safety:** Verify that the core type system design (P1T1 onwards) inherently prevents null reference errors (e.g., through non-nullable types by default and explicit `Option`/`Maybe` types).

### Q. Focus on Projection Readability & Standard Library Design

*   **Goal:** Enhance developer experience through readable syntax projections and a useful standard library providing common functional patterns.
*   **Practices:**
    *   `[ ]` **ASPE Readability Evaluation (P3T3):** Explicitly evaluate the readability of code generated by the ASPE (especially Pythonic) using code quality metrics or human review heuristics. Refine the ASPE translation to produce idiomatic and clean code in the target projection.
    *   `[ ]` **Design Core Standard Library (Native):** Plan and implement a small but well-designed *native* Synapse standard library early (potentially Phase 2/3) covering:
        *   Core data structures (List, Map, Set) with immutable and mutable variants.
        *   Common functional methods (`map`, `filter`, `fold`/`reduce`, `find`, etc.).
        *   Basic IO utilities (building on Effect system).
        *   String manipulation helpers.
    *   `[ ]` **Consistent API Design:** Ensure APIs within the standard library and core runtime follow consistent naming conventions and patterns (inspired by successful libraries like Kotlin's or Swift's).

### R. Mitigate Complexity & Learning Curve

*   **Goal:** Make Synapse's powerful features accessible despite their inherent complexity.
*   **Practices:**
    *   `[ ]` **Prioritize AI Assistance Quality:** Ensure the AI Tutor (P3T5), Explainable Errors (P3T4), and Proof Synthesis Assistance (P4T2) provide genuinely helpful, accurate, and context-aware guidance, specifically targeting complex features like dependent types or quantitative types. Test their effectiveness with example learning scenarios.
    *   `[ ]` **Layered Documentation:** Structure documentation to allow users to learn basics first and progressively dive into advanced features like full verification or metaprogramming. Provide clear tutorials for common tasks.
    *   `[ ]` **Consider Language Profiles/Levels (Optional Design):** Explore if defining official language "levels" (e.g., a "core" level without dependent types or manual resource management) could ease initial adoption, while still allowing opt-in to advanced features. Document this exploration in `DESIGN_LOG.md`.

### S. Ensure Ergonomic Resource Management (Quantitative Types)

*   **Goal:** Make the Quantitative Type system (P2T2) powerful yet usable, avoiding the friction reported with some manual memory management approaches.
*   **Practices:**
    *   `[ ]` **Focus on Inference:** Maximize type inference for quantitative types where possible to reduce annotation burden.
    *   `[ ]` **Clear Error Messages:** Provide exceptionally clear error messages for linearity/affinity/ownership violations, explaining *why* the usage is incorrect and suggesting fixes (leverage AI P3T4 here).
    *   `[ ]` **Document Patterns:** Document common patterns for working with linear/affine types effectively (e.g., passing ownership, borrowing, structured deallocation).

---

**(End of Extension)**