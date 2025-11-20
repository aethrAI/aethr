# Aethr

Local-first terminal intelligence: search, fix, and understand commands with privacy by default.

## Why Aethr

Aethr helps engineers work faster at the command line without surrendering data:

- Semantic recall of prior commands
- Deterministic and community-informed error remediation
- Context-aware suggestions (forthcoming)
- Local SQLite persistence with explicit opt-in sharing

## Core Principles

1. Local-first by architecture, not marketing copy
2. Deterministic before probabilistic (rules precede LLMs)
3. Transparent ranking of results (recency, frequency, context)
4. User agency over data (no silent collection)

## Fast Start

```bash
git clone https://github.com/aethrAI/aethr.git
cd aethr
cargo build --release
./target/release/aethr init
./target/release/aethr import
./target/release/aethr recall "docker build"
```

## Capabilities (Current)

| Area        | Feature                               | Status |
|-------------|----------------------------------------|--------|
| Search      | Semantic recall over history           | Stable |
| Fix         | Rule + community driven error help     | Stable |
| Predict     | Intent-based suggestions               | Beta   |
| Moat        | Curated shared fixes                   | Stable |
| Context     | Project tech stack detection           | Stable |
| LLM Assist  | Advanced reasoning layer               | Planned|

## Coming Next

The LLM layer will integrate after deterministic coverage is maximized. It will provide:

- Structured error interpretation
- Multi-step remediation suggestions
- Natural language intent mapping

Implementation will preserve local-first posture; remote augmentation will be explicit and optional.

## Navigation

| Start | Read | Apply | Maintain |
|-------|------|-------|----------|
| [Installation](getting-started/installation.md) | [Architecture](reference/architecture.md) | [Commands](usage/commands.md) | [Troubleshooting](operations/troubleshooting.md) |

## Design Goals

- Millisecond local queries
- Deterministic ranking formula (documented)
- Clear exit codes for CI integration
- Minimal cognitive overhead in TUI

Continue with: [Installation →](getting-started/installation.md)