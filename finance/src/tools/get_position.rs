use async_trait::async_trait;
use motosan_agent_tool::{Tool, ToolAnnotations, ToolContext, ToolDef, ToolOutput, Value};
use serde_json::json;

use super::{normalize_symbol, position_for, required_string};

pub struct GetPositionTool;

#[async_trait]
impl Tool for GetPositionTool {
    fn def(&self) -> ToolDef {
        ToolDef {
            name: "get_position".into(),
            description: "Return current mock portfolio holdings and cost basis for a stock symbol. Unknown symbols return a flat position.".into(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "symbol": { "type": "string", "description": "Ticker symbol, e.g. AAPL" }
                },
                "required": ["symbol"]
            }),
        }
    }

    fn annotations(&self) -> ToolAnnotations {
        ToolAnnotations {
            read_only: true,
            destructive: false,
            network_access: true,
            idempotent: true,
        }
    }

    async fn call(&self, args: Value, _ctx: &ToolContext) -> ToolOutput {
        let symbol = match required_string(&args, "symbol") {
            Ok(symbol) => normalize_symbol(&symbol),
            Err(message) => return ToolOutput::error(message),
        };
        let (quantity, cost_basis) = position_for(&symbol);

        ToolOutput::json(json!({
            "symbol": symbol,
            "quantity": quantity,
            "cost_basis": cost_basis,
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tools::test_support::block_on;

    #[test]
    fn happy_path_returns_position_json() {
        let out =
            block_on(GetPositionTool.call(json!({ "symbol": "AAPL" }), &ToolContext::default()));

        assert!(!out.is_error);
        let value = out.as_json().expect("json output");
        assert_eq!(value["symbol"], "AAPL");
        assert_eq!(value["quantity"], 50);
        assert_eq!(value["cost_basis"], 150.0);
    }

    #[test]
    fn unknown_symbol_returns_flat_position() {
        let out =
            block_on(GetPositionTool.call(json!({ "symbol": "XYZ" }), &ToolContext::default()));

        assert!(!out.is_error);
        let value = out.as_json().expect("json output");
        assert_eq!(value["symbol"], "XYZ");
        assert_eq!(value["quantity"], 0);
        assert_eq!(value["cost_basis"], 0.0);
    }

    #[test]
    fn bad_args_are_error() {
        let out = block_on(GetPositionTool.call(json!({}), &ToolContext::default()));

        assert!(out.is_error);
        assert_eq!(
            out.as_text(),
            Some("bad args: missing or invalid string field 'symbol'")
        );
    }
}
