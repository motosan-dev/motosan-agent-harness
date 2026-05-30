//! `TwoToolHarness` — a [`Harness`] exposing two stub
//! [`Tool`](motosan_agent_tool::Tool) implementations.
//!
//! Demonstrates the expected shape of a vertical bundle:
//!
//! - tool names are namespaced (`demo.echo`, `demo.add`) per the
//!   collision-avoidance guidance in the [`Harness`] docs,
//! - the harness owns its tools as `Arc`s and clones them out cheaply,
//! - a system prompt is provided so the persona is visible end-to-end.

use std::sync::Arc;

use async_trait::async_trait;
use motosan_agent_harness::Harness;
use motosan_agent_tool::{Tool, ToolAnnotations, ToolContext, ToolDef, ToolOutput, Value};

// ---------------------------------------------------------------------------
// Stub tools
// ---------------------------------------------------------------------------

/// Echoes back its `message` argument as text.
struct EchoTool;

#[async_trait]
impl Tool for EchoTool {
    fn def(&self) -> ToolDef {
        ToolDef::new(
            "demo_echo",
            "Echo the input message back to the caller.",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "message": { "type": "string" }
                },
                "required": ["message"]
            }),
        )
        .with_internal_name("demo.echo")
    }

    fn annotations(&self) -> ToolAnnotations {
        ToolAnnotations {
            read_only: true,
            destructive: false,
            network_access: false,
            idempotent: true,
        }
    }

    async fn call(&self, args: Value, _ctx: &ToolContext) -> ToolOutput {
        let msg = args.get("message").and_then(Value::as_str).unwrap_or("");
        ToolOutput::text(msg)
    }
}

/// Adds two integers `a` and `b`.
struct AddTool;

#[async_trait]
impl Tool for AddTool {
    fn def(&self) -> ToolDef {
        ToolDef::new(
            "demo_add",
            "Add two integers a and b.",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "a": { "type": "integer" },
                    "b": { "type": "integer" }
                },
                "required": ["a", "b"]
            }),
        )
        .with_internal_name("demo.add")
    }

    fn annotations(&self) -> ToolAnnotations {
        ToolAnnotations {
            read_only: true,
            destructive: false,
            network_access: false,
            idempotent: true,
        }
    }

    async fn call(&self, args: Value, _ctx: &ToolContext) -> ToolOutput {
        let a = args.get("a").and_then(Value::as_i64).unwrap_or(0);
        let b = args.get("b").and_then(Value::as_i64).unwrap_or(0);
        ToolOutput::text((a + b).to_string())
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
