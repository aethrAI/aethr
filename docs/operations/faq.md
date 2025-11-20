# FAQ

## Data & Privacy

**Does Aethr upload my history?** No. Local-first by default; remote features will require explicit opt-in.

**Is anonymous telemetry collected?** Not at this stage. Any future collection will be documented and disabled by default.

## Usage

**Why does recall return few results?** History import or seed moat may not have run. Execute `aethr import` and `aethr seed-moat`.

**How are results ranked?** Recency decay + frequency normalization + context boost. Formula documented in Architecture reference.

**Can I disable context detection?** Planned configuration toggle; currently always on and local only.

## Fix Command

**Where do fixes come from?** Deterministic rule set plus curated community entries shipped locally.

**What if no fix is found?** Fallback LLM layer (planned) will attempt structured interpretation.

## Predict Command

**Why is output limited?** Feature is in beta; expands as deterministic coverage and context heuristics mature.

## System

**Which shells are supported?** bash, zsh, fish history ingestion.

**Can I use in CI?** Yes; commands return exit codes. Parse stdout for structured output where relevant.

**How do I remove all data?** Delete `~/.aethr/` and optional PATH symlink.

## Roadmap

**When will LLM features arrive?** After deterministic remediation reaches target coverage. Emphasis remains on transparency and local control.

**Will there be a hosted sync?** Optional, organization-focused. Disabled by default.
