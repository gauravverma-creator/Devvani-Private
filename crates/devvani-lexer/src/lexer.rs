use crate::error::LexError;
use crate::sandhi::{SandhiEngine, SandhiMode};
use crate::token::{Span, Token, TokenKind};
use crate::unicode_map::{is_iast_identifier_continue, is_iast_identifier_start};
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
            let is_eof = token.kind == TokenKind::Samaapti;
            tokens.push(token);
            if is_eof {
                break;
            }
        }

        Ok(tokens)
    }

    fn next_token(&mut self) -> Result<Option<Token>, LexError> {
        self.skip_whitespace()?;

        let start_line = self.line;
        let start_col = self.col;
        let start_pos = self.pos;

        let c = match self.peek() {
            Some(c) => c,
            None => {
                return Ok(Some(Token {
                    kind: TokenKind::Samaapti,
                    span: Span {
                        line: self.line,
                        col: self.col,
                        len: 0,
                    },
                }))
            }
        };

        // Special standalone characters check BEFORE identifiers
        if c == 'ḥ' {
            self.advance();
            return Ok(Some(Token {
                kind: TokenKind::Visarga,
                span: Span {
                    line: start_line,
                    col: start_col,
                    len: self.pos - start_pos,
                },
            }));
        }
        if c == 'ṃ' {
            self.advance();
            return Ok(Some(Token {
                kind: TokenKind::Anusvara,
                span: Span {
                    line: start_line,
                    col: start_col,
                    len: self.pos - start_pos,
                },
            }));
        }

        // Devanagari Danda (। - U+0964)
        if c == '।' {
            self.advance();
            return Ok(Some(Token {
                kind: TokenKind::Danda,
                span: Span {
                    line: start_line,
                    col: start_col,
                    len: self.pos - start_pos,
                },
            }));
        }

        if is_iast_identifier_start(c) {
            return Ok(Some(self.lex_identifier_or_keyword()?));
        }

        if c.is_ascii_digit() {
            return Ok(Some(self.lex_anka()?));
        }

        if c == '"' {
            return Ok(Some(self.lex_vaak()?));
        }

        // Catch-all for any other characters
        let kind = match c {
            'ḥ' => {
                self.advance();
                TokenKind::Visarga
            }
            'ṃ' => {
                self.advance();
                TokenKind::Anusvara
            }
              '[' => {
                  self.advance();
                  TokenKind::LBracket
              }
              ']' => {
                  self.advance();
                  TokenKind::RBracket
              }
              '.' => {
                  self.advance();
                  TokenKind::Dot
              }
              '=' => {
                  self.advance();
                  TokenKind::Equals
              }
             _ => {
                 self.advance();
                 TokenKind::Unknown(c)
             }
         };

        Ok(Some(Token {
            kind,
            span: Span {
                line: start_line,
                col: start_col,
                len: self.pos - start_pos,
            },
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
        while self.peek() == Some('-') {
            let rest = &self.input[self.pos + 1..];
            let next_start = rest.chars().next();
            if next_start.map_or(false, |c| is_iast_identifier_start(c)) {
                id.push('-');
                self.advance();
                while let Some(c) = self.peek() {
                    if is_iast_identifier_continue(c) {
                        id.push(c);
                        self.advance();
                    } else {
                        break;
                    }
                }
            } else {
                break;
            }
        }
        let kind = match id.as_str() {
            "Lat" => TokenKind::Lat,
            "Lit" => TokenKind::Lit,
            "Lut" => TokenKind::Lut,
            "Lrt" => TokenKind::Lrt,
            "Let" => TokenKind::Let,
            "Lot" => TokenKind::Lot,
            "Lan" => TokenKind::Lan,
            "Vidhilin" => TokenKind::Vidhilin,
            "Asihlin" => TokenKind::Asihlin,
            "Lun" => TokenKind::Lun,
            "ca" | "Ca" => TokenKind::Ca,
            "va" | "Va" => TokenKind::Va,
            "na" | "Na" => TokenKind::Na,
            "iti" | "Iti" => TokenKind::Iti,
            "eva" | "Eva" => TokenKind::Eva,
            "api" | "Api" => TokenKind::Api,
            "tu" | "Tu" => TokenKind::Tu,
            "yadi" | "Yadi" => TokenKind::Yadi,
            "tarhi" | "Tarhi" => TokenKind::Tarhi,
            "anyatha" | "Anyatha" => TokenKind::Anyatha,
            "kintu" | "Kintu" => TokenKind::Kintu,
            "punah" | "Punah" => TokenKind::Punah,
            "atha" | "Atha" => TokenKind::Atha,
            "alam" | "Alam" => TokenKind::Alam,
            "Pra" => TokenKind::Pra,
            "Para" => TokenKind::Para,
            "Apa" => TokenKind::Apa,
            "Sam" => TokenKind::Sam,
            "Anu" => TokenKind::Anu,
            "Ava" => TokenKind::Ava,
            "Nis" => TokenKind::Nis,
            "Nir" => TokenKind::Nir,
            "Dus" => TokenKind::Dus,
            "Dur" => TokenKind::Dur,
            "Vi" => TokenKind::Vi,
            "Aa" => TokenKind::Aa,
            "Ni" => TokenKind::Ni,
            "Adhi" => TokenKind::Adhi,
            "Ati" => TokenKind::Ati,
            "Su" => TokenKind::Su,
            "Ud" => TokenKind::Ud,
            "Abhi" => TokenKind::Abhi,
            "Prati" => TokenKind::Prati,
            "Pari" => TokenKind::Pari,
            "Upa" => TokenKind::Upa,

            // New Sanskrit Keywords (IAST and Devanagari)
            "asti" | "अस्ति" => TokenKind::Asti,
            "bhavati" | "भवति" => TokenKind::Bhavati,
            "vadati" | "वदति" => TokenKind::Vadati,
            "pathati" | "पठति" => TokenKind::Pathati,
"yavat" | "यावत्" => TokenKind::Yavat,
             "tavat" | "तावत्" => TokenKind::Tavat,
             "kramasah" | "Kramasah" => TokenKind::Kramasah,
             "varam" | "वारम्" => TokenKind::Varam,
            "arambhah" | "आरम्भः" => TokenKind::Arambhah,
            "samaptih" | "समाप्तिः" => TokenKind::Samaptih,
            "yoga" | "योग" => TokenKind::Yoga,
            "viyoga" | "वियोग" => TokenKind::Viyoga,
            "guna" | "गुण" => TokenKind::Guna,
            "bhaga" | "भाग" => TokenKind::Bhaga,
            "sama" | "सम" => TokenKind::Sama,
            "asamah" | "असमः" => TokenKind::AsamaH,
"nyunah" | "न्यूनः" => TokenKind::NyuunaH,
             "adhikah" | "अधिकः" => TokenKind::AdhikaH,
             "avali" => TokenKind::Avali,
             "dravya" | "Dravya" => TokenKind::Dravya,
             "anga" | "Anga" => TokenKind::Anga,
             "nirmāṇa" | "Nirmāṇa" => TokenKind::Nirmana,
             "phalam" | "Phalam" => TokenKind::Phalam,
             "arogya" | "Arogya" => TokenKind::Arogya,
             "dosha" | "Dosha" => TokenKind::Dosha,
             "nidana" | "Nidana" => TokenKind::Nidana,
"samprapti" | "Samprapti" => TokenKind::Samprapti,

"dhātu" | "Dhātu" => TokenKind::Dhātu,

               // Ownership keywords
               "sandarbha" | "Sandarbha" => TokenKind::Sandarbha,
               "adhikara" | "Adhikara" => TokenKind::Adhikara,
               "vikara" | "Vikara" => TokenKind::Vikara,

            "sāmānya" | "Sāmānya" => TokenKind::Sāmānya,

            "dhara" | "Dhara" | "धरा" => TokenKind::Dharā,

             _ => TokenKind::Naama(id),
         };
        Ok(Token {
            kind,
            span: Span {
                line: start_line,
                col: start_col,
                len: self.pos - start_pos,
            },
        })
    }

    fn lex_anka(&mut self) -> Result<Token, LexError> {
        let start_line = self.line;
        let start_col = self.col;
        let start_pos = self.pos;
        let mut s = String::new();
        while let Some(c) = self.peek() {
            if c.is_ascii_digit() || c == '_' {
                if c != '_' {
                    s.push(c);
                }
                self.advance();
            } else {
                break;
            }
        }
        if self.peek() == Some('.') {
            s.push('.');
            self.advance();
            while let Some(c) = self.peek() {
                if c.is_ascii_digit() || c == '_' {
                    if c != '_' {
                        s.push(c);
                    }
                    self.advance();
                } else {
                    break;
                }
            }
            let val: f64 = s.parse().unwrap_or(0.0);
            return Ok(Token {
                kind: TokenKind::DashaamshaLiteral(val),
                span: Span {
                    line: start_line,
                    col: start_col,
                    len: self.pos - start_pos,
                },
            });
        }
        let val: i64 = s.parse().unwrap_or(0);
        Ok(Token {
            kind: TokenKind::PurnaankLiteral(val),
            span: Span {
                line: start_line,
                col: start_col,
                len: self.pos - start_pos,
            },
        })
    }

    fn lex_vaak(&mut self) -> Result<Token, LexError> {
        let start_line = self.line;
        let start_col = self.col;
        let start_pos = self.pos;
        self.advance();
        let mut s = String::new();
        while let Some(c) = self.peek() {
            if c == '"' {
                self.advance();
                return Ok(Token {
                    kind: TokenKind::VaakLiteral(s),
                    span: Span {
                        line: start_line,
                        col: start_col,
                        len: self.pos - start_pos,
                    },
                });
            }
            if c == '\\' {
                self.advance();
                let span = Span {
                    line: self.line,
                    col: self.col,
                    len: 1,
                };
                match self.peek() {
                    Some('r') => s.push('\r'),
                    Some('n') => s.push('\n'),
                    Some('t') => s.push('\t'),
                    Some('0') => s.push('\0'),
                    Some('\\') => s.push('\\'),
                    Some('"') => s.push('"'),
                    Some('u') => {
                        self.advance();
                        if self.peek() != Some('{') {
                            return Err(LexError::InvalidEscape { ch: 'u', span });
                        }
                        self.advance();
                        let mut hex_str = String::new();
                        while let Some(hc) = self.peek() {
                            if hc == '}' {
                                self.advance();
                                break;
                            }
                            if !hc.is_ascii_hexdigit() {
                                return Err(LexError::InvalidEscape { ch: hc, span });
                            }
                            hex_str.push(hc);
                            self.advance();
                        }
                        let code_point = u32::from_str_radix(&hex_str, 16).unwrap_or(0);
                        let ch = char::from_u32(code_point).unwrap_or('\u{FFFD}');
                        s.push(ch);
                    }
                    Some(unknown_ch) => {
                        return Err(LexError::InvalidEscape {
                            ch: unknown_ch,
                            span,
                        });
                    }
                    None => {
                        return Err(LexError::UnterminatedString {
                            span: Span {
                                line: start_line,
                                col: start_col,
                                len: self.pos - start_pos,
                            },
                        })
                    }
                }
                self.advance();
            } else {
                s.push(c);
                self.advance();
            }
        }
        Err(LexError::UnterminatedString {
            span: Span {
                line: start_line,
                col: start_col,
                len: self.pos - start_pos,
            },
        })
    }

    fn skip_whitespace(&mut self) -> Result<(), LexError> {
        while let Some(c) = self.peek() {
            if c.is_whitespace() {
                self.advance();
            } else {
                break;
            }
        }
        Ok(())
    }

    fn peek(&mut self) -> Option<char> {
        self.chars.peek().copied()
    }
    fn advance(&mut self) {
        if let Some(c) = self.chars.next() {
            self.pos += c.len_utf8();
            if c == '\n' {
                self.line += 1;
                self.col = 1;
            } else {
                self.col += 1;
            }
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
        assert_eq!(tokens[0].kind, TokenKind::Naama("rāma".to_string()));
        assert_eq!(tokens[1].kind, TokenKind::Naama("phala".to_string()));
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
    fn test_new_sanskrit_keywords() {
        let mut lexer = Lexer::new("asti bhavati vadati pathati ।");
        let tokens = lexer.tokenize(SandhiMode::Off).unwrap();
        assert_eq!(tokens[0].kind, TokenKind::Asti);
        assert_eq!(tokens[1].kind, TokenKind::Bhavati);
        assert_eq!(tokens[2].kind, TokenKind::Vadati);
        assert_eq!(tokens[3].kind, TokenKind::Pathati);
        assert_eq!(tokens[4].kind, TokenKind::Danda);
    }

    #[test]
    fn test_numbers() {
        let mut lexer = Lexer::new("1_000 123.456");
        let tokens = lexer.tokenize(SandhiMode::Off).unwrap();
        assert_eq!(tokens[0].kind, TokenKind::PurnaankLiteral(1000));
        assert_eq!(tokens[1].kind, TokenKind::DashaamshaLiteral(123.456));
    }

    #[test]
    fn test_strings() {
        let mut lexer = Lexer::new("\"hello\\nworld\"");
        let tokens = lexer.tokenize(SandhiMode::Off).unwrap();
        assert_eq!(
            tokens[0].kind,
            TokenKind::VaakLiteral("hello\nworld".to_string())
        );
    }

    #[test]
    fn test_hyphenated_identifier() {
        let mut lexer = Lexer::new("avartanah-dhatu");
        let tokens = lexer.tokenize(SandhiMode::Off).unwrap();
        assert_eq!(
            tokens[0].kind,
            TokenKind::Naama("avartanah-dhatu".to_string())
        );
    }

    #[test]
    fn test_hyphenated_identifier_multiple_hyphens() {
        let mut lexer = Lexer::new("avartanah-dhatu-karoti");
        let tokens = lexer.tokenize(SandhiMode::Off).unwrap();
        assert_eq!(
            tokens[0].kind,
            TokenKind::Naama("avartanah-dhatu-karoti".to_string())
        );
    }

    #[test]
    fn test_plain_identifier_regression() {
        let mut lexer = Lexer::new("phala");
        let tokens = lexer.tokenize(SandhiMode::Off).unwrap();
        assert_eq!(tokens[0].kind, TokenKind::Naama("phala".to_string()));
    }

    #[test]
    fn test_standalone_hyphen_not_swallowed() {
        let mut lexer = Lexer::new("foo - bar");
        let tokens = lexer.tokenize(SandhiMode::Off).unwrap();
        assert_eq!(tokens[0].kind, TokenKind::Naama("foo".to_string()));
        assert_eq!(tokens[1].kind, TokenKind::Unknown('-'));
        assert_eq!(tokens[2].kind, TokenKind::Naama("bar".to_string()));
    }

    #[test]
    fn test_lbracket_rbracket_tokens() {
        let mut lexer = Lexer::new("[ ]");
        let tokens = lexer.tokenize(SandhiMode::Off).unwrap();
        assert_eq!(tokens[0].kind, TokenKind::LBracket);
        assert_eq!(tokens[1].kind, TokenKind::RBracket);
    }

    #[test]
    fn test_brackets_in_array_literal() {
        let mut lexer = Lexer::new("[1, 2, 3]");
        let tokens = lexer.tokenize(SandhiMode::Off).unwrap();
        assert_eq!(tokens[0].kind, TokenKind::LBracket);
        assert_eq!(tokens[1].kind, TokenKind::PurnaankLiteral(1));
        assert_eq!(tokens[2].kind, TokenKind::Unknown(','));
        assert_eq!(tokens[3].kind, TokenKind::PurnaankLiteral(2));
        assert_eq!(tokens[4].kind, TokenKind::Unknown(','));
        assert_eq!(tokens[5].kind, TokenKind::PurnaankLiteral(3));
        assert_eq!(tokens[6].kind, TokenKind::RBracket);
    }

    #[test]
    fn test_kramasah_keyword() {
        let mut lexer = Lexer::new("kramasah");
        let tokens = lexer.tokenize(SandhiMode::Off).unwrap();
        assert_eq!(tokens[0].kind, TokenKind::Kramasah);
    }

    #[test]
    fn test_Kramasah_keyword() {
        let mut lexer = Lexer::new("Kramasah");
        let tokens = lexer.tokenize(SandhiMode::Off).unwrap();
        assert_eq!(tokens[0].kind, TokenKind::Kramasah);
    }

    #[test]
    fn test_avali_keyword() {
        let mut lexer = Lexer::new("avali");
        let tokens = lexer.tokenize(SandhiMode::Off).unwrap();
        assert_eq!(tokens[0].kind, TokenKind::Avali);
    }

    #[test]
    fn test_avali_array_literal() {
        let mut lexer = Lexer::new("avali[1, 2, 3]");
        let tokens = lexer.tokenize(SandhiMode::Off).unwrap();
        assert_eq!(tokens[0].kind, TokenKind::Avali);
        assert_eq!(tokens[1].kind, TokenKind::LBracket);
        assert_eq!(tokens[2].kind, TokenKind::PurnaankLiteral(1));
        assert_eq!(tokens[3].kind, TokenKind::Unknown(','));
        assert_eq!(tokens[4].kind, TokenKind::PurnaankLiteral(2));
        assert_eq!(tokens[5].kind, TokenKind::Unknown(','));
        assert_eq!(tokens[6].kind, TokenKind::PurnaankLiteral(3));
        assert_eq!(tokens[7].kind, TokenKind::RBracket);
    }

    #[test]
    fn test_phalam_keyword() {
        let mut lexer = Lexer::new("phalam");
        let tokens = lexer.tokenize(SandhiMode::Off).unwrap();
        assert_eq!(tokens[0].kind, TokenKind::Phalam);
    }

    #[test]
    fn test_Phalam_keyword() {
        let mut lexer = Lexer::new("Phalam");
        let tokens = lexer.tokenize(SandhiMode::Off).unwrap();
        assert_eq!(tokens[0].kind, TokenKind::Phalam);
    }

    #[test]
    fn test_arogya_keyword() {
        let mut lexer = Lexer::new("arogya");
        let tokens = lexer.tokenize(SandhiMode::Off).unwrap();
        assert_eq!(tokens[0].kind, TokenKind::Arogya);
    }

    #[test]
    fn test_Arogya_keyword() {
        let mut lexer = Lexer::new("Arogya");
        let tokens = lexer.tokenize(SandhiMode::Off).unwrap();
        assert_eq!(tokens[0].kind, TokenKind::Arogya);
    }

    #[test]
    fn test_dosha_keyword() {
        let mut lexer = Lexer::new("dosha");
        let tokens = lexer.tokenize(SandhiMode::Off).unwrap();
        assert_eq!(tokens[0].kind, TokenKind::Dosha);
    }

    #[test]
    fn test_Dosha_keyword() {
        let mut lexer = Lexer::new("Dosha");
        let tokens = lexer.tokenize(SandhiMode::Off).unwrap();
        assert_eq!(tokens[0].kind, TokenKind::Dosha);
    }

    #[test]
    fn test_nidana_keyword() {
        let mut lexer = Lexer::new("nidana");
        let tokens = lexer.tokenize(SandhiMode::Off).unwrap();
        assert_eq!(tokens[0].kind, TokenKind::Nidana);
    }

    #[test]
    fn test_Nidana_keyword() {
        let mut lexer = Lexer::new("Nidana");
        let tokens = lexer.tokenize(SandhiMode::Off).unwrap();
        assert_eq!(tokens[0].kind, TokenKind::Nidana);
    }

    #[test]
    fn test_samprapti_keyword() {
        let mut lexer = Lexer::new("samprapti");
        let tokens = lexer.tokenize(SandhiMode::Off).unwrap();
        assert_eq!(tokens[0].kind, TokenKind::Samprapti);
    }

    #[test]
    fn test_Samprapti_keyword() {
        let mut lexer = Lexer::new("Samprapti");
        let tokens = lexer.tokenize(SandhiMode::Off).unwrap();
        assert_eq!(tokens[0].kind, TokenKind::Samprapti);
    }

    #[test]
    fn test_sandarbha_keyword() {
        let mut lexer = Lexer::new("sandarbha");
        let tokens = lexer.tokenize(SandhiMode::Off).unwrap();
        assert_eq!(tokens[0].kind, TokenKind::Sandarbha);
    }

    #[test]
    fn test_Sandarbha_keyword() {
        let mut lexer = Lexer::new("Sandarbha");
        let tokens = lexer.tokenize(SandhiMode::Off).unwrap();
        assert_eq!(tokens[0].kind, TokenKind::Sandarbha);
    }

    #[test]
    fn test_adhikara_keyword() {
        let mut lexer = Lexer::new("adhikara");
        let tokens = lexer.tokenize(SandhiMode::Off).unwrap();
        assert_eq!(tokens[0].kind, TokenKind::Adhikara);
    }

    #[test]
    fn test_Adhikara_keyword() {
        let mut lexer = Lexer::new("Adhikara");
        let tokens = lexer.tokenize(SandhiMode::Off).unwrap();
        assert_eq!(tokens[0].kind, TokenKind::Adhikara);
    }

    #[test]
    fn test_vikara_keyword() {
        let mut lexer = Lexer::new("vikara");
        let tokens = lexer.tokenize(SandhiMode::Off).unwrap();
        assert_eq!(tokens[0].kind, TokenKind::Vikara);
    }

    #[test]
    fn test_Vikara_keyword() {
        let mut lexer = Lexer::new("Vikara");
        let tokens = lexer.tokenize(SandhiMode::Off).unwrap();
        assert_eq!(tokens[0].kind, TokenKind::Vikara);
    }

    #[test]
    fn test_sāmānya_keyword() {
        let mut lexer = Lexer::new("sāmānya");
        let tokens = lexer.tokenize(SandhiMode::Off).unwrap();
        assert_eq!(tokens[0].kind, TokenKind::Sāmānya);
    }

    #[test]
    fn test_Sāmānya_keyword() {
        let mut lexer = Lexer::new("Sāmānya");
        let tokens = lexer.tokenize(SandhiMode::Off).unwrap();
        assert_eq!(tokens[0].kind, TokenKind::Sāmānya);
    }

    #[test]
    fn test_dhātu_keyword() {
        let mut lexer = Lexer::new("dhātu");
        let tokens = lexer.tokenize(SandhiMode::Off).unwrap();
        assert_eq!(tokens[0].kind, TokenKind::Dhātu);
    }

    #[test]
    fn test_Dhātu_keyword() {
        let mut lexer = Lexer::new("Dhātu");
        let tokens = lexer.tokenize(SandhiMode::Off).unwrap();
        assert_eq!(tokens[0].kind, TokenKind::Dhātu);
    }

    #[test]
    fn test_sāmānya_dravya_sequence() {
        let mut lexer = Lexer::new("sāmānya T dravya Peti",
        );
        let tokens = lexer.tokenize(SandhiMode::Off).unwrap();
        assert_eq!(tokens[0].kind, TokenKind::Sāmānya);
        assert_eq!(tokens[1].kind, TokenKind::Naama("T".to_string()));
        assert_eq!(tokens[2].kind, TokenKind::Dravya);
        assert_eq!(tokens[3].kind, TokenKind::Naama("Peti".to_string()));
    }

    #[test]
    fn test_sāmānya_dhātu_sequence() {
        let mut lexer = Lexer::new("sāmānya T dhātu pratirūpa");
        let tokens = lexer.tokenize(SandhiMode::Off).unwrap();
        assert_eq!(tokens[0].kind, TokenKind::Sāmānya);
        assert_eq!(tokens[1].kind, TokenKind::Naama("T".to_string()));
        assert_eq!(tokens[2].kind, TokenKind::Dhātu);
        assert_eq!(tokens[3].kind, TokenKind::Naama("pratirūpa".to_string()));
    }
}
