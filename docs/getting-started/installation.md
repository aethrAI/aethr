# Installation

## Requirements

- Rust 1.70+ (use rustup)
- Git
- Linux, macOS, or WSL2

Optional packages (performance / clipboard integration):

```bash
# Debian / Ubuntu
sudo apt update && sudo apt install -y libssl-dev libsqlite3-dev sqlite3 xclip
```

## Procedure

Clone and build:

```bash
git clone https://github.com/aethrAI/aethr.git
cd aethr
cargo build --release
```

Initialize local state:

```bash
./target/release/aethr init
./target/release/aethr import   # ingest shell history
./target/release/aethr seed-moat  # load shared fixes
```

Verify:

```bash
./target/release/aethr status
```

Add binary to PATH (optional):

```bash
ln -s "$PWD/target/release/aethr" ~/.local/bin/aethr
```

## Uninstall

```bash
rm -f ~/.local/bin/aethr
rm -rf ~/.aethr
```

## Build Time

First release build: ~1–2 minutes depending on CPU and network. Subsequent builds are incremental.

## Environment Variables (Optional)

Set an Anthropic API key for emerging predictive / remediation features:

```bash
export ANTHROPIC_API_KEY=sk-... # optional
```

Absent a key, deterministic and community layers continue to function normally.
