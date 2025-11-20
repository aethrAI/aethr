# Features

## Search / Recall

Semantic matching over prior commands using SQLite FTS5 plus structured metadata. Ranking = `score = w_recency + w_frequency + w_context` with documented weights.

## Fix Pipeline

1. Deterministic rule match (pattern → remediation)
2. Community moat lookup
3. Planned: LLM interpretation when prior layers produce no result

Error entries record: pattern, remediation, confidence origin.

## Context Detection

File and directory heuristics establish active tech stack (e.g. `package.json` → Node.js). Context boosts ranking; all boosts are transparent in UI.

## Community Moat

Locally cached set of curated remediations. Distributed as seed data; no remote calls required.

## Predict (Beta)

Intent string + detected context → candidate composite commands. Currently limited to common workflows (build, run, deploy). Will expand post deterministic maturity.

## Privacy

All operations local by default. No automatic network traffic. Remote augmentation (LLM, sync) will be explicit and configurable.

## Extensibility

Planned plugin points:
- Additional ranking signals
- Organization scoped moat entries
- Custom rule packs

## Road to LLM Integration

LLM features will focus on structured error explanation, diff-based remediation, and natural language command synthesis. Activation requires explicit key provisioning.
