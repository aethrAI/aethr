# Installation

This guide shows the quickest way to get Aethr running. Most users should use the one-line installer. Source build instructions are provided for contributors.

## Supported platforms

- Linux (x86_64 / aarch64)
- macOS 12+
- Windows (via WSL2 if you want Linux semantics, but not required for macOS/Linux users)

## Quick install (recommended)

Run the installer script (reviews welcome — it’s a small audited shell wrapper):

```bash
curl -fsSL https://install.aethr-ai.dev | sh
```

After it completes, restart your shell or source your profile so the `aethr` binary is on `PATH`.

Initialize local state and import your history:

```bash
aethr init
aethr import        # optional: ingest existing shell history
aethr seed-moat     # optional: load shared deterministic fixes
```

Check status:

```bash
aethr status
```

## Optional packages (Debian/Ubuntu)

```bash
sudo apt update && sudo apt install -y libssl-dev libsqlite3-dev sqlite3 xclip
```

These enable faster builds and clipboard integration; Aethr still works without them.

## Source build (contributors / advanced)

```bash
git clone https://github.com/aethrAI/aethr.git
cd aethr
cargo build --release
./target/release/aethr init
```

Add manually to PATH (optional):

```bash
ln -s "$PWD/target/release/aethr" ~/.local/bin/aethr
```

## Uninstall

```bash
rm -f ~/.local/bin/aethr
rm -rf ~/.aethr
```

## Build time

First release build: ~1–2 minutes (network + compilation). Subsequent builds are incremental.

## About upcoming LLM features

Predictive and remediation capabilities (LLM-assisted) will arrive soon. They will be strictly opt-in and layered atop the existing deterministic and community fix pipeline. No action or API keys are required from you today.

Stay tuned in the roadmap for availability updates.
