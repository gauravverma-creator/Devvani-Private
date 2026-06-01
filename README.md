# ॥ देववाणी ॥
### The world's first Sanskrit-based AI programming language

Devvani is a high-performance, statically typed programming language designed specifically for Artificial Intelligence and Machine Learning, rooted in the logical structure of Sanskrit.

## 🚀 Installation

```bash
# Clone the repository
git clone https://github.com/your-username/devvani.git
cd devvani

# Build the project
cargo build --release
```

## ⚡ Quick Start

### Running a Devvani program
```bash
cargo run -p devvani-cli -- run examples/neural.dvn
```

### Starting the REPL
```bash
cargo run -p devvani-cli -- repl
```

## 📜 Language Syntax

Devvani uses Devanagari keywords for a native Sanskrit feel:

- `क्रिया` (Kriya) - Function definition
- `मान` (Maan) - Variable declaration
- `यदि/अन्यथा` (Yadi/Anyatha) - If/Else
- `जबतक` (Jabtak) - While loop
- `प्रत्यावर्तन` (Pratyavartan) - Return

Example:
```devvani
क्रिया योग(क, ख) {
    प्रत्यावर्तन क + ख
}

मान परिणाम = योग(१०, २०)
मुद्रण(परिणाम)
```

## 🧠 AI Features

- **Knowledge Type (`ज्ञान`)**: Built-in tensor support for AI models.
- **Matrix Math (`आव्यूह`)**: Optimized matrix operations.
- **AI Standard Library**: Native implementation of Tensors, Layers, and Models in `devvani-ai`.

## 📉 Compression Demo

Devvani features a **Semantic Compressor** that uses Sanskrit grammatical rules to optimize AI-related text data.

```bash
cargo run -p devvani-cli -- compress examples/compress_demo.txt
```

Example Output:
```text
Original tokens: 84
Compressed tokens: 62
Compression ratio: 26.19%
```

## 🛠️ Build & Test

```bash
# Run all tests
cargo test --workspace

# Build for release
cargo build --release
```
.