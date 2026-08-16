use crate::error::ParseError;
use crate::karaka_map::vibhakti_to_karaka;
use crate::symbol_table::{Symbol, SymbolKind, SymbolTable};
use devvani_ast::*;
use devvani_lexer::{Span, Token, TokenKind};

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
            while self.check(&TokenKind::Danda) {
                self.advance();
            }
        }
        Ok(ASTNode::KaryakramNode { shareera })
    }

    fn parse_vakya(&mut self) -> Result<ASTNode, ParseError> {
        while self.check(&TokenKind::Danda) {
            self.advance();
        }
        if self.check(&TokenKind::RBrace) {
            return Err(ParseError::Generic(
                "unexpected closing brace } outside of block".to_string(),
            ));
        }
        if self.is_at_end() {
            return Err(ParseError::Generic("Unexpected EOF".to_string()));
        }

        let tok = self.peek().clone();
        match tok.kind {
            TokenKind::Arambhah => self.parse_karyakram(),
            TokenKind::Yadi => self.parse_yadi(),
            TokenKind::Yavat => self.parse_yavat(),
            TokenKind::Kramasah => self.parse_kramasah(),
            TokenKind::Arogya => self.parse_arogya(),
            TokenKind::Dosha => self.parse_dosha(),
            TokenKind::Nidana => self.parse_nidana(),
             TokenKind::Sandarbha => self.parse_sandarbha(),
TokenKind::Sāmānya => self.parse_sāmānya_prefix(),
             TokenKind::Dhātu => {
                 self.advance();
                 self.parse_dhatu_def(vec![])
             }
             TokenKind::Dharā => self.parse_dhara(),
             TokenKind::Samyoga => self.parse_samyoga(),
             TokenKind::Manas => self.parse_manas(),
             TokenKind::Parikshaa => self.parse_parikshaa(false),
             TokenKind::Tarka => {
                 if self.check_ahead(1, &TokenKind::Parikshaa) {
                     self.advance();
                     self.parse_parikshaa(true)
                 } else {
                     Err(ParseError::TarkaWithoutParikshaa {
                         span: self.peek().span,
                     })
                 }
             }
             TokenKind::Nigamana => self.parse_nigamana_statement(),
             TokenKind::SadrishyaNigamana => self.parse_sadrishya_nigamana_statement(),
             TokenKind::AsadrishyaNigamana => self.parse_asadrishya_nigamana_statement(),
             TokenKind::LBracket => {
                 if self.check_ahead_is(1, &|k| matches!(k, TokenKind::Naama(_))) {
                     self.parse_dhara()
                 } else {
                     let expr = self.parse_arithmetic()?;
                     if self.check(&TokenKind::Vadati) {
                         self.advance();
                         self.expect(TokenKind::Danda)?;
                         Ok(ASTNode::VadatiNode {
                             mulya: Box::new(expr),
                         })
                     } else {
                         if self.check(&TokenKind::Danda) {
                             self.advance();
                         }
                         Ok(expr)
                     }
                 }
             }
             TokenKind::LBrace => {
                 if self.check_ahead(1, &TokenKind::Samyoga) {
                     self.parse_samyoga()
                 } else if self.check_ahead(1, &TokenKind::Manas) {
                     self.parse_manas()
                 } else {
                     Err(ParseError::Generic(
                         "unexpected LBrace: samyoga/manas block expected".to_string(),
                     ))
                 }
             }
             TokenKind::Naama(ref name) => {
                   if name.ends_with("-dhatu") {
                      let lookup = self.symbols.lookup(name);
                      if !matches!(lookup.map(|s| &s.kind), Some(SymbolKind::Dhatu { .. })) {
                          self.parse_dhatu_def(vec![])
                      } else {
                          let expr = self.parse_arithmetic()?;
                          if self.check(&TokenKind::Vadati) {
                              self.advance();
                              self.expect(TokenKind::Danda)?;
                              Ok(ASTNode::VadatiNode {
                                  mulya: Box::new(expr),
                              })
                          } else {
                              if self.check(&TokenKind::Danda) {
                                  self.advance();
                              }
                              Ok(expr)
                          }
                      }
                   } else if self.check_ahead(1, &TokenKind::Dravya) {
                       self.parse_dravya_def(vec![])
                   } else if self.check_ahead(1, &TokenKind::Nirmana) {
                       self.parse_nirmana()
                   } else if self.check_ahead(1, &TokenKind::Asti) {
                     self.parse_asti()
                 } else if self.check_ahead(1, &TokenKind::Bhavati) {
                     self.parse_bhavati()
                  } else if self.check_ahead(1, &TokenKind::Pathati) {
                      self.parse_pathati()
                   } else if self.check_ahead(1, &TokenKind::Bhej) {
                       self.parse_duta_bhej()
                   } else {
                     let expr = self.parse_arithmetic()?;
                     if self.check(&TokenKind::Vadati) {
                         self.advance();
                         self.expect(TokenKind::Danda)?;
                         Ok(ASTNode::VadatiNode {
                             mulya: Box::new(expr),
                         })
                     } else {
                         if self.check(&TokenKind::Danda) {
                             self.advance();
                         }
                         Ok(expr)
                     }
                 }
             }
            _ => {
                let expr = self.parse_arithmetic()?;
                if self.check(&TokenKind::Vadati) {
                    self.advance();
                    self.expect(TokenKind::Danda)?;
                    Ok(ASTNode::VadatiNode {
                        mulya: Box::new(expr),
                    })
                } else {
                    if self.check(&TokenKind::Danda) {
                        self.advance();
                    }
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

fn parse_dhatu_def(&mut self, generic_params: Vec<String>) -> Result<ASTNode, ParseError> {
         let name_tok = self.expect_identifier()?;
         let name = if let TokenKind::Naama(n) = name_tok.kind {
             n
         } else {
             unreachable!()
         };

         let _ = self.symbols.define(
             &name,
             Symbol {
                 name: name.clone(),
                 kind: SymbolKind::Dhatu {
                     gana: Gana::Bhvadi,
                     lakara: Lakara::Lat,
                 },
                 karaka: KarakaRole::Karta,
                 vibhakti: Vibhakti::Prathama,
                 linga: Linga::Pullinga,
                 vacana: Vacana::Eka,
                 defined_at: name_tok.span,
             },
         );

 let mut params = Vec::new();
          while !self.is_karoti() && !self.check(&TokenKind::Danda) && !self.check(&TokenKind::Phalam) && !self.is_at_end() {
              let p_tok = self.expect_identifier()?;
              let p_name = if let TokenKind::Naama(n) = p_tok.kind {
                  n
              } else {
                  unreachable!()
              };
              let vibhakti = self.match_vibhakti().unwrap_or(Vibhakti::Prathama);

 let mut is_borrowed = false;
               let mut is_mutable_borrow = false;
               let mut param_type_name = String::new();

               if self.check(&TokenKind::Adhikara) {
                   self.advance();
                   is_borrowed = true;
               } else if self.check(&TokenKind::Vikara) {
                   self.advance();
                   self.expect(TokenKind::Adhikara)?;
                   is_borrowed = true;
                   is_mutable_borrow = true;
               }

               if is_borrowed {
                   let type_tok = self.expect_identifier()?;
                   param_type_name = if let TokenKind::Naama(n) = type_tok.kind {
                       n
                   } else {
                       "unknown".to_string()
                   };
               }

 params.push(KarakaParam {
                   name: p_name,
                   role: vibhakti_to_karaka(&vibhakti),
                   vibhakti,
                   is_borrowed,
                   is_mutable_borrow,
                   type_name: param_type_name.clone(),
                   span: p_tok.span,
               });
          }

         let return_type = if self.check(&TokenKind::Phalam) {
             Some(Box::new(self.parse_phalam_type()?))
         } else {
             None
         };

         if self.is_karoti() {
             self.advance();
         }
         self.expect(TokenKind::Danda)?;

         self.symbols.push_scope();
         for p in &params {
             let _ = self.symbols.define(
                 &p.name,
                 Symbol {
                     name: p.name.clone(),
                     kind: SymbolKind::Param {
                         role: p.role.clone(),
                     },
                     karaka: p.role.clone(),
                     vibhakti: p.vibhakti.clone(),
                     linga: Linga::Pullinga,
                     vacana: Vacana::Eka,
                     defined_at: p.span,
                 },
             );
         }

         let mut body = Vec::new();
         self.current_dhatu_name.push(name.clone());
         while !self.check(&TokenKind::Iti) && !self.is_at_end() {
             body.push(self.parse_vakya()?);
         }
         self.current_dhatu_name.pop();
         self.symbols.pop_scope();
         self.expect(TokenKind::Iti)?;
         if self.check(&TokenKind::Danda) {
             self.advance();
         }

         Ok(ASTNode::DhatuDef {
             name,
             generic_params,
             gana: Gana::Bhvadi,
             lakara: Lakara::Lat,
             linga: Linga::Pullinga,
             vacana: Vacana::Eka,
             params,
             upasargas: vec![],
              return_karaka: None,
              return_type,
              body,
              span: name_tok.span,
          })
      }

     fn parse_dhara(&mut self) -> Result<ASTNode, ParseError> {
         let start_span = self.peek().span;
         self.advance(); // consume dharā

         let naamas: Vec<String>;
         let type_name: Option<String>;

         if self.check(&TokenKind::LBracket) {
             self.advance(); // consume LBracket
             let first_tok = self.expect_identifier()?;
             let first_naama = if let TokenKind::Naama(n) = first_tok.kind {
                 n
             } else {
                 unreachable!()
             };
             let mut names = vec![first_naama];
             while self.check(&TokenKind::Unknown(',')) {
                 self.advance(); // consume comma
                 let tok = self.expect_identifier()?;
                 if let TokenKind::Naama(n) = tok.kind {
                     names.push(n);
                 } else {
                     unreachable!()
                 }
             }
             self.expect(TokenKind::RBracket)?;
             naamas = names;
             type_name = None;
         } else {
             let name_tok = self.expect_identifier()?;
             let naama = if let TokenKind::Naama(n) = name_tok.kind {
                 n
             } else {
                 unreachable!()
             };
             naamas = vec![naama];

             type_name = if self.check(&TokenKind::Equals) {
                 None
             } else {
                 let type_tok = self.expect_identifier()?;
                 if let TokenKind::Naama(n) = type_tok.kind {
                     Some(n)
                 } else {
                     unreachable!()
                 }
             };
         }

         self.expect(TokenKind::Equals)?;
         let mulya = self.parse_arithmetic()?;
         self.expect(TokenKind::Danda)?;

         Ok(ASTNode::DharaNode {
             naamas,
             type_name,
             mulya: Box::new(mulya),
             is_mutable: false,
             span: start_span,
         })
     }

     fn parse_sāmānya_prefix(&mut self) -> Result<ASTNode, ParseError> {
         self.advance(); // consume Sāmānya

         let mut generic_params = Vec::new();
         while !self.check(&TokenKind::Dravya) && !self.check(&TokenKind::Dhātu) && !self.is_at_end() {
             let param_tok = self.expect_identifier()?;
             let param_name = if let TokenKind::Naama(n) = param_tok.kind {
                 n
             } else {
                 unreachable!()
             };
             generic_params.push(param_name);
         }

         if self.is_at_end() {
             return Err(ParseError::Generic(
                 "sāmānya must be followed by dravya or dhātu".to_string(),
             ));
         }

         match self.peek().kind.clone() {
             TokenKind::Dravya => {
                 self.advance(); // consume Dravya
                 self.parse_dravya_def(generic_params)
             }
             TokenKind::Dhātu => {
                 self.advance(); // consume Dhātu
                 self.parse_dhatu_def(generic_params)
             }
             _ => Err(ParseError::UnexpectedToken {
                 expected: "dravya or dhātu after sāmānya".to_string(),
                 found: self.peek().kind.clone(),
                 span: self.peek().span,
             }),
         }
     }

     fn parse_phalam_type(&mut self) -> Result<ASTNode, ParseError> {
        self.expect(TokenKind::Phalam)?;
        let success_tok = self.expect_identifier()?;
        let success_type = if let TokenKind::Naama(n) = success_tok.kind {
            n
        } else {
            unreachable!()
        };
        let error_tok = self.expect_identifier()?;
        let error_type = if let TokenKind::Naama(n) = error_tok.kind {
            n
        } else {
            unreachable!()
        };
        Ok(ASTNode::PhalamType {
            success_type,
            error_type,
            span: success_tok.span,
        })
    }

fn parse_dravya_def(&mut self, generic_params: Vec<String>) -> Result<ASTNode, ParseError> {
         let name_tok = self.expect_identifier()?;
         let name = if let TokenKind::Naama(n) = name_tok.kind {
             n
         } else {
             unreachable!()
         };

         if self.check(&TokenKind::Dravya) {
             self.advance();
         }

         let _ = self.symbols.define(
             &name,
             Symbol {
                 name: name.clone(),
                 kind: SymbolKind::Dravya,
                 karaka: KarakaRole::Karta,
                 vibhakti: Vibhakti::Prathama,
                 linga: Linga::Pullinga,
                 vacana: Vacana::Eka,
                 defined_at: name_tok.span,
             },
         );

         let mut angas = Vec::new();
         while !self.check(&TokenKind::Danda) && !self.is_at_end() {
             let anga_tok = self.expect_identifier()?;
             let anga_name = if let TokenKind::Naama(n) = anga_tok.kind {
                 n
             } else {
                 unreachable!()
             };
             if self.is_at_end() || self.check(&TokenKind::Danda) {
                 return Err(ParseError::UnexpectedToken {
                     expected: "type_name for anga".to_string(),
                     found: self.peek().kind.clone(),
                     span: self.peek().span,
                 });
             }
             let type_tok = self.expect_identifier()?;
             let type_name = if let TokenKind::Naama(n) = type_tok.kind {
                 n
             } else {
                 unreachable!()
             };
             angas.push(AngaField {
                 name: anga_name,
                 type_name,
                 span: anga_tok.span,
             });
         }

         self.expect(TokenKind::Danda)?;

         Ok(ASTNode::DravyaDef {
             name,
             generic_params,
             angas,
             span: name_tok.span,
         })
     }

    fn parse_nirmana(&mut self) -> Result<ASTNode, ParseError> {
        let name_tok = self.expect_identifier()?;
        let dravya_name = if let TokenKind::Naama(n) = name_tok.kind {
            n
        } else {
            unreachable!()
        };

        self.advance();

        let mut values = Vec::new();
        while !self.check(&TokenKind::Danda) && !self.is_at_end() {
            let expr = self.parse_arithmetic()?;
            values.push(expr);
        }

        self.expect(TokenKind::Danda)?;

        Ok(ASTNode::NirmanaNode {
            dravya_name,
            values,
            span: name_tok.span,
        })
    }

    fn parse_arogya(&mut self) -> Result<ASTNode, ParseError> {
        let arogya_tok = self.expect(TokenKind::Arogya)?;
        let value = self.parse_arithmetic()?;
        self.expect(TokenKind::Danda)?;
        Ok(ASTNode::ArogyaNode {
            value: Box::new(value),
            span: arogya_tok.span,
        })
    }

    fn parse_dosha(&mut self) -> Result<ASTNode, ParseError> {
        let dosha_tok = self.expect(TokenKind::Dosha)?;
        let value = self.parse_arithmetic()?;
        self.expect(TokenKind::Danda)?;
        Ok(ASTNode::DoshaNode {
            value: Box::new(value),
            span: dosha_tok.span,
        })
    }

    fn parse_nidana(&mut self) -> Result<ASTNode, ParseError> {
        let start_span = self.peek().span;
        self.advance(); // consume Nidana
        let target = self.parse_arithmetic()?;

        self.expect(TokenKind::Arogya)?;
        let arogya_bind_tok = self.expect_identifier()?;
        let arogya_bind = if let TokenKind::Naama(n) = arogya_bind_tok.kind {
            n
        } else {
            unreachable!()
        };

        let mut arogya_body = Vec::new();
        while !self.check(&TokenKind::Dosha) && !self.check(&TokenKind::Iti) && !self.is_at_end() {
            arogya_body.push(self.parse_vakya()?);
        }

        self.expect(TokenKind::Dosha)?;
        let dosha_bind_tok = self.expect_identifier()?;
        let dosha_bind = if let TokenKind::Naama(n) = dosha_bind_tok.kind {
            n
        } else {
            unreachable!()
        };

        let mut dosha_body = Vec::new();
        while !self.check(&TokenKind::Iti) && !self.is_at_end() {
            dosha_body.push(self.parse_vakya()?);
        }
        self.expect(TokenKind::Iti)?;
        if self.check(&TokenKind::Danda) {
            self.advance();
        }

        Ok(ASTNode::NidanaNode {
            target: Box::new(target),
            arogya_bind,
            arogya_body,
            dosha_bind,
            dosha_body,
span: start_span,
         })
     }

     fn parse_sandarbha(&mut self) -> Result<ASTNode, ParseError> {
         let start_span = self.peek().span;
         self.advance(); // consume Sandarbha

let is_mutable;
        if self.check(&TokenKind::Vikara) {
            self.advance();
            self.expect(TokenKind::Adhikara)?;
            is_mutable = true;
        } else if self.check(&TokenKind::Adhikara) {
            self.advance();
            is_mutable = false;
        } else {
            return Err(ParseError::UnexpectedToken {
                expected: "adhikara or vikara adhikara after sandarbha".to_string(),
                found: self.peek().kind.clone(),
                span: self.peek().span,
            });
        }

         let target = Box::new(self.parse_arithmetic()?);
         self.expect(TokenKind::Danda)?;

         Ok(ASTNode::SandarbhaNode {
             target,
             is_mutable,
             span: start_span,
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
        let naama = if let TokenKind::Naama(n) = naama_tok.kind {
            n
        } else {
            unreachable!()
        };
        self.expect(TokenKind::Asti)?;
        let mulya = self.parse_arithmetic()?;
        self.expect(TokenKind::Danda)?;
        Ok(ASTNode::AstiNode {
            naama,
            mulya: Box::new(mulya),
        })
    }

    fn parse_bhavati(&mut self) -> Result<ASTNode, ParseError> {
        let naama_tok = self.expect_identifier()?;
        let naama = if let TokenKind::Naama(n) = naama_tok.kind {
            n
        } else {
            unreachable!()
        };
        self.expect(TokenKind::Bhavati)?;
        let mulya = self.parse_arithmetic()?;
        self.expect(TokenKind::Danda)?;
        Ok(ASTNode::BhavatiNode {
            naama,
            mulya: Box::new(mulya),
        })
    }

    fn parse_pathati(&mut self) -> Result<ASTNode, ParseError> {
        let naama_tok = self.expect_identifier()?;
        let naama = if let TokenKind::Naama(n) = naama_tok.kind {
            n
        } else {
            unreachable!()
        };
        self.expect(TokenKind::Pathati)?;
        self.expect(TokenKind::Danda)?;
        Ok(ASTNode::PathatiNode { naama })
    }

    fn parse_yadi(&mut self) -> Result<ASTNode, ParseError> {
        self.expect(TokenKind::Yadi)?;
        let sthiti = Box::new(self.parse_arithmetic()?);
        self.expect(TokenKind::Tarhi)?;

        let mut tarhi = Vec::new();
        while !self.check(&TokenKind::Anyatha) && !self.check(&TokenKind::Iti) && !self.is_at_end()
        {
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
        if self.check(&TokenKind::Danda) {
            self.advance();
        }

        Ok(ASTNode::YadiNode {
            sthiti,
            tarhi,
            anyatha,
        })
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
        if self.check(&TokenKind::Danda) {
            self.advance();
        }

        Ok(ASTNode::YavatNode { sthiti, shareera })
    }

    fn parse_arithmetic(&mut self) -> Result<ASTNode, ParseError> {
        let mut left = self.parse_primary()?;

        // Handle postfix indexing: expr[index]
        if self.check(&TokenKind::LBracket) {
            let span = self.extract_span(&left);
            left = self.parse_vinyasa_access(left, span)?;
        }

        // Handle postfix field access: expr.field
        while self.check(&TokenKind::Dot) {
            let span = self.extract_span(&left);
            left = self.parse_samavaya_access(left, span)?;
        }

        while let Some(tok) = self.match_any(&[
            TokenKind::Yoga,
            TokenKind::Viyoga,
            TokenKind::Guna,
            TokenKind::Bhaga,
        ]) {
            let right = self.parse_primary()?;
            left = match tok.kind {
                TokenKind::Yoga => ASTNode::YogaNode {
                    vama: Box::new(left),
                    dakshina: Box::new(right),
                },
                TokenKind::Viyoga => ASTNode::ViyogaNode {
                    vama: Box::new(left),
                    dakshina: Box::new(right),
                },
                TokenKind::Guna => ASTNode::GunaNode {
                    vama: Box::new(left),
                    dakshina: Box::new(right),
                },
                TokenKind::Bhaga => ASTNode::BhagaNode {
                    vama: Box::new(left),
                    dakshina: Box::new(right),
                },
                _ => unreachable!(),
            };
        }

        if let Some(tok) = self.match_any(&[
            TokenKind::Sama,
            TokenKind::AsamaH,
            TokenKind::NyuunaH,
            TokenKind::AdhikaH,
        ]) {
            let right = self.parse_arithmetic()?;
            left = match tok.kind {
                TokenKind::Sama => ASTNode::SamaNode {
                    vama: Box::new(left),
                    dakshina: Box::new(right),
                },
                TokenKind::AsamaH => ASTNode::AsamaNode {
                    vama: Box::new(left),
                    dakshina: Box::new(right),
                },
                TokenKind::NyuunaH => ASTNode::NyuunaNode {
                    vama: Box::new(left),
                    dakshina: Box::new(right),
                },
                TokenKind::AdhikaH => ASTNode::AdhikaNode {
                    vama: Box::new(left),
                    dakshina: Box::new(right),
                },
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
            if self.check(&TokenKind::Danda) {
                self.advance();
            }
            left = ASTNode::PunahNode {
                varam: Box::new(left),
                shareera,
            };
         }

          if self.check(&TokenKind::Prapti) {
              let current = self.peek().clone();
              self.advance();
              left = ASTNode::PraptiNode {
                  handle: Box::new(left),
                  span: current.span,
              };
          }

          if self.check(&TokenKind::Parinama) {
              let current = self.peek().clone();
              self.advance();
              let dhatus = self.parse_parinama_dhatu_list()?;
              left = ASTNode::ParinamaNode {
                  mulyam: Box::new(left),
                  dhatus,
                  span: current.span,
              };
          }

          Ok(left)
      }

     fn parse_primary(&mut self) -> Result<ASTNode, ParseError> {
         let start_span = self.peek().span;
         let tok = self.advance();
         match tok.kind {
            TokenKind::LBracket => return self.parse_pankti_literal(tok.span),
            TokenKind::Avali => {
                return self.parse_avali_literal(tok.span);
            }
             TokenKind::Samyoga => {
                 self.expect(TokenKind::LBrace)?;
                 let mut body = Vec::new();
                 while !self.check(&TokenKind::RBrace) && !self.is_at_end() {
                     body.push(self.parse_vakya()?);
                 }
                 self.expect(TokenKind::RBrace)?;
                 return Ok(ASTNode::SamyogaNode { body, span: start_span });
             }
            TokenKind::Duta => {
                self.expect(TokenKind::Banaa)?;
                return Ok(ASTNode::DutaBanaaNode { span: start_span });
            }
             TokenKind::PurnaankLiteral(value) => Ok(ASTNode::PurnaankLiteral {
                 value,
                 span: tok.span,
             }),
            TokenKind::DashaamshaLiteral(value) => Ok(ASTNode::DashaamshaLiteral {
                value,
                span: tok.span,
            }),
            TokenKind::VaakLiteral(value) => Ok(ASTNode::VaakLiteral {
                value,
                span: tok.span,
            }),
             TokenKind::Naama(name) => {
                 if self.check(&TokenKind::Grahan) {
                     return self.parse_duta_grahan(name);
                 }
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

fn parse_pankti_literal(&mut self, span: Span) -> Result<ASTNode, ParseError> {
         let mut elements = Vec::new();
         if !self.check(&TokenKind::RBracket) {
             loop {
                 elements.push(self.parse_arithmetic()?);
                 if !self.check(&TokenKind::Unknown(',')) {
                     break;
                 }
                 self.advance();
             }
         }
         self.expect(TokenKind::RBracket)?;
         Ok(ASTNode::PanktiNode { elements, span })
     }

     fn parse_avali_literal(&mut self, span: Span) -> Result<ASTNode, ParseError> {
         self.expect(TokenKind::LBracket)?;
         let mut elements = Vec::new();
         if !self.check(&TokenKind::RBracket) {
             loop {
                 elements.push(self.parse_arithmetic()?);
                 if !self.check(&TokenKind::Unknown(',')) {
                     break;
                 }
                 self.advance();
             }
         }
         self.expect(TokenKind::RBracket)?;
         Ok(ASTNode::AvaliNode { elements, span })
     }

     fn parse_vinyasa_access(&mut self, target: ASTNode, span: Span) -> Result<ASTNode, ParseError> {
        // Consume the LBracket before parsing the inner expression
        self.advance(); // consume LBracket
        let index = self.parse_arithmetic()?;
        self.expect(TokenKind::RBracket)?;
        Ok(ASTNode::VinyasaNode {
            target: Box::new(target),
            index: Box::new(index),
            span,
        })
    }

    fn parse_samavaya_access(&mut self, target: ASTNode, span: Span) -> Result<ASTNode, ParseError> {
        self.advance(); // consume Dot
        let anga_tok = self.expect_identifier()?;
        let anga_name = if let TokenKind::Naama(n) = anga_tok.kind {
            n
        } else {
            unreachable!()
        };
        Ok(ASTNode::SamavayaNode {
            target: Box::new(target),
            anga_name,
            span,
        })
    }

    fn parse_kramasah(&mut self) -> Result<ASTNode, ParseError> {
        let start_span = self.peek().span;
        self.advance(); // consume Kramasah
        let item_tok = self.expect_identifier()?;
        let item_name = if let TokenKind::Naama(n) = item_tok.kind {
            n
        } else {
            unreachable!()
        };
        let iterable = self.parse_arithmetic()?;
        self.expect(TokenKind::Tavat)?;
        self.symbols.push_scope();
        let _ = self.symbols.define(
            &item_name,
            Symbol {
                name: item_name.clone(),
                kind: SymbolKind::Param {
                    role: KarakaRole::Karana,
                },
                karaka: KarakaRole::Karana,
                vibhakti: Vibhakti::Tritiya,
                linga: Linga::Pullinga,
                vacana: Vacana::Eka,
                defined_at: item_tok.span,
            },
        );
        let mut body = Vec::new();
        while !self.check(&TokenKind::Iti) && !self.is_at_end() {
            body.push(self.parse_vakya()?);
        }
        self.symbols.pop_scope();
        self.expect(TokenKind::Iti)?;
        if self.check(&TokenKind::Danda) {
            self.advance();
        }
         Ok(ASTNode::KramashahNode {
             item_name,
             iterable: Box::new(iterable),
             body,
             span: start_span,
         })
     }

     fn parse_samyoga(&mut self) -> Result<ASTNode, ParseError> {
         let start_span = self.peek().span;
         self.advance(); // consume Samyoga
         self.expect(TokenKind::LBrace)?;

         let mut body = Vec::new();
         while !self.check(&TokenKind::RBrace) && !self.is_at_end() {
             body.push(self.parse_vakya()?);
         }
         self.expect(TokenKind::RBrace)?;

         Ok(ASTNode::SamyogaNode { body, span: start_span })
     }

      fn parse_duta_bhej(&mut self) -> Result<ASTNode, ParseError> {
         let sender_tok = self.advance(); // Naama(sender)
          let sender = if let TokenKind::Naama(n) = sender_tok.kind {
              ASTNode::Nama {
                  base: n,
                  vibhakti: Vibhakti::Prathama,
                  linga: Linga::Pullinga,
                  vacana: Vacana::Eka,
                  span: sender_tok.span,
              }
           } else {
               unreachable!()
           };
          self.expect(TokenKind::Bhej)?;
          self.expect(TokenKind::Sandesha)?;
           let message = self.parse_arithmetic()?;
           self.expect(TokenKind::Danda)?;
         Ok(ASTNode::DutaBhejNode {
             sender: Box::new(sender),
             message: Box::new(message),
             span: sender_tok.span,
         })
     }

     fn parse_duta_grahan(&mut self, receiver_name: String) -> Result<ASTNode, ParseError> {
         let receiver = ASTNode::Nama {
             base: receiver_name,
             vibhakti: Vibhakti::Prathama,
             linga: Linga::Pullinga,
             vacana: Vacana::Eka,
             span: Span { line: 1, col: 1, len: 1 },
         };
         self.expect(TokenKind::Grahan)?;
         self.expect(TokenKind::Karo)?;
         self.expect(TokenKind::Danda)?;
         Ok(ASTNode::DutaGrahanNode {
             receiver: Box::new(receiver),
             span: Span { line: 1, col: 1, len: 1 },
         })
     }

       fn parse_manas(&mut self) -> Result<ASTNode, ParseError> {
           let start_span = self.peek().span;
           self.expect(TokenKind::Manas)?;
           let target = self.parse_arithmetic()?;
           self.expect(TokenKind::LBrace)?;

           let mut body = Vec::new();
           while !self.check(&TokenKind::RBrace) && !self.is_at_end() {
               body.push(self.parse_vakya()?);
           }
           self.expect(TokenKind::RBrace)?;

           Ok(ASTNode::ManasNode {
               target: Box::new(target),
               body,
               span: start_span,
           })
       }

      fn parse_parikshaa(&mut self, is_tarka: bool) -> Result<ASTNode, ParseError> {
          let start_span = self.peek().span;
          self.expect(TokenKind::Parikshaa)?;
          let name_tok = self.expect_identifier()?;
          let name = if let TokenKind::Naama(n) = name_tok.kind {
              n
          } else {
              unreachable!()
          };
          self.expect(TokenKind::LBrace)?;

          let mut body = Vec::new();
          while !self.check(&TokenKind::RBrace) && !self.is_at_end() {
              body.push(self.parse_vakya()?);
          }
          self.expect(TokenKind::RBrace)?;
          if self.check(&TokenKind::Danda) {
              self.advance();
          }

          Ok(ASTNode::ParikshaaNode {
              name,
              body,
              is_tarka,
              span: start_span,
          })
      }

      fn parse_nigamana_statement(&mut self) -> Result<ASTNode, ParseError> {
          let start_span = self.peek().span;
          self.expect(TokenKind::Nigamana)?;

          let mut args = Vec::new();
          while !self.check(&TokenKind::Danda) && !self.is_at_end() {
              args.push(self.parse_arithmetic()?);
          }
          self.expect(TokenKind::Danda)?;

          if args.len() != 1 {
              return Err(ParseError::AssertionArgCount {
                  keyword: "nigamana".to_string(),
                  expected: 1,
                  found: args.len(),
                  span: start_span,
              });
          }

          Ok(ASTNode::NigamanaNode {
              expr: Box::new(args.into_iter().next().unwrap()),
              span: start_span,
          })
      }

      fn parse_sadrishya_nigamana_statement(&mut self) -> Result<ASTNode, ParseError> {
          let start_span = self.peek().span;
          self.expect(TokenKind::SadrishyaNigamana)?;

          let mut args = Vec::new();
          while !self.check(&TokenKind::Danda) && !self.is_at_end() {
              args.push(self.parse_arithmetic()?);
          }
          self.expect(TokenKind::Danda)?;

          if args.len() != 2 {
              return Err(ParseError::AssertionArgCount {
                  keyword: "sadrishya-nigamana".to_string(),
                  expected: 2,
                  found: args.len(),
                  span: start_span,
              });
          }

          Ok(ASTNode::SadrishyaNigamanaNode {
              left: Box::new(args[0].clone()),
              right: Box::new(args[1].clone()),
              span: start_span,
          })
      }

      fn parse_asadrishya_nigamana_statement(&mut self) -> Result<ASTNode, ParseError> {
          let start_span = self.peek().span;
          self.expect(TokenKind::AsadrishyaNigamana)?;

          let mut args = Vec::new();
          while !self.check(&TokenKind::Danda) && !self.is_at_end() {
              args.push(self.parse_arithmetic()?);
          }
          self.expect(TokenKind::Danda)?;

          if args.len() != 2 {
              return Err(ParseError::AssertionArgCount {
                  keyword: "asadrishya-nigamana".to_string(),
                  expected: 2,
                  found: args.len(),
                  span: start_span,
              });
          }

          Ok(ASTNode::AsadrishyaNigamanaNode {
              left: Box::new(args[0].clone()),
              right: Box::new(args[1].clone()),
              span: start_span,
          })
      }

      fn parse_parinama_dhatu_list(&mut self) -> Result<Vec<String>, ParseError> {
          self.expect(TokenKind::LBracket)?;
          let mut dhatus = Vec::new();
          if !self.check(&TokenKind::RBracket) {
              loop {
                  let tok = self.expect_identifier()?;
                  if let TokenKind::Naama(n) = tok.kind {
                      dhatus.push(n);
                  } else {
                      unreachable!()
                  }
                  if !self.check(&TokenKind::Unknown(',')) {
                      break;
                  }
                  self.advance();
              }
          }
          self.expect(TokenKind::RBracket)?;
          Ok(dhatus)
      }

      fn parse_kriya_call(&mut self, name: String, span: Span) -> Result<ASTNode, ParseError> {
         let stop_kinds = [
             TokenKind::Danda,
             TokenKind::Iti,
             TokenKind::Vadati,
             TokenKind::Yoga,
             TokenKind::Viyoga,
             TokenKind::Guna,
             TokenKind::Bhaga,
             TokenKind::Sama,
             TokenKind::AsamaH,
             TokenKind::NyuunaH,
             TokenKind::AdhikaH,
             TokenKind::Varam,
             TokenKind::Tarhi,
             TokenKind::Anyatha,
             TokenKind::Samaapti,
              TokenKind::Samyoga,
              TokenKind::Manas,
              TokenKind::Prapti,
              TokenKind::Parinama,
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
        if v.is_some() {
            self.advance();
        }
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
        if self.is_at_end() {
            return false;
        }
        let next = &self.peek().kind;
        kinds.iter().any(|k| k == next)
    }

    fn check(&self, kind: &TokenKind) -> bool {
        if self.is_at_end() {
            return false;
        }
        &self.peek().kind == kind
    }

    fn check_ahead(&self, n: usize, kind: &TokenKind) -> bool {
        if self.pos + n >= self.tokens.len() {
            return false;
        }
        &self.tokens[self.pos + n].kind == kind
    }

    fn check_ahead_is(&self, n: usize, f: &dyn Fn(&TokenKind) -> bool) -> bool {
        if self.pos + n >= self.tokens.len() {
            return false;
        }
        f(&self.tokens[self.pos + n].kind)
    }

    fn match_any(&mut self, kinds: &[TokenKind]) -> Option<Token> {
        for kind in kinds {
            if self.check(kind) {
                return Some(self.advance());
            }
        }
        None
    }

    fn extract_span(&self, node: &ASTNode) -> Span {
        match node {
            ASTNode::PurnaankLiteral { span, .. } => *span,
            ASTNode::DashaamshaLiteral { span, .. } => *span,
            ASTNode::VaakLiteral { span, .. } => *span,
            ASTNode::Nama { span, .. } => *span,
            ASTNode::PanktiNode { span, .. } => *span,
            ASTNode::DravyaDef { span, .. } => *span,
            ASTNode::SamavayaNode { span, .. } => *span,
ASTNode::SamprapatiNode { span, .. } => *span,
             ASTNode::SandarbhaNode { span, .. } => *span,
             _ => Span {
                line: 1,
                col: 1,
                len: 1,
            },
        }
    }

    fn advance(&mut self) -> Token {
        if !self.is_at_end() {
            self.pos += 1;
        }
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
        Span {
            line: 1,
            col: 1,
            len: 1,
        }
    }

    fn nm(s: &str) -> Token {
        Token {
            kind: TokenKind::Naama(s.to_string()),
            span: span(),
        }
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
            nm("square-dhatu"),
            nm("n"),
            nm("karoti"),
            kw(TokenKind::Danda),
            nm("n"),
            kw(TokenKind::Iti),
            kw(TokenKind::Danda),
        ];
        let ast = parse_tokens(tokens).expect("should parse");

        match ast {
            ASTNode::KaryakramNode { shareera } => {
                assert_eq!(shareera.len(), 1);
                match &shareera[0] {
                    ASTNode::DhatuDef {
                        name, params, body, ..
                    } => {
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
            nm("factorial-dhatu"),
            nm("n"),
            nm("karoti"),
            kw(TokenKind::Danda),
            nm("n"),
            kw(TokenKind::Yoga),
            nm("factorial-dhatu"),
            nm("n"),
            kw(TokenKind::Iti),
            kw(TokenKind::Danda),
        ];
        let ast = parse_tokens(tokens).expect("should parse");

        match ast {
            ASTNode::KaryakramNode { shareera } => {
                match &shareera[0] {
                    ASTNode::DhatuDef { body, .. } => {
                        // body[0] = YogaNode { vama: Nama n, dakshina: AvartanaNode }
                        match &body[0] {
                            ASTNode::YogaNode { dakshina, .. } => match dakshina.as_ref() {
                                ASTNode::AvartanaNode { call, .. } => match call.as_ref() {
                                    ASTNode::KriyaCall { kriya, karma, .. } => {
                                        assert_eq!(kriya, "factorial-dhatu");
                                        assert_eq!(karma.len(), 1);
                                    }
                                    other => panic!(
                                        "expected KriyaCall inside AvartanaNode, got {:?}",
                                        other
                                    ),
                                },
                                other => panic!("expected AvartanaNode, got {:?}", other),
                            },
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
            nm("first-dhatu"),
            nm("x"),
            nm("karoti"),
            kw(TokenKind::Danda),
            nm("x"),
            kw(TokenKind::Iti),
            kw(TokenKind::Danda),
            nm("second-dhatu"),
            nm("y"),
            nm("karoti"),
            kw(TokenKind::Danda),
            nm("y"),
            kw(TokenKind::Yoga),
            nm("first-dhatu"),
            nm("x"),
            kw(TokenKind::Iti),
            kw(TokenKind::Danda),
        ];
        let ast = parse_tokens(tokens).expect("should parse");

        match ast {
            ASTNode::KaryakramNode { shareera } => match &shareera[1] {
                ASTNode::DhatuDef { body, .. } => match &body[0] {
                    ASTNode::YogaNode { dakshina, .. } => match dakshina.as_ref() {
                        ASTNode::KriyaCall { kriya, karma, .. } => {
                            assert_eq!(kriya, "first-dhatu");
                            assert_eq!(karma.len(), 1);
                        }
                        other => panic!("expected plain KriyaCall, got {:?}", other),
                    },
                    other => panic!("expected YogaNode, got {:?}", other),
                },
                other => panic!("expected DhatuDef, got {:?}", other),
            },
            other => panic!("expected KaryakramNode, got {:?}", other),
        }
    }

    // A reference to a name NOT registered as a Dhatu stays a plain Nama (no regression).
    #[test]
    fn test_unregistered_name_is_nama() {
        let tokens = vec![
            nm("foo-dhatu"),
            nm("a"),
            nm("karoti"),
            kw(TokenKind::Danda),
            nm("a"),
            kw(TokenKind::Yoga),
            nm("bar"),
            kw(TokenKind::Iti),
            kw(TokenKind::Danda),
        ];
        let ast = parse_tokens(tokens).expect("should parse");

        match ast {
            ASTNode::KaryakramNode { shareera } => match &shareera[0] {
                ASTNode::DhatuDef { body, .. } => match &body[0] {
                    ASTNode::YogaNode { dakshina, .. } => {
                        assert!(
                            matches!(dakshina.as_ref(), ASTNode::Nama { base, .. } if base == "bar")
                        );
                    }
                    other => panic!("expected YogaNode, got {:?}", other),
                },
                other => panic!("expected DhatuDef, got {:?}", other),
            },
            other => panic!("expected KaryakramNode, got {:?}", other),
        }
    }

    // Empty array literal [] parses to PanktiNode with 0 elements
    #[test]
    fn test_empty_array_literal() {
        let tokens = vec![kw(TokenKind::LBracket), kw(TokenKind::RBracket)];
        let ast = parse_tokens(tokens).expect("should parse");

        match ast {
            ASTNode::KaryakramNode { shareera } => {
                assert_eq!(shareera.len(), 1);
                assert!(
                    matches!(&shareera[0], ASTNode::PanktiNode { elements, .. } if elements.is_empty())
                );
            }
            other => panic!("expected KaryakramNode, got {:?}", other),
        }
    }

    // Array literal with 3 numeric elements parses correctly
    #[test]
    fn test_array_literal_with_three_elements() {
        let tokens = vec![
            kw(TokenKind::LBracket),
            kw(TokenKind::PurnaankLiteral(1)),
            kw(TokenKind::Unknown(',')),
            kw(TokenKind::PurnaankLiteral(2)),
            kw(TokenKind::Unknown(',')),
            kw(TokenKind::PurnaankLiteral(3)),
            kw(TokenKind::RBracket),
        ];
        let ast = parse_tokens(tokens).expect("should parse");

        match ast {
            ASTNode::KaryakramNode { shareera } => {
                assert_eq!(shareera.len(), 1);
                match &shareera[0] {
                    ASTNode::PanktiNode { elements, .. } => {
                        assert_eq!(elements.len(), 3);
                        assert!(
                            matches!(&elements[0], ASTNode::PurnaankLiteral { value, .. } if *value == 1)
                        );
                        assert!(
                            matches!(&elements[1], ASTNode::PurnaankLiteral { value, .. } if *value == 2)
                        );
                        assert!(
                            matches!(&elements[2], ASTNode::PurnaankLiteral { value, .. } if *value == 3)
                        );
                    }
                    other => panic!("expected PanktiNode, got {:?}", other),
                }
            }
            other => panic!("expected KaryakramNode, got {:?}", other),
        }
    }

    // Nested array literal [[1,2],[3,4]] parses correctly
    #[test]
    fn test_nested_array_literal() {
        let tokens = vec![
            kw(TokenKind::LBracket),
            kw(TokenKind::LBracket),
            kw(TokenKind::PurnaankLiteral(1)),
            kw(TokenKind::Unknown(',')),
            kw(TokenKind::PurnaankLiteral(2)),
            kw(TokenKind::RBracket),
            kw(TokenKind::Unknown(',')),
            kw(TokenKind::LBracket),
            kw(TokenKind::PurnaankLiteral(3)),
            kw(TokenKind::Unknown(',')),
            kw(TokenKind::PurnaankLiteral(4)),
            kw(TokenKind::RBracket),
            kw(TokenKind::RBracket),
        ];
        let ast = parse_tokens(tokens).expect("should parse");

        match ast {
            ASTNode::KaryakramNode { shareera } => {
                assert_eq!(shareera.len(), 1);
                match &shareera[0] {
                    ASTNode::PanktiNode { elements, .. } => {
                        assert_eq!(elements.len(), 2);
                        assert!(
                            matches!(&elements[0], ASTNode::PanktiNode { elements: inner, .. } if inner.len() == 2)
                        );
                        assert!(
                            matches!(&elements[1], ASTNode::PanktiNode { elements: inner, .. } if inner.len() == 2)
                        );
                    }
                    other => panic!("expected PanktiNode, got {:?}", other),
                }
            }
            other => panic!("expected KaryakramNode, got {:?}", other),
        }
    }

    // Simple indexing x[0] parses to VinyasaNode
    #[test]
    fn test_simple_indexing() {
        let tokens = vec![
            nm("arr"),
            kw(TokenKind::LBracket),
            kw(TokenKind::PurnaankLiteral(0)),
            kw(TokenKind::RBracket),
        ];
        let ast = parse_tokens(tokens).expect("should parse");

        match ast {
            ASTNode::KaryakramNode { shareera } => {
                assert_eq!(shareera.len(), 1);
                match &shareera[0] {
                    ASTNode::VinyasaNode { target, index, .. } => {
                        assert!(
                            matches!(target.as_ref(), ASTNode::Nama { base, .. } if base == "arr")
                        );
                        assert!(
                            matches!(index.as_ref(), ASTNode::PurnaankLiteral { value, .. } if *value == 0)
                        );
                    }
                    other => panic!("expected VinyasaNode, got {:?}", other),
                }
            }
            other => panic!("expected KaryakramNode, got {:?}", other),
        }
    }

    // Indexing with expression as index parses correctly
    #[test]
    fn test_indexing_with_expression() {
        let tokens = vec![
            nm("arr"),
            kw(TokenKind::LBracket),
            nm("i"),
            kw(TokenKind::RBracket),
        ];
        let ast = parse_tokens(tokens).expect("should parse");

        match ast {
            ASTNode::KaryakramNode { shareera } => {
                assert_eq!(shareera.len(), 1);
                match &shareera[0] {
                    ASTNode::VinyasaNode { target, index, .. } => {
                        assert!(
                            matches!(target.as_ref(), ASTNode::Nama { base, .. } if base == "arr")
                        );
                        assert!(
                            matches!(index.as_ref(), ASTNode::Nama { base, .. } if base == "i")
                        );
                    }
                    other => panic!("expected VinyasaNode, got {:?}", other),
                }
            }
other => panic!("expected KaryakramNode, got {:?}", other),
         }
     }

    // Basic kramasah parse with array literal iterable
    #[test]
    fn test_kramasah_basic_parse() {
        let tokens = vec![
            kw(TokenKind::Kramasah),
            nm("x"),
            kw(TokenKind::LBracket),
            kw(TokenKind::PurnaankLiteral(1)),
            kw(TokenKind::Unknown(',')),
            kw(TokenKind::PurnaankLiteral(2)),
            kw(TokenKind::Unknown(',')),
            kw(TokenKind::PurnaankLiteral(3)),
            kw(TokenKind::RBracket),
            kw(TokenKind::Tavat),
            nm("x"),
            kw(TokenKind::Vadati),
            kw(TokenKind::Danda),
            kw(TokenKind::Iti),
        ];
        let ast = parse_tokens(tokens).expect("should parse");

        match ast {
            ASTNode::KaryakramNode { shareera } => {
                assert_eq!(shareera.len(), 1);
                match &shareera[0] {
                    ASTNode::KramashahNode { item_name, body, .. } => {
                        assert_eq!(item_name, "x");
                        assert_eq!(body.len(), 1);
                    }
                    other => panic!("expected KramashahNode, got {:?}", other),
                }
            }
            other => panic!("expected KaryakramNode, got {:?}", other),
        }
    }

    // Kramasah with empty body (immediately followed by tavat and iti)
    #[test]
    fn test_kramasah_empty_body() {
        let tokens = vec![
            kw(TokenKind::Kramasah),
            nm("x"),
            kw(TokenKind::LBracket),
            kw(TokenKind::RBracket),
            kw(TokenKind::Tavat),
            kw(TokenKind::Iti),
        ];
        let result = parse_tokens(tokens);
        assert!(result.is_ok(), "empty body kramasah should parse without error");
    }

    // Kramasah with variable reference as iterable
    #[test]
    fn test_kramasah_over_named_pankti() {
        let tokens = vec![
            kw(TokenKind::Kramasah),
            nm("item"),
            nm("arr"),
            kw(TokenKind::Tavat),
            nm("item"),
            kw(TokenKind::Vadati),
            kw(TokenKind::Danda),
            kw(TokenKind::Iti),
        ];
        let ast = parse_tokens(tokens).expect("should parse");

        match ast {
            ASTNode::KaryakramNode { shareera } => {
                assert_eq!(shareera.len(), 1);
                match &shareera[0] {
                    ASTNode::KramashahNode { item_name, iterable, body, .. } => {
                        assert_eq!(item_name, "item");
                        assert!(matches!(iterable.as_ref(), ASTNode::Nama { base, .. } if base == "arr"));
                        assert_eq!(body.len(), 1);
                    }
                    other => panic!("expected KramashahNode, got {:?}", other),
                }
            }
            other => panic!("expected KaryakramNode, got {:?}", other),
        }
    }

    // Kramasah with multiple body statements
    #[test]
    fn test_kramasah_nested_body_statements() {
        let tokens = vec![
            kw(TokenKind::Kramasah),
            nm("x"),
            kw(TokenKind::LBracket),
            kw(TokenKind::PurnaankLiteral(1)),
            kw(TokenKind::Unknown(',')),
            kw(TokenKind::PurnaankLiteral(2)),
            kw(TokenKind::RBracket),
            kw(TokenKind::Tavat),
            nm("x"),
            kw(TokenKind::Vadati),
            kw(TokenKind::Danda),
            nm("x"),
            kw(TokenKind::Vadati),
            kw(TokenKind::Danda),
            kw(TokenKind::Iti),
        ];
        let ast = parse_tokens(tokens).expect("should parse");

        match ast {
            ASTNode::KaryakramNode { shareera } => {
                assert_eq!(shareera.len(), 1);
                match &shareera[0] {
                    ASTNode::KramashahNode { item_name, body, .. } => {
                        assert_eq!(item_name, "x");
                        assert_eq!(body.len(), 2, "body should have 2 statements");
                    }
                    other => panic!("expected KramashahNode, got {:?}", other),
                }
            }
other => panic!("expected KaryakramNode, got {:?}", other),
         }
     }

    // Empty avali array literal avali[] parses to AvaliNode with 0 elements
    #[test]
    fn test_avali_literal_empty() {
        let tokens = vec![kw(TokenKind::Avali), kw(TokenKind::LBracket), kw(TokenKind::RBracket)];
        let ast = parse_tokens(tokens).expect("should parse");

        match ast {
            ASTNode::KaryakramNode { shareera } => {
                assert_eq!(shareera.len(), 1);
                assert!(
                    matches!(&shareera[0], ASTNode::AvaliNode { elements, .. } if elements.is_empty())
                );
            }
            other => panic!("expected KaryakramNode, got {:?}", other),
        }
    }

    // Array literal with 3 numeric elements parses correctly
    #[test]
    fn test_avali_literal_basic() {
        let tokens = vec![
            kw(TokenKind::Avali),
            kw(TokenKind::LBracket),
            kw(TokenKind::PurnaankLiteral(1)),
            kw(TokenKind::Unknown(',')),
            kw(TokenKind::PurnaankLiteral(2)),
            kw(TokenKind::Unknown(',')),
            kw(TokenKind::PurnaankLiteral(3)),
            kw(TokenKind::RBracket),
        ];
        let ast = parse_tokens(tokens).expect("should parse");

        match ast {
            ASTNode::KaryakramNode { shareera } => {
                assert_eq!(shareera.len(), 1);
                match &shareera[0] {
                    ASTNode::AvaliNode { elements, .. } => {
                        assert_eq!(elements.len(), 3);
                        assert!(
                            matches!(&elements[0], ASTNode::PurnaankLiteral { value, .. } if *value == 1)
                        );
                        assert!(
                            matches!(&elements[1], ASTNode::PurnaankLiteral { value, .. } if *value == 2)
                        );
                        assert!(
                            matches!(&elements[2], ASTNode::PurnaankLiteral { value, .. } if *value == 3)
                        );
                    }
                    other => panic!("expected AvaliNode, got {:?}", other),
                }
            }
            other => panic!("expected KaryakramNode, got {:?}", other),
        }
    }

    // Nested avali array literal parses correctly
    #[test]
    fn test_avali_nested() {
        let tokens = vec![
            kw(TokenKind::Avali),
            kw(TokenKind::LBracket),
            kw(TokenKind::PurnaankLiteral(1)),
            kw(TokenKind::Unknown(',')),
            kw(TokenKind::PurnaankLiteral(2)),
            kw(TokenKind::RBracket),
        ];
        let ast = parse_tokens(tokens).expect("should parse");

        match ast {
            ASTNode::KaryakramNode { shareera } => {
                assert_eq!(shareera.len(), 1);
                match &shareera[0] {
                    ASTNode::AvaliNode { elements, .. } => {
                        assert_eq!(elements.len(), 2);
                        assert!(
                            matches!(&elements[0], ASTNode::PurnaankLiteral { value, .. } if *value == 1)
                        );
                        assert!(
                            matches!(&elements[1], ASTNode::PurnaankLiteral { value, .. } if *value == 2)
                        );
                    }
                    other => panic!("expected AvaliNode, got {:?}", other),
                }
            }
            other => panic!("expected KaryakramNode, got {:?}", other),
        }
    }

    // Simple dravya def with 2 fields parses correctly
    #[test]
    fn test_parse_simple_dravya_def() {
        let tokens = vec![
            nm("manushya"),
            kw(TokenKind::Dravya),
            nm("naama"),
            nm("vaak"),
            nm("sankhya"),
            nm("sankhya"),
            kw(TokenKind::Danda),
        ];
        let ast = parse_tokens(tokens).expect("should parse");

        match ast {
            ASTNode::KaryakramNode { shareera } => {
                assert_eq!(shareera.len(), 1);
                match &shareera[0] {
                    ASTNode::DravyaDef { name, angas, .. } => {
                        assert_eq!(name, "manushya");
                        assert_eq!(angas.len(), 2);
                        assert_eq!(angas[0].name, "naama");
                        assert_eq!(angas[0].type_name, "vaak");
                        assert_eq!(angas[1].name, "sankhya");
                        assert_eq!(angas[1].type_name, "sankhya");
                    }
                    other => panic!("expected DravyaDef, got {:?}", other),
                }
            }
            other => panic!("expected KaryakramNode, got {:?}", other),
        }
    }

    // Empty dravya def parses correctly
    #[test]
    fn test_parse_empty_dravya_def() {
        let tokens = vec![
            nm("manushya"),
            kw(TokenKind::Dravya),
            kw(TokenKind::Danda),
        ];
        let ast = parse_tokens(tokens).expect("should parse");

        match ast {
            ASTNode::KaryakramNode { shareera } => {
                assert_eq!(shareera.len(), 1);
                match &shareera[0] {
                    ASTNode::DravyaDef { name, angas, .. } => {
                        assert_eq!(name, "manushya");
                        assert!(angas.is_empty());
                    }
                    other => panic!("expected DravyaDef, got {:?}", other),
                }
            }
            other => panic!("expected KaryakramNode, got {:?}", other),
        }
    }

    // Simple samavaya access x.naama parses correctly
    #[test]
    fn test_parse_samavaya_access() {
        let tokens = vec![
            nm("x"),
            kw(TokenKind::Dot),
            nm("naama"),
        ];
        let ast = parse_tokens(tokens).expect("should parse");

        match ast {
            ASTNode::KaryakramNode { shareera } => {
                assert_eq!(shareera.len(), 1);
                match &shareera[0] {
                    ASTNode::SamavayaNode { anga_name, target, .. } => {
                        assert_eq!(anga_name, "naama");
                        assert!(
                            matches!(target.as_ref(), ASTNode::Nama { base, .. } if base == "x")
                        );
                    }
                    other => panic!("expected SamavayaNode, got {:?}", other),
                }
            }
            other => panic!("expected KaryakramNode, got {:?}", other),
        }
    }

    // Chained samavaya access x.a.b parses correctly
    #[test]
    fn test_parse_chained_samavaya_access() {
        let tokens = vec![
            nm("x"),
            kw(TokenKind::Dot),
            nm("a"),
            kw(TokenKind::Dot),
            nm("b"),
        ];
        let ast = parse_tokens(tokens).expect("should parse");

        match ast {
            ASTNode::KaryakramNode { shareera } => {
                assert_eq!(shareera.len(), 1);
                match &shareera[0] {
                    ASTNode::SamavayaNode { anga_name, target, .. } => {
                        assert_eq!(anga_name, "b");
                        match target.as_ref() {
                            ASTNode::SamavayaNode { anga_name: inner_name, target: inner_target, .. } => {
                                assert_eq!(inner_name, "a");
                                assert!(
                                    matches!(inner_target.as_ref(), ASTNode::Nama { base, .. } if base == "x")
                                );
                            }
                            other => panic!("expected inner SamavayaNode, got {:?}", other),
                        }
                    }
                    other => panic!("expected SamavayaNode, got {:?}", other),
                }
            }
            other => panic!("expected KaryakramNode, got {:?}", other),
        }
    }

    // Dravya is registered in symbol table during parsing
    #[test]
    fn test_dravya_registered_in_symbol_table() {
        let tokens = vec![
            nm("manushya"),
            kw(TokenKind::Dravya),
            nm("naama"),
            nm("vaak"),
            kw(TokenKind::Danda),
        ];
        let mut parser = Parser::new(tokens);
        let _ = parser.parse();

        let sym = parser.symbols.lookup("manushya");
        assert!(sym.is_some(), "dravya should be registered in symbol table");
        assert!(matches!(sym.unwrap().kind, SymbolKind::Dravya));
    }

    // Odd number of trailing Naama tokens before Danda should error
    #[test]
    fn test_parse_dravya_odd_field_count_errors() {
        let tokens = vec![
            nm("manushya"),
            kw(TokenKind::Dravya),
            nm("naama"),
            nm("vaak"),
            nm("sankhya"),
            kw(TokenKind::Danda),
        ];
        let result = parse_tokens(tokens);
        assert!(result.is_err(), "odd number of field tokens before Danda should error");
    }

    // Nirmana with values parses correctly
    #[test]
    fn test_parse_nirmana_with_values() {
        let tokens = vec![
            nm("manushya"),
            kw(TokenKind::Nirmana),
            Token {
                kind: TokenKind::VaakLiteral("raam".to_string()),
                span: span(),
            },
            kw(TokenKind::PurnaankLiteral(25)),
            kw(TokenKind::PurnaankLiteral(180)),
            kw(TokenKind::Danda),
        ];
        let ast = parse_tokens(tokens).expect("should parse");

        match ast {
            ASTNode::KaryakramNode { shareera } => {
                assert_eq!(shareera.len(), 1);
                match &shareera[0] {
                    ASTNode::NirmanaNode { dravya_name, values, .. } => {
                        assert_eq!(dravya_name, "manushya");
                        assert_eq!(values.len(), 3);
                        assert!(
                            matches!(&values[0], ASTNode::VaakLiteral { value, .. } if value == "raam")
                        );
                        assert!(
                            matches!(&values[1], ASTNode::PurnaankLiteral { value, .. } if *value == 25)
                        );
                        assert!(
                            matches!(&values[2], ASTNode::PurnaankLiteral { value, .. } if *value == 180)
                        );
                    }
                    other => panic!("expected NirmanaNode, got {:?}", other),
                }
            }
            other => panic!("expected KaryakramNode, got {:?}", other),
        }
    }

    // Nirmana without trailing Danda should error
    #[test]
    fn test_parse_nirmana_missing_danda() {
        let tokens = vec![
            nm("manushya"),
            kw(TokenKind::Nirmana),
            kw(TokenKind::PurnaankLiteral(1)),
            kw(TokenKind::PurnaankLiteral(2)),
        ];
        let result = parse_tokens(tokens);
        assert!(result.is_err(), "missing Danda before EOF should error");
    }

    // Nirmana with zero values parses correctly at parser level
    #[test]
    fn test_parse_nirmana_zero_values() {
        let tokens = vec![
            nm("manushya"),
            kw(TokenKind::Nirmana),
            kw(TokenKind::Danda),
        ];
        let ast = parse_tokens(tokens).expect("should parse");

        match ast {
            ASTNode::KaryakramNode { shareera } => {
                assert_eq!(shareera.len(), 1);
                match &shareera[0] {
                    ASTNode::NirmanaNode { dravya_name, values, .. } => {
                        assert_eq!(dravya_name, "manushya");
                        assert!(values.is_empty());
                    }
                    other => panic!("expected NirmanaNode, got {:?}", other),
                }
            }
            other => panic!("expected KaryakramNode, got {:?}", other),
        }
    }

    // Dhatu with phalam return type parses correctly
    #[test]
    fn test_parse_dhatu_with_phalam_return_type() {
        let tokens = vec![
            nm("bhojan-dhatu"),
            nm("param1"),
            kw(TokenKind::Phalam),
            nm("sankhya"),
            nm("vaak"),
            nm("karoti"),
            kw(TokenKind::Danda),
            nm("param1"),
            kw(TokenKind::Vadati),
            kw(TokenKind::Danda),
            kw(TokenKind::Iti),
            kw(TokenKind::Danda),
        ];
        let ast = parse_tokens(tokens).expect("should parse");

        match ast {
            ASTNode::KaryakramNode { shareera } => {
                assert_eq!(shareera.len(), 1);
                match &shareera[0] {
                    ASTNode::DhatuDef { return_type, name, params, body, .. } => {
                        assert_eq!(name, "bhojan-dhatu");
                        assert_eq!(params.len(), 1);
                        assert!(return_type.is_some());
                        match return_type.as_ref().unwrap().as_ref() {
                            ASTNode::PhalamType { success_type, error_type, .. } => {
                                assert_eq!(success_type, "sankhya");
                                assert_eq!(error_type, "vaak");
                            }
                            other => panic!("expected PhalamType, got {:?}", other),
                        }
                        assert_eq!(body.len(), 1);
                    }
                    other => panic!("expected DhatuDef, got {:?}", other),
                }
            }
            other => panic!("expected KaryakramNode, got {:?}", other),
        }
    }

    // Arogya expression parses correctly
    #[test]
    fn test_parse_arogya_expression() {
        let tokens = vec![
            kw(TokenKind::Arogya),
            kw(TokenKind::PurnaankLiteral(42)),
            kw(TokenKind::Danda),
        ];
        let ast = parse_tokens(tokens).expect("should parse");

        match ast {
            ASTNode::KaryakramNode { shareera } => {
                assert_eq!(shareera.len(), 1);
                match &shareera[0] {
                    ASTNode::ArogyaNode { value, .. } => {
                        assert!(
                            matches!(value.as_ref(), ASTNode::PurnaankLiteral { value, .. } if *value == 42)
                        );
                    }
                    other => panic!("expected ArogyaNode, got {:?}", other),
                }
            }
            other => panic!("expected KaryakramNode, got {:?}", other),
        }
    }

    // Dosha expression parses correctly
    #[test]
    fn test_parse_dosha_expression() {
        let tokens = vec![
            kw(TokenKind::Dosha),
            Token {
                kind: TokenKind::VaakLiteral("kuch galat hua".to_string()),
                span: span(),
            },
            kw(TokenKind::Danda),
        ];
        let ast = parse_tokens(tokens).expect("should parse");

        match ast {
            ASTNode::KaryakramNode { shareera } => {
                assert_eq!(shareera.len(), 1);
                match &shareera[0] {
                    ASTNode::DoshaNode { value, .. } => {
                        assert!(
                            matches!(value.as_ref(), ASTNode::VaakLiteral { value, .. } if value == "kuch galat hua")
                        );
                    }
                    other => panic!("expected DoshaNode, got {:?}", other),
                }
            }
            other => panic!("expected KaryakramNode, got {:?}", other),
        }
    }

    // Full Nidana block parses correctly with both arms
    #[test]
    fn test_parse_nidana_block() {
        let tokens = vec![
            kw(TokenKind::Nidana),
            nm("result"),
            kw(TokenKind::Arogya),
            nm("val"),
            nm("val"),
            kw(TokenKind::Vadati),
            kw(TokenKind::Danda),
            kw(TokenKind::Dosha),
            nm("err"),
            nm("err"),
            kw(TokenKind::Vadati),
            kw(TokenKind::Danda),
            kw(TokenKind::Iti),
            kw(TokenKind::Danda),
        ];
        let ast = parse_tokens(tokens).expect("should parse");

        match ast {
            ASTNode::KaryakramNode { shareera } => {
                assert_eq!(shareera.len(), 1);
                match &shareera[0] {
                    ASTNode::NidanaNode { target, arogya_bind, arogya_body, dosha_bind, dosha_body, .. } => {
                        assert!(
                            matches!(target.as_ref(), ASTNode::Nama { base, .. } if base == "result")
                        );
                        assert_eq!(arogya_bind, "val");
                        assert_eq!(dosha_bind, "err");
                        assert_eq!(arogya_body.len(), 1);
                        assert_eq!(dosha_body.len(), 1);
                    }
                    other => panic!("expected NidanaNode, got {:?}", other),
                }
            }
            other => panic!("expected KaryakramNode, got {:?}", other),
        }
    }

    // Generic dravya with one type param parses correctly
    #[test]
    fn test_parse_generic_dravya_one_param() {
        let tokens = vec![
            kw(TokenKind::Sāmānya),
            nm("T"),
            kw(TokenKind::Dravya),
            nm("Peti"),
            nm("naama"),
            nm("string"),
            kw(TokenKind::Danda),
        ];
        let ast = parse_tokens(tokens).expect("should parse");

        match ast {
            ASTNode::KaryakramNode { shareera } => {
                assert_eq!(shareera.len(), 1);
                match &shareera[0] {
                    ASTNode::DravyaDef {
                        name, generic_params, angas: _, ..
                    } => {
                        assert_eq!(name, "Peti");
                        assert_eq!(generic_params, &vec!["T".to_string()]);
                    }
                    other => panic!("expected DravyaDef, got {:?}", other),
                }
            }
            other => panic!("expected KaryakramNode, got {:?}", other),
        }
    }

    // Generic dravya with two type params parses correctly
    #[test]
    fn test_parse_generic_dravya_two_params() {
        let tokens = vec![
            kw(TokenKind::Sāmānya),
            nm("T"),
            nm("U"),
            kw(TokenKind::Dravya),
            nm("Peti"),
            nm("naama"),
            nm("string"),
            kw(TokenKind::Danda),
        ];
        let ast = parse_tokens(tokens).expect("should parse");

        match ast {
            ASTNode::KaryakramNode { shareera } => {
                assert_eq!(shareera.len(), 1);
                match &shareera[0] {
                    ASTNode::DravyaDef {
                        name, generic_params, angas: _, ..
                    } => {
                        assert_eq!(name, "Peti");
                        assert_eq!(generic_params, &vec!["T".to_string(), "U".to_string()]);
                    }
                    other => panic!("expected DravyaDef, got {:?}", other),
                }
            }
            other => panic!("expected KaryakramNode, got {:?}", other),
        }
    }

    // Generic dhātu with one type param parses correctly
    #[test]
    fn test_parse_generic_dhatu_one_param() {
        let tokens = vec![
            kw(TokenKind::Sāmānya),
            nm("T"),
            kw(TokenKind::Dhātu),
            nm("pratirūpa"),
            nm("n"),
            nm("karoti"),
            kw(TokenKind::Danda),
            nm("n"),
            kw(TokenKind::Iti),
            kw(TokenKind::Danda),
        ];
        let ast = parse_tokens(tokens).expect("should parse");

        match ast {
            ASTNode::KaryakramNode { shareera } => {
                assert_eq!(shareera.len(), 1);
                match &shareera[0] {
                    ASTNode::DhatuDef {
                        name, generic_params, ..
                    } => {
                        assert_eq!(name, "pratirūpa");
                        assert_eq!(generic_params, &vec!["T".to_string()]);
                    }
                    other => panic!("expected DhatuDef, got {:?}", other),
                }
            }
            other => panic!("expected KaryakramNode, got {:?}", other),
        }
    }

    // Non-generic dravya still parses correctly with empty generic_params (regression)
    #[test]
    fn test_parse_non_generic_dravya_regression() {
        let tokens = vec![
            nm("Manushya"),
            kw(TokenKind::Dravya),
            nm("naama"),
            nm("string"),
            kw(TokenKind::Danda),
        ];
        let ast = parse_tokens(tokens).expect("should parse");

        match ast {
            ASTNode::KaryakramNode { shareera } => {
                assert_eq!(shareera.len(), 1);
                match &shareera[0] {
                    ASTNode::DravyaDef {
                        name, generic_params, ..
                    } => {
                        assert_eq!(name, "Manushya");
                        assert!(generic_params.is_empty());
                    }
                    other => panic!("expected DravyaDef, got {:?}", other),
                }
            }
            other => panic!("expected KaryakramNode, got {:?}", other),
        }
    }

    // Non-generic dhātu still parses correctly with empty generic_params (regression)
    #[test]
    fn test_parse_non_generic_dhatu_regression() {
        let tokens = vec![
            kw(TokenKind::Dhātu),
            nm("pratirūpa"),
            nm("n"),
            nm("karoti"),
            kw(TokenKind::Danda),
            nm("n"),
            kw(TokenKind::Iti),
            kw(TokenKind::Danda),
        ];
        let ast = parse_tokens(tokens).expect("should parse");

        match ast {
            ASTNode::KaryakramNode { shareera } => {
                assert_eq!(shareera.len(), 1);
                match &shareera[0] {
                    ASTNode::DhatuDef {
                        name, generic_params, ..
                    } => {
                        assert_eq!(name, "pratirūpa");
                        assert!(generic_params.is_empty());
                    }
                    other => panic!("expected DhatuDef, got {:?}", other),
                }
            }
            other => panic!("expected KaryakramNode, got {:?}", other),
        }
    }

    // Malformed sāmānya not followed by dravya or dhātu produces expected error
    #[test]
    fn test_parse_sāmānya_not_followed_by_dravya_or_dhatu() {
        let tokens = vec![
            kw(TokenKind::Sāmānya),
            nm("T"),
            nm("foo"),
            kw(TokenKind::Danda),
        ];
        let result = parse_tokens(tokens);
        assert!(result.is_err(), "should fail when sāmānya not followed by dravya or dhātu");
    }

    // Dhara with explicit type parses correctly (regression)
    #[test]
    fn test_parse_dhara_with_explicit_type() {
        let tokens = vec![
            kw(TokenKind::Dharā),
            nm("x"),
            nm("i64"),
            kw(TokenKind::Equals),
            kw(TokenKind::PurnaankLiteral(5)),
            kw(TokenKind::Danda),
        ];
        let ast = parse_tokens(tokens).expect("should parse");

        match ast {
            ASTNode::KaryakramNode { shareera } => {
                assert_eq!(shareera.len(), 1);
                match &shareera[0] {
                    ASTNode::DharaNode { naamas, type_name, mulya, is_mutable, .. } => {
                        assert_eq!(naamas, &vec!["x".to_string()]);
                        assert_eq!(type_name, &Some("i64".to_string()));
                        assert!(!is_mutable);
                        assert!(matches!(mulya.as_ref(), ASTNode::PurnaankLiteral { value, .. } if *value == 5));
                    }
                    other => panic!("expected DharaNode, got {:?}", other),
                }
            }
            other => panic!("expected KaryakramNode, got {:?}", other),
        }
    }

    // Dhara without explicit type parses correctly and type_name is None (Anumana / Type Inference)
    #[test]
    fn test_parse_dhara_without_type() {
        let tokens = vec![
            kw(TokenKind::Dharā),
            nm("x"),
            kw(TokenKind::Equals),
            kw(TokenKind::PurnaankLiteral(5)),
            kw(TokenKind::Danda),
        ];
        let ast = parse_tokens(tokens).expect("should parse");

        match ast {
            ASTNode::KaryakramNode { shareera } => {
                assert_eq!(shareera.len(), 1);
                match &shareera[0] {
                    ASTNode::DharaNode { naamas, type_name, mulya, is_mutable, .. } => {
                        assert_eq!(naamas, &vec!["x".to_string()]);
                        assert!(type_name.is_none(), "type_name should be None for inferred type");
                        assert!(!is_mutable);
                        assert!(matches!(mulya.as_ref(), ASTNode::PurnaankLiteral { value, .. } if *value == 5));
                    }
                    other => panic!("expected DharaNode, got {:?}", other),
                }
            }
            other => panic!("expected KaryakramNode, got {:?}", other),
        }
    }

    // Dhātu without explicit return type parses correctly and return_type is None (Anumana)
    #[test]
    fn test_parse_dhatu_without_phalam_return_type() {
        let tokens = vec![
            nm("square-dhatu"),
            nm("n"),
            nm("karoti"),
            kw(TokenKind::Danda),
            nm("n"),
            kw(TokenKind::Yoga),
            nm("n"),
            kw(TokenKind::Iti),
            kw(TokenKind::Danda),
        ];
        let ast = parse_tokens(tokens).expect("should parse");

        match ast {
            ASTNode::KaryakramNode { shareera } => {
                assert_eq!(shareera.len(), 1);
                match &shareera[0] {
                    ASTNode::DhatuDef { name, return_type, .. } => {
                        assert_eq!(name, "square-dhatu");
                        assert!(return_type.is_none(), "return_type should be None when phalam is omitted");
                    }
                    other => panic!("expected DhatuDef, got {:?}", other),
                }
            }
             other => panic!("expected KaryakramNode, got {:?}", other),
         }
     }

    // --- CONCURRENCY PARSER TESTS ---

    // samyoga { ... । } as a bare statement parses to SamyogaNode
    #[test]
    fn test_parse_samyoga_statement() {
        let tokens = vec![
            kw(TokenKind::Samyoga),
            kw(TokenKind::LBrace),
            nm("x"),
            kw(TokenKind::Vadati),
            kw(TokenKind::Danda),
            kw(TokenKind::RBrace),
            kw(TokenKind::Danda),
        ];
        let ast = parse_tokens(tokens).expect("should parse samyoga statement");

        match ast {
            ASTNode::KaryakramNode { shareera } => {
                assert_eq!(shareera.len(), 1);
                match &shareera[0] {
                    ASTNode::SamyogaNode { body, .. } => {
                        assert_eq!(body.len(), 1);
                    }
                    other => panic!("expected SamyogaNode, got {:?}", other),
                }
            }
            other => panic!("expected KaryakramNode, got {:?}", other),
        }
    }

    // dhara h = samyoga { ... । } parses DharaNode with SamyogaNode as mulya
    #[test]
    fn test_parse_samyoga_as_dhara_initializer() {
        let tokens = vec![
            kw(TokenKind::Dharā),
            nm("h"),
            kw(TokenKind::Equals),
            kw(TokenKind::Samyoga),
            kw(TokenKind::LBrace),
            nm("x"),
            kw(TokenKind::Vadati),
            kw(TokenKind::Danda),
            kw(TokenKind::RBrace),
            kw(TokenKind::Danda),
        ];
        let ast = parse_tokens(tokens).expect("should parse samyoga as dhara initializer");

        match ast {
            ASTNode::KaryakramNode { shareera } => {
                assert_eq!(shareera.len(), 1);
                match &shareera[0] {
                    ASTNode::DharaNode { naamas, mulya, .. } => {
                        assert_eq!(naamas, &vec!["h".to_string()]);
                        match mulya.as_ref() {
                            ASTNode::SamyogaNode { body, .. } => {
                                assert_eq!(body.len(), 1);
                            }
                            other => panic!("expected SamyogaNode in mulya, got {:?}", other),
                        }
                    }
                    other => panic!("expected DharaNode, got {:?}", other),
                }
            }
            other => panic!("expected KaryakramNode, got {:?}", other),
        }
    }

    // prapti h as an expression parses to PraptiNode
    #[test]
    fn test_parse_prapti_expression() {
        let tokens = vec![
            kw(TokenKind::Dharā),
            nm("result"),
            kw(TokenKind::Equals),
            nm("h"),
            kw(TokenKind::Prapti),
            kw(TokenKind::Danda),
        ];
        let ast = parse_tokens(tokens).expect("should parse prapti expression");

        match ast {
            ASTNode::KaryakramNode { shareera } => {
                assert_eq!(shareera.len(), 1);
                match &shareera[0] {
                    ASTNode::DharaNode { mulya, .. } => match mulya.as_ref() {
                        ASTNode::PraptiNode { handle, .. } => {
                            assert!(
                                matches!(handle.as_ref(), ASTNode::Nama { base, .. } if base == "h")
                            );
                        }
                        other => panic!("expected PraptiNode, got {:?}", other),
                    },
                    other => panic!("expected DharaNode, got {:?}", other),
                }
            }
            other => panic!("expected KaryakramNode, got {:?}", other),
        }
    }

    // duta banaa parses as an expression usable in dhara [a, b] = ...
    #[test]
    fn test_parse_duta_banaa_expression() {
        let tokens = vec![
            kw(TokenKind::Dharā),
            kw(TokenKind::LBracket),
            nm("bhejaka"),
            kw(TokenKind::Unknown(',')),
            nm("grahaka"),
            kw(TokenKind::RBracket),
            kw(TokenKind::Equals),
            kw(TokenKind::Duta),
            kw(TokenKind::Banaa),
            kw(TokenKind::Danda),
        ];
        let ast = parse_tokens(tokens).expect("should parse duta banaa binding");

        match ast {
            ASTNode::KaryakramNode { shareera } => {
                assert_eq!(shareera.len(), 1);
                match &shareera[0] {
                    ASTNode::DharaNode { naamas, mulya, .. } => {
                        assert_eq!(naamas, &vec!["bhejaka".to_string(), "grahaka".to_string()]);
                        match mulya.as_ref() {
                            ASTNode::DutaBanaaNode { .. } => {}
                            other => panic!("expected DutaBanaaNode, got {:?}", other),
                        }
                    }
                    other => panic!("expected DharaNode, got {:?}", other),
                }
            }
            other => panic!("expected KaryakramNode, got {:?}", other),
        }
    }

    // bhejaka bhej sandesha <expr> । parses as DutaBhejNode statement
    #[test]
    fn test_parse_duta_bhej_statement() {
        let tokens = vec![
            nm("bhejaka"),
            kw(TokenKind::Bhej),
            kw(TokenKind::Sandesha),
            nm("msg"),
            kw(TokenKind::Danda),
        ];
        let ast = parse_tokens(tokens).expect("should parse bhej statement");

        match ast {
            ASTNode::KaryakramNode { shareera } => {
                assert_eq!(shareera.len(), 1);
                match &shareera[0] {
                    ASTNode::DutaBhejNode { sender, message, .. } => {
                        assert!(
                            matches!(sender.as_ref(), ASTNode::Nama { base, .. } if base == "bhejaka")
                        );
                        assert!(
                            matches!(message.as_ref(), ASTNode::Nama { base, .. } if base == "msg")
                        );
                    }
                    other => panic!("expected DutaBhejNode, got {:?}", other),
                }
            }
            other => panic!("expected KaryakramNode, got {:?}", other),
        }
    }

    // grahaka grahan karo parses as DutaGrahanNode expression
    #[test]
    fn test_parse_duta_grahan_expression() {
        let tokens = vec![
            kw(TokenKind::Dharā),
            nm("msg"),
            kw(TokenKind::Equals),
            nm("grahaka"),
            kw(TokenKind::Grahan),
            kw(TokenKind::Karo),
            kw(TokenKind::Danda),
        ];
        let ast = parse_tokens(tokens).expect("should parse grahan karo expression");

        match ast {
            ASTNode::KaryakramNode { shareera } => {
                assert_eq!(shareera.len(), 1);
                match &shareera[0] {
                    ASTNode::DharaNode { mulya, .. } => match mulya.as_ref() {
                        ASTNode::DutaGrahanNode { receiver, .. } => {
                            assert!(
                                matches!(receiver.as_ref(), ASTNode::Nama { base, .. } if base == "grahaka")
                            );
                        }
                        other => panic!("expected DutaGrahanNode, got {:?}", other),
                    },
                    other => panic!("expected DharaNode, got {:?}", other),
                }
            }
            other => panic!("expected KaryakramNode, got {:?}", other),
        }
    }

    // manas x { ... । } parses as ManasNode statement
    #[test]
    fn test_parse_manas_statement() {
        let tokens = vec![
            kw(TokenKind::Manas),
            nm("lock"),
            kw(TokenKind::LBrace),
            nm("x"),
            kw(TokenKind::Vadati),
            kw(TokenKind::Danda),
            kw(TokenKind::RBrace),
            kw(TokenKind::Danda),
        ];
        let ast = parse_tokens(tokens).expect("should parse manas statement");

        match ast {
            ASTNode::KaryakramNode { shareera } => {
                assert_eq!(shareera.len(), 1);
                match &shareera[0] {
                    ASTNode::ManasNode { target, body, .. } => {
                        assert!(
                            matches!(target.as_ref(), ASTNode::Nama { base, .. } if base == "lock")
                        );
                        assert_eq!(body.len(), 1);
                    }
                    other => panic!("expected ManasNode, got {:?}", other),
                }
            }
            other => panic!("expected KaryakramNode, got {:?}", other),
        }
    }

    // Combined test: spawn thread, join, and use channel between two samyoga blocks
    #[test]
    fn test_parse_concurrency_combined() {
        let tokens = vec![
            // dhara [bhejaka, grahaka] = duta banaa ।
            kw(TokenKind::Dharā),
            kw(TokenKind::LBracket),
            nm("bhejaka"),
            kw(TokenKind::Unknown(',')),
            nm("grahaka"),
            kw(TokenKind::RBracket),
            kw(TokenKind::Equals),
            kw(TokenKind::Duta),
            kw(TokenKind::Banaa),
            kw(TokenKind::Danda),
            // dhara h = samyoga { bhejaka bhej sandesha "hello"। }
            kw(TokenKind::Dharā),
            nm("h"),
            kw(TokenKind::Equals),
            kw(TokenKind::Samyoga),
            kw(TokenKind::LBrace),
            nm("bhejaka"),
            kw(TokenKind::Bhej),
            kw(TokenKind::Sandesha),
            kw(TokenKind::VaakLiteral("hello".to_string())),
            kw(TokenKind::Danda),
            kw(TokenKind::RBrace),
            kw(TokenKind::Danda),
            // dhara result = prapti h ।
            kw(TokenKind::Dharā),
            nm("result"),
            kw(TokenKind::Equals),
            nm("h"),
            kw(TokenKind::Prapti),
            kw(TokenKind::Danda),
            // grahaka grahan karo ।
            nm("grahaka"),
            kw(TokenKind::Grahan),
            kw(TokenKind::Karo),
            kw(TokenKind::Danda),
            // manas lock { dhara x = 1 । }
            kw(TokenKind::Manas),
            nm("lock"),
            kw(TokenKind::LBrace),
            kw(TokenKind::Dharā),
            nm("x"),
            kw(TokenKind::Equals),
            kw(TokenKind::PurnaankLiteral(1)),
            kw(TokenKind::Danda),
            kw(TokenKind::RBrace),
            kw(TokenKind::Danda),
        ];
        let ast = parse_tokens(tokens).expect("should parse combined concurrency snippet");

        match ast {
            ASTNode::KaryakramNode { shareera } => {
                assert_eq!(shareera.len(), 5);
                // 0: DharaNode [bhejaka, grahaka] = DutaBanaaNode
                match &shareera[0] {
                    ASTNode::DharaNode { mulya, .. } => {
                        assert!(matches!(mulya.as_ref(), ASTNode::DutaBanaaNode { .. }));
                    }
                    other => panic!("stmt 0: expected DharaNode, got {:?}", other),
                }
                // 1: DharaNode h = SamyogaNode
                match &shareera[1] {
                    ASTNode::DharaNode { mulya, .. } => {
                        assert!(matches!(mulya.as_ref(), ASTNode::SamyogaNode { .. }));
                    }
                    other => panic!("stmt 1: expected DharaNode, got {:?}", other),
                }
                // 2: DharaNode result = PraptiNode
                match &shareera[2] {
                    ASTNode::DharaNode { mulya, .. } => {
                        assert!(matches!(mulya.as_ref(), ASTNode::PraptiNode { .. }));
                    }
                    other => panic!("stmt 2: expected DharaNode, got {:?}", other),
                }
                // 3: DutaGrahanNode (standalone expression statement)
                match &shareera[3] {
                    ASTNode::DutaGrahanNode { .. } => {}
                    other => panic!("stmt 3: expected DutaGrahanNode, got {:?}", other),
                }
                // 4: ManasNode
                match &shareera[4] {
                    ASTNode::ManasNode { body, .. } => {
                        assert_eq!(body.len(), 1);
                    }
                    other => panic!("stmt 4: expected ManasNode, got {:?}", other),
                }
            }
            other => panic!("expected KaryakramNode, got {:?}", other),
        }
    }
}

// --- PARINAMA PIPELINE PARSER TESTS ---

#[cfg(test)]
mod parinama_tests {
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

    // x pariṇāma [f, g, h] parses to ParinamaNode with all three dhatus present
    #[test]
    fn test_parse_parinama_three_dhatus() {
        let tokens = vec![
            nm("x"),
            kw(TokenKind::Parinama),
            kw(TokenKind::LBracket),
            nm("f"),
            kw(TokenKind::Unknown(',')),
            nm("g"),
            kw(TokenKind::Unknown(',')),
            nm("h"),
            kw(TokenKind::RBracket),
        ];
        let ast = parse_tokens(tokens).expect("should parse parinama with three dhatus");

        match ast {
            ASTNode::KaryakramNode { shareera } => {
                assert_eq!(shareera.len(), 1);
                match &shareera[0] {
                    ASTNode::ParinamaNode { mulyam, dhatus, .. } => {
                        assert!(matches!(mulyam.as_ref(), ASTNode::Nama { base, .. } if base == "x"));
                        assert_eq!(dhatus, &vec!["f".to_string(), "g".to_string(), "h".to_string()]);
                    }
                    other => panic!("expected ParinamaNode, got {:?}", other),
                }
            }
            other => panic!("expected KaryakramNode, got {:?}", other),
        }
    }

    // x pariṇāma [f] parses to ParinamaNode with a single dhatu
    #[test]
    fn test_parse_parinama_single_dhatu() {
        let tokens = vec![
            nm("x"),
            kw(TokenKind::Parinama),
            kw(TokenKind::LBracket),
            nm("f"),
            kw(TokenKind::RBracket),
        ];
        let ast = parse_tokens(tokens).expect("should parse parinama with single dhatu");

        match ast {
            ASTNode::KaryakramNode { shareera } => {
                assert_eq!(shareera.len(), 1);
                match &shareera[0] {
                    ASTNode::ParinamaNode { mulyam, dhatus, .. } => {
                        assert!(matches!(mulyam.as_ref(), ASTNode::Nama { base, .. } if base == "x"));
                        assert_eq!(dhatus, &vec!["f".to_string()]);
                    }
                    other => panic!("expected ParinamaNode, got {:?}", other),
                }
            }
            other => panic!("expected KaryakramNode, got {:?}", other),
        }
    }

    // x pariṇāma [] parses to ParinamaNode with empty dhatus list (no panic)
    #[test]
    fn test_parse_parinama_empty_list() {
        let tokens = vec![
            nm("x"),
            kw(TokenKind::Parinama),
            kw(TokenKind::LBracket),
            kw(TokenKind::RBracket),
        ];
        let ast = parse_tokens(tokens).expect("should parse parinama with empty list without panic");

        match ast {
            ASTNode::KaryakramNode { shareera } => {
                assert_eq!(shareera.len(), 1);
                match &shareera[0] {
                    ASTNode::ParinamaNode { mulyam, dhatus, .. } => {
                        assert!(matches!(mulyam.as_ref(), ASTNode::Nama { base, .. } if base == "x"));
                        assert!(dhatus.is_empty(), "empty dhatu list should parse to empty vec");
                    }
                    other => panic!("expected ParinamaNode, got {:?}", other),
                }
            }
            other => panic!("expected KaryakramNode, got {:?}", other),
        }
    }

    // ParinamaNode is usable as the initializer of a dhara binding
    #[test]
    fn test_parse_parinama_as_dhara_initializer() {
        let tokens = vec![
            kw(TokenKind::Dharā),
            nm("result"),
            kw(TokenKind::Equals),
            nm("x"),
            kw(TokenKind::Parinama),
            kw(TokenKind::LBracket),
            nm("f"),
            kw(TokenKind::Unknown(',')),
            nm("g"),
            kw(TokenKind::RBracket),
            kw(TokenKind::Danda),
        ];
        let ast = parse_tokens(tokens).expect("should parse parinama as dhara initializer");

        match ast {
            ASTNode::KaryakramNode { shareera } => {
                assert_eq!(shareera.len(), 1);
                match &shareera[0] {
                    ASTNode::DharaNode { mulya, .. } => match mulya.as_ref() {
                        ASTNode::ParinamaNode { dhatus, .. } => {
                            assert_eq!(dhatus, &vec!["f".to_string(), "g".to_string()]);
                         }
                         other => panic!("expected ParinamaNode in mulya, got {:?}", other),
                     },
                     other => panic!("expected DharaNode, got {:?}", other),
                 }
             }
             other => panic!("expected KaryakramNode, got {:?}", other),
         }
     }
}

// --- PARIṬṢĀ (TESTING) PARSER TESTS ---

#[cfg(test)]
mod parikshaa_tests {
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

    // parikshaa test_name { nigamana x sama y। } parses correctly
    #[test]
    fn test_parse_plain_parikshaa_block() {
        let tokens = vec![
            kw(TokenKind::Parikshaa),
            nm("test_true"),
            kw(TokenKind::LBrace),
            kw(TokenKind::Nigamana),
            nm("x"),
            kw(TokenKind::Sama),
            nm("y"),
            kw(TokenKind::Danda),
            kw(TokenKind::RBrace),
            kw(TokenKind::Danda),
        ];
        let ast = parse_tokens(tokens).expect("should parse plain parikshaa block");

        match ast {
            ASTNode::KaryakramNode { shareera } => {
                assert_eq!(shareera.len(), 1);
                match &shareera[0] {
                    ASTNode::ParikshaaNode { name, body, is_tarka, .. } => {
                        assert_eq!(name, "test_true");
                        assert!(!is_tarka);
                        assert_eq!(body.len(), 1);
                    }
                    other => panic!("expected ParikshaaNode, got {:?}", other),
                }
            }
            other => panic!("expected KaryakramNode, got {:?}", other),
        }
    }

    // tarka parikshaa test_name { ... } parses correctly with is_tarka=true
    #[test]
    fn test_parse_tarka_parikshaa_block() {
        let tokens = vec![
            kw(TokenKind::Tarka),
            kw(TokenKind::Parikshaa),
            nm("should_panic"),
            kw(TokenKind::LBrace),
            kw(TokenKind::Nigamana),
            nm("x"),
            kw(TokenKind::Sama),
            nm("y"),
            kw(TokenKind::Danda),
            kw(TokenKind::RBrace),
            kw(TokenKind::Danda),
        ];
        let ast = parse_tokens(tokens).expect("should parse tarka parikshaa block");

        match ast {
            ASTNode::KaryakramNode { shareera } => {
                assert_eq!(shareera.len(), 1);
                match &shareera[0] {
                    ASTNode::ParikshaaNode { name, body, is_tarka, .. } => {
                        assert_eq!(name, "should_panic");
                        assert!(is_tarka);
                        assert_eq!(body.len(), 1);
                    }
                    other => panic!("expected ParikshaaNode, got {:?}", other),
                }
            }
            other => panic!("expected KaryakramNode, got {:?}", other),
        }
    }

    // nigamana x sama y। parses as NigamanaNode
    #[test]
    fn test_parse_nigamana_statement() {
        let tokens = vec![
            kw(TokenKind::Nigamana),
            nm("x"),
            kw(TokenKind::Sama),
            nm("y"),
            kw(TokenKind::Danda),
        ];
        let ast = parse_tokens(tokens).expect("should parse nigamana statement");

        match ast {
            ASTNode::KaryakramNode { shareera } => {
                assert_eq!(shareera.len(), 1);
                match &shareera[0] {
                    ASTNode::NigamanaNode { .. } => {}
                    other => panic!("expected NigamanaNode, got {:?}", other),
                }
            }
            other => panic!("expected KaryakramNode, got {:?}", other),
        }
    }

    // sadrishya-nigamana x y। parses as SadrishyaNigamanaNode
    #[test]
    fn test_parse_sadrishya_nigamana_statement() {
        let tokens = vec![
            kw(TokenKind::SadrishyaNigamana),
            nm("x"),
            nm("y"),
            kw(TokenKind::Danda),
        ];
        let ast = parse_tokens(tokens).expect("should parse sadrishya-nigamana statement");

        match ast {
            ASTNode::KaryakramNode { shareera } => {
                assert_eq!(shareera.len(), 1);
                match &shareera[0] {
                    ASTNode::SadrishyaNigamanaNode { .. } => {}
                    other => panic!("expected SadrishyaNigamanaNode, got {:?}", other),
                }
            }
            other => panic!("expected KaryakramNode, got {:?}", other),
        }
    }

    // asadrishya-nigamana x y। parses as AsadrishyaNigamanaNode
    #[test]
    fn test_parse_asadrishya_nigamana_statement() {
        let tokens = vec![
            kw(TokenKind::AsadrishyaNigamana),
            nm("x"),
            nm("y"),
            kw(TokenKind::Danda),
        ];
        let ast = parse_tokens(tokens).expect("should parse asadrishya-nigamana statement");

        match ast {
            ASTNode::KaryakramNode { shareera } => {
                assert_eq!(shareera.len(), 1);
                match &shareera[0] {
                    ASTNode::AsadrishyaNigamanaNode { .. } => {}
                    other => panic!("expected AsadrishyaNigamanaNode, got {:?}", other),
                }
            }
            other => panic!("expected KaryakramNode, got {:?}", other),
        }
    }

    // nigamana with 0 args errors (D083)
    #[test]
    fn test_parse_nigamana_wrong_arg_count_d083() {
        let tokens = vec![
            kw(TokenKind::Nigamana),
            kw(TokenKind::Danda),
        ];
        let result = parse_tokens(tokens);
        assert!(result.is_err(), "nigamana with 0 args should error");
    }

    // sadrishya-nigamana with 1 arg errors (D083)
    #[test]
    fn test_parse_sadrishya_nigamana_wrong_arg_count_d083() {
        let tokens = vec![
            kw(TokenKind::SadrishyaNigamana),
            nm("x"),
            kw(TokenKind::Danda),
        ];
        let result = parse_tokens(tokens);
        assert!(result.is_err(), "sadrishya-nigamana with 1 arg should error");
    }

    // asadrishya-nigamana with 3 args errors (D083)
    #[test]
    fn test_parse_asadrishya_nigamana_wrong_arg_count_d083() {
        let tokens = vec![
            kw(TokenKind::AsadrishyaNigamana),
            nm("x"),
            nm("y"),
            nm("z"),
            kw(TokenKind::Danda),
        ];
        let result = parse_tokens(tokens);
        assert!(result.is_err(), "asadrishya-nigamana with 3 args should error");
    }

    // tarka without parikshaa errors (D084)
    #[test]
    fn test_parse_tarka_without_parikshaa_d084() {
        let tokens = vec![
            kw(TokenKind::Tarka),
            nm("foo"),
            kw(TokenKind::Danda),
        ];
        let result = parse_tokens(tokens);
        assert!(result.is_err(), "tarka without parikshaa should error");
    }

    // parikshaa without name errors (D085)
    #[test]
    fn test_parse_parikshaa_missing_name_d085() {
        let tokens = vec![
            kw(TokenKind::Parikshaa),
            kw(TokenKind::LBrace),
            kw(TokenKind::Danda),
            kw(TokenKind::RBrace),
            kw(TokenKind::Danda),
        ];
        let result = parse_tokens(tokens);
        assert!(result.is_err(), "parikshaa without name should error");
    }

    // parikshaa without braces errors (D085)
    #[test]
    fn test_parse_parikshaa_missing_braces_d085() {
        let tokens = vec![
            kw(TokenKind::Parikshaa),
            nm("test"),
            kw(TokenKind::Danda),
        ];
        let result = parse_tokens(tokens);
        assert!(result.is_err(), "parikshaa without braces should error");
    }
}
