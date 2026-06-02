use thiserror::Error;

#[derive(Debug, Error)]
pub enum DevvaniLLVMError {
    #[error("LLVM initialization failed: {0}")]
    InitError(String),

    #[error("Type mapping failed for vibhakti: {0}")]
    TypeMapError(String),

    #[error("Target machine creation failed: {0}")]
    TargetError(String),

    #[error("Code generation failed: {0}")]
    CodeGenError(String),

    #[error("IR emission failed: {0}")]
    EmitError(String),
}
