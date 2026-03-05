use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

pub mod manager;
pub mod state;

pub use manager::ProcessManager;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessInfo {
    pub id: String,
    pub name: String,
    pub script: String,
    pub pid: Option<u32>,
    pub status: ProcessStatus,
    pub instances: usize,
    pub restart_count: u32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub cpu_percent: f32,
    pub memory_mb: f64,
    pub uptime_seconds: u64,
    pub env_vars: HashMap<String, String>,
    pub log_file: Option<String>,
    pub error_log_file: Option<String>,
    pub max_memory_restart: Option<u64>,
    pub watch: bool,
    pub cluster_mode: bool,
    // 日志轮转配置
    #[serde(default)]
    pub log_rotate_size: Option<String>,
    #[serde(default = "default_log_rotate_count")]
    pub log_rotate_count: u32,
    #[serde(default)]
    pub log_rotate_interval: Option<String>,
}

fn default_log_rotate_count() -> u32 {
    10
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ProcessStatus {
    Online,
    Stopped,
    Stopping,
    Launching,
    Errored,
    OneLaunchStatus,
}

impl std::fmt::Display for ProcessStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProcessStatus::Online => write!(f, "online"),
            ProcessStatus::Stopped => write!(f, "stopped"),
            ProcessStatus::Stopping => write!(f, "stopping"),
            ProcessStatus::Launching => write!(f, "launching"),
            ProcessStatus::Errored => write!(f, "errored"),
            ProcessStatus::OneLaunchStatus => write!(f, "one-launch-status"),
        }
    }
}

#[derive(Debug)]
pub struct ManagedProcess {
    #[allow(dead_code)]
    pub info: ProcessInfo,
    pub child: Option<tokio::process::Child>,
}

impl ProcessInfo {
    pub fn new(
        name: String,
        script: String,
        instances: usize,
        env_vars: HashMap<String, String>,
        log_file: Option<String>,
        error_log_file: Option<String>,
        max_memory_restart: Option<u64>,
        watch: bool,
        cluster_mode: bool,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4().to_string(),
            name,
            script,
            pid: None,
            status: ProcessStatus::Stopped,
            instances: instances.max(1),
            restart_count: 0,
            created_at: now,
            updated_at: now,
            cpu_percent: 0.0,
            memory_mb: 0.0,
            uptime_seconds: 0,
            env_vars,
            log_file,
            error_log_file,
            max_memory_restart,
            watch,
            cluster_mode,
            // 默认日志轮转配置
            log_rotate_size: None,
            log_rotate_count: 10,
            log_rotate_interval: None,
        }
    }

    pub fn update_status(&mut self, status: ProcessStatus) {
        self.status = status;
        self.updated_at = Utc::now();
    }
}
