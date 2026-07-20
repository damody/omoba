# Run 10000 Starting Gold Design

## Goal

Provide a test-only `run_10000.bat` launcher that starts every TD difficulty with 10,000 gold while preserving the normal 650 starting gold used by `run.bat`.

## Design

Add an optional `OMB_TD_STARTING_GOLD` environment override to `TdDifficultyConfig::from_env()`. Difficulty selection remains authoritative for lives, cost multiplier, and round count; only `starting_gold` is replaced when the override contains a valid non-negative `i32` value. A missing, empty, malformed, or negative override falls back to the selected difficulty's normal value.

Add `run_10000.bat` as a small wrapper. It sets `OMB_TD_STARTING_GOLD=10000`, calls the existing root `run.bat`, forwards all command-line arguments such as `--trace`, and returns the same exit code. The wrapper must use CRLF line endings.

This avoids duplicating the build, DLL staging, backend lifecycle, and frontend launch logic already owned by `run.bat`.

## Testing

- Keep existing assertions that all four difficulty profiles normally start with 650 gold.
- Add pure parsing/application tests proving novice, intermediate, advanced, and expert each start with 10,000 when the override is `10000`.
- Verify invalid and negative override values retain the normal difficulty value.
- Run focused initialization tests and the relevant `omoba-core` test suite.
- Verify the batch file has CRLF line endings and forwards arguments and exit status.

## Scope

- Do not change production difficulty values.
- Do not duplicate `run.bat` implementation.
- Do not modify `omfue`.
