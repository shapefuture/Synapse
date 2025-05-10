//! Proof synthesis and explainable errors assistance for Synapse
//! 
//! Provides utilities for:
//! 1. Explaining type/lint errors in natural language
//! 2. Suggesting proof tactics for discharing verification obligations
//! 3. Generating code fixes for common errors

use thiserror::Error;
use type_checker_l2::TypeError;
use serde::{Serialize, Deserialize};

/// Error returned by explainable error generation
#[derive(Error, Debug)]
pub enum ExplainError {
    #[error("Unknown error type: {0}")]
    UnknownErrorType(String),
    
    #[error("Missing context: {0}")]
    MissingContext(String),
    
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
}

/// Result type for explainable operations
pub type Result<T> = std::result::Result<T, ExplainError>;

/// Structured explanation for a type error
#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorExplanation {
    /// The original error message
    pub error_message: String,
    
    /// A more user-friendly explanation
    pub explanation: String,
    
    /// Suggested fix, if available
    pub suggested_fix: Option<String>,
    
    /// Code snippet to illustrate the fix, if available
    pub code_fix: Option<String>,
}

/// Explains a type error in user-friendly terms
pub fn explain_type_error(err: &TypeError, code_context: Option<&str>) -> Result<ErrorExplanation> {
    use TypeError::*;
    
    let (explanation, suggested_fix, code_fix) = match err {
        UnificationFail(t1, t2) => (
            format!("Cannot unify types: {} and {} are incompatible. This often happens when you're trying to use a value of one type where another type is expected.", t1, t2),
            Some("Check that the types of your expressions match what's expected at each position.".to_string()),
            Some(format!("// Example fix:\n// Instead of:\nlet x: {} = /* some {} value */;\n// Use:\nlet x: {} = /* some {} value */;\n// Or convert the value to match the expected type", t2, t1, t1, t1))
        ),
        
        OccursCheck(t1, t2) => (
            format!("Recursive type definition detected: {} occurs within {}. This usually happens with infinite types, which Synapse doesn't support.", t1, t2),
            Some("Revise your type definitions to avoid recursion without indirection.".to_string()),
            Some("// Example fix:\n// Instead of recursive types like:\n// type Tree = { value: Int, children: Tree[] }\n// Use indirection:\n// type Tree = { value: Int, children: TreeList }\n// type TreeList = Tree[]".to_string())
        ),
        
        UndefinedVariable(node_id) => (
            format!("Undefined variable referenced (node ID: {}). You're using a variable that hasn't been defined in the current scope.", node_id),
            Some("Make sure all variables are properly defined before use.".to_string()),
            Some("// Example fix:\n// Instead of:\nx + 1  // x not defined\n// Use:\nlet x = 5;\nx + 1".to_string())
        ),
        
        Unimplemented => (
            "This feature is not yet implemented in the type checker.".to_string(),
            Some("Try using a different approach with currently supported language features.".to_string()),
            None
        ),
        
        _ => return Err(ExplainError::UnknownErrorType(format!("{:?}", err)))
    };
    
    Ok(ErrorExplanation {
        error_message: format!("{}", err),
        explanation,
        suggested_fix: Some(suggested_fix),
        code_fix,
    })
}

/// Generates a proof tactic suggestion for a verification condition
pub fn suggest_proof_tactic(vc_description: &str, vc_type: &str) -> Result<String> {
    // Pattern-based suggestion
    match vc_type {
        "equality" => {
            Ok("Try using the 'rewrite' tactic to simplify both sides of the equality, or 'reflexivity' if they're identical after simplification.".to_string())
        },
        "inequality" => {
            Ok("For inequalities, consider 'apply' with appropriate arithmetic lemmas, or 'omega' for linear integer constraints.".to_string())
        },
        "implication" => {
            Ok("Start with 'intros' to move antecedents into the context, then work on the conclusion.".to_string())
        },
        "forall" => {
            Ok("Use 'intros' to bring quantified variables into context, then proceed with the body of the quantification.".to_string())
        },
        _ => {
            Err(ExplainError::UnknownErrorType(vc_type.to_string()))
        }
    }
}

/// Provides a simplified explanation of an effect capability error
pub fn explain_effect_error(allowed_effects: &[String], required_effect: &str) -> Result<ErrorExplanation> {
    let allowed = allowed_effects.join(", ");
    
    let explanation = format!(
        "Effect capability error: The code requires the '{}' effect, but only [{}] effects are allowed in this context.",
        required_effect, allowed
    );
    
    let suggested_fix = if allowed_effects.is_empty() {
        "Either annotate the function to explicitly allow this effect, or rewrite the code to avoid using this effect.".to_string()
    } else {
        format!("Add '{}' to the list of allowed effects, or rewrite to use only the allowed effects.", required_effect)
    };
    
    let code_fix = Some(format!(
        "// Example fix:\n// Instead of:\nfn pure_function() {{ ... perform_io() ... }}\n\n// Either annotate to allow the effect:\n#[allows({})]\nfn effectful_function() {{ ... perform_io() ... }}\n\n// Or rewrite to avoid the effect:\nfn pure_function() {{ ... // no IO operations ... }}", 
        required_effect
    ));
    
    Ok(ErrorExplanation {
        error_message: format!("Effect '{}' not allowed in this context", required_effect),
        explanation,
        suggested_fix: Some(suggested_fix),
        code_fix,
    })
}

/// Tests for the explainable module
#[cfg(test)]
mod tests {
    use super::*;
    use type_checker_l2::Type;
    
    #[test]
    fn test_explain_unification_error() {
        let err = TypeError::UnificationFail(
            Type::Int,
            Type::Bool
        );
        
        let explanation = explain_type_error(&err, None).unwrap();
        assert!(explanation.explanation.contains("incompatible"));
        assert!(explanation.explanation.contains("Int"));
        assert!(explanation.explanation.contains("Bool"));
    }
    
    #[test]
    fn test_explain_effect_error() {
        let allowed = vec!["Pure".to_string(), "State".to_string()];
        let required = "IO";
        
        let explanation = explain_effect_error(&allowed, required).unwrap();
        assert!(explanation.explanation.contains("IO"));
        assert!(explanation.explanation.contains("Pure"));
        assert!(explanation.explanation.contains("State"));
        assert!(explanation.suggested_fix.unwrap().contains("Add 'IO'"));
    }
}