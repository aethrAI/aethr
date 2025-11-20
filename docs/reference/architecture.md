# Architecture

## Layers

```
User Command
   │
   ├─ Parse & Normalize
   │
   ├─ Context Detection (project signatures)
   │
   ├─ Ranking Engine
   │     ├─ Recency decay
   │     ├─ Frequency weighting
   │     └─ Context boosts
   │
   ├─ Search (FTS5 + metadata filters)
   │
   ├─ Post-processing (dedupe, scoring)
   │
   └─ Output (TUI / stdout)
```

## Data Flow

| Component        | Responsibility                            | Persistence |
|------------------|--------------------------------------------|-------------|
| Collector        | Import shell history                      | `command_history` |
| Context Analyzer | Derive active tech stack                  | In-memory   |
| Rule Engine      | Deterministic error remediation           | `rules` (embedded) |
| Moat Loader      | Seed community fixes                      | `community_moat` |
| FTS Index        | Tokenization and match scoring            | `command_fts` / `community_moat_fts` |
| Predictor        | Intent mapping (beta)                     | In-memory   |

## Ranking Formula (Recall)

```
score = (r * WR) + (f * WF) + (c * WC)

r = recency factor (e^-Δt / τ)
f = normalized frequency
c = context indicator (0 or boost)
WR, WF, WC = tunable weights
```

Default weight distribution: 0.60 recency / 0.30 frequency / 0.10 context. Adjustments documented before change.

## Error Fix Selection

1. Rule exact/regex match → immediate return
2. Community candidates filtered by token overlap
3. Planned: LLM classification when (1,2) empty

## Boundaries

- No external calls in critical path
- LLM invocation isolated behind optional feature flag
- Database access abstracted behind small wrapper layer

## Performance Targets

| Operation      | Target (95th) |
|----------------|---------------|
| Recall query   | < 25 ms       |
| Fix rule match | < 5 ms        |
| Import batch   | < 2 s / 10k   |

Instrumentation hooks reserved for future profile collection (opt-in).
