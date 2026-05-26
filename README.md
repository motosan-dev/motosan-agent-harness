# motosan-agent-harness

Composition contract for the Motosan agent framework — the `Harness` trait.

A *harness* is a vertical bundle that ties a set of tools, hooks, a permission
policy, a memory schema, and a system prompt into one coherent domain
(finance, rental, devops, …). This crate owns the `Harness` trait and
nothing else.

## Layering

```text
   ┌────────────────────────────────────────────┐
   │ motosan-agent-harness-{finance,rental,…}   │  vertical implementations
   ├────────────────────────────────────────────┤
   │ motosan-agent-loop      (ReAct engine)     │  runs Harness + Tools
   ├────────────────────────────────────────────┤
   │ motosan-agent-harness   (Harness trait)    │  THIS CRATE
   ├────────────────────────────────────────────┤
   │ motosan-agent-tool │ motosan-ai │ sandbox  │  capability + infra
   ├────────────────────────────────────────────┤
   │ motosan-agent-primitives                   │  shared types + Hook + Permission
   └────────────────────────────────────────────┘
```

This crate sits **above** `motosan-agent-tool` and `motosan-agent-primitives`
so that it can reference both the `Tool` trait and the `Hook` /
`PermissionPolicy` / `MemorySchema` types without forcing those leaf crates
to depend on each other. See decision **D1=B** in the primitives
implementation plan for the rationale.

## What's in here

- `Harness` trait — the single composition contract every vertical
  implementation must satisfy.
- Two examples:
  - `examples/null_harness.rs` — the minimum viable harness.
  - `examples/two_tool_harness.rs` — a harness with two stub tools and a
    system prompt.

## Composition rules

When the agent loop stacks multiple harnesses into one session it must
honour:

- **Tool name uniqueness** across the union — collisions fail loudly.
- **Hook ordering** = registration order, then per-harness order.
- **PermissionPolicy composition** = most-restrictive-wins (`Deny` >
  `AskUser` > `Allow`) — order-independent.
- **MemorySchema** = union of declared keys; collisions are configuration
  errors.
- **System prompt** = concatenated in registration order with a blank line
  separator.

See the `Harness` rustdoc for the full contract.

## Quick start

```rust
use std::sync::Arc;
use motosan_agent_harness::Harness;
use motosan_agent_tool::Tool;

struct MyHarness;

impl Harness for MyHarness {
    fn name(&self) -> &str { "my-domain" }
    fn tools(&self) -> Vec<Arc<dyn Tool>> { Vec::new() }
}
```

Run the bundled examples:

```bash
cargo run --example null_harness
cargo run --example two_tool_harness
```

## Status

`0.1.0` — pre-1.0 API. Will be frozen once two real harnesses (finance +
rental) have been built against it and the awkwardness list resolved.

## License

Apache-2.0 — see [`LICENSE`](LICENSE).
