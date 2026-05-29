# Changelog

`motosan-agent-harness-finance` — finance vertical harness for the Motosan agent framework.

## 0.2.0 — 2026-05-29

CHANGED (breaking):
- Bumped `motosan-agent-primitives` to 0.2.0 and `motosan-agent-tool` to 0.5.0; bumped path/version of `motosan-agent-harness` to 0.2.0.
- `AuditLogHook::post_tool_use_failure` now records `ctx.result` (the real `ToolResult` the model sees) directly, dropping the synthetic `{ "failure": ... }` wrapper. Closes [AWKWARDNESS.md #3](AWKWARDNESS.md) (M10 D-M10-2).
- Finance tools now expose a distinct host-side `internal_name`: `finance.get_quote` / `finance.get_position` / `finance.place_order`. The LLM-facing `name` is unchanged (`get_quote` / `get_position` / `place_order`), preserving M9 prompt and provider compatibility. Closes [AWKWARDNESS.md #4](AWKWARDNESS.md) (M10 D-M10-4).
- `FinanceApprovalPolicy` test helper updated for the new `PermissionContext { recent_messages: &[Message] }` field added in primitives 0.2.0 (M10 D-M10-3); no behavior change for the policy itself.

## 0.1.0 — 2026-05-29

Initial release.

### Added

- **`FinanceHarness`** implementing the full 5-field `motosan_agent_harness::Harness` trait:
  - `tools()` — three finance tools: `get_quote`, `get_position`, `place_order`
  - `system_prompt()` — finance-domain persona instructing the LLM on the get_quote → evaluate → place_order flow
  - `permission_policy()` — `FinanceApprovalPolicy` (gates any tool with `annotations.destructive == true` via `Permission::AskUser`; read-only tools auto-allow)
  - `hooks()` — `AuditLogHook` writing JSONL audit records (configurable path via env `FINANCE_AUDIT_LOG` or constructor arg)
  - `memory_schema()` — `None` (no memory yet)

- **`tools::get_quote { symbol }`** — returns mocked quote `{ symbol, price, timestamp }`. Hardcoded prices for AAPL ($185), MSFT ($420), GOOGL ($175), TSLA ($245), NVDA ($880).

- **`tools::get_position { symbol }`** — returns mocked holdings.

- **`tools::place_order { symbol, side, quantity, max_price? }`** — `destructive: true` annotation, gated by `FinanceApprovalPolicy`. Returns mocked fill `{ order_id, status, executed_price }`.

- **`policy::FinanceApprovalPolicy`** — checks `ctx.annotations.destructive`; renders human-readable approval prompts from `tool_input` for `place_order` calls.

- **`audit::AuditLogHook`** — JSONL audit emitter writing on `session_start`, `post_tool_use`, `post_tool_use_failure`, `session_end`. File handle protected by `Arc<Mutex<File>>`.

### Demo

End-to-end validation via `agemo --harness finance --prompt "buy 10 AAPL if under $200"`:
- Approve scenario: `get_quote(AAPL)` → $185 → `place_order` triggers `AskUser` → user approves → `place_order` executes → audit records the full chain
- Deny scenario: same flow but user denies → `place_order` does NOT execute → audit records `post_tool_use_failure`
- Read-only scenario: "what's TSLA's price?" → `get_quote` auto-allows, no prompt

### Known awkwardness — see [AWKWARDNESS.md](AWKWARDNESS.md)

5 implementation-time friction items captured for M10:

1. `agemo` stdin conflict (fixed in agemo 0.1.3)
2. Read-only repos can mutate lockfile during build (process fix in M10)
3. `post_tool_use_failure` ctx is thin; needed synthetic `result` shape (M10 D-M10-2)
4. Tool naming guidance conflicts with provider conventions (M10 D-M10-4)
5. Denied permission loses tool identity in agemo wire (fixed in agemo 0.1.3)

### M9 milestone

Ships as part of M9 Step 1 Phase 2 — the first vertical harness consumer of the Motosan agent framework. Validated the M8.6.1 `LoopInterceptor` + `PermissionPolicy` + Hook wiring works end-to-end for a real domain use case.
