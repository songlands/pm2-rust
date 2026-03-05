use crate::process::ProcessInfo;
use colored::Colorize;
use tabled::{Table, Tabled};

#[derive(Tabled)]
struct ProcessRow {
    id: String,
    name: String,
    #[tabled(rename = "mode")]
    mode: String,
    #[tabled(rename = "pid")]
    pid: String,
    status: String,
    #[tabled(rename = "restart")]
    restart: String,
    uptime: String,
    #[tabled(rename = "cpu")]
    cpu: String,
    #[tabled(rename = "mem")]
    mem: String,
    #[tabled(rename = "user")]
    user: String,
    #[tabled(rename = "watching")]
    watching: String,
}

impl ProcessRow {
    fn from_process(process: &ProcessInfo) -> Self {
        let mode = if process.cluster_mode {
            "cluster".to_string()
        } else {
            "fork".to_string()
        };

        let pid = process
            .pid
            .map(|p| p.to_string())
            .unwrap_or_else(|| "N/A".to_string());

        let status = format_status(&process.status.to_string());

        let uptime = format_duration(process.uptime_seconds);

        let cpu = format!("{:.1}%", process.cpu_percent);
        let mem = format!("{:.1}MB", process.memory_mb);

        let user = std::env::var("USER")
            .or_else(|_| std::env::var("USERNAME"))
            .unwrap_or_else(|_| "unknown".to_string());

        let watching = if process.watch {
            "enabled".to_string()
        } else {
            "disabled".to_string()
        };

        Self {
            id: process.id[..8].to_string(),
            name: process.name.clone(),
            mode,
            pid,
            status,
            restart: process.restart_count.to_string(),
            uptime,
            cpu,
            mem,
            user,
            watching,
        }
    }
}

fn format_status(status: &str) -> String {
    match status {
        "online" => status.green().to_string(),
        "stopped" => status.bright_black().to_string(),
        "stopping" => status.yellow().to_string(),
        "launching" => status.blue().to_string(),
        "errored" => status.red().to_string(),
        _ => status.to_string(),
    }
}

fn format_duration(seconds: u64) -> String {
    if seconds < 60 {
        format!("{}s", seconds)
    } else if seconds < 3600 {
        format!("{}m", seconds / 60)
    } else if seconds < 86400 {
        format!("{}h", seconds / 3600)
    } else {
        format!("{}d", seconds / 86400)
    }
}

pub fn display_process_list(processes: &[&ProcessInfo]) {
    if processes.is_empty() {
        println!("{}", "No processes found".yellow());
        return;
    }

    let rows: Vec<ProcessRow> = processes
        .iter()
        .map(|p| ProcessRow::from_process(p))
        .collect();

    let table = Table::new(rows);
    println!("{}", table);

    println!("\n{}: {}", "Total processes".bold(), processes.len());

    let online_count = processes
        .iter()
        .filter(|p| p.status.to_string() == "online")
        .count();
    println!("{}: {}", "Online".green().bold(), online_count);
}

pub fn display_process_details(process: &ProcessInfo) {
    println!("{}", "Process Details".bold().underline());
    println!("{:<20} {}", "ID:", process.id);
    println!("{:<20} {}", "Name:", process.name);
    println!("{:<20} {}", "Script:", process.script);
    println!(
        "{:<20} {}",
        "Status:",
        format_status(&process.status.to_string())
    );
    println!(
        "{:<20} {}",
        "PID:",
        process
            .pid
            .map(|p| p.to_string())
            .unwrap_or_else(|| "N/A".to_string())
    );
    println!("{:<20} {}", "Instances:", process.instances);
    println!("{:<20} {}", "Restart Count:", process.restart_count);
    println!("{:<20} {}", "Uptime:", format_duration(process.uptime_seconds));
    println!("{:<20} {:.1}%", "CPU Usage:", process.cpu_percent);
    println!("{:<20} {:.1}MB", "Memory Usage:", process.memory_mb);
    println!("{:<20} {}", "Cluster Mode:", process.cluster_mode);
    println!("{:<20} {}", "Watch:", process.watch);

    if let Some(log_file) = &process.log_file {
        println!("{:<20} {}", "Log File:", log_file);
    }

    if let Some(error_log) = &process.error_log_file {
        println!("{:<20} {}", "Error Log:", error_log);
    }

    if !process.env_vars.is_empty() {
        println!("\n{}", "Environment Variables:".bold());
        for (key, value) in &process.env_vars {
            println!("  {}={}", key, value);
        }
    }
}

pub fn display_success(message: &str) {
    println!("{} {}", "✓".green(), message.green());
}

pub fn display_error(message: &str) {
    eprintln!("{} {}", "✗".red(), message.red());
}

pub fn display_warning(message: &str) {
    println!("{} {}", "⚠".yellow(), message.yellow());
}

pub fn display_info(message: &str) {
    println!("{} {}", "ℹ".blue(), message);
}
