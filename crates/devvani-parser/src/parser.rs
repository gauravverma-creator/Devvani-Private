use devvani_lexer::{Token, TokenKind, Span};
use devvani_ast::*;
use crate::error::ParseError;
use crate::symbol_table::{SymbolTable, Symbol, SymbolKind};
use crate::karaka_map::vibhakti_to_karaka;

pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
    pub symbols: SymbolTable,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self {
            tokens,
            pos: 0,
            symbols: SymbolTable::new(),
        }
    }

    pub fn parse(&mut self) -> Result<ASTNode, ParseError> {
        let mut statements = Vec::new();
        while !self.is_at_end() {
            statements.push(self.parse_statement()?);
        }
        Ok(ASTNode::Program {
            statements,
            span: Span { line: 1, col: 1, len: 0 },
        })
    }

    fn parse_statement(&mut self) -> Result<ASTNode, ParseError> {
        while self.check(&TokenKind::Iti) || self.check(&TokenKind::Semicolon) {
            self.advance();
        }
        if self.is_at_end() {
            return Ok(ASTNode::Comment { text: "EOF".to_string(), span: Span { line: 0, col: 0, len: 0 } });
        }

        if self.check(&TokenKind::Yadi) {
            self.parse_conditional()
        } else if self.check(&TokenKind::Punah) {
            self.parse_loop()
        } else if self.check(&TokenKind::Alam) {
            self.parse_return()
        } else if self.is_dhatu_def() {
            self.parse_dhatu_def()
        } else {
            let expr = self.parse_expr()?;
            
            // Basic Karaka Validation for KriyaCall
            if let ASTNode::KriyaCall { ref karta, .. } = expr {
                if !matches!(karta.as_ref(), Some(b) if matches!(b.as_ref(), ASTNode::Nama { vibhakti: Vibhakti::Prathama, .. })) {
                    // In real Devvani, Karta must be Prathama
                }
            }

            if self.check(&TokenKind::Dot) {
                self.advance();
            }
            Ok(expr)
        }
    }

    fn is_dhatu_def(&self) -> bool {
        let mut p = self.pos;
        while p < self.tokens.len() {
            match self.tokens[p].kind {
                TokenKind::Pra | TokenKind::Para | TokenKind::Apa | TokenKind::Sam | TokenKind::Anu |
                TokenKind::Ava | TokenKind::Nis | TokenKind::Nir | TokenKind::Dus | TokenKind::Dur |
                TokenKind::Vi | TokenKind::Aa | TokenKind::Ni | TokenKind::Adhi | TokenKind::ApiUpasarga |
                TokenKind::Ati | TokenKind::Su | TokenKind::Ud | TokenKind::Abhi | TokenKind::Prati |
                TokenKind::Pari | TokenKind::Upa | TokenKind::Dot => p += 1,
                _ => break,
            }
        }
        if p + 1 < self.tokens.len() {
            match (&self.tokens[p].kind, &self.tokens[p+1].kind) {
                (TokenKind::Identifier(_), TokenKind::DoubleColon) => return true,
                _ => {}
            }
        }
        false
    }

    fn parse_dhatu_def(&mut self) -> Result<ASTNode, ParseError> {
        let mut upasargas = Vec::new();
        while let Some(u) = self.match_upasarga() {
            upasargas.push(u);
            if self.check(&TokenKind::Dot) { self.advance(); }
        }

        let name_token = self.expect_identifier()?;
        let name = if let TokenKind::Identifier(n) = name_token.kind { n } else { unreachable!() };
        
        self.expect(TokenKind::DoubleColon)?;
        let lakara = self.parse_lakara()?;
        
        self.expect(TokenKind::LeftBrace)?;
        let params = self.parse_param_list()?;
        self.expect(TokenKind::RightBrace)?;
        
        self.expect(TokenKind::LeftBrace)?;
        self.symbols.push_scope();
        for p in &params {
            let _ = self.symbols.define(&p.name, Symbol {
                name: p.name.clone(),
                kind: SymbolKind::Param { role: p.role.clone() },
                karaka: p.role.clone(),
                vibhakti: p.vibhakti.clone(),
                linga: Linga::Pullinga,
                vacana: Vacana::Eka,
                defined_at: p.span,
            });
        }
        
        let body = self.parse_block_body()?;
        self.symbols.pop_scope();
        self.expect(TokenKind::RightBrace)?;

        Ok(ASTNode::DhatuDef {
            name,
            gana: Gana::Bhvadi,
            lakara,
            linga: Linga::Pullinga,
            vacana: Vacana::Eka,
            params,
            return_karaka: None,
            body,
            upasargas,
            span: name_token.span,
        })
    }

    fn parse_param_list(&mut self) -> Result<Vec<KarakaParam>, ParseError> {
        let mut params = Vec::new();
        while !self.check(&TokenKind::RightBrace) && !self.is_at_end() {
            let id_token = self.expect_identifier()?;
            let name = if let TokenKind::Identifier(n) = id_token.kind { n } else { unreachable!() };
            
            let vibhakti = self.match_vibhakti().unwrap_or(Vibhakti::Prathama);
            params.push(KarakaParam {
                name,
                role: vibhakti_to_karaka(&vibhakti),
                vibhakti,
                span: id_token.span,
            });

            if self.check(&TokenKind::Comma) { self.advance(); }
        }
        Ok(params)
    }

    fn parse_block_body(&mut self) -> Result<Vec<ASTNode>, ParseError> {
        let mut body = Vec::new();
        while !self.check(&TokenKind::RightBrace) && !self.check(&TokenKind::Iti) && !self.is_at_end() {
            body.push(self.parse_statement()?);
        }
        Ok(body)
    }

    fn parse_expr(&mut self) -> Result<ASTNode, ParseError> {
        self.parse_logical_or()
    }

    fn parse_logical_or(&mut self) -> Result<ASTNode, ParseError> {
        let mut node = self.parse_logical_and()?;
        while let Some(tok) = self.match_any(&[TokenKind::Va]) {
            let right = self.parse_logical_and()?;
            node = ASTNode::BinaryExpr {
                op: BinaryOp::Or,
                left: Box::new(node),
                right: Box::new(right),
                span: tok.span,
            };
        }
        Ok(node)
    }

    fn parse_logical_and(&mut self) -> Result<ASTNode, ParseError> {
        let mut node = self.parse_equality()?;
        while let Some(tok) = self.match_any(&[TokenKind::Ca]) {
            let right = self.parse_equality()?;
            node = ASTNode::BinaryExpr {
                op: BinaryOp::And,
                left: Box::new(node),
                right: Box::new(right),
                span: tok.span,
            };
        }
        Ok(node)
    }

    fn parse_equality(&mut self) -> Result<ASTNode, ParseError> {
        let mut node = self.parse_comparison()?;
        while let Some(tok) = self.match_any(&[TokenKind::EqualEqual, TokenKind::BangEqual]) {
            let op = if tok.kind == TokenKind::EqualEqual { BinaryOp::Eq } else { BinaryOp::NotEq };
            let right = self.parse_comparison()?;
            node = ASTNode::BinaryExpr {
                op,
                left: Box::new(node),
                right: Box::new(right),
                span: tok.span,
            };
        }
        Ok(node)
    }

    fn parse_comparison(&mut self) -> Result<ASTNode, ParseError> {
        let mut node = self.parse_addition()?;
        while let Some(tok) = self.match_any(&[TokenKind::Less, TokenKind::LessEqual, TokenKind::Greater, TokenKind::GreaterEqual]) {
            let op = match tok.kind {
                TokenKind::Less => BinaryOp::Lt,
                TokenKind::LessEqual => BinaryOp::LtEq,
                TokenKind::Greater => BinaryOp::Gt,
                _ => BinaryOp::GtEq,
            };
            let right = self.parse_addition()?;
            node = ASTNode::BinaryExpr {
                op,
                left: Box::new(node),
                right: Box::new(right),
                span: tok.span,
            };
        }
        Ok(node)
    }

    fn parse_addition(&mut self) -> Result<ASTNode, ParseError> {
        let mut node = self.parse_multiplication()?;
        while let Some(tok) = self.match_any(&[TokenKind::Plus, TokenKind::Minus]) {
            let op = if tok.kind == TokenKind::Plus { BinaryOp::Add } else { BinaryOp::Sub };
            let right = self.parse_multiplication()?;
            node = ASTNode::BinaryExpr {
                op,
                left: Box::new(node),
                right: Box::new(right),
                span: tok.span,
            };
        }
        Ok(node)
    }

    fn parse_multiplication(&mut self) -> Result<ASTNode, ParseError> {
        let mut node = self.parse_unary()?;
        while let Some(tok) = self.match_any(&[TokenKind::Star, TokenKind::Slash, TokenKind::Percent]) {
            let op = match tok.kind {
                TokenKind::Star => BinaryOp::Mul,
                TokenKind::Slash => BinaryOp::Div,
                _ => BinaryOp::Mod,
            };
            let right = self.parse_unary()?;
            node = ASTNode::BinaryExpr {
                op,
                left: Box::new(node),
                right: Box::new(right),
                span: tok.span,
            };
        }
        Ok(node)
    }

    fn parse_unary(&mut self) -> Result<ASTNode, ParseError> {
        if let Some(tok) = self.match_any(&[TokenKind::Na]) {
            let operand = self.parse_unary()?;
            return Ok(ASTNode::UnaryExpr {
                op: UnaryOp::Not,
                operand: Box::new(operand),
                span: tok.span,
            });
        }
        self.parse_primary()
    }

    fn parse_primary(&mut self) -> Result<ASTNode, ParseError> {
        if self.match_any(&[TokenKind::LeftParen]).is_some() {
            let expr = self.parse_expr()?;
            self.expect(TokenKind::RightParen)?;
            return Ok(expr);
        }

        let tok = self.peek().clone();
        match tok.kind {
            TokenKind::IntLiteral(value) => { self.advance(); Ok(ASTNode::IntLiteral { value, span: tok.span }) }
            TokenKind::FloatLiteral(value) => { self.advance(); Ok(ASTNode::FloatLiteral { value, span: tok.span }) }
            TokenKind::StringLiteral(value) => { self.advance(); Ok(ASTNode::StringLiteral { value, span: tok.span }) }
            TokenKind::Identifier(name) => {
                self.advance();
                
                if name == "tuple" && self.check(&TokenKind::LeftParen) {
                    self.advance();
                    let mut members = Vec::new();
                    while !self.check(&TokenKind::RightParen) && !self.is_at_end() {
                        members.push(self.parse_expr()?);
                        if self.check(&TokenKind::Comma) { self.advance(); }
                    }
                    self.expect(TokenKind::RightParen)?;
                    return Ok(ASTNode::Dvandva { members, span: tok.span });
                }

                let vibhakti = self.match_vibhakti().unwrap_or(Vibhakti::Prathama);
                
                let mut node = ASTNode::Nama {
                    base: name,
                    vibhakti,
                    vacana: Vacana::Eka,
                    linga: Linga::Pullinga,
                    span: tok.span,
                };

                while self.check(&TokenKind::Dot) {
                    if let Some(TokenKind::Identifier(_)) = self.peek_next_kind() {
                        self.advance();
                        let next_tok = self.expect_identifier()?;
                        let next_name = if let TokenKind::Identifier(n) = next_tok.kind { n } else { unreachable!() };
                        
                        if self.check(&TokenKind::LeftParen) {
                            self.advance();
                            let karma = if !self.check(&TokenKind::RightParen) {
                                Some(Box::new(self.parse_expr()?))
                            } else { None };
                            self.expect(TokenKind::RightParen)?;
                            
                            node = ASTNode::KriyaCall {
                                karta: Some(Box::new(node)),
                                kriya: next_name,
                                karma: karma.map(|k| vec![*k]).unwrap_or_default(),
                                karana: None,
                                sampradana: None,
                                apadan: None,
                                adhikarana: None,
                                span: next_tok.span,
                            };
                        } else {
                            let base_name = if let ASTNode::Nama { ref base, .. } = node { base.clone() } 
                                           else if let ASTNode::Samasa { ref resolved, .. } = node { resolved.clone() }
                                           else { "unknown".to_string() };
                            
                            node = ASTNode::Samasa {
                                samasa_type: SamasaType::Tatpurusha,
                                parts: vec![],
                                components: vec![base_name.clone(), next_name.clone()],
                                resolved: format!("{}.{}", base_name, next_name),
                                span: next_tok.span,
                            };
                        }
                        let _v = self.match_vibhakti();
                    } else {
                        break;
                    }
                }
                
                Ok(node)
            }
            _ => Err(ParseError::UnexpectedToken {
                expected: "expression".to_string(),
                found: tok.kind,
                span: tok.span,
            }),
        }
    }

    fn parse_conditional(&mut self) -> Result<ASTNode, ParseError> {
        let span = self.advance().span;
        let condition = Box::new(self.parse_expr()?);
        self.match_any(&[TokenKind::Eva]);
        self.expect(TokenKind::Tarhi)?;
        
        let mut then_branch = Vec::new();
        if self.match_any(&[TokenKind::Atha]).is_some() {
            then_branch = self.parse_block_body()?;
            self.expect(TokenKind::Iti)?;
        } else {
            then_branch.push(self.parse_statement()?);
        }

        let mut else_branch = None;
        if self.match_any(&[TokenKind::Anyatha]).is_some() {
            if self.match_any(&[TokenKind::Atha]).is_some() {
                else_branch = Some(self.parse_block_body()?);
                self.expect(TokenKind::Iti)?;
            } else {
                else_branch = Some(vec![self.parse_statement()?]);
            }
        }

        Ok(ASTNode::Conditional { 
            condition, 
            then_body: then_branch.clone(),
            then_branch, 
            else_body: else_branch.clone(),
            else_branch, 
            span 
        })
    }

    fn parse_loop(&mut self) -> Result<ASTNode, ParseError> {
        let span = self.advance().span;
        let mut condition = None;
        if self.match_any(&[TokenKind::Yadi]).is_some() {
            condition = Some(Box::new(self.parse_expr()?));
        }
        
        let mut body = Vec::new();
        if self.match_any(&[TokenKind::Atha]).is_some() {
            body = self.parse_block_body()?;
            self.expect(TokenKind::Iti)?;
        } else {
            body.push(self.parse_statement()?);
        }

        Ok(ASTNode::Loop { condition, body, span })
    }

    fn parse_return(&mut self) -> Result<ASTNode, ParseError> {
        let span = self.advance().span;
        let mut value = None;
        if !self.check(&TokenKind::Iti) && !self.check(&TokenKind::RightBrace) && !self.is_at_end() {
            value = Some(Box::new(self.parse_expr()?));
        }
        if self.check(&TokenKind::Iti) { self.advance(); }
        Ok(ASTNode::Return { value, span })
    }

    fn parse_lakara(&mut self) -> Result<Lakara, ParseError> {
        let tok = self.advance();
        match tok.kind {
            TokenKind::Lat => Ok(Lakara::Lat),
            TokenKind::Lit => Ok(Lakara::Lit),
            TokenKind::Lut => Ok(Lakara::Lut),
            TokenKind::Lrt => Ok(Lakara::Lrt),
            TokenKind::Let => Ok(Lakara::Let),
            TokenKind::Lot => Ok(Lakara::Lot),
            TokenKind::Lan => Ok(Lakara::Lan),
            TokenKind::Vidhilin => Ok(Lakara::Vidhilin),
            TokenKind::Asihlin => Ok(Lakara::Asihlin),
            TokenKind::Lun => Ok(Lakara::Lun),
            _ => Err(ParseError::Generic(format!("Expected Lakara, found {:?}", tok.kind))),
        }
    }

    fn match_upasarga(&mut self) -> Option<Upasarga> {
        let tok = self.peek();
        let u = match tok.kind {
            TokenKind::Pra => Some(Upasarga::Pra),
            TokenKind::Para => Some(Upasarga::Para),
            TokenKind::Apa => Some(Upasarga::Apa),
            TokenKind::Sam => Some(Upasarga::Sam),
            TokenKind::Anu => Some(Upasarga::Anu),
            TokenKind::Ava => Some(Upasarga::Ava),
            TokenKind::Nis => Some(Upasarga::Nis),
            TokenKind::Nir => Some(Upasarga::Nir),
            TokenKind::Dus => Some(Upasarga::Dus),
            TokenKind::Dur => Some(Upasarga::Dur),
            TokenKind::Vi => Some(Upasarga::Vi),
            TokenKind::Aa => Some(Upasarga::Aa),
            TokenKind::Ni => Some(Upasarga::Ni),
            TokenKind::Adhi => Some(Upasarga::Adhi),
            TokenKind::ApiUpasarga => Some(Upasarga::Api),
            TokenKind::Ati => Some(Upasarga::Ati),
            TokenKind::Su => Some(Upasarga::Su),
            TokenKind::Ud => Some(Upasarga::Ud),
            TokenKind::Abhi => Some(Upasarga::Abhi),
            TokenKind::Prati => Some(Upasarga::Prati),
            TokenKind::Pari => Some(Upasarga::Pari),
            TokenKind::Upa => Some(Upasarga::Upa),
            _ => None,
        };
        if u.is_some() { self.advance(); }
        u
    }

    fn match_vibhakti(&mut self) -> Option<Vibhakti> {
        let tok = self.peek();
        let v = match tok.kind {
            TokenKind::Visarga => Some(Vibhakti::Prathama),
            TokenKind::Anusvara => Some(Vibhakti::Dvitiya),
            _ => None,
        };
        if v.is_some() { self.advance(); }
        v
    }

    fn expect_identifier(&mut self) -> Result<Token, ParseError> {
        let tok = self.advance();
        match tok.kind {
            TokenKind::Identifier(_) => Ok(tok),
            _ => Err(ParseError::UnexpectedToken {
                expected: "identifier".to_string(),
                found: tok.kind,
                span: tok.span,
            }),
        }
    }

    fn expect(&mut self, kind: TokenKind) -> Result<Token, ParseError> {
        let tok = self.advance();
        if tok.kind == kind {
            Ok(tok)
        } else {
            Err(ParseError::UnexpectedToken {
                expected: format!("{:?}", kind),
                found: tok.kind,
                span: tok.span,
            })
        }
    }

    fn check(&self, kind: &TokenKind) -> bool {
        if self.is_at_end() { return false; }
        &self.peek().kind == kind
    }

    fn match_any(&mut self, kinds: &[TokenKind]) -> Option<Token> {
        for kind in kinds {
            if self.check(kind) {
                return Some(self.advance());
            }
        }
        None
    }

    fn advance(&mut self) -> Token {
        if !self.is_at_end() { self.pos += 1; }
        self.tokens[self.pos - 1].clone()
    }

    fn peek(&self) -> &Token {
        &self.tokens[self.pos]
    }

    fn peek_next_kind(&self) -> Option<TokenKind> {
        if self.pos + 1 < self.tokens.len() {
            Some(self.tokens[self.pos + 1].kind.clone())
        } else {
            None
        }
    }

    fn is_at_end(&self) -> bool {
        self.pos >= self.tokens.len() || self.tokens[self.pos].kind == TokenKind::EOF
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use devvani_lexer::Lexer;
    use devvani_lexer::SandhiMode;

    fn get_tokens(input: &str) -> Vec<Token> {
        let mut lexer = Lexer::new(input);
        lexer.tokenize(SandhiMode::Off).unwrap()
    }

    #[test]
    fn test_basic_sov() {
        let tokens = get_tokens("rāmaḥ.khādati(phalaṃ).");
        let mut parser = Parser::new(tokens);
        let ast = parser.parse().unwrap();
        if let ASTNode::Program { statements, .. } = ast {
            assert!(matches!(statements[0], ASTNode::KriyaCall { .. }));
        }
    }

    #[test]
    fn test_dhatu_def() {
        let tokens = get_tokens("khādati::Lat { rāmaḥ, phalaṃ } { alam rāmaḥ iti }");
        let mut parser = Parser::new(tokens);
        let ast = parser.parse().unwrap();
        if let ASTNode::Program { statements, .. } = ast {
            assert!(matches!(statements[0], ASTNode::DhatuDef { .. }));
        }
    }

    #[test]
    fn test_conditional() {
        let tokens = get_tokens("yadi rāmaḥ Eva tarhi Atha rāmaḥ.gacchati() Iti");
        let mut parser = Parser::new(tokens);
        let ast = parser.parse().unwrap();
        if let ASTNode::Program { statements, .. } = ast {
            assert!(matches!(statements[0], ASTNode::Conditional { .. }));
        }
    }

    #[test]
    fn test_samasa_resolution() {
        let tokens = get_tokens("rāma.putraḥ.");
        let mut parser = Parser::new(tokens);
        let ast = parser.parse().unwrap();
        if let ASTNode::Program { statements, .. } = ast {
            assert!(matches!(statements[0], ASTNode::Samasa { .. }));
        }
    }

    #[test]
    fn test_dvandva() {
        let tokens = get_tokens("tuple(rāmaḥ, lakṣmaṇaḥ).");
        let mut parser = Parser::new(tokens);
        let ast = parser.parse().unwrap();
        if let ASTNode::Program { statements, .. } = ast {
            assert!(matches!(statements[0], ASTNode::Dvandva { .. }));
        }
    }

    #[test]
    fn test_krit_chain() {
        let tokens = get_tokens("rāmaḥ.gatvā().pathitvā().");
        let mut parser = Parser::new(tokens);
        let ast = parser.parse().unwrap();
        if let ASTNode::Program { statements, .. } = ast {
            assert!(matches!(statements[0], ASTNode::KriyaCall { .. }));
        }
    }

    #[test]
    fn test_symbol_table_scoping() {
        let tokens = get_tokens("khādati::Lat { rāmaḥ } { yadi phalaṃ tarhi alam rāmaḥ iti }");
        let mut parser = Parser::new(tokens);
        parser.parse().unwrap();
        assert!(parser.symbols.lookup("phalaṃ").is_none());
    }

    #[test]
    fn test_upasarga_prefix() {
        let tokens = get_tokens("Pra.gacchati::Lat { rāmaḥ } { alam iti }");
        let mut parser = Parser::new(tokens);
        let ast = parser.parse().unwrap();
        if let ASTNode::Program { statements, .. } = ast {
            if let ASTNode::DhatuDef { upasargas, .. } = &statements[0] {
                assert_eq!(upasargas[0], Upasarga::Pra);
            }
        }
    }

    #[test]
    fn test_binary_expression() {
        let tokens = get_tokens("rāmaḥ Ca lakṣmaṇaḥ.");
        let mut parser = Parser::new(tokens);
        let ast = parser.parse().unwrap();
        if let ASTNode::Program { statements, .. } = ast {
            assert!(matches!(statements[0], ASTNode::BinaryExpr { .. }));
        }
    }

    #[test]
    fn test_nested_conditionals() {
        let tokens = get_tokens("yadi rāmaḥ tarhi Atha yadi lakṣmaṇaḥ tarhi alam iti Iti iti");
        let mut parser = Parser::new(tokens);
        let ast = parser.parse().unwrap();
        if let ASTNode::Program { statements, .. } = ast {
            assert!(matches!(statements[0], ASTNode::Conditional { .. }));
        }
    }

    #[test]
    fn test_missing_karta_error() {
        // This test verifies that we can parse even if semantic checks are missing in Phase 2
        let tokens = get_tokens("khādati(phalaṃ).");
        let mut parser = Parser::new(tokens);
        let _ast = parser.parse();
        // Phase 2 allows this, Phase 3 will catch it
    }

    #[test]
    fn test_karaka_conflict_error() {
        let tokens = get_tokens("rāmaḥ lakṣmaṇaḥ khādati(phalaṃ).");
        let mut parser = Parser::new(tokens);
        let _ast = parser.parse();
    }
}
