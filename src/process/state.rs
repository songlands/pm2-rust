use super::{ProcessInfo, ProcessStatus};
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use tokio::fs;
use tracing::info;

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct ProcessState {
    pub processes: HashMap<String, ProcessInfo>,
    pub version: String,
}

impl ProcessState {
    pub fn new() -> Self {
        Self {
            processes: HashMap::new(),
            version: env!("CARGO_PKG_VERSION").to_string(),
        }
    }

    pub fn get_state_file_path() -> Result<PathBuf> {
        let pm2_home = if let Ok(home) = std::env::var("PM2_HOME") {
            PathBuf::from(home)
        } else if let Some(home_dir) = dirs::home_dir() {
            home_dir.join(".pm2")
        } else {
            let current_dir = std::env::current_dir().context("Failed to get current directory")?;
            current_dir.join(".pm2")
        };
        Ok(pm2_home.join("process_state.json"))
    }

    pub async fn load() -> Result<Self> {
        let path = Self::get_state_file_path()?;

        if !path.exists() {
            info!("No existing state file found, creating new state");
            return Ok(Self::new());
        }

        let content = fs::read_to_string(&path).await.with_context(|| {
            format!("Failed to read state file: {}", path.display())
        })?;

        let state: ProcessState = serde_json::from_str(&content).with_context(|| {
            format!("Failed to parse state file: {}", path.display())
        })?;

        Ok(state)
    }

    pub async fn save(&self) -> Result<()> {
        let path = Self::get_state_file_path()?;

        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).await.with_context(|| {
                format!("Failed to create directory: {}", parent.display())
            })?;
        }

        let content = serde_json::to_string_pretty(self)?;
        fs::write(&path, content).await.with_context(|| {
            format!("Failed to write state file: {}", path.display())
        })?;

        Ok(())
    }

    pub fn add_process(&mut self, process: ProcessInfo) {
        self.processes.insert(process.name.clone(), process);
    }

    pub fn remove_process(&mut self, name: &str) -> Option<ProcessInfo> {
        self.processes.remove(name)
    }

    pub fn get_process(&self, name: &str) -> Option<&ProcessInfo> {
        self.processes.get(name)
    }

    #[allow(dead_code)]
    pub fn get_process_mut(&mut self, name: &str) -> Option<&mut ProcessInfo> {
        self.processes.get_mut(name)
    }

    pub fn find_by_id(&self, id: &str) -> Option<&ProcessInfo> {
        self.processes
            .values()
            .find(|p| p.id == id)
            .or_else(|| {
                self.processes
                    .values()
                    .find(|p| p.id.starts_with(id))
            })
    }

    #[allow(dead_code)]
    pub fn find_by_pid(&self, pid: u32) -> Option<&ProcessInfo> {
        self.processes.values().find(|p| p.pid == Some(pid))
    }

    pub fn list_processes(&self) -> Vec<&ProcessInfo> {
        self.processes.values().collect()
    }

    pub fn update_process_status(&mut self, name: &str, status: ProcessStatus) {
        if let Some(process) = self.processes.get_mut(name) {
            process.update_status(status);
        }
    }

    pub fn update_process_pid(&mut self, name: &str, pid: Option<u32>) {
        if let Some(process) = self.processes.get_mut(name) {
            process.pid = pid;
            process.updated_at = chrono::Utc::now();
        }
    }

    pub fn increment_restart_count(&mut self, name: &str) {
        if let Some(process) = self.processes.get_mut(name) {
            process.restart_count += 1;
        }
    }

    #[allow(dead_code)]
    pub fn update_metrics(&mut self, name: &str, cpu: f32, memory: f64, uptime: u64) {
        if let Some(process) = self.processes.get_mut(name) {
            process.cpu_percent = cpu;
            process.memory_mb = memory;
            process.uptime_seconds = uptime;
            process.updated_at = chrono::Utc::now();
        }
    }
}
