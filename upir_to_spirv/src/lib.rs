//! UPIR-to-SPIRV Backend (Phase 4 GPU)
//!
//! Lowers UPIR modules to SPIR-V binary for GPU execution.
//! Initial version handles basic arithmetic and function lowering.

use upir_core::ir::{Module, Function, Operation};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum SpirvError {
    #[error("Lowering error: {0}")]
    LoweringError(String),
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
    #[error("Backend error: {0}")]
    BackendError(String),
}

pub type Result<T> = std::result::Result<T, SpirvError>;

pub fn lower_upir_to_spirv(module: &Module) -> Result<Vec<u8>> {
    // Placeholder: serialize the module as JSON for now
    // In a real backend, this would emit SPIR-V binary
    let json = serde_json::to_vec(module)?;
    Ok(json)
}

#[cfg(test)]
mod tests {
    use super::*;
    use upir_core::ir::{Module, Function, Block, Operation};

    #[test]
    fn test_lower_trivial_module() {
        let module = Module {
            name: "gpu_test".to_string(),
            functions: vec![],
            datatype_decls: vec![],
            typeparam_decls: vec![],
            effect_decls: vec![],
        };
        let spirv = lower_upir_to_spirv(&module).unwrap();
        assert!(!spirv.is_empty());
    }
}
