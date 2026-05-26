//! `TwoToolHarness` — a [`Harness`] exposing two stub
//! [`Tool`](motosan_agent_tool::Tool) implementations.
//!
//! Demonstrates the expected shape of a vertical bundle:
//!
//! - tool names are namespaced (`demo.echo`, `demo.add`) per the
//!   collision-avoidance guidance in the [`Harness`] docs,
//! - the harness owns its tools as `Arc`s and clones them out cheaply,
//! - a system prompt is provided so the persona is visible end-to-end.

use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use motosan_agent_harness::Harness;
use motosan_agent_tool::{Tool, ToolContext, ToolDef, ToolResult, Value};

// ---------------------------------------------------------------------------
// Stub tools
// ---------------------------------------------------------------------------

/// Echoes back its `message` argument as text.
struct EchoTool;

impl Tool for EchoTool {
    fn def(&self) -> ToolDef {
        ToolDef {
            name: "demo.echo".into(),
            description: "Echo the input message back to the caller.".into(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "message": { "type": "string" }
                },
                "required": ["message"]
            }),
        }
    }

    fn call(
        &self,
        args: Value,
        _ctx: &ToolContext,
    ) -> Pin<Box<dyn Future<Output = ToolResult> + Send + '_>> {
        Box::pin(async move {
            let msg = args
                .get("message")
                .and_then(Value::as_str)
                .unwrap_or("")
                .to_string();
            ToolResult::text(msg)
        })
    }
}

/// Adds two integers `a` and `b`.
struct AddTool;

impl Tool for AddTool {
    fn def(&self) -> ToolDef {
        ToolDef {
            name: "demo.add".into(),
            description: "Add two integers a and b.".into(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "a": { "type": "integer" },
                    "b": { "type": "integer" }
                },
                "required": ["a", "b"]
            }),
        }
    }

    fn call(
        &self,
        args: Value,
        _ctx: &ToolContext,
    ) -> Pin<Box<dyn Future<Output = ToolResult> + Send + '_>> {
        Box::pin(async move {
            let a = args.get("a").and_then(Value::as_i64).unwrap_or(0);
            let b = args.get("b").and_then(Value::as_i64).unwrap_or(0);
            ToolResult::json(serde_json::json!({ "sum": a + b }))
        })
    }
}

// ---------------------------------------------------------------------------
// Harness
// ---------------------------------------------------------------------------

struct TwoToolHarness {
    tools: Vec<Arc<dyn Tool>>,
}

impl TwoToolHarness {
    fn new() -> Self {
        Self {
            tools: vec![Arc::new(EchoTool), Arc::new(AddTool)],
        }
    }
}

impl Harness for TwoToolHarness {
    fn name(&self) -> &str {
        "demo"
    }

    fn system_prompt(&self) -> Option<String> {
        Some("You are a demo agent with two trivial tools: echo and add.".into())
    }

    fn tools(&self) -> Vec<Arc<dyn Tool>> {
        self.tools.clone()
    }
}

fn main() {
    let h: Arc<dyn Harness> = Arc::new(TwoToolHarness::new());
    println!("harness name : {}", h.name());
    println!(
        "system prompt: {}",
        h.system_prompt().unwrap_or_else(|| "<none>".into())
    );
    println!("tools        :");
    for tool in h.tools() {
        let def = tool.def();
        println!("  - {} — {}", def.name, def.description);
    }
}
