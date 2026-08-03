use crate::node::ASTNode;

pub type VisitResult = Result<(), String>;

pub trait ASTVisitor {
    fn visit_karyakram(&mut self, shareera: &[ASTNode]) -> VisitResult;
    fn visit_dhatu_def(&mut self, node: &ASTNode) -> VisitResult;
    fn visit_dravya_def(&mut self, node: &ASTNode) -> VisitResult;
    fn visit_nirmana(&mut self, node: &ASTNode) -> VisitResult;
    fn visit_phalam_type(&mut self, node: &ASTNode) -> VisitResult;
    fn visit_arogya(&mut self, node: &ASTNode) -> VisitResult;
    fn visit_dosha(&mut self, node: &ASTNode) -> VisitResult;
    fn visit_nidana(&mut self, node: &ASTNode) -> VisitResult;
    fn visit_samprapti(&mut self, node: &ASTNode) -> VisitResult;
    fn visit_sandarbha(&mut self, node: &ASTNode) -> VisitResult;
    fn visit_samavaya(&mut self, node: &ASTNode) -> VisitResult;
    fn visit_kriya_call(&mut self, node: &ASTNode) -> VisitResult;
    fn visit_nama(&mut self, node: &ASTNode) -> VisitResult;
    fn visit_asti(&mut self, node: &ASTNode) -> VisitResult;
    fn visit_bhavati(&mut self, node: &ASTNode) -> VisitResult;
    fn visit_dhara(&mut self, node: &ASTNode) -> VisitResult;
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
    fn visit_avali(&mut self, node: &ASTNode) -> VisitResult;
    fn visit_vinyasa(&mut self, node: &ASTNode) -> VisitResult;
    fn visit_kramasah(&mut self, node: &ASTNode) -> VisitResult;

    fn visit(&mut self, node: &ASTNode) -> VisitResult {
        match node {
            ASTNode::KaryakramNode { shareera, .. } => self.visit_karyakram(shareera),
            ASTNode::DhatuDef { .. } => self.visit_dhatu_def(node),
            ASTNode::DravyaDef { .. } => self.visit_dravya_def(node),
             ASTNode::NirmanaNode { .. } => self.visit_nirmana(node),
             ASTNode::PhalamType { .. } => self.visit_phalam_type(node),
             ASTNode::ArogyaNode { .. } => self.visit_arogya(node),
             ASTNode::DoshaNode { .. } => self.visit_dosha(node),
ASTNode::NidanaNode { .. } => self.visit_nidana(node),
              ASTNode::SamprapatiNode { .. } => self.visit_samprapti(node),
              ASTNode::SandarbhaNode { .. } => self.visit_sandarbha(node),
              ASTNode::SamavayaNode { .. } => self.visit_samavaya(node),
            ASTNode::KriyaCall { .. } => self.visit_kriya_call(node),
            ASTNode::Nama { .. } => self.visit_nama(node),
            ASTNode::AstiNode { .. } => self.visit_asti(node),
            ASTNode::BhavatiNode { .. } => self.visit_bhavati(node),
            ASTNode::DharaNode { .. } => self.visit_dhara(node),
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
             ASTNode::AvaliNode { .. } => self.visit_avali(node),
             ASTNode::VinyasaNode { .. } => self.visit_vinyasa(node),
            ASTNode::KramashahNode { .. } => self.visit_kramasah(node),
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

    fn visit_dravya_def(&mut self, node: &ASTNode) -> VisitResult {
        if let ASTNode::DravyaDef { name, angas, .. } = node {
            self.print_indent();
            println!("DravyaDef [{}] {} angas", name, angas.len());
        }
        Ok(())
    }

    fn visit_nirmana(&mut self, node: &ASTNode) -> VisitResult {
        if let ASTNode::NirmanaNode { dravya_name, values, .. } = node {
            self.print_indent();
            println!("NirmanaNode [{}] {} values", dravya_name, values.len());
        }
        Ok(())
    }

    fn visit_phalam_type(&mut self, node: &ASTNode) -> VisitResult {
        if let ASTNode::PhalamType { success_type, error_type, .. } = node {
            self.print_indent();
            println!("PhalamType <{}, {}>", success_type, error_type);
        }
        Ok(())
    }

    fn visit_arogya(&mut self, node: &ASTNode) -> VisitResult {
        self.print_indent();
        println!("Arogya");
        self.indent += 1;
        if let ASTNode::ArogyaNode { value, .. } = node {
            self.visit(value)?;
        }
        self.indent -= 1;
        Ok(())
    }

    fn visit_dosha(&mut self, node: &ASTNode) -> VisitResult {
        self.print_indent();
        println!("Dosha");
        self.indent += 1;
        if let ASTNode::DoshaNode { value, .. } = node {
            self.visit(value)?;
        }
        self.indent -= 1;
        Ok(())
    }

    fn visit_nidana(&mut self, node: &ASTNode) -> VisitResult {
        if let ASTNode::NidanaNode { arogya_bind, dosha_bind, arogya_body, dosha_body, .. } = node {
            self.print_indent();
            println!("Nidana [arogya:{}] [dosha:{}]", arogya_bind, dosha_bind);
            self.indent += 1;
            for stmt in arogya_body {
                self.visit(stmt)?;
            }
            for stmt in dosha_body {
                self.visit(stmt)?;
            }
            self.indent -= 1;
        }
        Ok(())
    }

fn visit_samprapti(&mut self, node: &ASTNode) -> VisitResult {
         if let ASTNode::SamprapatiNode { expr, .. } = node {
             self.print_indent();
             println!("Samprapti");
             self.indent += 1;
             self.visit(expr)?;
             self.indent -= 1;
         }
         Ok(())
     }

     fn visit_sandarbha(&mut self, node: &ASTNode) -> VisitResult {
         if let ASTNode::SandarbhaNode { target, is_mutable, .. } = node {
             self.print_indent();
             println!("SandarbhaNode [mutable={}]", is_mutable);
             self.indent += 1;
             self.visit(target)?;
             self.indent -= 1;
         }
         Ok(())
     }

     fn visit_samavaya(&mut self, node: &ASTNode) -> VisitResult {
        if let ASTNode::SamavayaNode { anga_name, target, .. } = node {
            self.print_indent();
            println!("Samavaya .{}", anga_name);
            self.indent += 1;
            self.visit(target)?;
            self.indent -= 1;
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

    fn visit_dhara(&mut self, node: &ASTNode) -> VisitResult {
        if let ASTNode::DharaNode { naama, type_name, .. } = node {
            self.print_indent();
            if let Some(tn) = type_name {
                println!("Dhara [{}:{}]", naama, tn);
            } else {
                println!("Dhara [{}:inferred]", naama);
            }
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

    fn visit_avali(&mut self, node: &ASTNode) -> VisitResult {
        if let ASTNode::AvaliNode { elements, .. } = node {
            self.print_indent();
            println!("Avali [{} elements]", elements.len());
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

    fn visit_kramasah(&mut self, node: &ASTNode) -> VisitResult {
        if let ASTNode::KramashahNode { item_name, iterable, body, .. } = node {
            self.print_indent();
            println!("Kramasah [{}]", item_name);
            self.indent += 1;
            self.visit(iterable)?;
            for stmt in body {
                self.visit(stmt)?;
            }
            self.indent -= 1;
        }
        Ok(())
    }
}
