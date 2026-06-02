use inkwell::targets::{
    CodeModel, InitializationConfig, RelocMode,
    Target, TargetMachine, TargetTriple,
};
use inkwell::OptimizationLevel;
use crate::error::DevvaniLLVMError;

pub struct DevvaniTarget {
    pub machine: TargetMachine,
    pub triple: TargetTriple,
}

impl DevvaniTarget {
    pub fn new_native() -> Result<Self, DevvaniLLVMError> {
        Target::initialize_native(&InitializationConfig::default())
            .map_err(|e| DevvaniLLVMError::InitError(e.to_string()))?;

        let triple = TargetMachine::get_default_triple();
        let target = Target::from_triple(&triple)
            .map_err(|e| DevvaniLLVMError::TargetError(e.to_string()))?;

        let machine = target
            .create_target_machine(
                &triple,
                "generic",
                "",
                OptimizationLevel::Default,
                RelocMode::Default,
                CodeModel::Default,
            )
            .ok_or_else(|| DevvaniLLVMError::TargetError(
                "Failed to create target machine".to_string()
            ))?;

        Ok(Self { machine, triple })
    }
}
