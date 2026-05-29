use async_trait::async_trait;
use chrono::Utc;
use motosan_agent_tool::{Tool, ToolAnnotations, ToolContext, ToolDef, ToolOutput, Value};
use serde_json::json;

use super::{normalize_symbol, quote_for, required_string};

pub struct GetQuoteTool;

#[async_trait]
impl Tool for GetQuoteTool {
    fn def(&self) -> ToolDef {
        // M10 D-M10-4: keep the LLM-facing `name` short and unqualified, but
        // use a namespaced `internal_name` host-side to avoid collisions when
        // multiple harnesses are stacked. AWK#4 resolved.
        ToolDef::new(
            "get_quote",
            "Return the current mock market price for a stock symbol.",
            json!({
                "type": "object",
                "properties": {
                    "symbol": { "type": "string", "description": "Ticker symbol, e.g. AAPL" }
                },
                "required": ["symbol"]
            }),
        )
        .with_internal_name("finance.get_quote")
    }

    fn annotations(&self) -> ToolAnnotations {
        ToolAnnotations {
            read_only: true,
            destructive: false,
            network_access: true,
            idempotent: false,
        }
    }

    async fn call(&self, args: Value, _ctx: &ToolContext) -> ToolOutput {
        let symbol = match required_string(&args, "symbol") {
            Ok(symbol) => normalize_symbol(&symbol),
            Err(message) => return ToolOutput::error(message),
        };

        let Some(price) = quote_for(&symbol) else {
            return ToolOutput::error(format!("unknown symbol: {symbol}"));
        };

        ToolOutput::json(json!({
            "symbol": symbol,
            "price": price,
            "timestamp": Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tools::test_support::block_on;

    #[test]
    fn happy_path_returns_quote_json() {
        let out = block_on(GetQuoteTool.call(json!({ "symbol": "aapl" }), &ToolContext::default()));

        assert!(!out.is_error);
        let value = out.as_json().expect("json output");
        assert_eq!(value["symbol"], "AAPL");
        assert_eq!(value["price"], 185.0);
        assert!(value["timestamp"].as_str().unwrap().ends_with('Z'));
    }

    #[test]
    fn unknown_symbol_is_error() {
        let out = block_on(GetQuoteTool.call(json!({ "symbol": "XYZ" }), &ToolContext::default()));

        assert!(out.is_error);
        assert_eq!(out.as_text(), Some("unknown symbol: XYZ"));
    }

    #[test]
    fn bad_args_are_error() {
        let out = block_on(GetQuoteTool.call(json!({ "symbol": 123 }), &ToolContext::default()));

        assert!(out.is_error);
        assert_eq!(
            out.as_text(),
            Some("bad args: missing or invalid string field 'symbol'")
        );
    }
}
