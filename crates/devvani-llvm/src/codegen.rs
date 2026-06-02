use inkwell::values::{BasicValueEnum, FunctionValue};
use std::collections::HashMap;
use crate::error::DevvaniLLVMError;
use devvani_ast::node::{ASTNode, BinaryOp, KarakaParam};

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
        self.compile_node(ast)?;
        Ok(self.module.print_to_string().to_string())
    }

    /// Dispatch on ASTNode variant
    fn compile_node(&mut self, node: &ASTNode) -> Result<Option<BasicValueEnum<'ctx>>, DevvaniLLVMError> {
        match node {
            ASTNode::Program { statements, .. } => {
                for stmt in statements {
                    self.compile_node(stmt)?;
                }
                Ok(None)
            }
            ASTNode::DhatuDef { name, params, body, .. } => {
                self.compile_function(name, params, body)?;
                Ok(None)
            }
            ASTNode::Nama { base, .. } => {
                let val = self.variables.get(base)
                    .ok_or_else(|| DevvaniLLVMError::CodeGenError(format!("Unknown variable: {}", base)))?;
                Ok(Some(*val))
            }
            ASTNode::IntLiteral { value, .. } => {
                let val = self.context.i64_type().const_int(*value as u64, true);
                Ok(Some(val.into()))
            }
            ASTNode::StringLiteral { value, .. } => {
                let val = self.builder.build_global_string_ptr(value, "str")
                    .map_err(|e| DevvaniLLVMError::CodeGenError(e.to_string()))?;
                Ok(Some(val.as_pointer_value().into()))
            }
            ASTNode::BinaryExpr { op, left, right, .. } => {
                self.compile_binary_op(op, left, right)
            }
            ASTNode::Return { value, .. } => {
                self.compile_return(value.as_deref())
            }
            ASTNode::KriyaCall { kriya, karma, .. } => {
                self.compile_function_call(kriya, karma)
            }
            _ => Ok(None),
        }
    }

    /// Compile a Dhatu (function definition)
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

        // Register params in symbol table
        for (i, param_val) in function.get_param_iter().enumerate() {
            if let Some(param_info) = params.get(i) {
                param_val.set_name(&param_info.name);
                self.variables.insert(param_info.name.clone(), param_val.into());
            }
        }

        // Compile body
        for stmt in body {
            self.compile_node(stmt)?;
        }

        // Default return 0 if no explicit return
        if function.get_last_basic_block()
            .and_then(|bb| bb.get_terminator())
            .is_none()
        {
            self.builder.build_return(Some(&i64_type.const_int(0, false)))
                .map_err(|e| DevvaniLLVMError::CodeGenError(e.to_string()))?;
        }

        Ok(function)
    }

    /// Compile BinaryOp: +, -, *, /
    fn compile_binary_op(
        &mut self,
        op: &BinaryOp,
        left: &ASTNode,
        right: &ASTNode,
    ) -> Result<Option<BasicValueEnum<'ctx>>, DevvaniLLVMError> {
        let lhs = self.compile_node(left)?
            .ok_or_else(|| DevvaniLLVMError::CodeGenError("BinaryOp: no lhs".into()))?;
        let rhs = self.compile_node(right)?
            .ok_or_else(|| DevvaniLLVMError::CodeGenError("BinaryOp: no rhs".into()))?;

        let l = lhs.into_int_value();
        let r = rhs.into_int_value();

        let result = match op {
            BinaryOp::Add => self.builder.build_int_add(l, r, "addtmp")
                .map_err(|e| DevvaniLLVMError::CodeGenError(e.to_string()))?,
            BinaryOp::Sub => self.builder.build_int_sub(l, r, "subtmp")
                .map_err(|e| DevvaniLLVMError::CodeGenError(e.to_string()))?,
            BinaryOp::Mul => self.builder.build_int_mul(l, r, "multmp")
                .map_err(|e| DevvaniLLVMError::CodeGenError(e.to_string()))?,
            BinaryOp::Div => self.builder.build_int_signed_div(l, r, "divtmp")
                .map_err(|e| DevvaniLLVMError::CodeGenError(e.to_string()))?,
            _ => return Err(DevvaniLLVMError::CodeGenError(
                format!("Unsupported operator: {:?}", op)
            )),
        };

        Ok(Some(result.into()))
    }

    /// Compile Return statement
    fn compile_return(
        &mut self,
        value: Option<&ASTNode>,
    ) -> Result<Option<BasicValueEnum<'ctx>>, DevvaniLLVMError> {
        let val = if let Some(v) = value {
            self.compile_node(v)?
        } else {
            None
        };
        
        match val {
            Some(v) => {
                self.builder.build_return(Some(&v))
                    .map_err(|e| DevvaniLLVMError::CodeGenError(e.to_string()))?;
            }
            None => {
                self.builder.build_return(None)
                    .map_err(|e| DevvaniLLVMError::CodeGenError(e.to_string()))?;
            }
        }
        Ok(None)
    }

    /// Compile FunctionCall
    fn compile_function_call(
        &mut self,
        name: &str,
        args: &[ASTNode],
    ) -> Result<Option<BasicValueEnum<'ctx>>, DevvaniLLVMError> {
        let function = self.module.get_function(name)
            .ok_or_else(|| DevvaniLLVMError::CodeGenError(
                format!("Unknown function: {}", name)
            ))?;

        let mut compiled_args = Vec::new();
        for arg in args {
            let val = self.compile_node(arg)?
                .ok_or_else(|| DevvaniLLVMError::CodeGenError(
                    "FunctionCall arg has no value".into()
                ))?;
            compiled_args.push(val.into());
        }

        let call = self.builder
            .build_call(function, &compiled_args, "calltmp")
            .map_err(|e| DevvaniLLVMError::CodeGenError(e.to_string()))?;

        match call.try_as_basic_value() {
            inkwell::values::ValueKind::Basic(v) => Ok(Some(v)),
            inkwell::values::ValueKind::Instruction(_) => Ok(None),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use devvani_ast::node::{ASTNode, Lakara, Gana, Linga, Vacana, Span};
    use inkwell::context::Context;

    fn dummy_span() -> Span {
        Span { line: 0, col: 0, len: 0 }
    }

    #[test]
    fn test_emit_simple_function() {
        let context = Context::create();
        let mut emitter = IrEmitter::new(&context, "test");

        let ast = ASTNode::Program {
            statements: vec![
                ASTNode::DhatuDef {
                    name: "main".to_string(),
                    lakara: Lakara::Lat,
                    gana: Gana::Bhvadi,
                    linga: Linga::Pullinga,
                    vacana: Vacana::Eka,
                    params: vec![],
                    upasargas: vec![],
                    return_karaka: None,
                    body: vec![
                        ASTNode::Return {
                            value: Some(Box::new(ASTNode::IntLiteral { value: 0, span: dummy_span() })),
                            span: dummy_span(),
                        },
                    ],
                    span: dummy_span(),
                }
            ],
            span: dummy_span(),
        };

        let ir = emitter.emit_ir(&ast).unwrap();
        assert!(ir.contains("define"));
        assert!(ir.contains("main"));
        assert!(ir.contains("ret"));
    }

    #[test]
    fn test_emit_binary_op() {
        let context = Context::create();
        let mut emitter = IrEmitter::new(&context, "test_binop");

        let ast = ASTNode::Program {
            statements: vec![
                ASTNode::DhatuDef {
                    name: "add".to_string(),
                    lakara: Lakara::Lat,
                    gana: Gana::Bhvadi,
                    linga: Linga::Pullinga,
                    vacana: Vacana::Eka,
                    params: vec![],
                    upasargas: vec![],
                    return_karaka: None,
                    body: vec![
                        ASTNode::Return {
                            value: Some(Box::new(
                                ASTNode::BinaryExpr {
                                    op: BinaryOp::Add,
                                    left: Box::new(ASTNode::IntLiteral { value: 3, span: dummy_span() }),
                                    right: Box::new(ASTNode::IntLiteral { value: 4, span: dummy_span() }),
                                    span: dummy_span(),
                                }
                            )),
                            span: dummy_span(),
                        },
                    ],
                    span: dummy_span(),
                }
            ],
            span: dummy_span(),
        };

        let ir = emitter.emit_ir(&ast).unwrap();
        assert!(ir.contains("add"));
        assert!(ir.contains("ret"));
    }

    #[test]
    fn test_emit_nama_usage() {
        let context = Context::create();
        let mut emitter = IrEmitter::new(&context, "test_nama");
        
        // Manual insertion into symbol table for test
        emitter.variables.insert("x".to_string(), context.i64_type().const_int(42, false).into());

        let ast = ASTNode::Program {
            statements: vec![
                ASTNode::DhatuDef {
                    name: "test".to_string(),
                    lakara: Lakara::Lat,
                    gana: Gana::Bhvadi,
                    linga: Linga::Pullinga,
                    vacana: Vacana::Eka,
                    params: vec![],
                    upasargas: vec![],
                    return_karaka: None,
                    body: vec![
                        ASTNode::Return {
                            value: Some(Box::new(ASTNode::Nama {
                                base: "x".to_string(),
                                vibhakti: devvani_ast::node::Vibhakti::Prathama,
                                linga: Linga::Pullinga,
                                vacana: Vacana::Eka,
                                span: dummy_span(),
                            })),
                            span: dummy_span(),
                        },
                    ],
                    span: dummy_span(),
                }
            ],
            span: dummy_span(),
        };

        let ir = emitter.emit_ir(&ast).unwrap();
        assert!(ir.contains("ret i64 42"));
    }
}
