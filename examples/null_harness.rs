//! `NullHarness` — the simplest possible [`Harness`] implementation.
//!
//! Contributes no tools, no hooks, no policy, no memory, no system prompt.
//! Useful as a placeholder when constructing a session that only wants the
//! framework's defaults, and as a smoke test that the [`Harness`] trait
//! compiles in a downstream crate.

use std::sync::Arc;

use motosan_agent_harness::Harness;
use motosan_agent_tool::Tool;

struct NullHarness;

impl Harness for NullHarness {
    fn name(&self) -> &str {
        "null"
    }

    fn tools(&self) -> Vec<Arc<dyn Tool>> {
        Vec::new()
    }
}

fn main() {
    let h: Arc<dyn Harness> = Arc::new(NullHarness);
    println!("harness name        : {}", h.name());
    println!("tools                : {}", h.tools().len());
    println!("hooks                : {}", h.hooks().len());
    println!("permission policy?   : {}", h.permission_policy().is_some());
    println!("memory schema?       : {}", h.memory_schema().is_some());
    println!("system prompt?       : {}", h.system_prompt().is_some());
}
