# Aethr

Local-first terminal intelligence for engineers who want instant recall, deterministic fixes, and total privacy.

```bash
curl -fsSL https://install.aethr-ai.dev | sh
```

---

## Why teams choose Aethr

Aethr keeps engineers fast and confident at the command line while respecting privacy:

- **Semantic recall** — Find past invocations by intent, not exact syntax.
- **Deterministic fixes** — Community-vetted remediation before any LLM guesswork.
- **Situational awareness** — Understand project context to surface the right command.
- **Local-first storage** — SQLite-backed history with optional opt-in sharing.

---

## Core principles we refuse to compromise

1. **Local-first by architecture**, not just marketing copy.
2. **Deterministic before probabilistic** — rule engines precede LLM fallbacks.
3. **Transparent ranking** — recency, frequency, and context are always explained.
4. **User agency over data** — no silent collection, ever.

---

## Source install (power users)

```bash
git clone https://github.com/aethrAI/aethr.git
cd aethr
cargo build --release
./target/release/aethr init
./target/release/aethr import
./target/release/aethr recall "docker build"
```

---

## Capabilities that ship today

| Area        | Feature                               | Status |
|-------------|----------------------------------------|--------|
| **Search**  | Semantic recall over history           | Stable |
| **Fix**     | Rule + community driven error help     | Stable |
| **Predict** | Intent-based suggestions               | Beta   |
| **Moat**    | Curated shared fixes                   | Stable |
| **Context** | Project tech stack detection           | Stable |
| **LLM Assist** | Advanced reasoning layer            | Planned |

---

## Coming next on the roadmap

The LLM layer will integrate after deterministic coverage is maximized, unlocking:

- **Structured error interpretation** — Understand failures with full CLI context.
- **Multi-step remediation** — Guide complex recoveries without leaving the terminal.
- **Natural language intent mapping** — Speak your goal; let Aethr craft the command.

We preserve local-first guarantees: remote augmentation is explicit, optional, and transparent.

---

## Navigate the docs

| Get Started | Dive Deep | Explore | Maintain |
|-------------|-----------|---------|----------|
| [Installation](getting-started/installation.md) | [Architecture](reference/architecture.md) | [Commands](usage/commands.md) | [Troubleshooting](operations/troubleshooting.md) |
| [Quick Start](getting-started/quick-start.md) | [Database](reference/database.md) | [Features](usage/features.md) | [FAQ](operations/faq.md) |

---

## Design goals we live by

- ⚡ **Millisecond local queries** — snappy recall without network hops.
- 📐 **Deterministic ranking** — formulas are documented, auditable, tweakable.
- 🔄 **CI-friendly integration** — clear exit codes and structured outputs.
- 🧠 **Low cognitive load** — the TUI stays intuitive so you stay in flow.
