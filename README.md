# Aethr MVP — Context-Gated Terminal Intelligence

**Aethr** is a local-first terminal assistant that learns your command patterns and helps you:
- 🔍 **Recall** commands from your history with semantic search
- 🤖 **Predict** what to type based on your intent  
- 🔧 **Fix** errors with community solutions + rules engine
- 📊 **Understand** your workflow with context detection

**Core Promise**: Your terminal intelligence lives on your machine first. Optional cloud sync later.

---

## Quick Start (5 minutes)

### Prerequisites
- **Rust 1.70+** ([install](https://rustup.rs/))
- **SQLite3** (usually pre-installed)
- **Linux/macOS** (or WSL on Windows)

### Installation

```bash
# 1. Clone and build
git clone https://github.com/pinkabel/aethr.git
cd aethr
cargo build --release

# 2. Initialize local database
./target/release/aethr init

# 3. Seed with 54 community fixes
./target/release/aethr seed-moat

# 4. (Optional) Enable shell history logging
cat << 'EOF' >> ~/.bashrc
# Aethr command logging
export HISTFILE="$HOME/.aethr/commands.log"
EOF
# Then open a new terminal

# 5. Import your history
./target/release/aethr import

# 6. Start using!
./target/release/aethr                    # Interactive menu (NEW: no command needed!)
./target/release/aethr recall "docker"    # Semantic search
./target/release/aethr fix "permission"   # Error fixing
./target/release/aethr run                # TUI explorer
```

---

## Features & Examples

### 1. **Recall** — Semantic Search Over Your Commands

```bash
$ aethr recall "docker"

🔍 Searching command history...
�� Detected context: docker

📋 Top matches (your history + community):
⭐ 1. docker ps -a [used 5 times]
⭐ 2. docker run -it ubuntu:latest [used 12 times]
⭐ 3. docker logs -f <container_id> [used 3 times]
   4. docker build -t myapp:latest .
   5. sudo usermod -aG docker $USER && newgrp docker
```

**Smart Ranking**:
- ⭐ Commands that match your project context are highlighted
- `[used X times]` shows frequency (how often you've run it)
- Recent commands (24h) rank higher than old ones
- Combines: 60% recency + 40% frequency + context boost

### 2. **Fix** — Error Resolution with 3-Tier Pipeline

**Layer 1: Deterministic Rules** (fastest, highest confidence)
```bash
$ aethr fix "permission denied"

🔧 Analyzing error...

✅ RULE MATCH (Deterministic):
   Command: sudo chown $(whoami) -R . && chmod u+rwx -R .
   Confidence: 60%
   Explanation: Change ownership of current directory to your user.
```

**Layer 2: Community Moat** (proven solutions from real devs)
```bash
$ aethr fix "Cannot find module express"

💡 COMMUNITY MOAT SUGGESTIONS:
   (Fixes from real developers who solved this)

   1. npm install express
   2. npm ci
```

**Layer 3: LLM Fallback** (when rules & community don't help)
```bash
$ ANTHROPIC_API_KEY=sk-... aethr fix "weird error"

🤖 LLM SUGGESTION:
   Command: [AI-generated fix based on error text]
```

### 3. **Predict** — AI Command Suggestions

```bash
$ aethr predict "build and deploy to production"

📍 Detected context: nodejs, git

Prediction: docker build -t myapp:latest . && docker push myrepo/myapp:latest
```

LLM prompt automatically includes your project context for relevant suggestions.

### 4. **Interactive TUI** — Visual Command Explorer

```bash
$ aethr run
```

Features:
- 🎨 Color-coded results (context-relevant = yellow, selected = cyan)
- ⌨️ Arrow keys to navigate, Enter to copy to clipboard
- 🔍 Real-time search as you type
- 📊 Shows frequency count and context boosts
- 📱 Responsive, clean interface

---

## Context Detection

Aethr automatically detects your project type from the current directory:

| File/Folder | Detected | Boost |
|-------------|----------|-------|
| `package.json` | Node.js | npm/yarn/node commands |
| `requirements.txt` / `pyproject.toml` | Python | pip/python/venv commands |
| `Dockerfile` | Docker | docker commands |
| `Cargo.toml` | Rust | cargo commands |
| `go.mod` | Go | go commands |
| `pom.xml` | Java | maven/javac commands |
| Kubernetes YAML | Kubernetes | kubectl/helm commands |
| `.git/` | Git repo | git commands |

**Example**: In a Node.js + Docker project:
```bash
$ aethr recall "build"

📍 Detected context: nodejs, docker

📋 Top matches:
⭐ 1. npm run build           ← npm boost (2.5x)
⭐ 2. docker build -t app:v1  ← docker boost (2.5x)
```

---

## Commands Reference

```bash
# Core commands
aethr recall "<query>"        # Search your history
aethr predict "<intent>"      # AI suggestions
aethr fix "<error>"           # Fix errors (3-tier pipeline)
aethr run                      # Interactive TUI

# Setup
aethr init                     # Create database & config
aethr import                   # Import shell history
aethr seed-moat               # Load 54 community fixes
aethr status                   # Check config & API key

# Authentication
aethr login <token>           # Store API token (future)
```

All commands support `--help`:
```bash
aethr recall --help
aethr fix --help
```

---

## Real-World Scenarios

### Scenario 1: Fix Docker Permission Error

```bash
$ docker ps
permission denied while trying to connect to the Docker daemon

$ aethr fix "permission denied"
✅ RULE MATCH: sudo chown $(whoami) -R . && chmod u+rwx -R .
```

### Scenario 2: Search Git History in a Node Project

```bash
$ cd ~/my-node-app  # Has package.json
$ aethr recall "merge"

📍 Detected context: nodejs, git

📋 Top matches:
⭐ 1. git merge --no-ff develop
   2. git checkout -b feature/...
```

### Scenario 3: Deploy Kubernetes Service

```bash
$ cd ~/k8s-config  # Has deployment.yaml with apiVersion
$ aethr predict "deploy to production"

📍 Detected context: kubernetes, git

Prediction: kubectl apply -f deployment.yaml --namespace=prod
```

### Scenario 4: Python Virtual Environment Setup

```bash
$ aethr recall "venv"

📍 Detected context: python

📋 Top matches:
⭐ 1. python -m venv venv
⭐ 2. source venv/bin/activate
```

---

## Architecture

```
┌─────────────────────────────────────────────────┐
│           Command Query (recall/predict)        │
└────────────────┬────────────────────────────────┘
                 │
         ┌───────▼────────┐
         │ Context        │
         │ Detection      │ ← Detects git, docker, nodejs, python, k8s
         └───────┬────────┘
                 │
    ┌────────────▼────────────┐
    │   Smart Ranking         │
    │ (Recency + Frequency)   │ ← 60% recency + 40% frequency
    └────────────┬────────────┘
                 │
    ┌────────────▼───────────────────┐
    │  Context Boosting (2-2.5x)      │
    │  Prioritize matching tech stack │
    └────────────┬───────────────────┘
                 │
        ┌────────▼────────────┐
        │  FTS5 Search Results│
        │  + Community Moat   │
        └────────┬────────────┘
                 │
         ┌───────▼────────┐
         │ Local SQLite   │
         │ (personal hist)│
         │ + 54 moat cmds │
         └────────────────┘
```

---

## Data Storage

```
~/.aethr/
├── aethr.db              # SQLite database
│   ├── command_history   # Your commands + metadata
│   ├── command_fts       # Full-text search index
│   ├── community_moat    # 54 curated fixes
│   └── community_moat_fts # Moat search index
├── config.toml           # User config (future)
├── aethr_token           # API token (optional)
└── commands.log          # Shell history (if using hook)
```

---

## Environment Variables

```bash
# Claude API (for predict/fix LLM features)
export ANTHROPIC_API_KEY=sk-...

# Or use legacy name
export CLAUDE_API_KEY=sk-...
```

---

## Installation & Build

### From Source
```bash
# Prerequisites (Ubuntu/Debian)
sudo apt update && sudo apt install -y \
  build-essential pkg-config libssl-dev libsqlite3-dev sqlite3 xclip

# Build
cargo build --release

# Binary location
./target/release/aethr
```

### Optional: System-Wide Install
```bash
cargo install --path .
# Then use: aethr recall "..."
```

---

## Roadmap

- ✅ **Phase 1 (MVP - Local)**: Recall, context detection, smart ranking, TUI, fix rules
- 🟡 **Phase 2 (Cloud Integration)**: 
  - Cloud API for community moat sync
  - LLM integration for predictions
  - Token authentication
- 🔮 **Phase 3 (Monetization)**:
  - API pricing ($0.08/call like Stripe)
  - Premium features (team workspaces, advanced analytics)

---

## FAQ

**Q: Does Aethr upload my command history to the cloud?**  
A: No. Everything runs locally on your machine. Optional cloud sync is future work.

**Q: What does "Community Moat" mean?**  
A: 54 curated solutions from real developers solving common problems (docker, git, npm, python, k8s, etc.). Local copy, no cloud dependency.

**Q: How do I contribute fixes?**  
A: Future feature. Currently, the moat is seeded from proven solutions.

**Q: Does it work without an API key?**  
A: Yes! Recall, fix rules, and community moat work 100% offline. LLM features (predict, advanced fix) need ANTHROPIC_API_KEY.

**Q: Can I use it in my CI/CD pipeline?**  
A: Yes, all commands have exit codes. Example:
```bash
if aethr fix "deployment error" 2>/dev/null; then
  echo "Fix found!"
fi
```

---

## Contributing

We welcome contributions! Areas for help:
- Adding more moat entries (solutions for common errors)
- Better error detection patterns
- UI/UX improvements
- Test coverage
- Documentation

---

## License

MIT. See [LICENSE](LICENSE) for details.

---

## Credits

Built with ❤️ by [Pinkabel](https://github.com/pinkabel)

**Using**:
- [ratatui](https://github.com/ratatui-org/ratatui) — Terminal UI
- [rusqlite](https://github.com/rusqlite/rusqlite) — SQLite bindings
- [crossterm](https://github.com/crossterm-rs/crossterm) — Terminal control
- [serde](https://github.com/serde-rs/serde) — Serialization
- [regex](https://github.com/rust-lang/regex) — Pattern matching
- [colored](https://github.com/mackwic/colored) — Terminal colors

---

**Get Started**: `./target/release/aethr init && ./target/release/aethr recall "docker"`
