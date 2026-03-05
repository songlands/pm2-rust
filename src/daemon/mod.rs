use crate::process::{ProcessManager, ProcessStatus};
use anyhow::{Context, Result};
use std::path::PathBuf;
use std::time::Duration;
use tokio::signal::unix::{signal, SignalKind};
use tokio::time::interval;
use tracing::{error, info, warn};

pub mod server;

pub use server::DaemonServer;

pub struct Daemon {
    process_manager: ProcessManager,
    shutdown_signal: tokio::sync::watch::Sender<bool>,
}

impl Daemon {
    pub async fn new() -> Result<Self> {
        let process_manager = ProcessManager::new().await?;
        let (shutdown_signal, _) = tokio::sync::watch::channel(false);

        Ok(Self {
            process_manager,
            shutdown_signal,
        })
    }

    pub async fn run(&mut self) -> Result<()> {
        info!("Starting PM2 daemon...");

        // Ensure PM2 home directory exists
        self.ensure_pm2_home().await?;

        // Write PID file
        self.write_pid_file().await?;

        // Start monitoring tasks
        let mut monitor_interval = interval(Duration::from_secs(5));
        let mut restart_interval = interval(Duration::from_secs(10));

        // Setup signal handlers
        let mut sigterm = signal(SignalKind::terminate())?;
        let mut sigint = signal(SignalKind::interrupt())?;
        let mut sighup = signal(SignalKind::hangup())?;

        info!("PM2 daemon started successfully");

        loop {
            tokio::select! {
                _ = monitor_interval.tick() => {
                    if let Err(e) = self.monitor_processes().await {
                        error!("Error monitoring processes: {}", e);
                    }
                }
                _ = restart_interval.tick() => {
                    if let Err(e) = self.check_restarts().await {
                        error!("Error checking restarts: {}", e);
                    }
                }
                _ = sigterm.recv() => {
                    info!("Received SIGTERM, shutting down...");
                    break;
                }
                _ = sigint.recv() => {
                    info!("Received SIGINT, shutting down...");
                    break;
                }
                _ = sighup.recv() => {
                    info!("Received SIGHUP, reloading configuration...");
                    if let Err(e) = self.reload_configuration().await {
                        error!("Error reloading configuration: {}", e);
                    }
                }
            }
        }

        // Cleanup
        self.shutdown().await?;
        info!("PM2 daemon stopped");

        Ok(())
    }

    async fn monitor_processes(&mut self) -> Result<()> {
        // Update process metrics
        self.process_manager.update_metrics().await;

        // Check for crashed processes and restart them
        let processes_to_restart: Vec<String> = self
            .process_manager
            .list_processes()
            .iter()
            .filter(|p| p.status == ProcessStatus::Errored)
            .map(|p| p.name.clone())
            .collect();

        for name in processes_to_restart {
            warn!("Process '{}' has errored, attempting restart", name);
            if let Err(e) = self.process_manager.restart_process(&name).await {
                error!("Failed to restart process '{}': {}", name, e);
            }
        }

        Ok(())
    }

    async fn check_restarts(&mut self) -> Result<()> {
        // Check memory limits and other restart conditions
        let processes_to_restart: Vec<(String, f64, u64)> = self
            .process_manager
            .list_processes()
            .iter()
            .filter_map(|p| {
                if let Some(max_memory) = p.max_memory_restart {
                    if p.memory_mb > max_memory as f64 {
                        return Some((p.name.clone(), p.memory_mb, max_memory));
                    }
                }
                None
            })
            .collect();

        for (name, memory_mb, max_memory) in processes_to_restart {
            warn!(
                "Process '{}' exceeded memory limit ({}MB > {}MB), restarting",
                name, memory_mb, max_memory
            );
            if let Err(e) = self.process_manager.restart_process(&name).await {
                error!("Failed to restart process '{}': {}", name, e);
            }
        }

        Ok(())
    }

    async fn reload_configuration(&mut self) -> Result<()> {
        info!("Reloading configuration...");
        // Reload process state from disk
        self.process_manager = ProcessManager::new().await?;
        Ok(())
    }

    async fn ensure_pm2_home(&self) -> Result<()> {
        let pm2_home = get_pm2_home()?;
        tokio::fs::create_dir_all(&pm2_home).await?;
        Ok(())
    }

    async fn write_pid_file(&self) -> Result<()> {
        let pid_file = get_pid_file_path()?;
        let pid = std::process::id();
        tokio::fs::write(&pid_file, pid.to_string()).await?;
        info!("PID file written: {}", pid_file.display());
        Ok(())
    }

    async fn remove_pid_file(&self) -> Result<()> {
        let pid_file = get_pid_file_path()?;
        if pid_file.exists() {
            tokio::fs::remove_file(&pid_file).await?;
            info!("PID file removed: {}", pid_file.display());
        }
        Ok(())
    }

    async fn shutdown(&mut self) -> Result<()> {
        info!("Shutting down daemon...");

        // Signal shutdown
        let _ = self.shutdown_signal.send(true);

        // Remove PID file
        if let Err(e) = self.remove_pid_file().await {
            error!("Failed to remove PID file: {}", e);
        }

        // Save state
        if let Err(e) = self.process_manager.save_state().await {
            error!("Failed to save state: {}", e);
        }

        Ok(())
    }
}

pub fn get_pm2_home() -> Result<PathBuf> {
    // Use current directory for PM2 home to avoid permission issues
    let current_dir = std::env::current_dir().context("Failed to get current directory")?;
    Ok(current_dir.join(".pm2"))
}

pub fn get_pid_file_path() -> Result<PathBuf> {
    Ok(get_pm2_home()?.join("pm2.pid"))
}

pub fn is_daemon_running() -> Result<bool> {
    let pid_file = get_pid_file_path()?;

    if !pid_file.exists() {
        return Ok(false);
    }

    let pid_str = std::fs::read_to_string(&pid_file)?;
    let pid: i32 = pid_str.trim().parse()?;

    // Check if process exists
    #[cfg(unix)]
    {
        unsafe {
            let result = libc::kill(pid, 0);
            Ok(result == 0)
        }
    }

    #[cfg(not(unix))]
    {
        // For non-Unix systems, just check if PID file exists
        Ok(true)
    }
}

pub async fn start_daemon() -> Result<()> {
    if is_daemon_running()? {
        info!("Daemon is already running");
        return Ok(());
    }

    // Fork and detach (Unix only)
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        use std::process::Command;

        // Start daemon in background
        let current_exe = std::env::current_exe()?;
        let mut cmd = Command::new(current_exe);
        cmd.arg("daemon");
        cmd.arg("start");
        cmd.stdout(std::process::Stdio::null());
        cmd.stderr(std::process::Stdio::null());
        cmd.stdin(std::process::Stdio::null());

        // Detach from terminal
        unsafe {
            cmd.pre_exec(|| {
                libc::setsid();
                Ok(())
            });
        }

        cmd.spawn()?;
    }

    #[cfg(not(unix))]
    {
        // For non-Unix systems, run daemon directly
        let mut daemon = Daemon::new().await?;
        daemon.run().await?;
    }

    Ok(())
}

pub async fn stop_daemon() -> Result<()> {
    let pid_file = get_pid_file_path()?;

    if !pid_file.exists() {
        info!("Daemon is not running");
        return Ok(());
    }

    let pid_str = std::fs::read_to_string(&pid_file)?;
    let pid: i32 = pid_str.trim().parse()?;

    #[cfg(unix)]
    {
        unsafe {
            libc::kill(pid, libc::SIGTERM);
        }
    }

    // Wait for daemon to stop
    for _ in 0..30 {
        tokio::time::sleep(Duration::from_millis(100)).await;
        if !is_daemon_running()? {
            break;
        }
    }

    // Force kill if still running
    if is_daemon_running()? {
        #[cfg(unix)]
        {
            unsafe {
                libc::kill(pid, libc::SIGKILL);
            }
        }
    }

    // Remove PID file
    if pid_file.exists() {
        tokio::fs::remove_file(&pid_file).await?;
    }

    info!("Daemon stopped");
    Ok(())
}
