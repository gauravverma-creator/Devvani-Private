use devvani_llvm::codegen::IrEmitter;
use devvani_ast::node::{ASTNode, Lakara, Gana, Linga, Vacana, Span};
use inkwell::context::Context;

fn dummy_span() -> Span {
    Span { line: 0, col: 0, len: 0 }
}

#[test]
fn test_pipeline_simple_function() {
    let context = Context::create();
    let mut emitter = IrEmitter::new(&context, "test_pipeline");

    let ast = ASTNode::KaryakramNode {
        shareera: vec![
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
                    ASTNode::PurnaankLiteral { value: 0, span: dummy_span() },
                ],
                span: dummy_span(),
            }
        ],
    };

    let ir = emitter.emit_ir(&ast).unwrap();
    assert!(ir.contains("define"));
    assert!(ir.contains("main"));
}

#[test]
fn test_pipeline_with_variables() {
    let context = Context::create();
    let mut emitter = IrEmitter::new(&context, "test_vars");

    let ast = ASTNode::KaryakramNode {
        shareera: vec![
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
                    ASTNode::AstiNode {
                        naama: "x".to_string(),
                        mulya: Box::new(ASTNode::PurnaankLiteral { value: 42, span: dummy_span() }),
                    },
                    ASTNode::VadatiNode {
                        mulya: Box::new(ASTNode::Nama {
                            base: "x".to_string(),
                            vibhakti: devvani_ast::node::Vibhakti::Prathama,
                            linga: Linga::Pullinga,
                            vacana: Vacana::Eka,
                            span: dummy_span(),
                        })
                    }
                ],
                span: dummy_span(),
            }
        ],
    };

    let ir = emitter.emit_ir(&ast).unwrap();
    assert!(ir.contains("42"));
    assert!(ir.contains("printf"));
}
