use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

pub mod parser;

pub use parser::ConfigParser;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub name: String,
    pub script: String,
    #[serde(default = "default_instances")]
    pub instances: usize,
    #[serde(default)]
    pub exec_mode: ExecMode,
    #[serde(default)]
    pub watch: bool,
    #[serde(default)]
    pub ignore_watch: Vec<String>,
    #[serde(rename = "max_memory_restart")]
    pub max_memory_restart: Option<String>,
    #[serde(rename = "log_file")]
    pub log_file: Option<String>,
    #[serde(rename = "error_file")]
    pub error_file: Option<String>,
    #[serde(rename = "out_file")]
    pub out_file: Option<String>,
    #[serde(rename = "pid_file")]
    pub pid_file: Option<String>,
    #[serde(default)]
    pub env: HashMap<String, String>,
    #[serde(rename = "env_production", default)]
    pub env_production: HashMap<String, String>,
    #[serde(rename = "env_development", default)]
    pub env_development: HashMap<String, String>,
    #[serde(default)]
    pub args: Vec<String>,
    #[serde(default)]
    pub node_args: Vec<String>,
    #[serde(default = "default_cwd")]
    pub cwd: String,
    #[serde(default)]
    pub autorestart: bool,
    #[serde(rename = "max_restarts", default = "default_max_restarts")]
    pub max_restarts: u32,
    #[serde(rename = "min_uptime", default)]
    pub min_uptime: Option<String>,
    #[serde(rename = "listen_timeout", default)]
    pub listen_timeout: Option<u64>,
    #[serde(rename = "kill_timeout", default)]
    pub kill_timeout: Option<u64>,
    #[serde(default)]
    pub cron_restart: Option<String>,
    #[serde(default)]
    pub merge_logs: bool,
    #[serde(default)]
    pub log_date_format: Option<String>,
    // 日志轮转配置
    #[serde(rename = "log_rotate_size", default)]
    pub log_rotate_size: Option<String>,
    #[serde(rename = "log_rotate_count", default = "default_log_rotate_count")]
    pub log_rotate_count: u32,
    #[serde(rename = "log_rotate_interval", default)]
    pub log_rotate_interval: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EcosystemConfig {
    pub apps: Vec<AppConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ExecMode {
    Fork,
    Cluster,
}

impl Default for ExecMode {
    fn default() -> Self {
        ExecMode::Fork
    }
}

fn default_instances() -> usize {
    1
}

fn default_cwd() -> String {
    ".".to_string()
}

fn default_max_restarts() -> u32 {
    15
}

fn default_log_rotate_count() -> u32 {
    10
}

impl AppConfig {
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref();
        let content = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read config file: {}", path.display()))?;

        if path.extension().map(|e| e == "json").unwrap_or(false) {
            let config: AppConfig = serde_json::from_str(&content)
                .with_context(|| format!("Failed to parse JSON config: {}", path.display()))?;
            Ok(config)
        } else if path.extension().map(|e| e == "yaml" || e == "yml").unwrap_or(false) {
            let config: AppConfig = serde_yaml::from_str(&content)
                .with_context(|| format!("Failed to parse YAML config: {}", path.display()))?;
            Ok(config)
        } else if path.extension().map(|e| e == "toml").unwrap_or(false) {
            let config: AppConfig = toml::from_str(&content)
                .with_context(|| format!("Failed to parse TOML config: {}", path.display()))?;
            Ok(config)
        } else {
            anyhow::bail!("Unsupported config file format: {}", path.display())
        }
    }

    pub fn parse_memory_limit(&self) -> Option<u64> {
        self.max_memory_restart.as_ref().and_then(|s| {
            parse_memory_string(s)
        })
    }
}

impl EcosystemConfig {
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref();
        let content = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read config file: {}", path.display()))?;

        if path.extension().map(|e| e == "json").unwrap_or(false) {
            let config: EcosystemConfig = serde_json::from_str(&content)
                .with_context(|| format!("Failed to parse JSON config: {}", path.display()))?;
            Ok(config)
        } else if path.extension().map(|e| e == "yaml" || e == "yml").unwrap_or(false) {
            let config: EcosystemConfig = serde_yaml::from_str(&content)
                .with_context(|| format!("Failed to parse YAML config: {}", path.display()))?;
            Ok(config)
        } else if path.extension().map(|e| e == "toml").unwrap_or(false) {
            let config: EcosystemConfig = toml::from_str(&content)
                .with_context(|| format!("Failed to parse TOML config: {}", path.display()))?;
            Ok(config)
        } else {
            anyhow::bail!("Unsupported config file format: {}", path.display())
        }
    }
}

fn parse_memory_string(s: &str) -> Option<u64> {
    let s = s.trim().to_lowercase();
    
    if s.ends_with("gb") || s.ends_with("g") {
        s.trim_end_matches("gb")
            .trim_end_matches("g")
            .trim()
            .parse::<u64>()
            .ok()
            .map(|v| v * 1024)
    } else if s.ends_with("mb") || s.ends_with("m") {
        s.trim_end_matches("mb")
            .trim_end_matches("m")
            .trim()
            .parse::<u64>()
            .ok()
    } else if s.ends_with("kb") || s.ends_with("k") {
        s.trim_end_matches("kb")
            .trim_end_matches("k")
            .trim()
            .parse::<u64>()
            .ok()
            .map(|v| v / 1024)
    } else {
        s.parse::<u64>().ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_memory_string() {
        assert_eq!(parse_memory_string("100M"), Some(100));
        assert_eq!(parse_memory_string("100MB"), Some(100));
        assert_eq!(parse_memory_string("1G"), Some(1024));
        assert_eq!(parse_memory_string("1GB"), Some(1024));
        assert_eq!(parse_memory_string("512"), Some(512));
    }
}
