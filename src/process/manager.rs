use super::{state::ProcessState, ManagedProcess, ProcessInfo, ProcessStatus};
use crate::log::{parse_interval_string, parse_size_string, LogRotator};
use anyhow::{Context, Result};
use std::collections::HashMap;
use std::time::Duration;
use sysinfo::{ProcessRefreshKind, RefreshKind, System};
use tokio::process::Command;
use tracing::{error, info, warn};

pub struct ProcessManager {
    state: ProcessState,
    running_processes: HashMap<String, ManagedProcess>,
    system: System,
    log_rotators: HashMap<String, (LogRotator, LogRotator)>,
}

impl ProcessManager {
    pub async fn new() -> Result<Self> {
        let state = ProcessState::load().await?;
        let system = System::new_with_specifics(
            RefreshKind::new().with_processes(ProcessRefreshKind::new()),
        );

        Ok(Self {
            state,
            running_processes: HashMap::new(),
            system,
            log_rotators: HashMap::new(),
        })
    }

    fn create_log_rotators(&self, process_info: &ProcessInfo) -> (LogRotator, LogRotator) {
        use crate::log::LogManager;

        let log_manager = LogManager::new().unwrap_or_default();

        // 解析轮转配置
        let rotate_size = process_info
            .log_rotate_size
            .as_ref()
            .and_then(|s| parse_size_string(s));
        let rotate_count = process_info.log_rotate_count;
        let rotate_interval = process_info
            .log_rotate_interval
            .as_ref()
            .and_then(|s| parse_interval_string(s));

        let out_rotator = log_manager.create_rotator(
            &process_info.name,
            rotate_size,
            rotate_count,
            rotate_interval,
        );

        let err_rotator = log_manager.create_error_rotator(
            &process_info.name,
            rotate_size,
            rotate_count,
            rotate_interval,
        );

        (out_rotator, err_rotator)
    }

    pub async fn start_process(&mut self, mut process_info: ProcessInfo) -> Result<()> {
        let name = process_info.name.clone();

        // Check if process already exists and is running
        if let Some(existing) = self.state.get_process(&name) {
            if existing.status == ProcessStatus::Online {
                anyhow::bail!("Process '{}' is already running", name);
            }
        }

        // Update status to launching
        process_info.update_status(ProcessStatus::Launching);
        self.state.add_process(process_info.clone());
        self.state.save().await?;

        // Start the process
        match self.spawn_process(&process_info).await {
            Ok(child) => {
                let pid = child.id();
                info!("Started process '{}' with PID {:?}", name, pid);

                process_info.pid = pid;
                process_info.update_status(ProcessStatus::Online);

                self.state.update_process_pid(&name, pid);
                self.state.update_process_status(&name, ProcessStatus::Online);
                self.state.save().await?;

                let managed = ManagedProcess {
                    info: process_info.clone(),
                    child: Some(child),
                };

                self.running_processes.insert(name.clone(), managed);

                // 创建日志轮转器
                let rotators = self.create_log_rotators(&process_info);
                self.log_rotators.insert(name.clone(), rotators);

                Ok(())
            }
            Err(e) => {
                process_info.update_status(ProcessStatus::Errored);
                self.state.update_process_status(&name, ProcessStatus::Errored);
                self.state.save().await?;

                error!("Failed to start process '{}': {}", name, e);
                Err(e)
            }
        }
    }

    async fn spawn_process(&self, process_info: &ProcessInfo) -> Result<tokio::process::Child> {
        let script = &process_info.script;

        let (program, args) = if script.ends_with(".js") {
            ("node", vec![script.as_str()])
        } else if script.ends_with(".py") {
            ("python3", vec![script.as_str()])
        } else if script.ends_with(".sh") {
            ("bash", vec![script.as_str()])
        } else {
            (script.as_str(), vec![])
        };

        let mut cmd = Command::new(program);
        cmd.args(&args)
            .envs(&process_info.env_vars)
            .kill_on_drop(false);

        let pm2_home = if let Ok(home) = std::env::var("PM2_HOME") {
            std::path::PathBuf::from(home)
        } else if let Some(home_dir) = dirs::home_dir() {
            home_dir.join(".pm2")
        } else {
            let current_dir = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
            current_dir.join(".pm2")
        };
        let logs_dir = pm2_home.join("logs");
        tokio::fs::create_dir_all(&logs_dir).await?;

        let log_file = process_info.log_file.clone().unwrap_or_else(|| {
            logs_dir.join(format!("{}-out.log", process_info.name)).to_string_lossy().to_string()
        });
        let error_file = process_info.error_log_file.clone().unwrap_or_else(|| {
            logs_dir.join(format!("{}-error.log", process_info.name)).to_string_lossy().to_string()
        });

        let log_path = std::path::Path::new(&log_file);
        if let Some(parent) = log_path.parent() {
            tokio::fs::create_dir_all(parent).await.with_context(|| {
                format!("Failed to create log directory: {}", parent.display())
            })?;
        }
        let stdout = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&log_file)
            .with_context(|| format!("Failed to open log file: {}", log_file))?;
        cmd.stdout(stdout);

        let error_path = std::path::Path::new(&error_file);
        if let Some(parent) = error_path.parent() {
            tokio::fs::create_dir_all(parent).await.with_context(|| {
                format!("Failed to create error log directory: {}", parent.display())
            })?;
        }
        let stderr = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&error_file)
            .with_context(|| format!("Failed to open error log file: {}", error_file))?;
        cmd.stderr(stderr);

        let child = cmd.spawn().with_context(|| {
            format!("Failed to spawn process: {} {}", program, args.join(" "))
        })?;

        Ok(child)
    }

    pub async fn stop_process(&mut self, name: &str) -> Result<()> {
        let process_info = self
            .state
            .get_process(name)
            .context(format!("Process '{}' not found", name))?
            .clone();

        if process_info.status != ProcessStatus::Online {
            warn!("Process '{}' is not running", name);
            return Ok(());
        }

        self.state
            .update_process_status(name, ProcessStatus::Stopping);
        self.state.save().await?;

        // Try to stop from running_processes first
        if let Some(mut managed) = self.running_processes.remove(name) {
            if let Some(mut child) = managed.child.take() {
                // Try graceful shutdown first
                if let Some(pid) = child.id() {
                    #[cfg(unix)]
                    {
                        unsafe {
                            libc::kill(pid as i32, libc::SIGTERM);
                        }
                    }
                }

                // Wait for graceful shutdown
                match tokio::time::timeout(Duration::from_secs(5), child.wait()).await {
                    Ok(Ok(_)) => {
                        info!("Process '{}' stopped gracefully", name);
                    }
                    _ => {
                        // Force kill
                        let _ = child.kill().await;
                        warn!("Process '{}' force killed", name);
                    }
                }
            }
        } else {
            // Process not in running_processes, try to kill by PID
            if let Some(pid) = process_info.pid {
                info!("Process '{}' not in running_processes, killing by PID {}", name, pid);
                
                #[cfg(unix)]
                {
                    // Try graceful shutdown first
                    unsafe {
                        libc::kill(pid as i32, libc::SIGTERM);
                    }
                    
                    // Wait a bit for graceful shutdown
                    tokio::time::sleep(Duration::from_millis(500)).await;
                    
                    // Check if process still exists
                    let still_running = unsafe { libc::kill(pid as i32, 0) == 0 };
                    
                    if still_running {
                        // Force kill
                        unsafe {
                            libc::kill(pid as i32, libc::SIGKILL);
                        }
                        warn!("Process '{}' force killed by PID {}", name, pid);
                    } else {
                        info!("Process '{}' stopped gracefully by PID {}", name, pid);
                    }
                }
            }
        }

        self.state.update_process_status(name, ProcessStatus::Stopped);
        self.state.update_process_pid(name, None);
        self.state.save().await?;

        Ok(())
    }

    pub async fn restart_process(&mut self, name: &str) -> Result<()> {
        let process_info = self
            .state
            .get_process(name)
            .context(format!("Process '{}' not found", name))?
            .clone();

        // Stop if running
        if process_info.status == ProcessStatus::Online {
            self.stop_process(name).await?;
        }

        // Increment restart count
        self.state.increment_restart_count(name);
        self.state.save().await?;

        // Start again
        self.start_process(process_info).await?;

        info!("Restarted process '{}'", name);
        Ok(())
    }

    pub async fn delete_process(&mut self, name: &str) -> Result<()> {
        // Stop if running
        if let Some(process) = self.state.get_process(name) {
            if process.status == ProcessStatus::Online {
                self.stop_process(name).await?;
            }
        }

        self.state.remove_process(name);
        self.state.save().await?;

        info!("Deleted process '{}'", name);
        Ok(())
    }

    pub fn get_process(&self, name: &str) -> Option<&ProcessInfo> {
        self.state.get_process(name)
    }

    pub fn list_processes(&self) -> Vec<&ProcessInfo> {
        self.state.list_processes()
    }

    pub async fn update_metrics(&mut self) {
        self.reap_zombie_processes().await;

        self.system
            .refresh_processes_specifics(ProcessRefreshKind::new());

        for (name, process_info) in self.state.processes.iter_mut() {
            if let Some(pid) = process_info.pid {
                if let Some(process) = self.system.process(sysinfo::Pid::from(pid as usize)) {
                    let cpu_usage = process.cpu_usage();
                    let memory_mb = process.memory() as f64 / 1024.0 / 1024.0;
                    let uptime = process.run_time();

                    process_info.cpu_percent = cpu_usage;
                    process_info.memory_mb = memory_mb;
                    process_info.uptime_seconds = uptime;

                    if let Some(max_memory) = process_info.max_memory_restart {
                        if memory_mb > max_memory as f64 {
                            warn!(
                                "Process '{}' exceeded memory limit ({}MB > {}MB), restarting",
                                name, memory_mb, max_memory
                            );
                        }
                    }
                } else {
                    if process_info.status == ProcessStatus::Online {
                        warn!("Process '{}' (PID: {}) not found, may have crashed", name, pid);
                        process_info.status = ProcessStatus::Errored;
                        process_info.pid = None;
                    }
                }
            }
        }

        let _ = self.state.save().await;
    }

    async fn reap_zombie_processes(&mut self) {
        let mut to_remove = Vec::new();

        for (name, managed) in self.running_processes.iter_mut() {
            if let Some(child) = &mut managed.child {
                match child.try_wait() {
                    Ok(Some(status)) => {
                        info!("Process '{}' exited with status: {:?}", name, status);
                        to_remove.push(name.clone());

                        if let Some(process_info) = self.state.processes.get_mut(name) {
                            process_info.status = ProcessStatus::Stopped;
                            process_info.pid = None;
                        }
                    }
                    Ok(None) => {}
                    Err(e) => {
                        error!("Error checking process '{}': {}", name, e);
                    }
                }
            }
        }

        for name in &to_remove {
            self.running_processes.remove(name);
        }

        if !to_remove.is_empty() {
            let _ = self.state.save().await;
        }
    }

    #[allow(dead_code)]
    pub async fn start_monitoring(&mut self) {
        let mut interval = tokio::time::interval(Duration::from_secs(5));

        loop {
            interval.tick().await;
            self.update_metrics().await;
        }
    }

    pub async fn save_state(&self) -> Result<()> {
        self.state.save().await
    }

    #[allow(dead_code)]
    pub fn get_state(&self) -> &ProcessState {
        &self.state
    }

    #[allow(dead_code)]
    pub fn get_state_mut(&mut self) -> &mut ProcessState {
        &mut self.state
    }

    pub async fn find_process_by_id(&self, id: &str) -> Option<ProcessInfo> {
        self.state.find_by_id(id).cloned()
    }
}
