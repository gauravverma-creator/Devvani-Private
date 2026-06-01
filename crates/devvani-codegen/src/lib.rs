use inkwell::context::Context;
use inkwell::builder::Builder;
use inkwell::module::Module;
use inkwell::values::*;
use inkwell::types::*;
use inkwell::AddressSpace;
use inkwell::IntPredicate;
use std::collections::HashMap;
use devvani_ast::*;

pub struct DevvaniCodegen<'ctx> {
    pub context: &'ctx Context,
    pub builder: Builder<'ctx>,
    pub module: Module<'ctx>,
    variables: HashMap<String, PointerValue<'ctx>>,
    functions: HashMap<String, FunctionValue<'ctx>>,
}

#[derive(Debug)]
pub enum CodegenError {
    UndefinedVariable(String),
    UndefinedFunction(String),
    TypeMismatch(String),
    LLVMError(String),
    UnsupportedNode(String),
}

#[derive(Debug, Clone, Copy)]
pub enum TargetArch {
    Native,
    X86_64,
    Arm64,
    Wasm32,
}

impl<'ctx> DevvaniCodegen<'ctx> {
    pub fn new(context: &'ctx Context, module_name: &str) -> Self {
        let builder = context.create_builder();
        let module = context.create_module(module_name);
        Self {
            context,
            builder,
            module,
            variables: HashMap::new(),
            functions: HashMap::new(),
        }
    }

    pub fn compile_program(&mut self, program: &ASTNode) -> Result<(), CodegenError> {
        self.declare_builtins();
        
        let i32_type = self.context.i32_type();
        let main_fn_type = i32_type.fn_type(&[], false);
        let main_fn = self.module.add_function("main", main_fn_type, None);
        let entry_bb = self.context.append_basic_block(main_fn, "entry");
        self.builder.position_at_end(entry_bb);

        if let ASTNode::Program { statements, .. } = program {
            for stmt in statements {
                self.generate_statement(stmt)?;
            }
        }
        
        let exit_fn = self.module.get_function("exit").unwrap();
        self.builder.build_call(exit_fn, &[i32_type.const_int(0, false).into()], "callexit").map_err(|e| CodegenError::LLVMError(format!("{:?}", e)))?;
        
        if entry_bb.get_terminator().is_none() {
            self.builder.build_return(Some(&i32_type.const_int(0, false))).map_err(|e| CodegenError::LLVMError(format!("{:?}", e)))?;
        } else if let Some(term) = entry_bb.get_terminator() {
            if term.get_opcode() != inkwell::values::InstructionOpcode::Return {
                 self.builder.build_return(Some(&i32_type.const_int(0, false))).map_err(|e| CodegenError::LLVMError(format!("{:?}", e)))?;
            }
        }
        
        Ok(())
    }

    fn generate_statement(&mut self, node: &ASTNode) -> Result<(), CodegenError> {
        match node {
            ASTNode::DhatuDef { name, params, body, .. } => {
                let param_names: Vec<String> = params.iter().map(|p| p.name.clone()).collect();
                self.compile_function(name, &param_names, body)?;
                Ok(())
            }
            ASTNode::Return { value, .. } => {
                if let Some(val) = value {
                    let ret_val = self.generate_expression(val)?;
                    self.builder.build_return(Some(&ret_val)).map_err(|e| CodegenError::LLVMError(format!("{:?}", e)))?;
                } else {
                    self.builder.build_return(None).map_err(|e| CodegenError::LLVMError(format!("{:?}", e)))?;
                }
                Ok(())
            }
            ASTNode::Conditional { condition, then_branch, else_branch, .. } => {
                self.compile_if(condition, then_branch, else_branch)
            }
            ASTNode::Loop { condition, body, .. } => {
                self.compile_while(condition, body)
            }
            ASTNode::KriyaCall { .. } | ASTNode::BinaryExpr { .. } | ASTNode::UnaryExpr { .. } |
            ASTNode::Nama { .. } | ASTNode::Samasa { .. } | ASTNode::IntLiteral { .. } |
            ASTNode::FloatLiteral { .. } | ASTNode::StringLiteral { .. } | ASTNode::BoolLiteral { .. } |
            ASTNode::Dvandva { .. } | ASTNode::KritChain { .. } => {
                self.generate_expression(node)?;
                Ok(())
            }
            ASTNode::Comment { .. } => Ok(()),
            _ => Err(CodegenError::UnsupportedNode(format!("{:?}", node))),
        }
    }

    fn generate_expression(&mut self, expr: &ASTNode) -> Result<BasicValueEnum<'ctx>, CodegenError> {
        match expr {
            ASTNode::Nama { base, .. } => {
                let ptr = self.variables.get(base).ok_or_else(|| CodegenError::UndefinedVariable(base.clone()))?;
                let val = self.builder.build_load(self.context.i64_type(), *ptr, base).map_err(|e| CodegenError::LLVMError(format!("{:?}", e)))?;
                Ok(val)
            }
            ASTNode::IntLiteral { value, .. } => {
                Ok(self.context.i64_type().const_int(*value as u64, false).into())
            }
            ASTNode::FloatLiteral { value, .. } => {
                Ok(self.context.f64_type().const_float(*value).into())
            }
            ASTNode::BoolLiteral { value, .. } => {
                Ok(self.context.bool_type().const_int(if *value { 1 } else { 0 }, false).into())
            }
            ASTNode::StringLiteral { value, .. } => {
                let global = self.builder.build_global_string_ptr(value, "str").map_err(|e| CodegenError::LLVMError(format!("{:?}", e)))?;
                Ok(global.as_basic_value_enum())
            }
            ASTNode::BinaryExpr { left, op, right, .. } => {
                self.compile_binary_op(left, op, right)
            }
            ASTNode::KriyaCall { kriya, karma, .. } => {
                let mut args = Vec::new();
                if !karma.is_empty() {
                    args.push(&karma[0]);
                }
                self.compile_function_call(kriya, &args)
            }
            _ => Err(CodegenError::UnsupportedNode(format!("{:?}", expr))),
        }
    }

    pub fn compile_function(&mut self, name: &str, params: &[String], body: &[ASTNode]) -> Result<FunctionValue<'ctx>, CodegenError> {
        let current_bb = self.builder.get_insert_block();

        let i64_type = self.context.i64_type();
        let param_types: Vec<BasicMetadataTypeEnum> = params.iter().map(|_| i64_type.into()).collect();
        let fn_type = i64_type.fn_type(&param_types, false);
        let function = self.module.add_function(name, fn_type, None);

        let basic_block = self.context.append_basic_block(function, "entry");
        self.builder.position_at_end(basic_block);

        let old_vars = self.variables.clone();
        self.variables.clear();
        for (i, arg) in function.get_param_iter().enumerate() {
            let param_name = &params[i];
            let alloca = self.builder.build_alloca(i64_type, param_name).map_err(|e| CodegenError::LLVMError(format!("{:?}", e)))?;
            self.builder.build_store(alloca, arg).map_err(|e| CodegenError::LLVMError(format!("{:?}", e)))?;
            self.variables.insert(param_name.clone(), alloca);
        }

        for stmt in body {
            self.generate_statement(stmt)?;
        }

        if basic_block.get_terminator().is_none() {
            self.builder.build_return(Some(&i64_type.const_int(0, false))).map_err(|e| CodegenError::LLVMError(format!("{:?}", e)))?;
        }

        self.variables = old_vars;
        self.functions.insert(name.to_string(), function);

        if let Some(bb) = current_bb {
            self.builder.position_at_end(bb);
        }

        Ok(function)
    }

    pub fn compile_if(&mut self, cond: &ASTNode, then: &[ASTNode], else_: &Option<Vec<ASTNode>>) -> Result<(), CodegenError> {
        let cond_val = self.generate_expression(cond)?;
        let cond_bool = self.builder.build_int_compare(IntPredicate::NE, cond_val.into_int_value(), self.context.i64_type().const_int(0, false), "ifcond").map_err(|e| CodegenError::LLVMError(format!("{:?}", e)))?;

        let function = self.builder.get_insert_block().unwrap().get_parent().unwrap();
        let then_bb = self.context.append_basic_block(function, "then");
        let else_bb = self.context.append_basic_block(function, "else");
        let merge_bb = self.context.append_basic_block(function, "ifcont");

        self.builder.build_conditional_branch(cond_bool, then_bb, else_bb).map_err(|e| CodegenError::LLVMError(format!("{:?}", e)))?;

        self.builder.position_at_end(then_bb);
        for stmt in then {
            self.generate_statement(stmt)?;
        }
        self.builder.build_unconditional_branch(merge_bb).map_err(|e| CodegenError::LLVMError(format!("{:?}", e)))?;

        self.builder.position_at_end(else_bb);
        if let Some(else_stmts) = else_ {
            for stmt in else_stmts {
                self.generate_statement(stmt)?;
            }
        }
        self.builder.build_unconditional_branch(merge_bb).map_err(|e| CodegenError::LLVMError(format!("{:?}", e)))?;

        self.builder.position_at_end(merge_bb);
        Ok(())
    }

    pub fn compile_while(&mut self, cond: &Option<Box<ASTNode>>, body: &[ASTNode]) -> Result<(), CodegenError> {
        let function = self.builder.get_insert_block().unwrap().get_parent().unwrap();
        let cond_bb = self.context.append_basic_block(function, "loop_cond");
        let body_bb = self.context.append_basic_block(function, "loop_body");
        let exit_bb = self.context.append_basic_block(function, "loop_exit");

        self.builder.build_unconditional_branch(cond_bb).map_err(|e| CodegenError::LLVMError(format!("{:?}", e)))?;

        self.builder.position_at_end(cond_bb);
        if let Some(c) = cond {
            let cond_val = self.generate_expression(c)?;
            let cond_bool = self.builder.build_int_compare(IntPredicate::NE, cond_val.into_int_value(), self.context.i64_type().const_int(0, false), "whilecond").map_err(|e| CodegenError::LLVMError(format!("{:?}", e)))?;
            self.builder.build_conditional_branch(cond_bool, body_bb, exit_bb).map_err(|e| CodegenError::LLVMError(format!("{:?}", e)))?;
        } else {
            self.builder.build_unconditional_branch(body_bb).map_err(|e| CodegenError::LLVMError(format!("{:?}", e)))?;
        }

        self.builder.position_at_end(body_bb);
        for stmt in body {
            self.generate_statement(stmt)?;
        }
        self.builder.build_unconditional_branch(cond_bb).map_err(|e| CodegenError::LLVMError(format!("{:?}", e)))?;

        self.builder.position_at_end(exit_bb);
        Ok(())
    }

    pub fn compile_binary_op(&mut self, left: &ASTNode, op: &BinaryOp, right: &ASTNode) -> Result<BasicValueEnum<'ctx>, CodegenError> {
        let l = self.generate_expression(left)?;
        let r = self.generate_expression(right)?;

        match op {
            BinaryOp::Add => {
                let res = self.builder.build_int_add(l.into_int_value(), r.into_int_value(), "tmpadd").map_err(|e| CodegenError::LLVMError(format!("{:?}", e)))?;
                Ok(res.into())
            }
            BinaryOp::Sub => {
                let res = self.builder.build_int_sub(l.into_int_value(), r.into_int_value(), "tmpsub").map_err(|e| CodegenError::LLVMError(format!("{:?}", e)))?;
                Ok(res.into())
            }
            BinaryOp::Mul => {
                let res = self.builder.build_int_mul(l.into_int_value(), r.into_int_value(), "tmpmul").map_err(|e| CodegenError::LLVMError(format!("{:?}", e)))?;
                Ok(res.into())
            }
            BinaryOp::Div => {
                let res = self.builder.build_int_signed_div(l.into_int_value(), r.into_int_value(), "tmpdiv").map_err(|e| CodegenError::LLVMError(format!("{:?}", e)))?;
                Ok(res.into())
            }
            BinaryOp::Mod => {
                let res = self.builder.build_int_signed_rem(l.into_int_value(), r.into_int_value(), "tmprem").map_err(|e| CodegenError::LLVMError(format!("{:?}", e)))?;
                Ok(res.into())
            }
            BinaryOp::Eq => {
                let res = self.builder.build_int_compare(IntPredicate::EQ, l.into_int_value(), r.into_int_value(), "tmpeq").map_err(|e| CodegenError::LLVMError(format!("{:?}", e)))?;
                let extended = self.builder.build_int_z_extend(res, self.context.i64_type(), "tmpeqext").map_err(|e| CodegenError::LLVMError(format!("{:?}", e)))?;
                Ok(extended.into())
            }
            BinaryOp::Neq | BinaryOp::NotEq => {
                let res = self.builder.build_int_compare(IntPredicate::NE, l.into_int_value(), r.into_int_value(), "tmpne").map_err(|e| CodegenError::LLVMError(format!("{:?}", e)))?;
                let extended = self.builder.build_int_z_extend(res, self.context.i64_type(), "tmpneext").map_err(|e| CodegenError::LLVMError(format!("{:?}", e)))?;
                Ok(extended.into())
            }
            BinaryOp::Gt => {
                let res = self.builder.build_int_compare(IntPredicate::SGT, l.into_int_value(), r.into_int_value(), "tmpsgt").map_err(|e| CodegenError::LLVMError(format!("{:?}", e)))?;
                let extended = self.builder.build_int_z_extend(res, self.context.i64_type(), "tmpsgtext").map_err(|e| CodegenError::LLVMError(format!("{:?}", e)))?;
                Ok(extended.into())
            }
            BinaryOp::GtEq => {
                let res = self.builder.build_int_compare(IntPredicate::SGE, l.into_int_value(), r.into_int_value(), "tmpsge").map_err(|e| CodegenError::LLVMError(format!("{:?}", e)))?;
                let extended = self.builder.build_int_z_extend(res, self.context.i64_type(), "tmpsgeext").map_err(|e| CodegenError::LLVMError(format!("{:?}", e)))?;
                Ok(extended.into())
            }
            BinaryOp::LtEq => {
                let res = self.builder.build_int_compare(IntPredicate::SLE, l.into_int_value(), r.into_int_value(), "tmpsle").map_err(|e| CodegenError::LLVMError(format!("{:?}", e)))?;
                let extended = self.builder.build_int_z_extend(res, self.context.i64_type(), "tmpsleext").map_err(|e| CodegenError::LLVMError(format!("{:?}", e)))?;
                Ok(extended.into())
            }
            BinaryOp::Lt => {
                let res = self.builder.build_int_compare(IntPredicate::SLT, l.into_int_value(), r.into_int_value(), "tmpslt").map_err(|e| CodegenError::LLVMError(format!("{:?}", e)))?;
                let extended = self.builder.build_int_z_extend(res, self.context.i64_type(), "tmpsltext").map_err(|e| CodegenError::LLVMError(format!("{:?}", e)))?;
                Ok(extended.into())
            }
            BinaryOp::And => {
                let l_bool = self.builder.build_int_compare(IntPredicate::NE, l.into_int_value(), self.context.i64_type().const_int(0, false), "land_l").map_err(|e| CodegenError::LLVMError(format!("{:?}", e)))?;
                let r_bool = self.builder.build_int_compare(IntPredicate::NE, r.into_int_value(), self.context.i64_type().const_int(0, false), "land_r").map_err(|e| CodegenError::LLVMError(format!("{:?}", e)))?;
                let res = self.builder.build_and(l_bool, r_bool, "tmpand").map_err(|e| CodegenError::LLVMError(format!("{:?}", e)))?;
                let extended = self.builder.build_int_z_extend(res, self.context.i64_type(), "tmpandext").map_err(|e| CodegenError::LLVMError(format!("{:?}", e)))?;
                Ok(extended.into())
            }
            BinaryOp::Or => {
                let l_bool = self.builder.build_int_compare(IntPredicate::NE, l.into_int_value(), self.context.i64_type().const_int(0, false), "lor_l").map_err(|e| CodegenError::LLVMError(format!("{:?}", e)))?;
                let r_bool = self.builder.build_int_compare(IntPredicate::NE, r.into_int_value(), self.context.i64_type().const_int(0, false), "lor_r").map_err(|e| CodegenError::LLVMError(format!("{:?}", e)))?;
                let res = self.builder.build_or(l_bool, r_bool, "tmpor").map_err(|e| CodegenError::LLVMError(format!("{:?}", e)))?;
                let extended = self.builder.build_int_z_extend(res, self.context.i64_type(), "tmporext").map_err(|e| CodegenError::LLVMError(format!("{:?}", e)))?;
                Ok(extended.into())
            }
            BinaryOp::Not => {
                 let res = self.builder.build_not(l.into_int_value(), "tmpnot").map_err(|e| CodegenError::LLVMError(format!("{:?}", e)))?;
                 Ok(res.into())
            }
        }
    }

    fn compile_function_call(&mut self, name: &str, args: &[&ASTNode]) -> Result<BasicValueEnum<'ctx>, CodegenError> {
        if name == "मुद्रण" {
            let mut compiled_args = Vec::new();
            for arg in args {
                compiled_args.push(self.generate_expression(arg)?);
            }
            
            let printf = self.module.get_function("printf").unwrap();
            let fflush = self.module.get_function("fflush").unwrap();
            for arg in compiled_args {
                let format_str = if arg.is_pointer_value() {
                    self.builder.build_global_string_ptr("%s\n", "fmt_s").unwrap()
                } else {
                    self.builder.build_global_string_ptr("%lld\n", "fmt_d").unwrap()
                };
                self.builder.build_call(printf, &[format_str.as_pointer_value().into(), arg.into()], "callprintf").map_err(|e| CodegenError::LLVMError(format!("{:?}", e)))?;
                self.builder.build_call(fflush, &[self.context.ptr_type(AddressSpace::default()).const_null().into()], "callfflush").map_err(|e| CodegenError::LLVMError(format!("{:?}", e)))?;
            }
            return Ok(self.context.i64_type().const_int(0, false).into());
        }

        let function = self.module.get_function(name).ok_or_else(|| CodegenError::UndefinedFunction(name.to_string()))?;

        let mut compiled_args = Vec::new();
        for arg in args {
            compiled_args.push(self.generate_expression(arg)?.into());
        }
        let call = self.builder.build_call(function, &compiled_args, "tmpcall").map_err(|e| CodegenError::LLVMError(format!("{:?}", e)))?;
        
        Ok(call.try_as_basic_value().basic().unwrap_or(self.context.i64_type().const_int(0, false).into()))
    }

    pub fn declare_builtins(&mut self) {
        let i32_type = self.context.i32_type();
        let ptr_type = self.context.ptr_type(AddressSpace::default());
        
        let printf_type = i32_type.fn_type(&[ptr_type.into()], true);
        self.module.add_function("printf", printf_type, None);
        
        let fflush_type = i32_type.fn_type(&[ptr_type.into()], false);
        self.module.add_function("fflush", fflush_type, None);
        
        let exit_type = self.context.void_type().fn_type(&[i32_type.into()], false);
        self.module.add_function("exit", exit_type, None);

        let mudran_type = self.context.i64_type().fn_type(&[self.context.i64_type().into()], false);
        let mudran_fn = self.module.add_function("मुद्रण", mudran_type, None);
        let bb = self.context.append_basic_block(mudran_fn, "entry");
        self.builder.position_at_end(bb);
        let format_str = self.builder.build_global_string_ptr("%lld\n", "fmt").unwrap();
        let arg = mudran_fn.get_first_param().unwrap();
        self.builder.build_call(self.module.get_function("printf").unwrap(), &[format_str.as_pointer_value().into(), arg.into()], "callprintf").unwrap();
        self.builder.build_call(self.module.get_function("fflush").unwrap(), &[ptr_type.const_null().into()], "callfflush").unwrap();
        self.builder.build_return(Some(&self.context.i64_type().const_int(0, false))).unwrap();
    }

    pub fn get_ir(&self) -> String {
        self.module.print_to_string().to_string()
    }

    pub fn write_binary(&self, output_path: &str, _target: TargetArch) -> Result<(), CodegenError> {
        self.module.print_to_file(std::path::Path::new(output_path)).map_err(|e| CodegenError::LLVMError(format!("{:?}", e)))?;
        Ok(())
    }
}
