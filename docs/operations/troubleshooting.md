# Troubleshooting

## Installation

| Symptom | Cause | Action |
|---------|-------|--------|
| `cargo: command not found` | Rust not installed | Install via rustup script |
| Build takes >5m | Network or dependency fetch | Re-run; verify connectivity |
| Binary missing | Build incomplete | Check `cargo build --release` output |

### Rust Setup

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source "$HOME/.cargo/env"
```

## Runtime

| Issue | Explanation | Resolution |
|-------|-------------|------------|
| Slow recall | DB fragmentation | `VACUUM;` (future command) |
| Empty results | History not imported | Run `aethr import` |
| Fix returns nothing | No rule / moat match | Fallback planned (LLM) |
| Permission errors | File ownership mismatch | Adjust ownership or path |

## Path Problems

Ensure `~/.local/bin` is on PATH:

```bash
echo $PATH | grep -q "$HOME/.local/bin" || echo "Missing"
```

Add if absent:

```bash
printf '\nexport PATH="$HOME/.local/bin:$PATH"\n' >> ~/.bashrc
source ~/.bashrc
```

## Database Integrity

Check status:
```bash
aethr status
```

If corruption suspected, backup then re-init:
```bash
cp ~/.aethr/aethr.db ~/aethr.db.bak
rm -rf ~/.aethr/
aethr init
aethr import
aethr seed-moat
```

## Reporting

Include:
- OS / shell
- Command executed
- Output snippet
- `aethr status` summary

Submit via GitHub issue tracker.
