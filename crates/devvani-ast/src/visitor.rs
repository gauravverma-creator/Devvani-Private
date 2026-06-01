
use crate::node::ASTNode;

pub type VisitResult = Result<(), String>;

pub trait ASTVisitor {
    fn visit_program(&mut self, statements: &[ASTNode]) -> VisitResult;
    fn visit_dhatu_def(&mut self, node: &ASTNode) -> VisitResult;
    fn visit_kriya_call(&mut self, node: &ASTNode) -> VisitResult;
    fn visit_nama(&mut self, node: &ASTNode) -> VisitResult;
    fn visit_conditional(&mut self, node: &ASTNode) -> VisitResult;
    fn visit_loop(&mut self, node: &ASTNode) -> VisitResult;
    fn visit_binary_expr(&mut self, node: &ASTNode) -> VisitResult;
    fn visit_literal(&mut self, node: &ASTNode) -> VisitResult;
    fn visit_samasa(&mut self, node: &ASTNode) -> VisitResult;
    fn visit_krit_chain(&mut self, node: &ASTNode) -> VisitResult;
    
    fn visit(&mut self, node: &ASTNode) -> VisitResult {
        match node {
            ASTNode::Program { statements, .. } => self.visit_program(statements),
            ASTNode::DhatuDef { .. } => self.visit_dhatu_def(node),
            ASTNode::KriyaCall { .. } => self.visit_kriya_call(node),
            ASTNode::Nama { .. } => self.visit_nama(node),
            ASTNode::Conditional { .. } => self.visit_conditional(node),
            ASTNode::Loop { .. } => self.visit_loop(node),
            ASTNode::BinaryExpr { .. } => self.visit_binary_expr(node),
            ASTNode::IntLiteral { .. } | ASTNode::FloatLiteral { .. } | 
            ASTNode::StringLiteral { .. } | ASTNode::BoolLiteral { .. } => self.visit_literal(node),
            ASTNode::Samasa { .. } => self.visit_samasa(node),
            ASTNode::KritChain { .. } => self.visit_krit_chain(node),
            _ => Ok(()),
        }
    }
}

pub struct PrettyPrinter {
    pub indent: usize,
}

impl PrettyPrinter {
    pub fn new() -> Self {
        Self { indent: 0 }
    }
    fn print_indent(&self) {
        for _ in 0..self.indent { print!("  "); }
    }
}

impl ASTVisitor for PrettyPrinter {
    fn visit_program(&mut self, statements: &[ASTNode]) -> VisitResult {
        println!("Program");
        self.indent += 1;
        for stmt in statements {
            self.visit(stmt)?;
        }
        self.indent -= 1;
        Ok(())
    }
    
    fn visit_dhatu_def(&mut self, node: &ASTNode) -> VisitResult {
        if let ASTNode::DhatuDef { name, lakara, .. } = node {
            self.print_indent();
            println!("DhatuDef [{}] lakara={:?}", name, lakara);
        }
        Ok(())
    }

    fn visit_kriya_call(&mut self, node: &ASTNode) -> VisitResult {
        if let ASTNode::KriyaCall { kriya, .. } = node {
            self.print_indent();
            println!("KriyaCall [{}]", kriya);
        }
        Ok(())
    }

    fn visit_nama(&mut self, node: &ASTNode) -> VisitResult {
        if let ASTNode::Nama { base, vibhakti, .. } = node {
            self.print_indent();
            println!("Nama [{}] {:?}", base, vibhakti);
        }
        Ok(())
    }

    fn visit_conditional(&mut self, _node: &ASTNode) -> VisitResult {
        self.print_indent();
        println!("Conditional");
        Ok(())
    }

    fn visit_loop(&mut self, _node: &ASTNode) -> VisitResult {
        self.print_indent();
        println!("Loop");
        Ok(())
    }

    fn visit_binary_expr(&mut self, node: &ASTNode) -> VisitResult {
        if let ASTNode::BinaryExpr { op, .. } = node {
            self.print_indent();
            println!("BinaryExpr {:?}", op);
        }
        Ok(())
    }

    fn visit_literal(&mut self, node: &ASTNode) -> VisitResult {
        self.print_indent();
        println!("Literal {:?}", node);
        Ok(())
    }

    fn visit_samasa(&mut self, node: &ASTNode) -> VisitResult {
        if let ASTNode::Samasa { resolved, .. } = node {
            self.print_indent();
            println!("Samasa [{}]", resolved);
        }
        Ok(())
    }

    fn visit_krit_chain(&mut self, _node: &ASTNode) -> VisitResult {
        self.print_indent();
        println!("KritChain");
        Ok(())
    }
}
