//! `FinanceHarness` — domain harness for the M9 finance demo.
//!
//! Phase 1 stub: returns empty/default for all 5 Harness fields.
//! Phase 2 fills in tools, policy, hook, system prompt.

use std::sync::Arc;

use motosan_agent_harness::Harness;
use motosan_agent_primitives::{Hook, MemorySchema, PermissionPolicy};
use motosan_agent_tool::Tool;

pub struct FinanceHarness;

impl FinanceHarness {
    pub fn new() -> Self {
        Self
    }
}

impl Default for FinanceHarness {
    fn default() -> Self {
        Self::new()
    }
}

impl Harness for FinanceHarness {
    fn name(&self) -> &str {
        "finance"
    }

    fn tools(&self) -> Vec<Arc<dyn Tool>> {
        Vec::new()
    }

    fn system_prompt(&self) -> Option<String> {
        None
    }

    fn hooks(&self) -> Vec<Arc<dyn Hook>> {
        Vec::new()
    }

    fn permission_policy(&self) -> Option<Arc<dyn PermissionPolicy>> {
        None
    }

    fn memory_schema(&self) -> Option<MemorySchema> {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stub_harness_name_is_finance() {
        assert_eq!(FinanceHarness::new().name(), "finance");
    }

    #[test]
    fn stub_harness_has_no_tools() {
        assert!(FinanceHarness::new().tools().is_empty());
    }
}
