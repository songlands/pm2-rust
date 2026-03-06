use super::{AppConfig, EcosystemConfig};
use anyhow::{Context, Result};
use std::collections::HashMap;
use std::path::Path;

pub struct ConfigParser;

impl ConfigParser {
    pub fn parse_file<P: AsRef<Path>>(path: P) -> Result<Vec<AppConfig>> {
        let path = path.as_ref();
        let content = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read config file: {}", path.display()))?;

        // Try to parse as ecosystem config first
        if let Ok(ecosystem) = Self::parse_ecosystem_config(&content, path) {
            return Ok(ecosystem.apps);
        }

        // Try to parse as single app config
        if let Ok(app) = Self::parse_app_config(&content, path) {
            return Ok(vec![app]);
        }

        anyhow::bail!("Failed to parse config file: {}", path.display())
    }

    fn parse_ecosystem_config<P: AsRef<Path>>(content: &str, path: P) -> Result<EcosystemConfig> {
        let path = path.as_ref();
        
        if path.extension().map(|e| e == "json").unwrap_or(false) {
            serde_json::from_str(content)
                .with_context(|| "Failed to parse JSON ecosystem config")
        } else if path.extension().map(|e| e == "yaml" || e == "yml").unwrap_or(false) {
            serde_yaml::from_str(content)
                .with_context(|| "Failed to parse YAML ecosystem config")
        } else if path.extension().map(|e| e == "toml").unwrap_or(false) {
            toml::from_str(content)
                .with_context(|| "Failed to parse TOML ecosystem config")
        } else {
            // Try JSON first, then YAML, then TOML
            serde_json::from_str(content)
                .or_else(|_| serde_yaml::from_str(content))
                .or_else(|_| toml::from_str(content))
                .with_context(|| "Failed to parse ecosystem config")
        }
    }

    fn parse_app_config<P: AsRef<Path>>(content: &str, path: P) -> Result<AppConfig> {
        let path = path.as_ref();
        
        if path.extension().map(|e| e == "json").unwrap_or(false) {
            serde_json::from_str(content)
                .with_context(|| "Failed to parse JSON app config")
        } else if path.extension().map(|e| e == "yaml" || e == "yml").unwrap_or(false) {
            serde_yaml::from_str(content)
                .with_context(|| "Failed to parse YAML app config")
        } else if path.extension().map(|e| e == "toml").unwrap_or(false) {
            toml::from_str(content)
                .with_context(|| "Failed to parse TOML app config")
        } else {
            // Try JSON first, then YAML, then TOML
            serde_json::from_str(content)
                .or_else(|_| serde_yaml::from_str(content))
                .or_else(|_| toml::from_str(content))
                .with_context(|| "Failed to parse app config")
        }
    }

    pub fn parse_env_vars(env_list: &[String]) -> HashMap<String, String> {
        let mut env_map = HashMap::new();
        
        for env in env_list {
            if let Some((key, value)) = env.split_once('=') {
                env_map.insert(key.to_string(), value.to_string());
            }
        }
        
        env_map
    }
}
