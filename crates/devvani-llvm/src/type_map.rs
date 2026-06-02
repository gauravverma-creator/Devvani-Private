use inkwell::context::Context;
use inkwell::types::BasicTypeEnum;
use crate::error::DevvaniLLVMError;

/// Vibhakti cases from Sanskrit grammar mapped to LLVM types
#[derive(Debug, Clone, PartialEq)]
pub enum Vibhakti {
    Prathama,   // Nominative  → Subject   → i64
    Dvitiya,    // Accusative  → Object    → i64
    Tritiya,    // Instrumental→ Helper    → ptr (i8*)
    Chaturthi,  // Dative      → Return    → i64
    Panchami,   // Ablative    → Source    → ptr (i8*)
    Shashthi,   // Genitive    → Parent    → ptr (i8*)
    Saptami,    // Locative    → Scope     → ptr (i8*)
}

pub struct TypeMapper<'ctx> {
    context: &'ctx Context,
}

impl<'ctx> TypeMapper<'ctx> {
    pub fn new(context: &'ctx Context) -> Self {
        Self { context }
    }

    pub fn vibhakti_to_llvm(
        &self,
        vibhakti: &Vibhakti,
    ) -> Result<BasicTypeEnum<'ctx>, DevvaniLLVMError> {
        match vibhakti {
            Vibhakti::Prathama  => Ok(self.context.i64_type().into()),
            Vibhakti::Dvitiya   => Ok(self.context.i64_type().into()),
            Vibhakti::Chaturthi => Ok(self.context.i64_type().into()),
            Vibhakti::Tritiya   => Ok(self.context.i8_type()
                                        .ptr_type(inkwell::AddressSpace::default()).into()),
            Vibhakti::Panchami  => Ok(self.context.i8_type()
                                        .ptr_type(inkwell::AddressSpace::default()).into()),
            Vibhakti::Shashthi  => Ok(self.context.i8_type()
                                        .ptr_type(inkwell::AddressSpace::default()).into()),
            Vibhakti::Saptami   => Ok(self.context.i8_type()
                                        .ptr_type(inkwell::AddressSpace::default()).into()),
        }
    }
}
