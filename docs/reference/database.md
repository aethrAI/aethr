# Database

## Schema Overview

| Table              | Purpose                                |
|--------------------|----------------------------------------|
| `command_history`  | Raw commands + timestamps + cwd        |
| `command_fts`      | FTS5 virtual table for semantic recall |
| `community_moat`   | Curated fixes                          |
| `community_moat_fts` | FTS5 index for fixes                 |
| `meta`             | Versioning / migration markers         |

## Command History Fields

| Field      | Type     | Notes                              |
|------------|----------|------------------------------------|
| `id`       | INTEGER  | Autoincrement                      |
| `raw`      | TEXT     | Original command line              |
| `ts`       | INTEGER  | Unix epoch seconds                 |
| `cwd`      | TEXT     | Working directory                  |
| `ctx`      | TEXT     | Derived context tags (CSV)         |
| `freq`     | INTEGER  | Incremented usage count            |

## FTS Strategy

FTS5 used with custom tokenization defaults (simple). Stop-word removal deferred to preserve technical terms. Ranking merges FTS score with secondary signals.

## Migrations

Simple forward-only migrations controlled by numeric schema version in `meta`. On startup:

1. Read version
2. Apply missing migrations sequentially
3. Vacuum/reindex if required

## Integrity

Periodic (future) checksum over critical tables. User-facing command `aethr status` will expose anomalies.

## Deletion Policy

No silent pruning. Expired commands may receive recency decay but remain queryable unless user opts to purge.

## Future Additions

- Table for user-contributed fixes with provenance
- Delta sync metadata for organization deployments
- Anonymous telemetry buffer (opt-in) for ranking improvements
