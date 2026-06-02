use inkwell::context::Context;
use devvani_llvm::codegen::IrEmitter;
use devvani_ast::node::{ASTNode, Lakara, Gana, Linga, Vacana, Span};

fn dummy_span() -> Span {
    Span { line: 0, col: 0, len: 0 }
}

#[test]
fn test_full_ir_pipeline() {
    let context = Context::create();
    let mut emitter = IrEmitter::new(&context, "pipeline_test");

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
    assert!(ir.contains("ret i64 0"));
}

#[test]
fn test_obj_emission() {
    use devvani_llvm::target::DevvaniTarget;
    use inkwell::targets::FileType;
    use std::fs;
    use std::path::Path;

    let context = Context::create();
    let mut emitter = IrEmitter::new(&context, "obj_test");

    let ast = ASTNode::Program {
        statements: vec![
            ASTNode::DhatuDef {
                name: "obj_main".to_string(),
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

    emitter.emit_ir(&ast).unwrap();

    let target = DevvaniTarget::new_native().unwrap();
    let out_path = "/tmp/devvani_test_output.o";
    let out = Path::new(out_path);

    let result = target.machine.write_to_file(
        &emitter.module,
        FileType::Object,
        out,
    );
    
    assert!(result.is_ok());
    assert!(out.exists());

    // Cleanup
    let _ = fs::remove_file(out);
}
