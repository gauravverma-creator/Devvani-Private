# Changelog

## [0.1.0] — 2025 (Initial Release)

### Added
- Phase 1: Full IAST Unicode lexer with 5 Sandhi rules
- Phase 2: SOV parser with Karaka mapping, 17 ASTNode types
- Phase 3: Complete type system
  - Vibhakti (7 cases) → type roles
  - Lakara (10 tenses) → scope + async semantics  
  - Vacana (3 numbers) → cardinality (Single/Pair/Vec)
  - Linga (3 genders)  → mutability (let/let mut/&T)
  - Samasa (5 compounds) → Rust type composition
- Phase 4: Code generation
  - Rust source emission
  - Bytecode instruction set
  - Sanskrit-style error diagnostics (D001–D010)
  - CLI: compile / check / lex / parse commands
- 63 tests passing across 7 crates
