// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2026 Schwimmbeck Dominik

//! Global configuration loaded from `~/.berth/berth.toml`.

use serde::{Deserialize, Serialize};
use std::fs;

use crate::paths;

/// Top-level global configuration.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GlobalConfig {
    #[serde(default)]
    pub registry: RegistryConfig,
    #[serde(default)]
    pub runtime: RuntimeConfig,
    #[serde(default)]
    pub security: SecurityConfig,
    #[serde(default)]
    pub clients: ClientsConfig,
}

/// Registry connection settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryConfig {
    #[serde(default = "default_registry_url")]
    pub url: String,
    #[serde(default = "default_cache_ttl")]
    pub cache_ttl: u64,
}

impl Default for RegistryConfig {
    fn default() -> Self {
        Self {
            url: default_registry_url(),
            cache_ttl: default_cache_ttl(),
        }
    }
}

/// Runtime behaviour defaults.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeConfig {
    #[serde(default)]
    pub auto_restart: bool,
    #[serde(default = "default_health_check_interval")]
    pub health_check_interval: u64,
    #[serde(default = "default_log_level")]
    pub log_level: String,
    #[serde(default = "default_max_servers")]
    pub max_servers: u64,
}

impl Default for RuntimeConfig {
    fn default() -> Self {
        Self {
            auto_restart: false,
            health_check_interval: default_health_check_interval(),
            log_level: default_log_level(),
            max_servers: default_max_servers(),
        }
    }
}

/// Security policy defaults.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    #[serde(default = "default_trust_level")]
    pub default_trust_level: String,
    #[serde(default = "default_true")]
    pub audit_enabled: bool,
    #[serde(default)]
    pub sandbox_enabled: bool,
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            default_trust_level: default_trust_level(),
            audit_enabled: true,
            sandbox_enabled: false,
        }
    }
}

/// Client auto-linking preferences.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ClientsConfig {
    #[serde(default)]
    pub auto_link: Vec<String>,
}

fn default_registry_url() -> String {
    "https://registry.berth.dev".to_string()
}
fn default_cache_ttl() -> u64 {
    3600
}
fn default_health_check_interval() -> u64 {
    30
}
fn default_log_level() -> String {
    "warn".to_string()
}
fn default_max_servers() -> u64 {
    20
}
fn default_trust_level() -> String {
    "community".to_string()
}
fn default_true() -> bool {
    true
}

/// Loads global configuration from `~/.berth/berth.toml`.
///
/// Returns [`GlobalConfig::default()`] when the file is missing or unparseable.
pub fn load() -> GlobalConfig {
    let path = match paths::global_config_path() {
        Some(p) => p,
        None => return GlobalConfig::default(),
    };
    let content = match fs::read_to_string(&path) {
        Ok(c) => c,
        Err(_) => return GlobalConfig::default(),
    };
    toml::from_str(&content).unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_when_no_file() {
        let cfg = GlobalConfig::default();
        assert!(!cfg.runtime.auto_restart);
        assert_eq!(cfg.runtime.health_check_interval, 30);
        assert_eq!(cfg.security.default_trust_level, "community");
        assert!(cfg.security.audit_enabled);
        assert!(cfg.clients.auto_link.is_empty());
    }

    #[test]
    fn partial_file_fills_defaults() {
        let toml_str = r#"
[runtime]
auto_restart = true
"#;
        let cfg: GlobalConfig = toml::from_str(toml_str).unwrap();
        assert!(cfg.runtime.auto_restart);
        assert_eq!(cfg.runtime.health_check_interval, 30);
        assert_eq!(cfg.registry.url, "https://registry.berth.dev");
    }

    #[test]
    fn round_trip_serialization() {
        let cfg = GlobalConfig::default();
        let serialized = toml::to_string_pretty(&cfg).unwrap();
        let deserialized: GlobalConfig = toml::from_str(&serialized).unwrap();
        assert_eq!(cfg.runtime.auto_restart, deserialized.runtime.auto_restart);
        assert_eq!(cfg.registry.url, deserialized.registry.url);
    }
}
