//! `FinanceHarness` — domain harness for the M9 finance demo.

use std::{env, path::PathBuf, sync::Arc};

use motosan_agent_harness::Harness;
use motosan_agent_primitives::{Hook, MemorySchema, PermissionPolicy};
use motosan_agent_tool::Tool;

pub mod audit;
pub mod policy;
pub mod tools;

use audit::AuditLogHook;
use policy::FinanceApprovalPolicy;
use tools::{GetPositionTool, GetQuoteTool, PlaceOrderTool};

pub struct FinanceHarness {
    tools: Vec<Arc<dyn Tool>>,
    policy: Arc<FinanceApprovalPolicy>,
}

impl FinanceHarness {
    pub fn new() -> Self {
        Self {
            tools: vec![
                Arc::new(GetQuoteTool),
                Arc::new(GetPositionTool),
                Arc::new(PlaceOrderTool),
            ],
            policy: Arc::new(FinanceApprovalPolicy),
        }
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
        self.tools.clone()
    }

    fn system_prompt(&self) -> Option<String> {
        Some(
            r#"You are a finance trading assistant with access to these tools:
- get_quote(symbol): current price
- get_position(symbol): your current holdings
- place_order(symbol, side, quantity, max_price?): execute a trade

For trade requests:
1. Check current price via get_quote
2. If user specified a condition (e.g. "if under $200"), evaluate it
3. If trade is warranted, call place_order — this will require human approval
4. Report the outcome clearly

Be concise. Format numbers with currency symbols."#
                .into(),
        )
    }

    fn hooks(&self) -> Vec<Arc<dyn Hook>> {
        let path = env::var_os("FINANCE_AUDIT_LOG")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("./finance_audit.log"));
        vec![Arc::new(AuditLogHook::new(path))]
    }

    fn permission_policy(&self) -> Option<Arc<dyn PermissionPolicy>> {
        Some(self.policy.clone())
    }

    fn memory_schema(&self) -> Option<MemorySchema> {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn harness_name_is_finance() {
        assert_eq!(FinanceHarness::new().name(), "finance");
    }

    #[test]
    fn harness_has_finance_tools() {
        let tool_names: Vec<String> = FinanceHarness::new()
            .tools()
            .into_iter()
            .map(|tool| tool.def().name.clone())
            .collect();

        assert_eq!(tool_names, vec!["get_quote", "get_position", "place_order"]);
    }

    #[test]
    fn harness_has_system_prompt_policy_and_hook() {
        let harness = FinanceHarness::new();
        let log_path = std::env::temp_dir().join(format!(
            "finance-harness-test-{}.jsonl",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("time")
                .as_nanos()
        ));
        std::env::set_var("FINANCE_AUDIT_LOG", &log_path);

        assert!(harness
            .system_prompt()
            .unwrap()
            .contains("Check current price via get_quote"));
        assert!(harness.permission_policy().is_some());
        assert_eq!(harness.hooks().len(), 1);
        assert!(harness.memory_schema().is_none());

        std::env::remove_var("FINANCE_AUDIT_LOG");
        let _ = std::fs::remove_file(log_path);
    }
}
