use async_trait::async_trait;
use motosan_agent_tool::{Tool, ToolAnnotations, ToolContext, ToolDef, ToolOutput, Value};
use serde_json::json;

use super::{normalize_symbol, quote_for, required_string};

pub struct PlaceOrderTool;

#[async_trait]
impl Tool for PlaceOrderTool {
    fn def(&self) -> ToolDef {
        ToolDef {
            name: "place_order".into(),
            description: "Place a mock stock order. This is destructive and always requires human approval before execution.".into(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "symbol": { "type": "string", "description": "Ticker symbol, e.g. AAPL" },
                    "side": { "type": "string", "enum": ["buy", "sell"] },
                    "quantity": { "type": "integer", "minimum": 1 },
                    "max_price": { "type": "number", "description": "Optional maximum execution price for buy orders" }
                },
                "required": ["symbol", "side", "quantity"]
            }),
        }
    }

    fn annotations(&self) -> ToolAnnotations {
        ToolAnnotations {
            read_only: false,
            destructive: true,
            network_access: true,
            idempotent: false,
        }
    }

    async fn call(&self, args: Value, _ctx: &ToolContext) -> ToolOutput {
        let symbol = match required_string(&args, "symbol") {
            Ok(symbol) => normalize_symbol(&symbol),
            Err(message) => return ToolOutput::error(message),
        };
        let side = match required_string(&args, "side") {
            Ok(side) if side == "buy" || side == "sell" => side,
            Ok(_) => return ToolOutput::error("bad args: side must be 'buy' or 'sell'"),
            Err(message) => return ToolOutput::error(message),
        };
        let quantity = match args.get("quantity").and_then(Value::as_u64) {
            Some(q) if q > 0 && q <= u32::MAX as u64 => q as u32,
            _ => return ToolOutput::error("bad args: quantity must be a positive integer"),
        };
        let max_price = match args.get("max_price") {
            None | Some(Value::Null) => None,
            Some(value) => match value.as_f64() {
                Some(price) if price.is_finite() && price > 0.0 => Some(price),
                _ => return ToolOutput::error("bad args: max_price must be a positive number"),
            },
        };

        let Some(price) = quote_for(&symbol) else {
            return ToolOutput::error(format!("unknown symbol: {symbol}"));
        };

        let status = if side == "buy" && max_price.is_some_and(|limit| price > limit) {
            "rejected"
        } else {
            "filled"
        };

        ToolOutput::json(json!({
            "order_id": format!("mock-{side}-{quantity}-{symbol}"),
            "status": status,
            "executed_price": price,
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tools::test_support::block_on;

    #[test]
    fn happy_path_fills_buy_order() {
        let out = block_on(PlaceOrderTool.call(
            json!({ "symbol": "AAPL", "side": "buy", "quantity": 10, "max_price": 200.0 }),
            &ToolContext::default(),
        ));

        assert!(!out.is_error);
        let value = out.as_json().expect("json output");
        assert_eq!(value["order_id"], "mock-buy-10-AAPL");
        assert_eq!(value["status"], "filled");
        assert_eq!(value["executed_price"], 185.0);
    }

    #[test]
    fn buy_order_over_limit_is_rejected() {
        let out = block_on(PlaceOrderTool.call(
            json!({ "symbol": "AAPL", "side": "buy", "quantity": 10, "max_price": 100.0 }),
            &ToolContext::default(),
        ));

        assert!(!out.is_error);
        let value = out.as_json().expect("json output");
        assert_eq!(value["status"], "rejected");
        assert_eq!(value["executed_price"], 185.0);
    }

    #[test]
    fn unknown_symbol_is_error() {
        let out = block_on(PlaceOrderTool.call(
            json!({ "symbol": "XYZ", "side": "buy", "quantity": 1 }),
            &ToolContext::default(),
        ));

        assert!(out.is_error);
        assert_eq!(out.as_text(), Some("unknown symbol: XYZ"));
    }

    #[test]
    fn bad_args_are_error() {
        let out = block_on(PlaceOrderTool.call(
            json!({ "symbol": "AAPL", "side": "hold", "quantity": 1 }),
            &ToolContext::default(),
        ));

        assert!(out.is_error);
        assert_eq!(
            out.as_text(),
            Some("bad args: side must be 'buy' or 'sell'")
        );
    }
}
