# Finance Harness AWKWARDNESS.md (M9 Step 1 Phase 2 draft)

This is the running implementation-time friction list. These are items hit while filling in the Phase 1 stub, not copied from `M9_GATE_DIAGRAM.md §5`.

## 1. `agemo` stdin cannot both carry the prompt and later approval

- What felt forced: the documented `echo "buy 10 AAPL if under $200" | agemo ...` shape consumes all of stdin in `read_prompt()`, leaving no live stdin for the later AskUser approval bridge.
- Workaround used: run demos with `--prompt "buy 10 AAPL if under $200"` and pipe/type `approve` or `deny` only for the approval response.
- Desired API/support: separate prompt input from interactive approval input, or document `--prompt` as mandatory for approval demos.
- M10 priority hint: High for CLI/demo reliability; low for core harness API.

## 2. Verifying read-only downstream builds can mutate read-only repos

- What felt forced: running `cargo build` in `agemo` updated its `Cargo.lock` from loop `0.25.0` to `0.25.1`, despite `agemo` being read-only for this phase. I had to restore the lockfile before implementation.
- Workaround used: revert the lockfile and avoid committing any `agemo` change; build artifacts only live in `target/`.
- Desired API/support: precondition docs should specify `cargo build --locked` expectations, or the read-only repo should already have an up-to-date lockfile.
- M10 priority hint: Medium; stale locks make milestone verification noisy and risky.

## 3. Audit failure rows require a synthetic `result` shape

- What felt forced: `post_tool_use` receives a full `ToolResult`, but `post_tool_use_failure` receives only `ToolFailure`. The required JSONL format wants a `result` field in both cases, so failure rows need an invented `{ "failure": ... }` result object.
- Workaround used: keep `event = "post_tool_use_failure"`, set `is_error = true`, and put the serialized failure enum under `result.failure`.
- Desired API/support: provide the final error `ToolResult` (as seen by the model) in the failure hook context, or standardize an audit event schema in primitives.
- M10 priority hint: Medium; audit consumers should not have to special-case framework failure shapes.

## 4. Tool naming guidance conflicts with provider/practical prompt conventions

- What felt forced: the Harness trait docs recommend namespaced names like `finance.place_order`, while the Phase 2 design and system prompt use `place_order`/`get_quote`. Provider tool-name restrictions and LLM prompt clarity both favor simple names for this demo.
- Workaround used: implement unqualified tool names (`get_quote`, `get_position`, `place_order`) for M9 demo compatibility.
- Desired API/support: a first-class display name / namespace mapping, so tools can be collision-safe internally without exposing awkward provider names.
- M10 priority hint: Medium before multiple harnesses are stacked.

## 5. Denied permission loses tool identity in `agemo`'s wire transcript

- What felt forced: the deny demo correctly produced an AskUser prompt and the hook audit row for `place_order`, but `agemo`'s JSONL transcript emitted the denied tool result as `tool_call_start` with `name: "unknown"` / `id: "call_unknown"`.
- Workaround used: rely on the audit log for the canonical denied-tool identity and call out the transcript quirk in the checkpoint report.
- Desired API/support: the loop or host event mapping should retain the original tool call metadata for denied permission completions.
- M10 priority hint: High for audit/UI correctness; users need to see which destructive call was denied.
