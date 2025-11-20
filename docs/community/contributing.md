# Contributing

## Scope

Focus on deterministic correctness and transparency. Avoid premature abstraction.

## Areas of Impact

| Domain | Contributions |
|--------|---------------|
| Rules  | Add precise error → remediation pairs |
| Ranking | Tune weights with rationale + benchmarks |
| Context | Improve detection signatures |
| Moat   | Curate widely encountered fixes |
| Docs   | Extend clarity; remove ambiguity |

## Process

1. Open issue describing change + rationale
2. Provide benchmark or example scenario (if ranking/performance)
3. Submit PR with focused diff (avoid unrelated formatting)
4. Include tests where feasible

## Style

- Rust 2021 edition conventions
- Avoid unnecessary unsafe blocks
- Descriptive function and module names
- Limit external dependencies (audit surface)

## Testing

Add targeted tests for new rule logic and ranking adjustments. Avoid broad end-to-end additions unless needed to illustrate change.

## Performance Considerations

Recall path should remain <25ms p95 on typical developer hardware. Provide regression evidence if modifying core query logic.

## Documentation

Update reference pages when adjusting formulas, schema, or ranking behavior. Keep README marketing minimal; prefer technical precision.

## License

Contributions under MIT license.
