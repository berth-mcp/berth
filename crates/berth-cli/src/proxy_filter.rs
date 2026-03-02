// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2026 Schwimmbeck Dominik

//! Exec-permission filtering for MCP proxy sessions.
//!
//! When a server declares specific exec permissions, the proxy intercepts
//! `tools/call` requests and blocks any tool name not in the allowed set.

use std::collections::HashSet;
use std::io::{self, BufRead, BufReader, Write};

/// Result of filtering a single JSON-RPC line from the client.
pub enum FilterResult {
    /// Forward the line unchanged to the server.
    Forward(String),
    /// Deny the request and return an error response to the client.
    Deny(String),
    /// Not a tool call — pass through unchanged.
    PassThrough(String),
}

/// Filters MCP `tools/call` requests based on declared exec permissions.
pub struct ProxyFilter {
    allowed: Option<HashSet<String>>,
}

impl ProxyFilter {
    /// Creates a new filter from declared exec permissions.
    ///
    /// An empty list or a list containing `"*"` allows all tool calls.
    pub fn new(exec_permissions: &[String]) -> Self {
        if exec_permissions.is_empty() || exec_permissions.iter().any(|p| p == "*") {
            Self { allowed: None }
        } else {
            Self {
                allowed: Some(exec_permissions.iter().cloned().collect()),
            }
        }
    }

    /// Returns `true` if the filter allows all tool calls (no restrictions).
    pub fn allows_all(&self) -> bool {
        self.allowed.is_none()
    }

    /// Inspects a line from the client and decides whether to forward or deny.
    pub fn filter_message(&self, line: &str) -> FilterResult {
        let allowed = match &self.allowed {
            Some(set) => set,
            None => return FilterResult::PassThrough(line.to_string()),
        };

        let value: serde_json::Value = match serde_json::from_str(line.trim()) {
            Ok(v) => v,
            Err(_) => return FilterResult::PassThrough(line.to_string()),
        };

        let method = match value.get("method").and_then(|m| m.as_str()) {
            Some(m) => m,
            None => return FilterResult::PassThrough(line.to_string()),
        };

        if method != "tools/call" {
            return FilterResult::Forward(line.to_string());
        }

        let tool_name = value
            .get("params")
            .and_then(|p| p.get("name"))
            .and_then(|n| n.as_str())
            .unwrap_or("");

        if allowed.contains(tool_name) {
            FilterResult::Forward(line.to_string())
        } else {
            let id = &value["id"];
            let denial = build_denial_response(id, tool_name);
            FilterResult::Deny(denial)
        }
    }
}

/// Builds a JSON-RPC error response for a denied tool call.
fn build_denial_response(id: &serde_json::Value, tool_name: &str) -> String {
    serde_json::json!({
        "jsonrpc": "2.0",
        "id": id,
        "error": {
            "code": -32600,
            "message": format!("Tool '{}' is not allowed by exec permissions", tool_name)
        }
    })
    .to_string()
}

/// Runs the filtered proxy, forwarding messages between client (stdin/stdout)
/// and the server process (piped stdin/stdout).
///
/// Client→server messages are filtered; server→client messages pass through.
pub fn run_filtered_proxy(
    server_stdin: std::process::ChildStdin,
    server_stdout: std::process::ChildStdout,
    filter: &ProxyFilter,
) -> io::Result<Vec<String>> {
    let mut denied_tools = Vec::new();
    let mut server_writer = io::BufWriter::new(server_stdin);

    // Server→client relay in a background thread
    let handle = std::thread::spawn(move || {
        let reader = BufReader::new(server_stdout);
        let mut stdout = io::stdout().lock();
        for line in reader.lines() {
            match line {
                Ok(l) => {
                    let _ = writeln!(stdout, "{l}");
                    let _ = stdout.flush();
                }
                Err(_) => break,
            }
        }
    });

    // Client→server relay with filtering
    let stdin = io::stdin().lock();
    let reader = BufReader::new(stdin);
    for line in reader.lines() {
        let line = match line {
            Ok(l) => l,
            Err(_) => break,
        };

        match filter.filter_message(&line) {
            FilterResult::Forward(msg) | FilterResult::PassThrough(msg) => {
                writeln!(server_writer, "{msg}")?;
                server_writer.flush()?;
            }
            FilterResult::Deny(err_json) => {
                // Extract tool name for audit
                if let Ok(val) = serde_json::from_str::<serde_json::Value>(line.trim()) {
                    if let Some(name) = val
                        .get("params")
                        .and_then(|p| p.get("name"))
                        .and_then(|n| n.as_str())
                    {
                        denied_tools.push(name.to_string());
                    }
                }
                let mut stdout = io::stdout().lock();
                let _ = writeln!(stdout, "{err_json}");
                let _ = stdout.flush();
            }
        }
    }

    let _ = handle.join();
    Ok(denied_tools)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn passthrough_when_no_restrictions() {
        let filter = ProxyFilter::new(&[]);
        assert!(filter.allows_all());
        let msg = r#"{"jsonrpc":"2.0","method":"tools/call","id":1,"params":{"name":"anything"}}"#;
        match filter.filter_message(msg) {
            FilterResult::PassThrough(_) => {}
            _ => panic!("expected PassThrough"),
        }
    }

    #[test]
    fn wildcard_allows_all() {
        let filter = ProxyFilter::new(&["*".to_string()]);
        assert!(filter.allows_all());
    }

    #[test]
    fn allowed_tool_call_forwarded() {
        let filter = ProxyFilter::new(&["read_file".to_string(), "write_file".to_string()]);
        let msg = r#"{"jsonrpc":"2.0","method":"tools/call","id":1,"params":{"name":"read_file"}}"#;
        match filter.filter_message(msg) {
            FilterResult::Forward(_) => {}
            _ => panic!("expected Forward"),
        }
    }

    #[test]
    fn denied_tool_call_returns_error() {
        let filter = ProxyFilter::new(&["read_file".to_string()]);
        let msg = r#"{"jsonrpc":"2.0","method":"tools/call","id":42,"params":{"name":"exec_cmd"}}"#;
        match filter.filter_message(msg) {
            FilterResult::Deny(err) => {
                let v: serde_json::Value = serde_json::from_str(&err).unwrap();
                assert_eq!(v["id"], 42);
                assert_eq!(v["error"]["code"], -32600);
                assert!(v["error"]["message"].as_str().unwrap().contains("exec_cmd"));
            }
            _ => panic!("expected Deny"),
        }
    }

    #[test]
    fn non_tool_call_methods_pass_through() {
        let filter = ProxyFilter::new(&["read_file".to_string()]);
        let msg = r#"{"jsonrpc":"2.0","method":"initialize","id":1,"params":{}}"#;
        match filter.filter_message(msg) {
            FilterResult::Forward(_) => {}
            _ => panic!("expected Forward for non-tool-call method"),
        }
    }
}
