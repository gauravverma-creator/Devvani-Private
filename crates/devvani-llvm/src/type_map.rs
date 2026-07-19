use crate::error::DevvaniLLVMError;
use devvani_ast::node::Vibhakti;
use devvani_typesystem::vibhakti::DevvaniType;
use inkwell::context::Context;
use inkwell::types::BasicTypeEnum;
use inkwell::AddressSpace;

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
            Vibhakti::Prathama => Ok(self.context.i64_type().into()),
            Vibhakti::Dvitiya => Ok(self.context.i64_type().into()),
            Vibhakti::Chaturthi => Ok(self.context.i64_type().into()),
            Vibhakti::Tritiya => Ok(self
                .context
                .i8_type()
                .ptr_type(inkwell::AddressSpace::default())
                .into()),
            Vibhakti::Panchami => Ok(self
                .context
                .i8_type()
                .ptr_type(inkwell::AddressSpace::default())
                .into()),
            Vibhakti::Shashthi => Ok(self
                .context
                .i8_type()
                .ptr_type(inkwell::AddressSpace::default())
                .into()),
            Vibhakti::Saptami => Ok(self
                .context
                .i8_type()
                .ptr_type(inkwell::AddressSpace::default())
                .into()),
        }
    }

    pub fn devvani_type_to_llvm(
        &self,
        devvani_type: &DevvaniType,
    ) -> Result<BasicTypeEnum<'ctx>, DevvaniLLVMError> {
        match devvani_type {
            DevvaniType::Vaak => Ok(self
                .context
                .i8_type()
                .ptr_type(AddressSpace::default())
                .into()),
            DevvaniType::VaakBorrow => Ok(self
                .context
                .i8_type()
                .ptr_type(AddressSpace::default())
                .into()),
            _ => Err(DevvaniLLVMError::TypeMapError(format!(
                "Unsupported DevvaniType for LLVM: {:?}",
                devvani_type
            ))),
        }
    }
}
