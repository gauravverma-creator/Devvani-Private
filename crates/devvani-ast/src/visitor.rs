use crate::node::ASTNode;

pub type VisitResult = Result<(), String>;

pub trait ASTVisitor {
    fn visit_karyakram(&mut self, shareera: &[ASTNode]) -> VisitResult;
    fn visit_dhatu_def(&mut self, node: &ASTNode) -> VisitResult;
    fn visit_kriya_call(&mut self, node: &ASTNode) -> VisitResult;
    fn visit_nama(&mut self, node: &ASTNode) -> VisitResult;
    fn visit_asti(&mut self, node: &ASTNode) -> VisitResult;
    fn visit_bhavati(&mut self, node: &ASTNode) -> VisitResult;
    fn visit_arithmetic(&mut self, node: &ASTNode) -> VisitResult;
    fn visit_comparison(&mut self, node: &ASTNode) -> VisitResult;
    fn visit_io(&mut self, node: &ASTNode) -> VisitResult;
    fn visit_yadi(&mut self, node: &ASTNode) -> VisitResult;
    fn visit_yavat(&mut self, node: &ASTNode) -> VisitResult;
    fn visit_punah(&mut self, node: &ASTNode) -> VisitResult;
    fn visit_literal(&mut self, node: &ASTNode) -> VisitResult;
    fn visit_samasa(&mut self, node: &ASTNode) -> VisitResult;
    fn visit_krit_chain(&mut self, node: &ASTNode) -> VisitResult;
    fn visit_avatarana(&mut self, node: &ASTNode) -> VisitResult;
    fn visit_pankti(&mut self, node: &ASTNode) -> VisitResult;
    fn visit_vinyasa(&mut self, node: &ASTNode) -> VisitResult;

    fn visit(&mut self, node: &ASTNode) -> VisitResult {
        match node {
            ASTNode::KaryakramNode { shareera, .. } => self.visit_karyakram(shareera),
            ASTNode::DhatuDef { .. } => self.visit_dhatu_def(node),
            ASTNode::KriyaCall { .. } => self.visit_kriya_call(node),
            ASTNode::Nama { .. } => self.visit_nama(node),
            ASTNode::AstiNode { .. } => self.visit_asti(node),
            ASTNode::BhavatiNode { .. } => self.visit_bhavati(node),
            ASTNode::YogaNode { .. }
            | ASTNode::ViyogaNode { .. }
            | ASTNode::GunaNode { .. }
            | ASTNode::BhagaNode { .. } => self.visit_arithmetic(node),
            ASTNode::SamaNode { .. }
            | ASTNode::AsamaNode { .. }
            | ASTNode::NyuunaNode { .. }
            | ASTNode::AdhikaNode { .. } => self.visit_comparison(node),
            ASTNode::VadatiNode { .. } | ASTNode::PathatiNode { .. } => self.visit_io(node),
            ASTNode::YadiNode { .. } => self.visit_yadi(node),
            ASTNode::YavatNode { .. } => self.visit_yavat(node),
            ASTNode::PunahNode { .. } => self.visit_punah(node),
            ASTNode::PurnaankLiteral { .. }
            | ASTNode::DashaamshaLiteral { .. }
            | ASTNode::VaakLiteral { .. } => self.visit_literal(node),
            ASTNode::Samasa { .. } => self.visit_samasa(node),
            ASTNode::KritChain { .. } => self.visit_krit_chain(node),
            ASTNode::AvartanaNode { .. } => self.visit_avatarana(node),
            ASTNode::PanktiNode { .. } => self.visit_pankti(node),
            ASTNode::VinyasaNode { .. } => self.visit_vinyasa(node),
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
        for _ in 0..self.indent {
            print!("  ");
        }
    }
}

impl ASTVisitor for PrettyPrinter {
    fn visit_karyakram(&mut self, shareera: &[ASTNode]) -> VisitResult {
        println!("Karyakram");
        self.indent += 1;
        for stmt in shareera {
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

    fn visit_asti(&mut self, node: &ASTNode) -> VisitResult {
        if let ASTNode::AstiNode { naama, .. } = node {
            self.print_indent();
            println!("Asti [{}]", naama);
        }
        Ok(())
    }

    fn visit_bhavati(&mut self, node: &ASTNode) -> VisitResult {
        if let ASTNode::BhavatiNode { naama, .. } = node {
            self.print_indent();
            println!("Bhavati [{}]", naama);
        }
        Ok(())
    }

    fn visit_arithmetic(&mut self, node: &ASTNode) -> VisitResult {
        self.print_indent();
        println!("Arithmetic {:?}", node);
        Ok(())
    }

    fn visit_comparison(&mut self, node: &ASTNode) -> VisitResult {
        self.print_indent();
        println!("Comparison {:?}", node);
        Ok(())
    }

    fn visit_io(&mut self, node: &ASTNode) -> VisitResult {
        self.print_indent();
        println!("IO {:?}", node);
        Ok(())
    }

    fn visit_yadi(&mut self, _node: &ASTNode) -> VisitResult {
        self.print_indent();
        println!("Yadi");
        Ok(())
    }

    fn visit_yavat(&mut self, _node: &ASTNode) -> VisitResult {
        self.print_indent();
        println!("Yavat");
        Ok(())
    }

    fn visit_punah(&mut self, _node: &ASTNode) -> VisitResult {
        self.print_indent();
        println!("Punah");
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

    fn visit_avatarana(&mut self, node: &ASTNode) -> VisitResult {
        if let ASTNode::AvartanaNode { call, .. } = node {
            self.print_indent();
            println!("Avartana");
            self.indent += 1;
            self.visit(call)?;
            self.indent -= 1;
        }
        Ok(())
    }

    fn visit_pankti(&mut self, node: &ASTNode) -> VisitResult {
        if let ASTNode::PanktiNode { elements, .. } = node {
            self.print_indent();
            println!("Pankti [{} elements]", elements.len());
            self.indent += 1;
            for elem in elements {
                self.visit(elem)?;
            }
            self.indent -= 1;
        }
        Ok(())
    }

    fn visit_vinyasa(&mut self, node: &ASTNode) -> VisitResult {
        if let ASTNode::VinyasaNode { target, index, .. } = node {
            self.print_indent();
            println!("Vinyasa");
            self.indent += 1;
            self.visit(target)?;
            self.visit(index)?;
            self.indent -= 1;
        }
        Ok(())
    }
}
