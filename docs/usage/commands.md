# Commands

All commands support `--help` for inline reference.

## Interactive Interface

```bash
aethr
```
Starts the terminal user interface for browsing history and fixes.

## Recall

```bash
aethr recall "query"
```
Semantic retrieval over your command history plus curated entries.

Ranking factors:
- Recency (time decay)
- Frequency (usage count)
- Context match (project tech stack)

## Fix

```bash
aethr fix "error message"
```
Provides candidate remediations via rule engine then community entries. LLM layer will augment when deterministic sources exhaust.

## Predict (Beta)

```bash
aethr predict "intent description"
```
Generates suggested actions based on current directory context and intent string. Falls back gracefully if insufficient context.

## Import

```bash
aethr import
```
Ingests existing shell history (bash/zsh/fish) into local database.

## Init

```bash
aethr init
```
Creates database schema and default config state if absent.

## Seed Moat

```bash
aethr seed-moat
```
Adds curated community fixes to your local database. Safe to re-run for updates.

## Status

```bash
aethr status
```
Displays environment summary, database health, optional API key presence.

## Login

```bash
aethr login <token>
```
Stores authentication token for future remote services. Currently non-essential.

## Daemon (Planned)

```bash
aethr daemon start
```
Background process for sync and advanced telemetry (opt-in, future phase).
