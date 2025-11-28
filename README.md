# Aethr — Terminal Intelligence That Learns

**Aethr** is a local-first terminal assistant that learns your command patterns and helps you:
- **Recall** commands with semantic search
- **Fix** errors with community-powered solutions
- **Auto-save** commands silently in the background

Your terminal intelligence lives on your machine. Optional community sync for shared fixes.

---

## Quick Start

### Installation

```bash
# Clone and build
git clone https://github.com/aethrAI/aethr.git
cd aethr
cargo build --release

# Add to PATH
sudo cp target/release/aethr /usr/local/bin/
# Or: export PATH="$PATH:$(pwd)/target/release"

# Initialize
aethr init
```

### Setup Auto-Save

```bash
# Install shell hook (adds to ~/.bashrc or ~/.zshrc)
aethr hook --install

# Reload shell
source ~/.bashrc  # or ~/.zshrc
```

### Import Existing History

```bash
aethr import
```

---

## Commands

### `aethr` — Interactive Mode

Just run `aethr` with no arguments for an interactive prompt:

```
$ aethr

  Welcome to Aethr
  Terminal Intelligence That Learns

  Type a command or use / for options

  > /recall docker build
```

**Slash commands:**
- `/recall <query>` — Search command history
- `/fix <error>` — Get fix suggestions
- `/import` — Import shell history
- `/status` — Check Aethr status
- `/help` — Show help
- `/exit` — Exit

---

### `aethr recall <query>` — Search Commands

Search your command history with smart ranking:

```bash
$ aethr recall docker build

 Searching: "docker build"
 Context: docker

 Results:

  *1 docker build -t myapp:latest .  (5x)
  *2 docker build --no-cache -t api .  (2x)
   3 docker-compose build  (3x)

 * = boosted by current context
```

**Features:**
- Full-text search with FTS5
- Context-aware boosting (detects project type)
- Recency + frequency scoring
- Shows usage count

---

### `aethr fix <error>` — Fix Errors

Get fix suggestions from 3 layers:

```bash
$ aethr fix "command not found: node"

 Analyzing: "command not found: node"

 Layer 3 Community Brain

  1. brew install node
    88% success (17 uses)

  2. nvm install node
    89% success (9 uses)

 Did this fix work? [Y/n]
```

**3-Layer Resolution:**
1. **Rules** — Deterministic fixes for common errors
2. **Local History** — Your past solutions (coming soon)
3. **Community Brain** — Crowdsourced fixes with success rates

When you confirm a fix works, it's logged to improve future suggestions.

---

### `aethr import` — Import History

Import commands from your shell history:

```bash
$ aethr import

 Importing shell history...

 + Imported 1063 commands from bash history

 + Total: 1063 commands imported
```

Supports: bash, zsh, fish

---

### `aethr status` — Check Status

```bash
$ aethr status

Aethr Status

  Configuration
    Config dir:    /home/user/.aethr
    Auto-save:     enabled (local only)
    Shell hook:    installed

  Database
    Path:          /home/user/.aethr/aethr.db
    Local history: 1063 commands
    Community Brain: 42 fixes

  Authentication
    Status:        not logged in
```

---

### `aethr hook` — Shell Integration

Setup automatic command saving:

```bash
# Show setup instructions
aethr hook

# Output hook for bash
aethr hook bash

# Output hook for zsh
aethr hook zsh

# Auto-install to shell config
aethr hook --install
```

---

## How It Works

### Auto-Save

When enabled, Aethr silently logs every command you run:

```
Command → ~/.aethr/commands.log → Processed on next aethr run → Saved to SQLite
```

No daemon required. Commands are imported automatically when you run any aethr command.

### Context Detection

Aethr detects your current project context by looking for:

| File | Context Tag |
|------|-------------|
| `package.json` | nodejs |
| `requirements.txt`, `pyproject.toml` | python |
| `Dockerfile` | docker |
| `Cargo.toml` | rust |
| `go.mod` | go |
| `*.tf` | terraform |
| `kubectl`, `k8s` | kubernetes |

Commands matching your context get boosted in search results.

### Community Brain

The Community Brain is a shared database of fixes:

- Each fix has a **success rate** based on user feedback
- Fixes are matched by error pattern + context
- When you confirm a fix works, it improves the success rate
- Anonymous — no personal data is shared

---

## Configuration

Config stored in `~/.aethr/config.json`:

```json
{
  "auto_save": true,
  "share_to_community": false,
  "shell_hook_installed": true
}
```

### First Run Options

On `aethr init`, you choose:

1. **Enable auto-save (local only)** — Commands saved privately
2. **Enable auto-save + Community Brain** — Help improve Aethr for everyone
3. **Disable auto-save** — Import history manually

---

## Data Storage

All data is stored locally in `~/.aethr/`:

```
~/.aethr/
├── config.json      # Settings
├── aethr.db         # SQLite database (commands + fixes)
├── commands.log     # Pending commands from shell hook
└── token            # Auth token (optional)
```

---

## Requirements

- **Rust 1.70+** (for building)
- **Linux/macOS** (or WSL on Windows)

---

## Development

```bash
# Build debug
cargo build

# Build release
cargo build --release

# Run tests
cargo test

# Run with logging
RUST_LOG=debug cargo run
```

---

## Roadmap

### v1 (Current)
- [x] Semantic recall with context boosting
- [x] Fix with 3-layer resolution
- [x] Auto-save shell hook
- [x] Community Brain with success rates
- [x] Interactive mode

### v2 (Planned)
- [ ] LLM-powered fix suggestions
- [ ] Natural language recall ("how did I install python")
- [ ] Predict/autocomplete
- [ ] Browser OAuth login
- [ ] Cloud sync

---

## License

MIT

---

## Links

- Website: [aethr-ai.dev](https://aethr-ai.dev)
- GitHub: [github.com/aethrAI/aethr](https://github.com/aethrAI/aethr)
