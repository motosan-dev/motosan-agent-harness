use std::{
    fs::{File, OpenOptions},
    io::Write,
    path::PathBuf,
    sync::{Arc, Mutex},
};

use async_trait::async_trait;
use chrono::Utc;
use motosan_agent_primitives::{
    Hook, HookResult, PostToolUseCtx, PostToolUseFailureCtx, SessionEndCtx, SessionStartCtx,
};
use serde_json::json;

pub struct AuditLogHook {
    file: Arc<Mutex<File>>,
}

impl AuditLogHook {
    pub fn new(path: PathBuf) -> Self {
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)
            .unwrap_or_else(|err| {
                panic!("failed to open finance audit log {}: {err}", path.display())
            });

        Self {
            file: Arc::new(Mutex::new(file)),
        }
    }

    fn write_record(&self, mut record: serde_json::Value) {
        if let Some(obj) = record.as_object_mut() {
            obj.insert(
                "ts".into(),
                json!(Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true)),
            );
        }

        let Ok(line) = serde_json::to_string(&record) else {
            return;
        };
        let Ok(mut file) = self.file.lock() else {
            return;
        };
        let _ = writeln!(file, "{line}");
        let _ = file.flush();
    }
}

#[async_trait]
impl Hook for AuditLogHook {
    async fn session_start(&self, ctx: &SessionStartCtx) -> HookResult {
        self.write_record(json!({
            "event": "session_start",
            "session_id": ctx.session_id,
        }));
        HookResult::cont()
    }

    async fn post_tool_use(&self, ctx: &PostToolUseCtx) -> HookResult {
        self.write_record(json!({
            "event": "post_tool_use",
            "session_id": ctx.session_id,
            "tool": ctx.tool_call.name,
            "args": ctx.tool_call.input,
            "result": ctx.tool_result,
            "is_error": false,
        }));
        HookResult::cont()
    }

    async fn post_tool_use_failure(&self, ctx: &PostToolUseFailureCtx) -> HookResult {
        self.write_record(json!({
            "event": "post_tool_use_failure",
            "session_id": ctx.session_id,
            "tool": ctx.tool_call.name,
            "args": ctx.tool_call.input,
            "result": ctx.result,
            "is_error": true,
        }));
        HookResult::cont()
    }

    async fn session_end(&self, ctx: &SessionEndCtx) -> HookResult {
        self.write_record(json!({
            "event": "session_end",
            "session_id": ctx.session_id,
            "stop_reason": ctx.stop_reason,
        }));
        HookResult::cont()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tools::test_support::block_on;
    use motosan_agent_primitives::{ContentBlock, StopReason, ToolCall, ToolFailure, ToolResult};
    use serde_json::json;
    use std::{
        fs,
        path::PathBuf,
        time::{SystemTime, UNIX_EPOCH},
    };

    fn temp_log_path(name: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time")
            .as_nanos();
        std::env::temp_dir().join(format!("finance-audit-{name}-{nanos}.jsonl"))
    }

    #[test]
    fn writes_session_and_success_jsonl_records() {
        let path = temp_log_path("success");
        let hook = AuditLogHook::new(path.clone());

        block_on(hook.session_start(&SessionStartCtx {
            session_id: "session-1".into(),
            cancellation_token: Default::default(),
        }));
        block_on(hook.post_tool_use(&PostToolUseCtx {
            session_id: "session-1".into(),
            tool_call: ToolCall {
                id: "call-1".into(),
                name: "get_quote".into(),
                input: json!({ "symbol": "AAPL" }),
            },
            tool_result: ToolResult {
                tool_use_id: "call-1".into(),
                content: vec![ContentBlock::Json {
                    value: json!({ "symbol": "AAPL", "price": 185.0 }),
                }],
                is_error: false,
            },
            cancellation_token: Default::default(),
        }));
        block_on(hook.session_end(&SessionEndCtx {
            session_id: "session-1".into(),
            stop_reason: StopReason::Completed,
            cancellation_token: Default::default(),
        }));

        let lines: Vec<serde_json::Value> = fs::read_to_string(&path)
            .expect("read log")
            .lines()
            .map(|line| serde_json::from_str(line).expect("json line"))
            .collect();
        let _ = fs::remove_file(path);

        assert_eq!(lines.len(), 3);
        assert_eq!(lines[0]["event"], "session_start");
        assert!(lines[0]["ts"].as_str().unwrap().ends_with('Z'));
        assert_eq!(lines[1]["event"], "post_tool_use");
        assert_eq!(lines[1]["tool"], "get_quote");
        assert_eq!(lines[1]["args"], json!({ "symbol": "AAPL" }));
        assert_eq!(lines[1]["is_error"], false);
        assert_eq!(lines[2]["event"], "session_end");
    }

    #[test]
    fn writes_failure_jsonl_record() {
        let path = temp_log_path("failure");
        let hook = AuditLogHook::new(path.clone());

        block_on(hook.post_tool_use_failure(&PostToolUseFailureCtx {
            session_id: "session-1".into(),
            tool_call: ToolCall {
                id: "call-2".into(),
                name: "place_order".into(),
                input: json!({ "symbol": "AAPL", "side": "buy", "quantity": 10 }),
            },
            failure: ToolFailure::Error {
                message: "permission denied".into(),
            },
            result: ToolResult {
                tool_use_id: "call-2".into(),
                content: vec![ContentBlock::Text { text: "permission denied".into() }],
                is_error: true,
            },
            cancellation_token: Default::default(),
        }));

        let line = fs::read_to_string(&path).expect("read log");
        let _ = fs::remove_file(path);
        let value: serde_json::Value = serde_json::from_str(line.trim()).expect("json line");

        assert_eq!(value["event"], "post_tool_use_failure");
        assert_eq!(value["tool"], "place_order");
        assert_eq!(value["is_error"], true);
        // AWK#3 / D-M10-2: result is the real ToolResult, not a {failure} wrapper.
        assert_eq!(value["result"]["is_error"], true);
        assert_eq!(value["result"]["content"][0]["text"], "permission denied");
    }
}
