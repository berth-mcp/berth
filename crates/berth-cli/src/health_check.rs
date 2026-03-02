// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2026 Schwimmbeck Dominik

//! MCP protocol health checks via JSON-RPC initialize handshake.

use std::collections::BTreeMap;
use std::io::{BufRead, BufReader, Write};
use std::process::{Command, Stdio};
use std::sync::mpsc;
use std::time::Duration;

/// Result of an MCP protocol health probe.
#[derive(Debug)]
pub struct HealthCheckResult {
    pub mcp_responsive: Option<bool>,
    pub protocol_version: Option<String>,
    pub server_info: Option<String>,
    pub error: Option<String>,
}

const INITIALIZE_REQUEST: &str = r#"{"jsonrpc":"2.0","method":"initialize","id":1,"params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"berth","version":"0.1.0"}}}"#;

/// Spawns a short-lived process and sends the MCP `initialize` handshake.
///
/// Returns whether the server responded with a valid JSON-RPC result.
pub fn probe_mcp_health(
    command: &str,
    args: &[String],
    env: &BTreeMap<String, String>,
    timeout: Duration,
) -> HealthCheckResult {
    let mut child = match Command::new(command)
        .args(args)
        .envs(env)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
    {
        Ok(c) => c,
        Err(e) => {
            return HealthCheckResult {
                mcp_responsive: Some(false),
                protocol_version: None,
                server_info: None,
                error: Some(format!("spawn failed: {e}")),
            };
        }
    };

    // Write initialize request
    if let Some(mut stdin) = child.stdin.take() {
        let msg = format!("{INITIALIZE_REQUEST}\n");
        let _ = stdin.write_all(msg.as_bytes());
        let _ = stdin.flush();
    }

    // Read response with timeout in a separate thread
    let (tx, rx) = mpsc::channel();
    let stdout = child.stdout.take();
    std::thread::spawn(move || {
        if let Some(stdout) = stdout {
            let mut reader = BufReader::new(stdout);
            let mut line = String::new();
            if reader.read_line(&mut line).is_ok() && !line.is_empty() {
                let _ = tx.send(line);
            }
        }
    });

    let result = match rx.recv_timeout(timeout) {
        Ok(line) => match parse_initialize_response(&line) {
            Some((version, info)) => HealthCheckResult {
                mcp_responsive: Some(true),
                protocol_version: Some(version),
                server_info: Some(info),
                error: None,
            },
            None => HealthCheckResult {
                mcp_responsive: Some(false),
                protocol_version: None,
                server_info: None,
                error: Some("invalid response".to_string()),
            },
        },
        Err(_) => HealthCheckResult {
            mcp_responsive: Some(false),
            protocol_version: None,
            server_info: None,
            error: Some("timeout".to_string()),
        },
    };

    let _ = child.kill();
    let _ = child.wait();
    result
}

/// Extracts protocol version and server name from a JSON-RPC initialize response.
fn parse_initialize_response(line: &str) -> Option<(String, String)> {
    let value: serde_json::Value = serde_json::from_str(line.trim()).ok()?;
    let result = value.get("result")?;
    let version = result
        .get("protocolVersion")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown")
        .to_string();
    let info = result
        .get("serverInfo")
        .and_then(|v| v.get("name"))
        .and_then(|v| v.as_str())
        .unwrap_or("unknown")
        .to_string();
    Some((version, info))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_valid_initialize_response() {
        let resp = r#"{"jsonrpc":"2.0","id":1,"result":{"protocolVersion":"2024-11-05","capabilities":{},"serverInfo":{"name":"test-server","version":"1.0"}}}"#;
        let (version, info) = parse_initialize_response(resp).unwrap();
        assert_eq!(version, "2024-11-05");
        assert_eq!(info, "test-server");
    }

    #[cfg(unix)]
    #[test]
    fn probe_timeout_returns_error() {
        let result = probe_mcp_health(
            "sh",
            &["-c".to_string(), "sleep 60".to_string()],
            &BTreeMap::new(),
            Duration::from_millis(100),
        );
        assert_eq!(result.mcp_responsive, Some(false));
        assert_eq!(result.error.as_deref(), Some("timeout"));
    }
}
