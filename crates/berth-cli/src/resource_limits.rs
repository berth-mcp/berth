// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2026 Schwimmbeck Dominik

//! Process resource limits for MCP server sandboxing.
//!
//! Supports memory and file-descriptor limits via `setrlimit` on Unix.

use std::collections::BTreeMap;

/// Parsed resource-limit configuration.
#[derive(Debug, Default)]
pub struct ResourceLimits {
    pub max_memory_bytes: Option<u64>,
    pub max_file_descriptors: Option<u64>,
}

pub const KEY_MAX_MEMORY: &str = "berth.max-memory";
pub const KEY_MAX_FILE_DESCRIPTORS: &str = "berth.max-file-descriptors";

/// Returns `true` if `key` is a recognised resource-limit config key.
pub fn is_resource_limit_key(key: &str) -> bool {
    matches!(key, KEY_MAX_MEMORY | KEY_MAX_FILE_DESCRIPTORS)
}

/// Validates a resource-limit config value before persistence.
pub fn validate_resource_limit_value(key: &str, value: &str) -> Result<(), String> {
    match key {
        KEY_MAX_MEMORY => {
            parse_memory_value(value)?;
            Ok(())
        }
        KEY_MAX_FILE_DESCRIPTORS => {
            value
                .trim()
                .parse::<u64>()
                .map_err(|_| format!("Invalid value for {key}: expected a positive integer"))?;
            Ok(())
        }
        _ => Ok(()),
    }
}

/// Parses a memory value with optional K/M/G suffix into bytes.
pub fn parse_memory_value(s: &str) -> Result<u64, String> {
    let s = s.trim();
    if s.is_empty() {
        return Err("Empty memory value".to_string());
    }

    let (num_str, multiplier) = if let Some(n) = s.strip_suffix('G') {
        (n, 1024 * 1024 * 1024)
    } else if let Some(n) = s.strip_suffix('M') {
        (n, 1024 * 1024)
    } else if let Some(n) = s.strip_suffix('K') {
        (n, 1024)
    } else {
        (s, 1u64)
    };

    let num: u64 = num_str
        .trim()
        .parse()
        .map_err(|_| format!("Invalid memory value: {s}"))?;
    Ok(num * multiplier)
}

/// Parses resource limits from a server config map.
pub fn parse_resource_limits(config: &BTreeMap<String, String>) -> Result<ResourceLimits, String> {
    let max_memory_bytes = config
        .get(KEY_MAX_MEMORY)
        .filter(|v| !v.trim().is_empty())
        .map(|v| parse_memory_value(v))
        .transpose()?;

    let max_file_descriptors = config
        .get(KEY_MAX_FILE_DESCRIPTORS)
        .filter(|v| !v.trim().is_empty())
        .map(|v| {
            v.trim()
                .parse::<u64>()
                .map_err(|_| format!("Invalid value for {KEY_MAX_FILE_DESCRIPTORS}: {v}"))
        })
        .transpose()?;

    Ok(ResourceLimits {
        max_memory_bytes,
        max_file_descriptors,
    })
}

/// Applies resource limits to a command via `pre_exec` + `setrlimit` on Unix.
#[cfg(unix)]
pub fn apply_resource_limits(cmd: &mut std::process::Command, limits: &ResourceLimits) {
    use std::os::unix::process::CommandExt;

    let mem = limits.max_memory_bytes;
    let fds = limits.max_file_descriptors;
    if mem.is_none() && fds.is_none() {
        return;
    }

    unsafe {
        cmd.pre_exec(move || {
            if let Some(bytes) = mem {
                let rlim = libc::rlimit {
                    rlim_cur: bytes,
                    rlim_max: bytes,
                };
                if libc::setrlimit(libc::RLIMIT_AS, &rlim) != 0 {
                    return Err(std::io::Error::last_os_error());
                }
            }
            if let Some(n) = fds {
                let rlim = libc::rlimit {
                    rlim_cur: n,
                    rlim_max: n,
                };
                if libc::setrlimit(libc::RLIMIT_NOFILE, &rlim) != 0 {
                    return Err(std::io::Error::last_os_error());
                }
            }
            Ok(())
        });
    }
}

/// No-op on non-Unix platforms.
#[cfg(not(unix))]
pub fn apply_resource_limits(_cmd: &mut std::process::Command, _limits: &ResourceLimits) {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_memory_value_handles_suffixes() {
        assert_eq!(parse_memory_value("512M").unwrap(), 512 * 1024 * 1024);
        assert_eq!(parse_memory_value("1G").unwrap(), 1024 * 1024 * 1024);
        assert_eq!(parse_memory_value("64K").unwrap(), 64 * 1024);
        assert_eq!(parse_memory_value("1048576").unwrap(), 1048576);
    }

    #[test]
    fn parse_memory_value_rejects_invalid() {
        assert!(parse_memory_value("abc").is_err());
        assert!(parse_memory_value("").is_err());
    }

    #[test]
    fn is_resource_limit_key_works() {
        assert!(is_resource_limit_key(KEY_MAX_MEMORY));
        assert!(is_resource_limit_key(KEY_MAX_FILE_DESCRIPTORS));
        assert!(!is_resource_limit_key("token"));
    }

    #[test]
    fn parse_resource_limits_from_config() {
        let mut config = BTreeMap::new();
        config.insert(KEY_MAX_MEMORY.to_string(), "256M".to_string());
        config.insert(KEY_MAX_FILE_DESCRIPTORS.to_string(), "1024".to_string());
        let limits = parse_resource_limits(&config).unwrap();
        assert_eq!(limits.max_memory_bytes, Some(256 * 1024 * 1024));
        assert_eq!(limits.max_file_descriptors, Some(1024));
    }
}
