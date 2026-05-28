//! The [`Harness`] trait — the composition contract for a vertical agent bundle.
//!
//! A *harness* answers the question "what does my agent know how to do, and
//! under what rules?" by bundling together:
//!
//! - A **name** (used for logs / event streams / disambiguation when multiple
//!   harnesses stack).
//! - An optional **system prompt** prepended to the conversation.
//! - A list of [`Tool`](motosan_agent_tool::Tool) implementations the LLM may
//!   call.
//! - A list of [`Hook`](motosan_agent_primitives::Hook) middleware that
//!   observes / rewrites lifecycle events.
//! - An optional [`PermissionPolicy`](motosan_agent_primitives::PermissionPolicy)
//!   that gates tool execution.
//! - An optional [`MemorySchema`](motosan_agent_primitives::MemorySchema)
//!   declaring the memory keys the harness reads / writes.
//!
//! ## Stacking harnesses
//!
//! The agent loop is allowed to compose **multiple harnesses** into one
//! session — for example a base `DevopsHarness` plus a vertical
//! `FinanceHarness` plus a session-specific `UserOverrideHarness`. The
//! composition rules below are not enforced by this trait (it has no runtime),
//! they are the **contract every loop implementation must honour**:
//!
//! ### Tool name uniqueness
//!
//! Tool names (`Tool::def().name`) MUST be unique across the union of all
//! stacked harnesses. The loop is required to detect collisions at session
//! construction time and **fail loudly** rather than silently shadow a tool.
//! Harness authors should namespace tool names with a short prefix
//! (`finance.place_order`, `rental.get_listings`, …) to make collisions
//! unlikely in practice.
//!
//! ### Hook ordering
//!
//! Hooks fire in **registration order**: the union is built by concatenating
//! `hooks()` from each harness in the order they were added to the session.
//! Within a single harness, the order returned by `hooks()` is preserved.
//! This matters because hooks can rewrite tool input by returning
//! [`HookResult::Continue { updated_input: Some(_) }`](motosan_agent_primitives::HookResult)
//! — an earlier hook's rewrite is visible to later hooks in the chain.
//!
//! ### PermissionPolicy composition: most-restrictive wins
//!
//! Per decision **D3** in the primitives plan, when multiple stacked
//! harnesses each return `Some(policy)` from
//! [`Harness::permission_policy`], the loop combines them into a single
//! composite policy using **most-restrictive-wins** semantics:
//!
//! - If **any** policy returns [`Permission::Deny`](motosan_agent_primitives::Permission), the composite is `Deny`.
//! - Otherwise if **any** policy returns [`Permission::AskUser`](motosan_agent_primitives::Permission), the composite is `AskUser`.
//! - Otherwise (every policy returned [`Permission::Allow`](motosan_agent_primitives::Permission)), the composite is `Allow`.
//!
//! Composition is **order-independent**: stacking `A` over `B` produces the
//! same effective permission as stacking `B` over `A`. This means a
//! harness can only ever make permissions *more* restrictive than a sibling,
//! never less. There is no "override" escape hatch by design.
//!
//! ### Memory schema merging
//!
//! When multiple harnesses declare a [`MemorySchema`](motosan_agent_primitives::MemorySchema),
//! the loop is expected to **union** the declared keys. Keys collisions are a
//! configuration error and SHOULD fail at session construction in the same
//! way that tool-name collisions do; this trait does not enforce that
//! because the storage backend is out of scope here.
//!
//! ### System prompt merging
//!
//! System prompts from multiple harnesses are concatenated in registration
//! order with a blank line between them. Harnesses that need to override an
//! earlier prompt rather than append must be the only harness providing one,
//! or must rely on an explicit loop-level builder API.

use std::sync::Arc;

use motosan_agent_primitives::{Hook, MemorySchema, PermissionPolicy};
use motosan_agent_tool::Tool;

/// A vertical agent bundle.
///
/// Implementors group a domain's tools, hooks, permission policy, memory
/// schema, and system prompt under one name. See the [module-level
/// docs][self] for the composition rules that loop implementations must
/// honour when stacking harnesses.
///
/// All accessor methods return *owned* values (or `Arc`s) because the loop
/// typically calls them once at session construction and stores the result.
/// Implementations are expected to be cheap — usually returning cloned
/// `Arc`s of long-lived shared state.
///
/// # Example
///
/// (Marked `ignore` because the link step pulls `libiconv` via transitive
/// `chrono` / `iana_time_zone` deps, and that lookup fails on environments
/// where the macOS SDK lib path isn't on the linker search path. The
/// example still serves as documentation; runnable coverage is in
/// `tests/` and the `examples/` binaries.)
///
/// ```ignore
/// use std::sync::Arc;
/// use motosan_agent_harness::Harness;
/// use motosan_agent_tool::Tool;
///
/// struct NullHarness;
///
/// impl Harness for NullHarness {
///     fn name(&self) -> &str { "null" }
///     fn tools(&self) -> Vec<Arc<dyn Tool>> { Vec::new() }
/// }
///
/// let h = NullHarness;
/// assert_eq!(h.name(), "null");
/// assert!(h.tools().is_empty());
/// assert!(h.hooks().is_empty());
/// assert!(h.permission_policy().is_none());
/// assert!(h.memory_schema().is_none());
/// assert!(h.system_prompt().is_none());
/// ```
pub trait Harness: Send + Sync {
    /// Stable identifier for this harness, used in logs, telemetry, and
    /// collision diagnostics when multiple harnesses stack.
    ///
    /// Should be a short kebab-case string (`"finance"`, `"rental-ops"`).
    /// Uniqueness across stacked harnesses is recommended but not enforced
    /// by this trait.
    fn name(&self) -> &str;

    /// Optional system prompt prepended to the conversation.
    ///
    /// When multiple harnesses are stacked the loop concatenates their
    /// prompts in registration order separated by a blank line. Return
    /// `None` for harnesses that contribute only tools / hooks and have no
    /// persona to declare.
    fn system_prompt(&self) -> Option<String> {
        None
    }

    /// Tools the LLM may call while this harness is active.
    ///
    /// Tool names (`Tool::def().name`) MUST be unique within the union of
    /// all stacked harnesses; the loop is expected to detect collisions at
    /// session construction and fail loudly.
    fn tools(&self) -> Vec<Arc<dyn Tool>>;

    /// Lifecycle hooks observing / rewriting the agent loop.
    ///
    /// Hooks fire in the order they are returned here, after which hooks
    /// from later-registered harnesses fire (registration-order semantics).
    /// The default returns an empty list — most harnesses do not need
    /// hooks.
    fn hooks(&self) -> Vec<Arc<dyn Hook>> {
        Vec::new()
    }

    /// Optional permission policy gating this harness's tools.
    ///
    /// When several stacked harnesses each return `Some(policy)`, the loop
    /// composes them via **most-restrictive-wins** (see module-level docs
    /// and primitives plan decision D3). The default returns `None`,
    /// meaning the harness contributes no opinion and inherits whatever
    /// policy the rest of the stack defines.
    fn permission_policy(&self) -> Option<Arc<dyn PermissionPolicy>> {
        None
    }

    /// Optional memory schema declaring keys this harness reads / writes.
    ///
    /// The loop unions schemas from all stacked harnesses; key collisions
    /// are a configuration error. Returns `None` for harnesses that do not
    /// touch memory.
    fn memory_schema(&self) -> Option<MemorySchema> {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Empty;
    impl Harness for Empty {
        fn name(&self) -> &str {
            "empty"
        }
        fn tools(&self) -> Vec<Arc<dyn Tool>> {
            Vec::new()
        }
    }

    #[test]
    fn defaults_are_empty_or_none() {
        let h = Empty;
        assert_eq!(h.name(), "empty");
        assert!(h.tools().is_empty());
        assert!(h.hooks().is_empty());
        assert!(h.permission_policy().is_none());
        assert!(h.memory_schema().is_none());
        assert!(h.system_prompt().is_none());
    }

    #[test]
    fn harness_is_object_safe() {
        // If this compiles, `dyn Harness` is object-safe — required for
        // loop crates that hold `Arc<dyn Harness>` per registered vertical.
        let _h: Arc<dyn Harness> = Arc::new(Empty);
    }
}
