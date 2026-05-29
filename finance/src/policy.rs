use async_trait::async_trait;
use motosan_agent_primitives::{Permission, PermissionContext, PermissionPolicy};

pub struct FinanceApprovalPolicy;

#[async_trait]
impl PermissionPolicy for FinanceApprovalPolicy {
    async fn check(&self, ctx: &PermissionContext<'_>) -> Permission {
        if ctx.annotations.destructive {
            Permission::AskUser {
                prompt: Some(approval_prompt(ctx.tool_name, ctx.tool_input)),
            }
        } else {
            Permission::Allow
        }
    }
}

fn approval_prompt(tool_name: &str, input: &serde_json::Value) -> String {
    if tool_name == "place_order" {
        if let (Some(side), Some(quantity), Some(symbol)) = (
            input.get("side").and_then(serde_json::Value::as_str),
            input.get("quantity").and_then(serde_json::Value::as_u64),
            input.get("symbol").and_then(serde_json::Value::as_str),
        ) {
            return format!(
                "Approve {side} {quantity} {}?",
                symbol.trim().to_ascii_uppercase()
            );
        }
    }

    format!("Approve call to tool '{tool_name}'?")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tools::test_support::block_on;
    use motosan_agent_primitives::{PermissionMode, ToolAnnotations};
    use serde_json::json;

    fn ctx<'a>(
        tool_name: &'a str,
        input: &'a serde_json::Value,
        annotations: &'a ToolAnnotations,
    ) -> PermissionContext<'a> {
        PermissionContext {
            session_id: "test-session",
            tool_use_id: "call-1",
            tool_name,
            tool_input: input,
            annotations,
            mode: PermissionMode::AcceptEdits,
            // M10 D-M10-3: PermissionContext gained `recent_messages` in
            // primitives 0.2.0. Empty slice = cold start, sufficient for
            // policy tests that don't depend on conversation history.
            recent_messages: &[],
        }
    }

    #[test]
    fn destructive_tool_asks_user_with_order_prompt() {
        let input = json!({ "symbol": "aapl", "side": "buy", "quantity": 10 });
        let annotations = ToolAnnotations {
            read_only: false,
            destructive: true,
            network_access: true,
            idempotent: false,
        };

        let permission =
            block_on(FinanceApprovalPolicy.check(&ctx("place_order", &input, &annotations)));

        assert_eq!(
            permission,
            Permission::AskUser {
                prompt: Some("Approve buy 10 AAPL?".into())
            }
        );
    }

    #[test]
    fn malformed_destructive_tool_asks_user_with_fallback_prompt() {
        let input = json!({ "symbol": "AAPL" });
        let annotations = ToolAnnotations {
            read_only: false,
            destructive: true,
            network_access: true,
            idempotent: false,
        };

        let permission =
            block_on(FinanceApprovalPolicy.check(&ctx("place_order", &input, &annotations)));

        assert_eq!(
            permission,
            Permission::AskUser {
                prompt: Some("Approve call to tool 'place_order'?".into())
            }
        );
    }

    #[test]
    fn read_only_tool_is_allowed() {
        let input = json!({ "symbol": "AAPL" });
        let annotations = ToolAnnotations {
            read_only: true,
            destructive: false,
            network_access: true,
            idempotent: false,
        };

        let permission =
            block_on(FinanceApprovalPolicy.check(&ctx("get_quote", &input, &annotations)));

        assert_eq!(permission, Permission::Allow);
    }
}
