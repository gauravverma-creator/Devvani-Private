# Devvani Lexer

The `devvani-lexer` crate provides a robust lexer for the Devvani programming language, inspired by Panini's Sanskrit grammar.

## Token Categories

- **Identifiers & Literals**: Supports IAST Unicode identifiers and standard numeric/string literals.
- **Vibhakti (Case Markers)**: `Prathama` through `Saptami` for type annotations.
- **Vacana (Number)**: `Ekavachana`, `Dvivachanaa, `Bahuvachana`.
- **Linga (Gender/Mutability)**: `Pullinga` (mutable), `Strilinga` (immutable), `Napumsakalinga` (const).
- **Lakara (Tense/Scope)**: `Lat`, `Lit`, `Lrt`, etc., for function markers.
- **Gana (Verb Classes)**: All 10 verb classes supported.
- **Upasarga (Prefixes)**: Compiler directives and module paths.
- **Nipata (Particles)**: Keywords and operators like `yadi` (if), `tarhi` (then), `ca` (and).
- **Special Tokens**: `Visarg`` (ḥ) and `Anusvara` (ṃ).
- **Punctuation & Structure**: Standard symbols like `.`, `::`, `->`, `=>`, and braces.

## Sandhi Rules Implemented

The lexer includes an optional pre-processing pass that applies Sanskrit Sandhi rules:

1. **Savarna Dirgha**: Identical vowels merge into long forms (e.g., `a+`` → `ḁ`).
2. **Guna**: `a/ā� before `i/ī/u/ū` (e.g., `a+i` �R `e`).
3. **Vriddhi**: `ḡ` before `e/i/o/u` (e.g., `ḡ+i` ⶒ `ai`).
4. **Yan Sandhi**: `/u` before different vowels (e.g., `i+a` ⶒ `ya`).
5. **Visarga Sandhi**: Phonological changes for `ḅ� (e.g., `ḅ+c` �R `śc`).

## How to Run Tests

From the workspace root:
``@bash
cargo test -p devvani-lexer
``@

## Usage

``@rust
use devvani_lexer;:{Lexer, SandhiMode};

let mut lexer = Lexer::new("rámaḥ phalaṃ khādati.");
let tokens = lexer.tokenize(SandhiMode::Auto).unwrap();
```
