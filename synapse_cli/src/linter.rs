/// Basic Level 0 linter for Synapse ASG.
///
/// Checks for graph integrity, scope issues, and simple type failures.
/// To be extended in future phases with deeper semantic analysis.

use asg_core::AsgGraph;

/// Represents a lint error detected by the linter.
#[derive(Debug)]
pub struct LintError {
    pub code: &'static str,
    pub message: String,
    pub node_id: u64,
    pub source_location: Option<SourceLocation>,
}

// Placeholder: Use real metadata once ASG contains source locations
#[derive(Debug)]
pub struct SourceLocation {
    pub filename: String,
    pub start_line: u32,
    pub start_col: u32,
    pub end_line: u32,
    pub end_col: u32,
}

/// Level 0 linter: detects graph and scope errors, not full types.
///
/// Returns a vector of LintError structs.
pub fn lint_graph(_graph: &AsgGraph) -> Vec<LintError> {
    // Skeleton implementation. Real implementation should:
    // * Traverse the ASG nodes
    // * Check referenced node IDs actually exist (edge correctness)
    // * For TermVariable, ensure definition_node_id resolves to an enclosing binder (scope check)
    // * For TermApplication, check function node is a lambda (if trivially available)
    // * For TermAssign, check target is a reference (if trivially available)
    // * Collect source location info from node metadata (when present)
    //
    // Return list of LintError as needed.
    vec![]
}