use inkwell::values::{BasicValueEnum, FunctionValue};
use std::collections::HashMap;
use crate::error::DevvaniLLVMError;
use devvani_ast::node::{ASTNode, KarakaParam};

/// Extends CodeGen from lib.rs with IR emission methods
pub struct IrEmitter<'ctx> {
    pub context: &'ctx inkwell::context::Context,
    pub module: inkwell::module::Module<'ctx>,
    pub builder: inkwell::builder::Builder<'ctx>,
    /// Symbol table: variable name → LLVM value
    pub variables: HashMap<String, BasicValueEnum<'ctx>>,
}

impl<'ctx> IrEmitter<'ctx> {
    pub fn new(context: &'ctx inkwell::context::Context, module_name: &str) -> Self {
        Self {
            context,
            module: context.create_module(module_name),
            builder: context.create_builder(),
            variables: HashMap::new(),
        }
    }

    /// Entry point: compile full AST to IR string
    pub fn emit_ir(&mut self, ast: &ASTNode) -> Result<String, DevvaniLLVMError> {
        // Create an anonymous top-level function if we are emitting at global scope
        let i64_type = self.context.i64_type();
        let fn_type = i64_type.fn_type(&[], false);
        let function = self.module.add_function("_devvani_main", fn_type, None);
        let entry_block = self.context.append_basic_block(function, "entry");
        self.builder.position_at_end(entry_block);

        self.compile_node(ast)?;

        if function.get_last_basic_block().and_then(|bb| bb.get_terminator()).is_none() {
            self.builder.build_return(Some(&i64_type.const_int(0, false))).map_err(|e| DevvaniLLVMError::CodeGenError(e.to_string()))?;
        }

        Ok(self.module.print_to_string().to_string())
    }

    /// Dispatch on ASTNode variant
    fn compile_node(&mut self, node: &ASTNode) -> Result<Option<BasicValueEnum<'ctx>>, DevvaniLLVMError> {
        match node {
            ASTNode::KaryakramNode { shareera } => {
                for stmt in shareera {
                    self.compile_node(stmt)?;
                }
                Ok(None)
            }
            ASTNode::DhatuDef { name, params, body, .. } => {
                // Save current builder position
                let current_bb = self.builder.get_insert_block();
                
                self.compile_function(name, params, body)?;
                
                // Restore builder position
                if let Some(bb) = current_bb {
                    self.builder.position_at_end(bb);
                }
                Ok(None)
            }
            ASTNode::Nama { base, .. } => {
                let val = self.variables.get(base)
                    .ok_or_else(|| DevvaniLLVMError::CodeGenError(format!("Unknown variable: {}", base)))?;
                Ok(Some(*val))
            }
            ASTNode::PurnaankLiteral { value, .. } => {
                let val = self.context.i64_type().const_int(*value as u64, true);
                Ok(Some(val.into()))
            }
            ASTNode::DashaamshaLiteral { value, .. } => {
                let val = self.context.f64_type().const_float(*value);
                Ok(Some(val.into()))
            }
            ASTNode::VaakLiteral { value, .. } => {
                let val = self.builder.build_global_string_ptr(value, "str")
                    .map_err(|e| DevvaniLLVMError::CodeGenError(e.to_string()))?;
                Ok(Some(val.as_pointer_value().into()))
            }
            ASTNode::AstiNode { naama, mulya } | ASTNode::BhavatiNode { naama, mulya } => {
                let val = self.compile_node(mulya)?
                    .ok_or_else(|| DevvaniLLVMError::CodeGenError("AstiNode: no mulya".into()))?;
                self.variables.insert(naama.clone(), val);
                Ok(Some(val))
            }
            ASTNode::YogaNode { vama, dakshina } => {
                let lhs = self.compile_node(vama)?.ok_or_else(|| DevvaniLLVMError::CodeGenError("Yoga: no vama".into()))?;
                let rhs = self.compile_node(dakshina)?.ok_or_else(|| DevvaniLLVMError::CodeGenError("Yoga: no dakshina".into()))?;
                match (lhs, rhs) {
                    (BasicValueEnum::IntValue(l), BasicValueEnum::IntValue(r)) => {
                        Ok(Some(self.builder.build_int_add(l, r, "addtmp").map_err(|e| DevvaniLLVMError::CodeGenError(e.to_string()))?.into()))
                    }
                    (BasicValueEnum::FloatValue(l), BasicValueEnum::FloatValue(r)) => {
                        Ok(Some(self.builder.build_float_add(l, r, "faddtmp").map_err(|e| DevvaniLLVMError::CodeGenError(e.to_string()))?.into()))
                    }
                    _ => Err(DevvaniLLVMError::CodeGenError("Yoga: type mismatch".into()))
                }
            }
            ASTNode::ViyogaNode { vama, dakshina } => {
                let lhs = self.compile_node(vama)?.ok_or_else(|| DevvaniLLVMError::CodeGenError("Viyoga: no vama".into()))?;
                let rhs = self.compile_node(dakshina)?.ok_or_else(|| DevvaniLLVMError::CodeGenError("Viyoga: no dakshina".into()))?;
                match (lhs, rhs) {
                    (BasicValueEnum::IntValue(l), BasicValueEnum::IntValue(r)) => {
                        Ok(Some(self.builder.build_int_sub(l, r, "subtmp").map_err(|e| DevvaniLLVMError::CodeGenError(e.to_string()))?.into()))
                    }
                    (BasicValueEnum::FloatValue(l), BasicValueEnum::FloatValue(r)) => {
                        Ok(Some(self.builder.build_float_sub(l, r, "fsubtmp").map_err(|e| DevvaniLLVMError::CodeGenError(e.to_string()))?.into()))
                    }
                    _ => Err(DevvaniLLVMError::CodeGenError("Viyoga: type mismatch".into()))
                }
            }
            ASTNode::GunaNode { vama, dakshina } => {
                let lhs = self.compile_node(vama)?.ok_or_else(|| DevvaniLLVMError::CodeGenError("Guna: no vama".into()))?;
                let rhs = self.compile_node(dakshina)?.ok_or_else(|| DevvaniLLVMError::CodeGenError("Guna: no dakshina".into()))?;
                match (lhs, rhs) {
                    (BasicValueEnum::IntValue(l), BasicValueEnum::IntValue(r)) => {
                        Ok(Some(self.builder.build_int_mul(l, r, "multmp").map_err(|e| DevvaniLLVMError::CodeGenError(e.to_string()))?.into()))
                    }
                    (BasicValueEnum::FloatValue(l), BasicValueEnum::FloatValue(r)) => {
                        Ok(Some(self.builder.build_float_mul(l, r, "fmultmp").map_err(|e| DevvaniLLVMError::CodeGenError(e.to_string()))?.into()))
                    }
                    _ => Err(DevvaniLLVMError::CodeGenError("Guna: type mismatch".into()))
                }
            }
            ASTNode::BhagaNode { vama, dakshina } => {
                let lhs = self.compile_node(vama)?.ok_or_else(|| DevvaniLLVMError::CodeGenError("Bhaga: no vama".into()))?;
                let rhs = self.compile_node(dakshina)?.ok_or_else(|| DevvaniLLVMError::CodeGenError("Bhaga: no dakshina".into()))?;
                match (lhs, rhs) {
                    (BasicValueEnum::IntValue(l), BasicValueEnum::IntValue(r)) => {
                        Ok(Some(self.builder.build_int_signed_div(l, r, "divtmp").map_err(|e| DevvaniLLVMError::CodeGenError(e.to_string()))?.into()))
                    }
                    (BasicValueEnum::FloatValue(l), BasicValueEnum::FloatValue(r)) => {
                        Ok(Some(self.builder.build_float_div(l, r, "fdivtmp").map_err(|e| DevvaniLLVMError::CodeGenError(e.to_string()))?.into()))
                    }
                    _ => Err(DevvaniLLVMError::CodeGenError("Bhaga: type mismatch".into()))
                }
            }
            ASTNode::SamaNode { vama, dakshina } => {
                let lhs = self.compile_node(vama)?.ok_or_else(|| DevvaniLLVMError::CodeGenError("Sama: no vama".into()))?;
                let rhs = self.compile_node(dakshina)?.ok_or_else(|| DevvaniLLVMError::CodeGenError("Sama: no dakshina".into()))?;
                match (lhs, rhs) {
                    (BasicValueEnum::IntValue(l), BasicValueEnum::IntValue(r)) => {
                        Ok(Some(self.builder.build_int_compare(inkwell::IntPredicate::EQ, l, r, "eqtmp").map_err(|e| DevvaniLLVMError::CodeGenError(e.to_string()))?.into()))
                    }
                    (BasicValueEnum::FloatValue(l), BasicValueEnum::FloatValue(r)) => {
                        Ok(Some(self.builder.build_float_compare(inkwell::FloatPredicate::OEQ, l, r, "feqtmp").map_err(|e| DevvaniLLVMError::CodeGenError(e.to_string()))?.into()))
                    }
                    _ => Err(DevvaniLLVMError::CodeGenError("Sama: type mismatch".into()))
                }
            }
            ASTNode::AsamaNode { vama, dakshina } => {
                let lhs = self.compile_node(vama)?.ok_or_else(|| DevvaniLLVMError::CodeGenError("Asama: no vama".into()))?;
                let rhs = self.compile_node(dakshina)?.ok_or_else(|| DevvaniLLVMError::CodeGenError("Asama: no dakshina".into()))?;
                match (lhs, rhs) {
                    (BasicValueEnum::IntValue(l), BasicValueEnum::IntValue(r)) => {
                        Ok(Some(self.builder.build_int_compare(inkwell::IntPredicate::NE, l, r, "netmp").map_err(|e| DevvaniLLVMError::CodeGenError(e.to_string()))?.into()))
                    }
                    (BasicValueEnum::FloatValue(l), BasicValueEnum::FloatValue(r)) => {
                        Ok(Some(self.builder.build_float_compare(inkwell::FloatPredicate::ONE, l, r, "fnetmp").map_err(|e| DevvaniLLVMError::CodeGenError(e.to_string()))?.into()))
                    }
                    _ => Err(DevvaniLLVMError::CodeGenError("Asama: type mismatch".into()))
                }
            }
            ASTNode::NyuunaNode { vama, dakshina } => {
                let lhs = self.compile_node(vama)?.ok_or_else(|| DevvaniLLVMError::CodeGenError("Nyuuna: no vama".into()))?;
                let rhs = self.compile_node(dakshina)?.ok_or_else(|| DevvaniLLVMError::CodeGenError("Nyuuna: no dakshina".into()))?;
                match (lhs, rhs) {
                    (BasicValueEnum::IntValue(l), BasicValueEnum::IntValue(r)) => {
                        Ok(Some(self.builder.build_int_compare(inkwell::IntPredicate::SLT, l, r, "slttmp").map_err(|e| DevvaniLLVMError::CodeGenError(e.to_string()))?.into()))
                    }
                    (BasicValueEnum::FloatValue(l), BasicValueEnum::FloatValue(r)) => {
                        Ok(Some(self.builder.build_float_compare(inkwell::FloatPredicate::OLT, l, r, "folttmp").map_err(|e| DevvaniLLVMError::CodeGenError(e.to_string()))?.into()))
                    }
                    _ => Err(DevvaniLLVMError::CodeGenError("Nyuuna: type mismatch".into()))
                }
            }
            ASTNode::AdhikaNode { vama, dakshina } => {
                let lhs = self.compile_node(vama)?.ok_or_else(|| DevvaniLLVMError::CodeGenError("Adhika: no vama".into()))?;
                let rhs = self.compile_node(dakshina)?.ok_or_else(|| DevvaniLLVMError::CodeGenError("Adhika: no dakshina".into()))?;
                match (lhs, rhs) {
                    (BasicValueEnum::IntValue(l), BasicValueEnum::IntValue(r)) => {
                        Ok(Some(self.builder.build_int_compare(inkwell::IntPredicate::SGT, l, r, "sgttmp").map_err(|e| DevvaniLLVMError::CodeGenError(e.to_string()))?.into()))
                    }
                    (BasicValueEnum::FloatValue(l), BasicValueEnum::FloatValue(r)) => {
                        Ok(Some(self.builder.build_float_compare(inkwell::FloatPredicate::OGT, l, r, "fogttmp").map_err(|e| DevvaniLLVMError::CodeGenError(e.to_string()))?.into()))
                    }
                    _ => Err(DevvaniLLVMError::CodeGenError("Adhika: type mismatch".into()))
                }
            }
            ASTNode::VadatiNode { mulya } => {
                let val = self.compile_node(mulya)?.ok_or_else(|| DevvaniLLVMError::CodeGenError("Vadati: no mulya".into()))?;
                self.compile_printf(val)?;
                Ok(None)
            }
            ASTNode::PathatiNode { naama } => {
                let val = self.compile_scanf()?;
                self.variables.insert(naama.clone(), val);
                Ok(Some(val))
            }
            ASTNode::YadiNode { sthiti, tarhi, anyatha } => {
                self.compile_yadi(sthiti, tarhi, anyatha.as_deref())
            }
            ASTNode::YavatNode { sthiti, shareera } => {
                self.compile_yavat(sthiti, shareera)
            }
            ASTNode::PunahNode { varam, shareera } => {
                self.compile_punah(varam, shareera)
            }
            ASTNode::KriyaCall { kriya, karma, .. } => {
                self.compile_function_call(kriya, karma)
            }
            _ => Ok(None),
        }
    }

    fn compile_function(
        &mut self,
        name: &str,
        params: &[KarakaParam],
        body: &[ASTNode],
    ) -> Result<FunctionValue<'ctx>, DevvaniLLVMError> {
        let i64_type = self.context.i64_type();
        let param_types: Vec<inkwell::types::BasicMetadataTypeEnum> = params
            .iter()
            .map(|_| i64_type.into())
            .collect();

        let fn_type = i64_type.fn_type(&param_types, false);
        let function = self.module.add_function(name, fn_type, None);
        let entry_block = self.context.append_basic_block(function, "entry");
        self.builder.position_at_end(entry_block);

        for (i, param_val) in function.get_param_iter().enumerate() {
            if let Some(param_info) = params.get(i) {
                param_val.set_name(&param_info.name);
                self.variables.insert(param_info.name.clone(), param_val.into());
            }
        }

        for stmt in body {
            self.compile_node(stmt)?;
        }

        if function.get_last_basic_block().and_then(|bb| bb.get_terminator()).is_none() {
            self.builder.build_return(Some(&i64_type.const_int(0, false))).map_err(|e| DevvaniLLVMError::CodeGenError(e.to_string()))?;
        }

        Ok(function)
    }

    fn compile_yadi(&mut self, sthiti: &ASTNode, tarhi: &[ASTNode], anyatha: Option<&[ASTNode]>) -> Result<Option<BasicValueEnum<'ctx>>, DevvaniLLVMError> {
        let cond = self.compile_node(sthiti)?.ok_or_else(|| DevvaniLLVMError::CodeGenError("Yadi: no sthiti".into()))?.into_int_value();
        let parent = self.builder.get_insert_block().unwrap().get_parent().unwrap();
        let then_bb = self.context.append_basic_block(parent, "then");
        let else_bb = self.context.append_basic_block(parent, "else");
        let merge_bb = self.context.append_basic_block(parent, "ifcont");

        self.builder.build_conditional_branch(cond, then_bb, else_bb).map_err(|e| DevvaniLLVMError::CodeGenError(e.to_string()))?;

        self.builder.position_at_end(then_bb);
        for stmt in tarhi { self.compile_node(stmt)?; }
        self.builder.build_unconditional_branch(merge_bb).map_err(|e| DevvaniLLVMError::CodeGenError(e.to_string()))?;

        self.builder.position_at_end(else_bb);
        if let Some(body) = anyatha {
            for stmt in body { self.compile_node(stmt)?; }
        }
        self.builder.build_unconditional_branch(merge_bb).map_err(|e| DevvaniLLVMError::CodeGenError(e.to_string()))?;

        self.builder.position_at_end(merge_bb);
        Ok(None)
    }

    fn compile_yavat(&mut self, sthiti: &ASTNode, shareera: &[ASTNode]) -> Result<Option<BasicValueEnum<'ctx>>, DevvaniLLVMError> {
        let parent = self.builder.get_insert_block().unwrap().get_parent().unwrap();
        let cond_bb = self.context.append_basic_block(parent, "whilecond");
        let body_bb = self.context.append_basic_block(parent, "whilebody");
        let after_bb = self.context.append_basic_block(parent, "afterwhile");

        self.builder.build_unconditional_branch(cond_bb).map_err(|e| DevvaniLLVMError::CodeGenError(e.to_string()))?;
        self.builder.position_at_end(cond_bb);
        let cond = self.compile_node(sthiti)?.ok_or_else(|| DevvaniLLVMError::CodeGenError("Yavat: no sthiti".into()))?.into_int_value();
        self.builder.build_conditional_branch(cond, body_bb, after_bb).map_err(|e| DevvaniLLVMError::CodeGenError(e.to_string()))?;

        self.builder.position_at_end(body_bb);
        for stmt in shareera { self.compile_node(stmt)?; }
        self.builder.build_unconditional_branch(cond_bb).map_err(|e| DevvaniLLVMError::CodeGenError(e.to_string()))?;

        self.builder.position_at_end(after_bb);
        Ok(None)
    }

    fn compile_punah(&mut self, varam: &ASTNode, shareera: &[ASTNode]) -> Result<Option<BasicValueEnum<'ctx>>, DevvaniLLVMError> {
        let n = self.compile_node(varam)?.ok_or_else(|| DevvaniLLVMError::CodeGenError("Punah: no varam".into()))?.into_int_value();
        let parent = self.builder.get_insert_block().unwrap().get_parent().unwrap();
        
        let i_ptr = self.builder.build_alloca(self.context.i64_type(), "i").map_err(|e| DevvaniLLVMError::CodeGenError(e.to_string()))?;
        self.builder.build_store(i_ptr, self.context.i64_type().const_int(0, false)).map_err(|e| DevvaniLLVMError::CodeGenError(e.to_string()))?;

        let cond_bb = self.context.append_basic_block(parent, "loopcond");
        let body_bb = self.context.append_basic_block(parent, "loopbody");
        let after_bb = self.context.append_basic_block(parent, "afterloop");

        self.builder.build_unconditional_branch(cond_bb).map_err(|e| DevvaniLLVMError::CodeGenError(e.to_string()))?;
        self.builder.position_at_end(cond_bb);
        let i_val = self.builder.build_load(i_ptr, "i_load").map_err(|e| DevvaniLLVMError::CodeGenError(e.to_string()))?.into_int_value();
        let cond = self.builder.build_int_compare(inkwell::IntPredicate::SLT, i_val, n, "lttmp").map_err(|e| DevvaniLLVMError::CodeGenError(e.to_string()))?;
        self.builder.build_conditional_branch(cond, body_bb, after_bb).map_err(|e| DevvaniLLVMError::CodeGenError(e.to_string()))?;

        self.builder.position_at_end(body_bb);
        for stmt in shareera { self.compile_node(stmt)?; }
        let next_i = self.builder.build_int_add(i_val, self.context.i64_type().const_int(1, false), "nexti").map_err(|e| DevvaniLLVMError::CodeGenError(e.to_string()))?;
        self.builder.build_store(i_ptr, next_i).map_err(|e| DevvaniLLVMError::CodeGenError(e.to_string()))?;
        self.builder.build_unconditional_branch(cond_bb).map_err(|e| DevvaniLLVMError::CodeGenError(e.to_string()))?;

        self.builder.position_at_end(after_bb);
        Ok(None)
    }

    fn compile_printf(&mut self, val: BasicValueEnum<'ctx>) -> Result<(), DevvaniLLVMError> {
        let printf = self.module.get_function("printf").unwrap_or_else(|| {
            let printf_type = self.context.i32_type().fn_type(&[self.context.i8_type().ptr_type(inkwell::AddressSpace::default()).into()], true);
            self.module.add_function("printf", printf_type, None)
        });

        let fmt = match val {
            BasicValueEnum::IntValue(_) => "%lld\n",
            BasicValueEnum::FloatValue(_) => "%f\n",
            _ => "%s\n",
        };
        let fmt_ptr = self.builder.build_global_string_ptr(fmt, "fmt").map_err(|e| DevvaniLLVMError::CodeGenError(e.to_string()))?;
        self.builder.build_call(printf, &[fmt_ptr.as_pointer_value().into(), val.into()], "printftmp").map_err(|e| DevvaniLLVMError::CodeGenError(e.to_string()))?;
        Ok(())
    }

    fn compile_scanf(&mut self) -> Result<BasicValueEnum<'ctx>, DevvaniLLVMError> {
        let scanf = self.module.get_function("scanf").unwrap_or_else(|| {
            let scanf_type = self.context.i32_type().fn_type(&[self.context.i8_type().ptr_type(inkwell::AddressSpace::default()).into()], true);
            self.module.add_function("scanf", scanf_type, None)
        });

        let dest = self.builder.build_alloca(self.context.i64_type(), "scanftmp").map_err(|e| DevvaniLLVMError::CodeGenError(e.to_string()))?;
        let fmt_ptr = self.builder.build_global_string_ptr("%lld", "fmtscan").map_err(|e| DevvaniLLVMError::CodeGenError(e.to_string()))?;
        self.builder.build_call(scanf, &[fmt_ptr.as_pointer_value().into(), dest.into()], "scanftmpcall").map_err(|e| DevvaniLLVMError::CodeGenError(e.to_string()))?;
        Ok(self.builder.build_load(dest, "scanload").map_err(|e| DevvaniLLVMError::CodeGenError(e.to_string()))?)
    }

    fn compile_function_call(&mut self, name: &str, args: &[ASTNode]) -> Result<Option<BasicValueEnum<'ctx>>, DevvaniLLVMError> {
        let function = self.module.get_function(name).ok_or_else(|| DevvaniLLVMError::CodeGenError(format!("Unknown function: {}", name)))?;
        let mut compiled_args = Vec::new();
        for arg in args {
            let val = self.compile_node(arg)?.ok_or_else(|| DevvaniLLVMError::CodeGenError("FunctionCall arg has no value".into()))?;
            compiled_args.push(val.into());
        }
        let call = self.builder.build_call(function, &compiled_args, "calltmp").map_err(|e| DevvaniLLVMError::CodeGenError(e.to_string()))?;
        match call.try_as_basic_value() {
            inkwell::values::ValueKind::Basic(v) => Ok(Some(v)),
            inkwell::values::ValueKind::Instruction(_) => Ok(None),
        }
    }
}
