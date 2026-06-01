use crate::token::{Token, TokenKind, Span};
use crate::error::LexError;
use crate::sandhi::{SandhiEngine, SandhiMode};
use crate::unicode_map::{is_iast_identifier_start, is_iast_identifier_continue};
use std::iter::Peekable;
use std::str::Chars;

pub struct Lexer<'a> {
    input: &'a str,
    chars: Peekable<Chars<'a>>,
    line: usize,
    col: usize,
    pos: usize,
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Self {
        Self {
            input,
            chars: input.chars().peekable(),
            line: 1,
            col: 1,
            pos: 0,
        }
    }

    pub fn tokenize(&mut self, sandhi_mode: SandhiMode) -> Result<Vec<Token>, LexError> {
        let mut sandhi_engine = SandhiEngine::new(sandhi_mode);
        let processed_input = sandhi_engine.apply(self.input);
        
        let mut tokens = Vec::new();
        let mut sub_lexer = Lexer::new(&processed_input);

        while let Some(token) = sub_lexer.next_token()? {
            let is_eof = token.kind == TokenKind::EOF;
            tokens.push(token);
            if is_eof { break; }
        }

        Ok(tokens)
    }

    fn next_token(&mut self) -> Result<Option<Token>, LexError> {
        self.skip_whitespace_and_comments()?;

        let start_line = self.line;
        let start_col = self.col;
        let start_pos = self.pos;

        let c = match self.peek() {
            Some(c) => c,
            None => return Ok(Some(Token {
                kind: TokenKind::EOF,
                span: Span { line: self.line, col: self.col, len: 0 },
            })),
        };

        // Special standalone characters check BEFORE identifiers
        if c == 'ḥ' {
            self.advance();
            return Ok(Some(Token {
                kind: TokenKind::Visarga,
                span: Span { line: start_line, col: start_col, len: self.pos - start_pos },
            }));
        }
        if c == 'ṃ' {
            self.advance();
            return Ok(Some(Token {
                kind: TokenKind::Anusvara,
                span: Span { line: start_line, col: start_col, len: self.pos - start_pos },
            }));
        }

        if is_iast_identifier_start(c) {
            return Ok(Some(self.lex_identifier_or_keyword()?));
        }

        if c.is_ascii_digit() {
            return Ok(Some(self.lex_number()?));
        }

        if c == '"' {
            return Ok(Some(self.lex_string()?));
        }

        let kind = match c {
            '.' => { self.advance(); TokenKind::Dot }
            ':' => {
                self.advance();
                if self.peek() == Some(':') {
                    self.advance();
                    TokenKind::DoubleColon
                } else {
                    TokenKind::Semicolon
                }
            }
            ';' => { self.advance(); TokenKind::Semicolon }
            '(' => { self.advance(); TokenKind::LeftParen }
            ')' => { self.advance(); TokenKind::RightParen }
            '{' => { self.advance(); TokenKind::LeftBrace }
            '}' => { self.advance(); TokenKind::RightBrace }
            '[' => { self.advance(); TokenKind::LeftBracket }
            ']' => { self.advance(); TokenKind::RightBracket }
            ',' => { self.advance(); TokenKind::Comma }
            '=' => {
                self.advance();
                if self.peek() == Some('=') {
                    self.advance();
                    TokenKind::EqualEqual
                } else if self.peek() == Some('>') {
                    self.advance();
                    TokenKind::FatArrow
                } else {
                    TokenKind::Equals
                }
            }
            '+' => { self.advance(); TokenKind::Plus }
            '-' => {
                self.advance();
                if self.peek() == Some('>') {
                    self.advance();
                    TokenKind::Arrow
                } else {
                    TokenKind::Minus
                }
            }
            '*' => { self.advance(); TokenKind::Star }
            '/' => { self.advance(); TokenKind::Slash }
            '%' => { self.advance(); TokenKind::Percent }
            '<' => {
                self.advance();
                if self.peek() == Some('=') {
                    self.advance();
                    TokenKind::LessEqual
                } else {
                    TokenKind::Less
                }
            }
            '>' => {
                self.advance();
                if self.peek() == Some('=') {
                    self.advance();
                    TokenKind::GreaterEqual
                } else {
                    TokenKind::Greater
                }
            }
            '!' => {
                self.advance();
                if self.peek() == Some('=') {
                    self.advance();
                    TokenKind::BangEqual
                } else {
                    TokenKind::Na
                }
            }
            'ḥ' => { self.advance(); TokenKind::Visarga }
            'ṃ' => { self.advance(); TokenKind::Anusvara }
            _ => {
                self.advance();
                TokenKind::Unknown(c)
            }
        };

        Ok(Some(Token {
            kind,
            span: Span { line: start_line, col: start_col, len: self.pos - start_pos },
        }))
    }

    fn lex_identifier_or_keyword(&mut self) -> Result<Token, LexError> {
        let start_line = self.line;
        let start_col = self.col;
        let start_pos = self.pos;
        let mut id = String::new();
        while let Some(c) = self.peek() {
            if is_iast_identifier_continue(c) {
                id.push(c);
                self.advance();
            } else {
                break;
            }
        }
        let kind = match id.as_str() {
            "Lat" => TokenKind::Lat, "Lit" => TokenKind::Lit, "Lut" => TokenKind::Lut,
            "Lrt" => TokenKind::Lrt, "Let" => TokenKind::Let, "Lot" => TokenKind::Lot,
            "Lan" => TokenKind::Lan, "Vidhilin" => TokenKind::Vidhilin,
            "Asihlin" => TokenKind::Asihlin, "Lun" => TokenKind::Lun,
            "ca" | "Ca" => TokenKind::Ca, "va" | "Va" => TokenKind::Va, "na" | "Na" => TokenKind::Na,
            "iti" | "Iti" => TokenKind::Iti, "eva" | "Eva" => TokenKind::Eva, "api" | "Api" => TokenKind::Api,
            "tu" | "Tu" => TokenKind::Tu, "yadi" | "Yadi" => TokenKind::Yadi, "tarhi" | "Tarhi" => TokenKind::Tarhi,
            "anyatha" | "Anyatha" => TokenKind::Anyatha, "kintu" | "Kintu" => TokenKind::Kintu,
            "punah" | "Punah" => TokenKind::Punah, "atha" | "Atha" => TokenKind::Atha, "alam" | "Alam" => TokenKind::Alam,
            "Pra" => TokenKind::Pra, "Para" => TokenKind::Para, "Apa" => TokenKind::Apa,
            "Sam" => TokenKind::Sam, "Anu" => TokenKind::Anu, "Ava" => TokenKind::Ava,
            "Nis" => TokenKind::Nis, "Nir" => TokenKind::Nir, "Dus" => TokenKind::Dus,
            "Dur" => TokenKind::Dur, "Vi" => TokenKind::Vi, "Aa" => TokenKind::Aa,
            "Ni" => TokenKind::Ni, "Adhi" => TokenKind::Adhi,
            "Ati" => TokenKind::Ati, "Su" => TokenKind::Su, "Ud" => TokenKind::Ud,
            "Abhi" => TokenKind::Abhi, "Prati" => TokenKind::Prati, "Pari" => TokenKind::Pari,
            "Upa" => TokenKind::Upa,
            _ => TokenKind::Identifier(id),
        };
        Ok(Token { kind, span: Span { line: start_line, col: start_col, len: self.pos - start_pos } })
    }

    fn lex_number(&mut self) -> Result<Token, LexError> {
        let start_line = self.line;
        let start_col = self.col;
        let start_pos = self.pos;
        let mut s = String::new();
        while let Some(c) = self.peek() {
            if c.is_ascii_digit() || c == '_' {
                if c != '_' { s.push(c); }
                self.advance();
            } else { break; }
        }
        if self.peek() == Some('.') {
            s.push('.'); self.advance();
            while let Some(c) = self.peek() {
                if c.is_ascii_digit() || c == '_' {
                    if c != '_' { s.push(c); }
                    self.advance();
                } else { break; }
            }
            let val: f64 = s.parse().unwrap_or(0.0);
            return Ok(Token { kind: TokenKind::FloatLiteral(val), span: Span { line: start_line, col: start_col, len: self.pos - start_pos } });
        }
        let val: i64 = s.parse().unwrap_or(0);
        Ok(Token { kind: TokenKind::IntLiteral(val), span: Span { line: start_line, col: start_col, len: self.pos - start_pos } })
    }

    fn lex_string(&mut self) -> Result<Token, LexError> {
        let start_line = self.line;
        let start_col = self.col;
        let start_pos = self.pos;
        self.advance();
        let mut s = String::new();
        while let Some(c) = self.peek() {
            if c == '"' {
                self.advance();
                return Ok(Token { kind: TokenKind::StringLiteral(s), span: Span { line: start_line, col: start_col, len: self.pos - start_pos } });
            }
            if c == '\\' {
                self.advance();
                match self.peek() {
                    Some('n') => s.push('\n'), Some('t') => s.push('\t'),
                    Some('\\') => s.push('\\'), Some('"') => s.push('"'),
                    _ => s.push('\\'),
                }
                self.advance();
            } else { s.push(c); self.advance(); }
        }
        Err(LexError::UnterminatedString { span: Span { line: start_line, col: start_col, len: self.pos - start_pos } })
    }

    fn skip_whitespace_and_comments(&mut self) -> Result<(), LexError> {
        while let Some(c) = self.peek() {
            if c.is_whitespace() { self.advance(); }
            else if c == '/' {
                if self.peek_next() == Some('/') {
                    while let Some(c) = self.peek() {
                        if c == '\n' { break; }
                        self.advance();
                    }
                } else if self.peek_next() == Some('*') { self.lex_block_comment()?; }
                else { break; }
            } else { break; }
        }
        Ok(())
    }

    fn lex_block_comment(&mut self) -> Result<(), LexError> {
        let start_line = self.line;
        let start_col = self.col;
        let start_pos = self.pos;
        self.advance(); self.advance();
        let mut depth = 1;
        while let Some(c) = self.peek() {
            if c == '/' && self.peek_next() == Some('*') { self.advance(); self.advance(); depth += 1; }
            else if c == '*' && self.peek_next() == Some('/') { self.advance(); self.advance(); depth -= 1; if depth == 0 { return Ok(()); } }
            else { self.advance(); }
        }
        Err(LexError::UnterminatedBlockComment { span: Span { line: start_line, col: start_col, len: self.pos - start_pos } })
    }

    fn peek(&mut self) -> Option<char> { self.chars.peek().copied() }
    fn peek_next(&mut self) -> Option<char> { let mut cloned = self.chars.clone(); cloned.next(); cloned.peek().copied() }
    fn advance(&mut self) {
        if let Some(c) = self.chars.next() {
            self.pos += c.len_utf8();
            if c == '\n' { self.line += 1; self.col = 1; }
            else { self.col += 1; }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::token::TokenKind;

    #[test]
    fn test_basic_identifiers() {
        let mut lexer = Lexer::new("rāma phala");
        let tokens = lexer.tokenize(SandhiMode::Off).unwrap();
        assert_eq!(tokens[0].kind, TokenKind::Identifier("rāma".to_string()));
        assert_eq!(tokens[1].kind, TokenKind::Identifier("phala".to_string()));
    }

    #[test]
    fn test_keywords() {
        let mut lexer = Lexer::new("Lat yadi na iti");
        let tokens = lexer.tokenize(SandhiMode::Off).unwrap();
        assert_eq!(tokens[0].kind, TokenKind::Lat);
        assert_eq!(tokens[1].kind, TokenKind::Yadi);
        assert_eq!(tokens[2].kind, TokenKind::Na);
        assert_eq!(tokens[3].kind, TokenKind::Iti);
    }

    #[test]
    fn test_sandhi_savarna_dirgha() {
        let mut lexer = Lexer::new("a+a i+i u+u");
        let tokens = lexer.tokenize(SandhiMode::Auto).unwrap();
        assert_eq!(tokens[0].kind, TokenKind::Identifier("ā".to_string()));
        assert_eq!(tokens[1].kind, TokenKind::Identifier("ī".to_string()));
        assert_eq!(tokens[2].kind, TokenKind::Identifier("ū".to_string()));
    }

    #[test]
    fn test_sandhi_guna() {
        let mut lexer = Lexer::new("a+i a+u");
        let tokens = lexer.tokenize(SandhiMode::Auto).unwrap();
        assert_eq!(tokens[0].kind, TokenKind::Identifier("e".to_string()));
        assert_eq!(tokens[1].kind, TokenKind::Identifier("o".to_string()));
    }

    #[test]
    fn test_operators() {
        let mut lexer = Lexer::new("-> => :: == != <= >=");
        let tokens = lexer.tokenize(SandhiMode::Off).unwrap();
        assert_eq!(tokens[0].kind, TokenKind::Arrow);
        assert_eq!(tokens[1].kind, TokenKind::FatArrow);
        assert_eq!(tokens[2].kind, TokenKind::DoubleColon);
        assert_eq!(tokens[3].kind, TokenKind::EqualEqual);
        assert_eq!(tokens[4].kind, TokenKind::BangEqual);
        assert_eq!(tokens[5].kind, TokenKind::LessEqual);
        assert_eq!(tokens[6].kind, TokenKind::GreaterEqual);
    }

    #[test]
    fn test_strings() {
        let mut lexer = Lexer::new("\"hello\\nworld\"");
        let tokens = lexer.tokenize(SandhiMode::Off).unwrap();
        assert_eq!(tokens[0].kind, TokenKind::StringLiteral("hello\nworld".to_string()));
    }

    #[test]
    fn test_comments() {
        let mut lexer = Lexer::new("// line comment\n/* block\ncomment */ identifier");
        let tokens = lexer.tokenize(SandhiMode::Off).unwrap();
        assert_eq!(tokens[0].kind, TokenKind::Identifier("identifier".to_string()));
    }

    #[test]
    fn test_numbers() {
        let mut lexer = Lexer::new("1_000 123.456");
        let tokens = lexer.tokenize(SandhiMode::Off).unwrap();
        assert_eq!(tokens[0].kind, TokenKind::IntLiteral(1000));
        assert_eq!(tokens[1].kind, TokenKind::FloatLiteral(123.456));
    }

    #[test]
    fn test_special_tokens() {
        let mut lexer = Lexer::new("ḥ ṃ");
        let tokens = lexer.tokenize(SandhiMode::Off).unwrap();
        assert_eq!(tokens[0].kind, TokenKind::Visarga);
        assert_eq!(tokens[1].kind, TokenKind::Anusvara);
    }

    #[test]
    fn test_upasargas() {
        let mut lexer = Lexer::new("Pra Para Apa Sam");
        let tokens = lexer.tokenize(SandhiMode::Off).unwrap();
        assert_eq!(tokens[0].kind, TokenKind::Pra);
        assert_eq!(tokens[1].kind, TokenKind::Para);
        assert_eq!(tokens[2].kind, TokenKind::Apa);
        assert_eq!(tokens[3].kind, TokenKind::Sam);
    }
}
