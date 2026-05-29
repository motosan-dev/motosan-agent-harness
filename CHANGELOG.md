# Changelog

## 0.2.0 — 2026-05-29

CHANGED (breaking):
- Bumped `motosan-agent-primitives` to 0.2.0 and `motosan-agent-tool` to 0.5.0 across the workspace.
- Examples updated for the new `ToolDef::new(name, description, input_schema)` constructor (the previous public struct-literal form is gone).
- Finance member also bumped to 0.2.0 — see `finance/CHANGELOG.md` for AWK#3 (audit) and AWK#4 (tool namespacing) resolution.

## 0.1.2 — 2026-05-28

ADDED:
- Converted the crate root into a Cargo workspace while keeping the `motosan-agent-harness` package at the workspace root.
- Added `finance/` workspace member with a Phase 1 stub `motosan-agent-harness-finance` crate and empty `FinanceHarness`.

## 0.1.1 — 2026-05-26

CHANGED:
- Bumped motosan-agent-tool to 0.4.
- Examples updated for new async-trait Tool + mandatory annotations() + ToolOutput.
