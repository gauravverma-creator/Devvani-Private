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
                    self.parse_dhatu_def()
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
        while !self.check(&TokenKind::Iti) && !self.is_at_end() {
            body.push(self.parse_vakya()?);
        }
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
