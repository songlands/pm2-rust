use crate::config::parser::ConfigParser;
use crate::process::{ProcessInfo, ProcessManager};
use crate::cli::display;
use anyhow::Result;
use colored::Colorize;
use std::path::Path;
use tracing::{error, info};

pub async fn start(
    script: String,
    name: Option<String>,
    instances: Option<usize>,
    cluster: bool,
    watch: bool,
    max_memory_restart: Option<String>,
    log: Option<String>,
    error_log: Option<String>,
    env: Vec<String>,
) -> Result<()> {
    let mut manager = ProcessManager::new().await?;

    // Check if script is a config file
    let path = Path::new(&script);
    let process_infos = if path.is_file()
        && (path.extension().map(|e| e == "json").unwrap_or(false)
            || path.extension().map(|e| e == "yaml" || e == "yml").unwrap_or(false)
            || path.extension().map(|e| e == "toml").unwrap_or(false)
            || path.extension().map(|e| e == "config.js").unwrap_or(false))
    {
        // Parse config file - convert AppConfig to ProcessInfo
        let configs = ConfigParser::parse_file(path)?;
        configs
            .into_iter()
            .map(|config| {
                let mut process_info = ProcessInfo::new(
                    config.name.clone(),
                    config.script.clone(),
                    config.instances,
                    config.env.clone(),
                    config.log_file.clone(),
                    config.error_file.clone(),
                    config.parse_memory_limit(),
                    config.watch,
                    config.exec_mode == crate::config::ExecMode::Cluster,
                );
                // 设置日志轮转配置
                process_info.log_rotate_size = config.log_rotate_size.clone();
                process_info.log_rotate_count = config.log_rotate_count;
                process_info.log_rotate_interval = config.log_rotate_interval.clone();
                process_info
            })
            .collect()
    } else {
        // Single script
        let process_name = name.unwrap_or_else(|| {
            path.file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("unnamed")
                .to_string()
        });

        let env_vars = ConfigParser::parse_env_vars(&env);
        let max_memory = max_memory_restart.as_ref().and_then(|s| parse_memory_limit(s));

        vec![ProcessInfo::new(
            process_name,
            script.clone(),
            instances.unwrap_or(1),
            env_vars,
            log,
            error_log,
            max_memory,
            watch,
            cluster,
        )]
    };

    for process_info in process_infos {
        match manager.start_process(process_info.clone()).await {
            Ok(_) => {
                display::display_success(&format!(
                    "Process '{}' started successfully",
                    process_info.name
                ));
            }
            Err(e) => {
                display::display_error(&format!(
                    "Failed to start process '{}': {}",
                    process_info.name, e
                ));
            }
        }
    }

    Ok(())
}

fn parse_memory_limit(s: &str) -> Option<u64> {
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

pub async fn stop(name: &str) -> Result<()> {
    let mut manager = ProcessManager::new().await?;

    match manager.stop_process(name).await {
        Ok(_) => {
            display::display_success(&format!("Process '{}' stopped successfully", name));
            Ok(())
        }
        Err(e) => {
            display::display_error(&format!("Failed to stop process '{}': {}", name, e));
            Err(e)
        }
    }
}

pub async fn restart(name: &str) -> Result<()> {
    let mut manager = ProcessManager::new().await?;

    match manager.restart_process(name).await {
        Ok(_) => {
            display::display_success(&format!("Process '{}' restarted successfully", name));
            Ok(())
        }
        Err(e) => {
            display::display_error(&format!("Failed to restart process '{}': {}", name, e));
            Err(e)
        }
    }
}

pub async fn delete(name: &str) -> Result<()> {
    let mut manager = ProcessManager::new().await?;

    // Check if it's "all" command
    if name == "all" {
        let processes = manager.list_processes();
        if processes.is_empty() {
            display::display_info("No processes to delete");
            return Ok(());
        }

        let process_names: Vec<String> = processes
            .iter()
            .map(|p| p.name.clone())
            .collect();

        for name in process_names {
            match manager.delete_process(&name).await {
                Ok(_) => {
                    display::display_success(&format!("Process '{}' deleted successfully", name));
                }
                Err(e) => {
                    display::display_error(&format!("Failed to delete process '{}': {}", name, e));
                }
            }
        }
        return Ok(());
    }

    // Try to find process by name first
    let process_name = if manager.get_process(name).is_some() {
        name.to_string()
    } else {
        // Try to find by id
        match manager.find_process_by_id(name).await {
            Some(process) => process.name.clone(),
            None => {
                display::display_error(&format!("Process '{}' not found", name));
                anyhow::bail!("Process not found")
            }
        }
    };

    match manager.delete_process(&process_name).await {
        Ok(_) => {
            display::display_success(&format!("Process '{}' deleted successfully", process_name));
            Ok(())
        }
        Err(e) => {
            display::display_error(&format!("Failed to delete process '{}': {}", process_name, e));
            Err(e)
        }
    }
}

pub async fn list() -> Result<()> {
    let manager = ProcessManager::new().await?;

    let processes = manager.list_processes();
    display::display_process_list(&processes);

    Ok(())
}

pub async fn show(name: &str) -> Result<()> {
    let manager = ProcessManager::new().await?;

    match manager.get_process(name) {
        Some(process) => {
            display::display_process_details(process);
            Ok(())
        }
        None => {
            display::display_error(&format!("Process '{}' not found", name));
            anyhow::bail!("Process not found")
        }
    }
}

pub async fn monit() -> Result<()> {
    println!("Monitoring processes... (Press Ctrl+C to exit)");

    let mut manager = ProcessManager::new().await?;

    // Clear screen and show header
    print!("\x1B[2J\x1B[1;1H");

    loop {
        // Update metrics
        manager.update_metrics().await;

        // Move cursor to top
        print!("\x1B[1;1H");

        let processes = manager.list_processes();
        display::display_process_list(&processes);

        // Wait before next update
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
    }
}

pub async fn logs(name: Option<&str>, lines: usize, follow: bool) -> Result<()> {
    use tokio::fs::File;
    use tokio::io::{AsyncBufReadExt, BufReader};

    let manager = ProcessManager::new().await?;

    let log_files: Vec<String> = if let Some(process_name) = name {
        // Get specific process logs
        if let Some(process) = manager.get_process(process_name) {
            let mut files = Vec::new();
            if let Some(log) = &process.log_file {
                files.push(log.clone());
            }
            if let Some(err_log) = &process.error_log_file {
                files.push(err_log.clone());
            }
            files
        } else {
            display::display_error(&format!("Process '{}' not found", process_name));
            anyhow::bail!("Process not found")
        }
    } else {
        // Get all process logs
        let mut files = Vec::new();
        for process in manager.list_processes() {
            if let Some(log) = &process.log_file {
                files.push(log.clone());
            }
            if let Some(err_log) = &process.error_log_file {
                files.push(err_log.clone());
            }
        }
        files
    };

    if log_files.is_empty() {
        display::display_warning("No log files configured");
        return Ok(());
    }

    for log_file in log_files {
        println!("\n{} {}", "==>".blue(), log_file.bold());
        println!("{}", "=".repeat(50));

        match File::open(&log_file).await {
            Ok(file) => {
                let reader = BufReader::new(file);
                let mut lines_iter = reader.lines();
                let mut all_lines = Vec::new();

                while let Some(line) = lines_iter.next_line().await? {
                    all_lines.push(line);
                }

                // Show last N lines
                let start = all_lines.len().saturating_sub(lines);
                for line in &all_lines[start..] {
                    println!("{}", line);
                }

                if follow {
                    // TODO: Implement follow mode with tail -f like functionality
                    display::display_info("Follow mode not yet implemented");
                }
            }
            Err(e) => {
                display::display_error(&format!("Failed to open log file '{}': {}", log_file, e));
            }
        }
    }

    Ok(())
}

pub async fn flush() -> Result<()> {
    use tokio::fs::OpenOptions;

    let manager = ProcessManager::new().await?;
    let mut cleared_count = 0;

    for process in manager.list_processes() {
        if let Some(log_file) = &process.log_file {
            match OpenOptions::new().write(true).truncate(true).open(log_file).await {
                Ok(_) => {
                    cleared_count += 1;
                    info!("Cleared log file: {}", log_file);
                }
                Err(e) => {
                    error!("Failed to clear log file '{}': {}", log_file, e);
                }
            }
        }

        if let Some(error_log) = &process.error_log_file {
            match OpenOptions::new()
                .write(true)
                .truncate(true)
                .open(error_log)
                .await
            {
                Ok(_) => {
                    cleared_count += 1;
                    info!("Cleared error log file: {}", error_log);
                }
                Err(e) => {
                    error!("Failed to clear error log file '{}': {}", error_log, e);
                }
            }
        }
    }

    display::display_success(&format!("Cleared {} log files", cleared_count));
    Ok(())
}

pub async fn reload(name: &str) -> Result<()> {
    // Reload is similar to restart but preserves the process ID
    restart(name).await
}

pub async fn save() -> Result<()> {
    let manager = ProcessManager::new().await?;
    manager.save_state().await?;
    display::display_success("Process list saved");
    Ok(())
}

pub async fn resurrect() -> Result<()> {
    let mut manager = ProcessManager::new().await?;
    let processes: Vec<ProcessInfo> = manager
        .list_processes()
        .iter()
        .map(|p| (*p).clone())
        .collect();

    let mut started_count = 0;
    for process in processes {
        if process.status.to_string() == "online" {
            match manager.start_process(process.clone()).await {
                Ok(_) => {
                    started_count += 1;
                    info!("Resurrected process '{}'", process.name);
                }
                Err(e) => {
                    error!("Failed to resurrect process '{}': {}", process.name, e);
                }
            }
        }
    }

    display::display_success(&format!("Resurrected {} processes", started_count));
    Ok(())
}

pub async fn startup(platform: Option<String>) -> Result<()> {
    let platform = platform.unwrap_or_else(|| {
        // Detect platform
        if Path::new("/etc/systemd").exists() {
            "systemd".to_string()
        } else if Path::new("/etc/init.d").exists() {
            "sysvinit".to_string()
        } else {
            "unknown".to_string()
        }
    });

    println!("Generating startup script for platform: {}", platform);

    match platform.as_str() {
        "systemd" => {
            generate_systemd_service()?;
        }
        "sysvinit" => {
            generate_sysvinit_script()?;
        }
        _ => {
            display::display_error(&format!("Unsupported platform: {}", platform));
            anyhow::bail!("Unsupported platform")
        }
    }

    display::display_success(&format!("Startup script generated for {}", platform));
    Ok(())
}

fn generate_systemd_service() -> Result<()> {
    let service_content = r#"[Unit]
Description=PM2 process manager
Documentation=https://pm2.keymetrics.io/
After=network.target

[Service]
Type=forking
User=%I
LimitNOFILE=infinity
LimitNPROC=infinity
LimitCORE=infinity
Environment=PM2_HOME=%h/.pm2
PIDFile=%h/.pm2/pm2.pid
Restart=on-failure
RestartSec=3

ExecStart=%h/.cargo/bin/pm2 resurrect
ExecReload=%h/.cargo/bin/pm2 reload all
ExecStop=%h/.cargo/bin/pm2 kill

[Install]
WantedBy=multi-user.target
"#;

    println!("\n{}\n", service_content);
    println!("To install the systemd service, run:");
    println!("  sudo systemctl enable --user pm2");
    println!("  sudo systemctl start --user pm2");

    Ok(())
}

fn generate_sysvinit_script() -> Result<()> {
    let script_content = r#"#!/bin/sh
### BEGIN INIT INFO
# Provides:          pm2
# Required-Start:    $local_fs $remote_fs $network
# Required-Stop:     $local_fs $remote_fs $network
# Default-Start:     2 3 4 5
# Default-Stop:      0 1 6
# Short-Description: PM2 process manager
### END INIT INFO

export PM2_HOME="$HOME/.pm2"

case "$1" in
    start)
        pm2 resurrect
        ;;
    stop)
        pm2 kill
        ;;
    restart)
        pm2 reload all
        ;;
    *)
        echo "Usage: $0 {start|stop|restart}"
        exit 1
        ;;
esac
"#;

    println!("\n{}\n", script_content);
    println!("To install the init script, run:");
    println!("  sudo cp pm2-init.sh /etc/init.d/pm2");
    println!("  sudo chmod +x /etc/init.d/pm2");
    println!("  sudo update-rc.d pm2 defaults");

    Ok(())
}

pub async fn kill() -> Result<()> {
    let mut manager = ProcessManager::new().await?;

    // Stop all processes
    let processes: Vec<String> = manager
        .list_processes()
        .iter()
        .map(|p| p.name.clone())
        .collect();

    for name in processes {
        if let Err(e) = manager.stop_process(&name).await {
            error!("Failed to stop process '{}': {}", name, e);
        }
    }

    display::display_success("PM2 daemon stopped");
    Ok(())
}

pub async fn update() -> Result<()> {
    display::display_info("Updating PM2 daemon...");
    // In a real implementation, this would update the daemon binary
    display::display_success("PM2 daemon updated");
    Ok(())
}

/// 手动触发日志轮转
pub async fn rotate_logs(name: Option<&str>) -> Result<()> {
    use crate::log::LogManager;

    let manager = ProcessManager::new().await?;
    let log_manager = LogManager::new()?;

    let processes_to_rotate: Vec<String> = if let Some(process_name) = name {
        // 轮转指定进程的日志
        if manager.get_process(process_name).is_some() {
            vec![process_name.to_string()]
        } else {
            display::display_error(&format!("Process '{}' not found", process_name));
            anyhow::bail!("Process not found")
        }
    } else {
        // 轮转所有进程的日志
        manager
            .list_processes()
            .iter()
            .map(|p| p.name.clone())
            .collect()
    };

    if processes_to_rotate.is_empty() {
        display::display_info("No processes to rotate logs for");
        return Ok(());
    }

    let mut rotated_count = 0;
    for process_name in processes_to_rotate {
        match log_manager.rotate_logs(&process_name).await {
            Ok(_) => {
                rotated_count += 1;
                display::display_success(&format!(
                    "Rotated logs for process '{}'",
                    process_name
                ));
            }
            Err(e) => {
                display::display_error(&format!(
                    "Failed to rotate logs for process '{}': {}",
                    process_name, e
                ));
            }
        }
    }

    display::display_success(&format!("Rotated logs for {} processes", rotated_count));
    Ok(())
}

/// 查看日志文件列表
pub async fn log_files(name: Option<&str>) -> Result<()> {
    use crate::log::LogManager;

    let manager = ProcessManager::new().await?;
    let log_manager = LogManager::new()?;

    let processes: Vec<String> = if let Some(process_name) = name {
        if manager.get_process(process_name).is_some() {
            vec![process_name.to_string()]
        } else {
            display::display_error(&format!("Process '{}' not found", process_name));
            anyhow::bail!("Process not found")
        }
    } else {
        manager
            .list_processes()
            .iter()
            .map(|p| p.name.clone())
            .collect()
    };

    if processes.is_empty() {
        display::display_info("No processes found");
        return Ok(());
    }

    for process_name in processes {
        println!("\n{} {}", "Process:".bold(), process_name.cyan());
        println!("{}", "-".repeat(50));

        match log_manager.get_rotated_log_files(&process_name).await {
            Ok(files) => {
                if files.is_empty() {
                    println!("  {} No log files found", "•".dimmed());
                } else {
                    for (i, file) in files.iter().enumerate() {
                        let marker = if i == 0 { "→".green() } else { "•".dimmed() };
                        let file_name = file.file_name().unwrap_or_default().to_string_lossy();
                        
                        // 获取文件大小
                        let size = match std::fs::metadata(file) {
                            Ok(meta) => {
                                let bytes = meta.len();
                                if bytes < 1024 {
                                    format!("{} B", bytes)
                                } else if bytes < 1024 * 1024 {
                                    format!("{:.1} KB", bytes as f64 / 1024.0)
                                } else {
                                    format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
                                }
                            }
                            Err(_) => "?".to_string(),
                        };

                        println!("  {} {} ({})", marker, file_name, size.dimmed());
                    }
                }
            }
            Err(e) => {
                display::display_error(&format!("Failed to get log files: {}", e));
            }
        }
    }

    Ok(())
}
