pub mod codegen;
pub mod error;
pub mod target;
pub mod type_map;

use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::module::Module;

use crate::error::DevvaniLLVMError;
use crate::target::DevvaniTarget;

pub struct CodeGen<'ctx> {
    pub context: &'ctx Context,
    pub module: Module<'ctx>,
    pub builder: Builder<'ctx>,
    pub target: DevvaniTarget,
}

impl<'ctx> CodeGen<'ctx> {
    pub fn new(context: &'ctx Context, module_name: &str) -> Result<Self, DevvaniLLVMError> {
        let module = context.create_module(module_name);
        let builder = context.create_builder();
        let target = DevvaniTarget::new_native()?;

        Ok(Self {
            context,
            module,
            builder,
            target,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::type_map::TypeMapper;
    use devvani_ast::node::Vibhakti;

    #[test]
    fn test_codegen_init() {
        let context = Context::create();
        let codegen = CodeGen::new(&context, "test_module");
        assert!(codegen.is_ok());
    }

    #[test]
    fn test_vibhakti_type_mapping() {
        let context = Context::create();
        let mapper = TypeMapper::new(&context);
        let ty = mapper.vibhakti_to_llvm(&Vibhakti::Prathama);
        assert!(ty.is_ok());
        let ty2 = mapper.vibhakti_to_llvm(&Vibhakti::Saptami);
        assert!(ty2.is_ok());
    }
}
