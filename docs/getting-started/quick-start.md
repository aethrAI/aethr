# Quick Start

## 1. Build

```bash
git clone https://github.com/aethrAI/aethr.git
cd aethr
cargo build --release
```

## 2. Initialize

```bash
./target/release/aethr init
./target/release/aethr import
./target/release/aethr seed-moat
```

## 3. First Queries

Recall a prior command:
```bash
./target/release/aethr recall "docker build"
```

Get an error remediation:
```bash
./target/release/aethr fix "permission denied"
```

Open the interactive interface:
```bash
./target/release/aethr
```

## 4. Path Convenience (Optional)

```bash
ln -s "$PWD/target/release/aethr" ~/.local/bin/aethr
```

Then use:
```bash
aethr recall "compose" 
aethr fix "module not found"
```

## 5. Deterministic Layers

Sequence of remediation attempts:
1. Rule engine (fast, explicit)
2. Community moat (curated prior fixes)
3. LLM augmentation (planned)

## 6. Data Location

```text
~/.aethr/
├── aethr.db
├── commands.log (optional shell hook)
└── aethr_token (optional)
```

Proceed to [Commands](../usage/commands.md).
