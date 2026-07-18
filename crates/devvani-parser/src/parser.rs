use devvani_lexer::{Token, TokenKind, Span};
use devvani_ast::*;
use crate::error::ParseError;
use crate::symbol_table::{SymbolTable, Symbol, SymbolKind};
use crate::karaka_map::vibhakti_to_karaka;

pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
    pub symbols: SymbolTable,
    current_dhatu_name: Vec<String>,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self {
            tokens,
            pos: 0,
            symbols: SymbolTable::new(),
            current_dhatu_name: Vec::new(),
        }
    }

    pub fn parse(&mut self) -> Result<ASTNode, ParseError> {
        let mut shareera = Vec::new();
        while !self.is_at_end() {
            shareera.push(self.parse_vakya()?);
        }
        Ok(ASTNode::KaryakramNode {
            shareera,
        })
    }

    fn parse_vakya(&mut self) -> Result<ASTNode, ParseError> {
        while self.check(&TokenKind::Danda) {
            self.advance();
        }
        if self.is_at_end() {
            return Err(ParseError::Generic("Unexpected EOF".to_string()));
        }

        let tok = self.peek().clone();
        match tok.kind {
            TokenKind::Arambhah => self.parse_karyakram(),
            TokenKind::Yadi => self.parse_yadi(),
            TokenKind::Yavat => self.parse_yavat(),
            TokenKind::Naama(ref name) => {
                if name.ends_with("-dhatu") {
                    let lookup = self.symbols.lookup(name);
                    if !matches!(lookup.map(|s| &s.kind), Some(SymbolKind::Dhatu { .. })) {
                        self.parse_dhatu_def()
                    } else {
                        let expr = self.parse_arithmetic()?;
                        if self.check(&TokenKind::Vadati) {
                            self.advance();
                            self.expect(TokenKind::Danda)?;
                            Ok(ASTNode::VadatiNode { mulya: Box::new(expr) })
                        } else {
                            if self.check(&TokenKind::Danda) { self.advance(); }
                            Ok(expr)
                        }
                    }
                } else if self.check_ahead(1, &TokenKind::Asti) {
                    self.parse_asti()
                } else if self.check_ahead(1, &TokenKind::Bhavati) {
                    self.parse_bhavati()
                } else if self.check_ahead(1, &TokenKind::Pathati) {
                    self.parse_pathati()
                } else {
                    let expr = self.parse_arithmetic()?;
                    if self.check(&TokenKind::Vadati) {
                        self.advance();
                        self.expect(TokenKind::Danda)?;
                        Ok(ASTNode::VadatiNode { mulya: Box::new(expr) })
                    } else {
                        if self.check(&TokenKind::Danda) { self.advance(); }
                        Ok(expr)
                    }
                }
            }
            _ => {
                let expr = self.parse_arithmetic()?;
                if self.check(&TokenKind::Vadati) {
                    self.advance();
                    self.expect(TokenKind::Danda)?;
                    Ok(ASTNode::VadatiNode { mulya: Box::new(expr) })
                } else {
                    if self.check(&TokenKind::Danda) { self.advance(); }
                    Ok(expr)
                }
            }
        }
    }

    fn parse_karyakram(&mut self) -> Result<ASTNode, ParseError> {
        self.expect(TokenKind::Arambhah)?;
        let mut shareera = Vec::new();
        while !self.check(&TokenKind::Samaptih) && !self.is_at_end() {
            shareera.push(self.parse_vakya()?);
        }
        self.expect(TokenKind::Samaptih)?;
        Ok(ASTNode::KaryakramNode { shareera })
    }

    fn parse_dhatu_def(&mut self) -> Result<ASTNode, ParseError> {
        let name_tok = self.expect_identifier()?;
        let name = if let TokenKind::Naama(n) = name_tok.kind { n } else { unreachable!() };

        let _ = self.symbols.define(&name, Symbol {
            name: name.clone(),
            kind: SymbolKind::Dhatu { gana: Gana::Bhvadi, lakara: Lakara::Lat },
            karaka: KarakaRole::Karta,
            vibhakti: Vibhakti::Prathama,
            linga: Linga::Pullinga,
            vacana: Vacana::Eka,
            defined_at: name_tok.span,
        });

        let mut params = Vec::new();
        while !self.is_karoti() && !self.check(&TokenKind::Danda) && !self.is_at_end() {
            let p_tok = self.expect_identifier()?;
            let p_name = if let TokenKind::Naama(n) = p_tok.kind { n } else { unreachable!() };
            let vibhakti = self.match_vibhakti().unwrap_or(Vibhakti::Prathama);
            params.push(KarakaParam {
                name: p_name,
                role: vibhakti_to_karaka(&vibhakti),
                vibhakti,
                span: p_tok.span,
            });
        }
        
        if self.is_karoti() { self.advance(); }
        self.expect(TokenKind::Danda)?;
        
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
        
        let mut body = Vec::new();
        self.current_dhatu_name.push(name.clone());
        while !self.check(&TokenKind::Iti) && !self.is_at_end() {
            body.push(self.parse_vakya()?);
        }
        self.current_dhatu_name.pop();
        self.symbols.pop_scope();
        self.expect(TokenKind::Iti)?;
        if self.check(&TokenKind::Danda) { self.advance(); }

        Ok(ASTNode::DhatuDef {
            name,
            gana: Gana::Bhvadi,
            lakara: Lakara::Lat,
            linga: Linga::Pullinga,
            vacana: Vacana::Eka,
            params,
            return_karaka: None,
            body,
            upasargas: vec![],
            span: name_tok.span,
        })
    }

    fn is_karoti(&self) -> bool {
        if let TokenKind::Naama(n) = &self.peek().kind {
            n == "karoti"
        } else {
            false
        }
    }

    fn parse_asti(&mut self) -> Result<ASTNode, ParseError> {
        let naama_tok = self.expect_identifier()?;
        let naama = if let TokenKind::Naama(n) = naama_tok.kind { n } else { unreachable!() };
        self.expect(TokenKind::Asti)?;
        let mulya = self.parse_arithmetic()?;
        self.expect(TokenKind::Danda)?;
        Ok(ASTNode::AstiNode { naama, mulya: Box::new(mulya) })
    }

    fn parse_bhavati(&mut self) -> Result<ASTNode, ParseError> {
        let naama_tok = self.expect_identifier()?;
        let naama = if let TokenKind::Naama(n) = naama_tok.kind { n } else { unreachable!() };
        self.expect(TokenKind::Bhavati)?;
        let mulya = self.parse_arithmetic()?;
        self.expect(TokenKind::Danda)?;
        Ok(ASTNode::BhavatiNode { naama, mulya: Box::new(mulya) })
    }

    fn parse_pathati(&mut self) -> Result<ASTNode, ParseError> {
        let naama_tok = self.expect_identifier()?;
        let naama = if let TokenKind::Naama(n) = naama_tok.kind { n } else { unreachable!() };
        self.expect(TokenKind::Pathati)?;
        self.expect(TokenKind::Danda)?;
        Ok(ASTNode::PathatiNode { naama })
    }

    fn parse_yadi(&mut self) -> Result<ASTNode, ParseError> {
        self.expect(TokenKind::Yadi)?;
        let sthiti = Box::new(self.parse_arithmetic()?);
        self.expect(TokenKind::Tarhi)?;
        
        let mut tarhi = Vec::new();
        while !self.check(&TokenKind::Anyatha) && !self.check(&TokenKind::Iti) && !self.is_at_end() {
            tarhi.push(self.parse_vakya()?);
        }
        
        let mut anyatha = None;
        if self.check(&TokenKind::Anyatha) {
            self.advance();
            let mut body = Vec::new();
            while !self.check(&TokenKind::Iti) && !self.is_at_end() {
                body.push(self.parse_vakya()?);
            }
            anyatha = Some(body);
        }
        
        self.expect(TokenKind::Iti)?;
        if self.check(&TokenKind::Danda) { self.advance(); }
        
        Ok(ASTNode::YadiNode { sthiti, tarhi, anyatha })
    }

    fn parse_yavat(&mut self) -> Result<ASTNode, ParseError> {
        self.expect(TokenKind::Yavat)?;
        let sthiti = Box::new(self.parse_arithmetic()?);
        self.expect(TokenKind::Tavat)?;
        
        let mut shareera = Vec::new();
        while !self.check(&TokenKind::Iti) && !self.is_at_end() {
            shareera.push(self.parse_vakya()?);
        }
        self.expect(TokenKind::Iti)?;
        if self.check(&TokenKind::Danda) { self.advance(); }
        
        Ok(ASTNode::YavatNode { sthiti, shareera })
    }

    fn parse_arithmetic(&mut self) -> Result<ASTNode, ParseError> {
        let mut left = self.parse_primary()?;
        
        while let Some(tok) = self.match_any(&[TokenKind::Yoga, TokenKind::Viyoga, TokenKind::Guna, TokenKind::Bhaga]) {
            let right = self.parse_primary()?;
            left = match tok.kind {
                TokenKind::Yoga => ASTNode::YogaNode { vama: Box::new(left), dakshina: Box::new(right) },
                TokenKind::Viyoga => ASTNode::ViyogaNode { vama: Box::new(left), dakshina: Box::new(right) },
                TokenKind::Guna => ASTNode::GunaNode { vama: Box::new(left), dakshina: Box::new(right) },
                TokenKind::Bhaga => ASTNode::BhagaNode { vama: Box::new(left), dakshina: Box::new(right) },
                _ => unreachable!(),
            };
        }
        
        if let Some(tok) = self.match_any(&[TokenKind::Sama, TokenKind::AsamaH, TokenKind::NyuunaH, TokenKind::AdhikaH]) {
            let right = self.parse_arithmetic()?;
            left = match tok.kind {
                TokenKind::Sama => ASTNode::SamaNode { vama: Box::new(left), dakshina: Box::new(right) },
                TokenKind::AsamaH => ASTNode::AsamaNode { vama: Box::new(left), dakshina: Box::new(right) },
                TokenKind::NyuunaH => ASTNode::NyuunaNode { vama: Box::new(left), dakshina: Box::new(right) },
                TokenKind::AdhikaH => ASTNode::AdhikaNode { vama: Box::new(left), dakshina: Box::new(right) },
                _ => unreachable!(),
            };
        }

        if self.check(&TokenKind::Varam) {
            self.advance();
            let mut shareera = Vec::new();
            while !self.check(&TokenKind::Iti) && !self.is_at_end() {
                shareera.push(self.parse_vakya()?);
            }
            self.expect(TokenKind::Iti)?;
            if self.check(&TokenKind::Danda) { self.advance(); }
            left = ASTNode::PunahNode { varam: Box::new(left), shareera };
        }

        Ok(left)
    }

    fn parse_primary(&mut self) -> Result<ASTNode, ParseError> {
        let tok = self.advance();
        match tok.kind {
            TokenKind::PurnaankLiteral(value) => Ok(ASTNode::PurnaankLiteral { value, span: tok.span }),
            TokenKind::DashaamshaLiteral(value) => Ok(ASTNode::DashaamshaLiteral { value, span: tok.span }),
            TokenKind::VaakLiteral(value) => Ok(ASTNode::VaakLiteral { value, span: tok.span }),
            TokenKind::Naama(name) => {
                if let Some(sym) = self.symbols.lookup(&name) {
                    if matches!(sym.kind, SymbolKind::Dhatu { .. }) {
                        return self.parse_kriya_call(name, tok.span);
                    }
                }
                let vibhakti = self.match_vibhakti().unwrap_or(Vibhakti::Prathama);
                Ok(ASTNode::Nama {
                    base: name,
                    vibhakti,
                    vacana: Vacana::Eka,
                    linga: Linga::Pullinga,
                    span: tok.span,
                })
            }
            _ => Err(ParseError::UnexpectedToken {
                expected: "expression".to_string(),
                found: tok.kind,
                span: tok.span,
            }),
        }
    }

    fn parse_kriya_call(&mut self, name: String, span: Span) -> Result<ASTNode, ParseError> {
        let stop_kinds = [
            TokenKind::Danda, TokenKind::Iti, TokenKind::Vadati,
            TokenKind::Yoga, TokenKind::Viyoga, TokenKind::Guna, TokenKind::Bhaga,
            TokenKind::Sama, TokenKind::AsamaH, TokenKind::NyuunaH, TokenKind::AdhikaH,
            TokenKind::Varam, TokenKind::Tarhi, TokenKind::Anyatha, TokenKind::Samaapti,
        ];

        let mut karma = Vec::new();
        while !self.check_any(&stop_kinds) && !self.is_at_end() {
            let arg = self.parse_primary()?;
            karma.push(arg);
        }

        let kriya_call = ASTNode::KriyaCall {
            karta: None,
            kriya: name.clone(),
            karma,
            karana: None,
            sampradana: None,
            apadan: None,
            adhikarana: None,
            span,
        };

        if let Some(current) = self.current_dhatu_name.last() {
            if *current == name {
                return Ok(ASTNode::AvartanaNode {
                    call: Box::new(kriya_call),
                    span,
                });
            }
        }

        Ok(kriya_call)
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
            TokenKind::Naama(_) => Ok(tok),
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

    fn check_any(&self, kinds: &[TokenKind]) -> bool {
        if self.is_at_end() { return false; }
        let next = &self.peek().kind;
        kinds.iter().any(|k| k == next)
    }

    fn check(&self, kind: &TokenKind) -> bool {
        if self.is_at_end() { return false; }
        &self.peek().kind == kind
    }

    fn check_ahead(&self, n: usize, kind: &TokenKind) -> bool {
        if self.pos + n >= self.tokens.len() { return false; }
        &self.tokens[self.pos + n].kind == kind
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

    fn is_at_end(&self) -> bool {
        self.pos >= self.tokens.len() || self.tokens[self.pos].kind == TokenKind::Samaapti
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn span() -> Span {
        Span { line: 1, col: 1, len: 1 }
    }

    fn nm(s: &str) -> Token {
        Token { kind: TokenKind::Naama(s.to_string()), span: span() }
    }

    fn kw(kind: TokenKind) -> Token {
        Token { kind, span: span() }
    }

    fn parse_tokens(tokens: Vec<Token>) -> Result<ASTNode, ParseError> {
        let mut parser = Parser::new(tokens);
        parser.parse()
    }

    // Regression: a simple non-recursive dhatu parses exactly as before.
    #[test]
    fn test_simple_dhatu_regression() {
        let tokens = vec![
            nm("square-dhatu"), nm("n"), nm("karoti"), kw(TokenKind::Danda),
            nm("n"), kw(TokenKind::Iti), kw(TokenKind::Danda),
        ];
        let ast = parse_tokens(tokens).expect("should parse");

        match ast {
            ASTNode::KaryakramNode { shareera } => {
                assert_eq!(shareera.len(), 1);
                match &shareera[0] {
                    ASTNode::DhatuDef { name, params, body, .. } => {
                        assert_eq!(name, "square-dhatu");
                        assert_eq!(params.len(), 1);
                        assert_eq!(params[0].name, "n");
                        assert_eq!(body.len(), 1);
                        assert!(matches!(&body[0], ASTNode::Nama { base, .. } if base == "n"));
                    }
                    other => panic!("expected DhatuDef, got {:?}", other),
                }
            }
            other => panic!("expected KaryakramNode, got {:?}", other),
        }
    }

    // A dhatu whose body calls its own name -> AvartanaNode wrapping KriyaCall.
    #[test]
    fn test_self_recursive_dhatu_is_avatarana() {
        let tokens = vec![
            nm("factorial-dhatu"), nm("n"), nm("karoti"), kw(TokenKind::Danda),
            nm("n"), kw(TokenKind::Yoga),
            nm("factorial-dhatu"), nm("n"),
            kw(TokenKind::Iti), kw(TokenKind::Danda),
        ];
        let ast = parse_tokens(tokens).expect("should parse");

        match ast {
            ASTNode::KaryakramNode { shareera } => {
                match &shareera[0] {
                    ASTNode::DhatuDef { body, .. } => {
                        // body[0] = YogaNode { vama: Nama n, dakshina: AvartanaNode }
                        match &body[0] {
                            ASTNode::YogaNode { dakshina, .. } => {
                                match dakshina.as_ref() {
                                    ASTNode::AvartanaNode { call, .. } => {
                                        match call.as_ref() {
                                            ASTNode::KriyaCall { kriya, karma, .. } => {
                                                assert_eq!(kriya, "factorial-dhatu");
                                                assert_eq!(karma.len(), 1);
                                            }
                                            other => panic!("expected KriyaCall inside AvartanaNode, got {:?}", other),
                                        }
                                    }
                                    other => panic!("expected AvartanaNode, got {:?}", other),
                                }
                            }
                            other => panic!("expected YogaNode in body, got {:?}", other),
                        }
                    }
                    other => panic!("expected DhatuDef, got {:?}", other),
                }
            }
            other => panic!("expected KaryakramNode, got {:?}", other),
        }
    }

    // A dhatu calling a DIFFERENT registered dhatu -> plain KriyaCall (not AvartanaNode).
    #[test]
    fn test_call_to_other_dhatu_is_kriya_call() {
        let tokens = vec![
            nm("first-dhatu"), nm("x"), nm("karoti"), kw(TokenKind::Danda),
            nm("x"), kw(TokenKind::Iti), kw(TokenKind::Danda),
            nm("second-dhatu"), nm("y"), nm("karoti"), kw(TokenKind::Danda),
            nm("y"), kw(TokenKind::Yoga),
            nm("first-dhatu"), nm("x"),
            kw(TokenKind::Iti), kw(TokenKind::Danda),
        ];
        let ast = parse_tokens(tokens).expect("should parse");

        match ast {
            ASTNode::KaryakramNode { shareera } => {
                match &shareera[1] {
                    ASTNode::DhatuDef { body, .. } => {
                        match &body[0] {
                            ASTNode::YogaNode { dakshina, .. } => {
                                match dakshina.as_ref() {
                                    ASTNode::KriyaCall { kriya, karma, .. } => {
                                        assert_eq!(kriya, "first-dhatu");
                                        assert_eq!(karma.len(), 1);
                                    }
                                    other => panic!("expected plain KriyaCall, got {:?}", other),
                                }
                            }
                            other => panic!("expected YogaNode, got {:?}", other),
                        }
                    }
                    other => panic!("expected DhatuDef, got {:?}", other),
                }
            }
            other => panic!("expected KaryakramNode, got {:?}", other),
        }
    }

    // A reference to a name NOT registered as a Dhatu stays a plain Nama (no regression).
    #[test]
    fn test_unregistered_name_is_nama() {
        let tokens = vec![
            nm("foo-dhatu"), nm("a"), nm("karoti"), kw(TokenKind::Danda),
            nm("a"), kw(TokenKind::Yoga), nm("bar"),
            kw(TokenKind::Iti), kw(TokenKind::Danda),
        ];
        let ast = parse_tokens(tokens).expect("should parse");

        match ast {
            ASTNode::KaryakramNode { shareera } => {
                match &shareera[0] {
                    ASTNode::DhatuDef { body, .. } => {
                        match &body[0] {
                            ASTNode::YogaNode { dakshina, .. } => {
                                assert!(matches!(dakshina.as_ref(), ASTNode::Nama { base, .. } if base == "bar"));
                            }
                            other => panic!("expected YogaNode, got {:?}", other),
                        }
                    }
                    other => panic!("expected DhatuDef, got {:?}", other),
                }
            }
            other => panic!("expected KaryakramNode, got {:?}", other),
        }
    }
}
