# Devvani (देववाणी)
> A Sanskrit-grammar-based programming language inspired by Pāṇini's Aṣṭādhyāyī

## What is Devvani?
Devvani is a high-performance programming language built on the logical and structural foundations of Sanskrit grammar. It leverages the ancient rules of Pāṇini's Aṣṭādhyāyī to create a mathematically precise and grammatically consistent environment for modern software development.

The language maps traditional Sanskrit concepts directly to programming features. In Devvani, noun case endings (Vibhakti) determine type roles, verb tenses (Lakara) define function scope and asynchrony, and grammatical number (Vacana) governs cardinality and collection types. This unique approach allows developers to write code that is both human-readable (to those familiar with Sanskrit) and strictly verified by the underlying grammar engine.

Key mappings include:
- **Vibhakti** (Cases) → Type System / Roles (Subject, Object, Instrument, etc.)
- **Dhātu** (Verb Roots) → Function Definitions
- **Lakara** (Tense/Mood) → Scope & Async Behavior
- **Vacana** (Number) → Cardinality (Single, Pair, Collection)
- **Liṅga** (Gender) → Mutability & Ownership Semantics
- **Samāsa** (Compounds) → Type Hierarchies & Module Paths

## Quick Example
Devvani utilizes Sanskrit sentence structure (SOV - Subject Object Verb) for expressive and type-safe code.

| Devvani (.dvn) | Rust Equivalent (Generated) |
| :--- | :--- |
| `rāmaḥ phalam khādati.` | `rama.khadati(phalam);` |

## Architecture
Devvani operates through a multi-stage pipeline that transforms Sanskrit-inspired source code into valid Rust or bytecode.

```text
  .dvn file
     ↓
  Lexer (devvani-lexer)        — Unicode/IAST tokenizer + Sandhi Engine
     ↓
  Parser (devvani-parser)      — SOV parser + Karaka mapping
     ↓
  Type System (devvani-typesystem) — Vibhakti, Lakara, Vacana, Linga Resolution
     ↓
  Codegen (devvani-codegen)    — Rust source emission + Bytecode generation
     ↓
  Rust output / Bytecode
```

## Grammar → Language Features

| Sanskrit Concept | Grammar Term | Devvani Role | Example |
| :--- | :--- | :--- | :--- |
| Case endings | Vibhakti (7) | Type system | `rāmaḥ` → Subject |
| Verb roots | Dhātu | Functions | `khādati` → `fn khadati()` |
| Tense/Mood | Lakara (10) | Scope/Async | `Lrt` → `async fn` |
| Number | Vacana (3) | Cardinality | `Bahu` → `Vec<T>` |
| Gender | Liṅga (3) | Mutability | `Strilinga` → `let mut` |
| Compounds | Samāsa (5) | Type composition | `Rama.Putra` → `rama.putra` |
| Sound rules | Sandhi | Token merging | `vidyā+ālaya` → `vidyālaya` |

## Installation
```bash
git clone https://github.com/gauravverma-creator/Devvani-private.git
cd Devvani
cargo build --workspace
```

## Usage
Devvani provides a comprehensive CLI for managing the compilation pipeline.

```bash
# Compile a .dvn file
cargo run -p devvani-cli -- compile examples/hello.dvn

# Check types only
cargo run -p devvani-cli -- check examples/hello.dvn

# Lex tokens
cargo run -p devvani-cli -- lex examples/hello.dvn

# Parse AST
cargo run -p devvani-cli -- parse examples/hello.dvn
```

## Error Messages
Devvani features a Sanskrit-aware diagnostic engine that references Paninian concepts and Sutras.

```text
── दोष Dosha D001 | Aparicita Nama ──────────────────
 अपरिचित नाम: 'rāma' Prathama Vibhakti mein Kartā ke roop mein nahi mila. Pehle ise define karo.
 Sutra: 1.4.54 (Kartā — svatantraḥ kartā)
 Hint: 'rāma' ko pehle declare karo: rāmaḥ
────────────────────────────────────────────────
```

## Crate Structure

| Crate | Purpose | Status |
| :--- | :--- | :--- |
| devvani-lexer | IAST Unicode tokenizer | ✅ |
| devvani-ast | AST node definitions | ✅ |
| devvani-parser | SOV parser + Karaka mapping | ✅ |
| devvani-typesystem | Vibhakti/Lakara/Vacana/Linga | ✅ |
| devvani-codegen | Rust source + bytecode output | ✅ |
| devvani-compiler | Pipeline orchestrator | ✅ |
| devvani-cli | Command-line interface | ✅ |
| devvani-number | Sanskrit numeral system | 🔄 |

## Test Status
```bash
cargo test --workspace
```
63/63 tests passing.

## Roadmap
- [x] Phase 1: Lexer
- [x] Phase 2: Parser + AST
- [x] Phase 3: Type System (Vibhakti, Lakara, Vacana, Linga)
- [x] Phase 4: Codegen + CLI + Diagnostics
- [ ] Phase 5: Standard library (Sanskrit built-ins)
- [ ] Phase 6: Self-hosting — Devvani written in Devvani

## License
No license
