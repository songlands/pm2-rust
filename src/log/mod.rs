use anyhow::{Context, Result};
use chrono::Local;
use std::path::PathBuf;
use tokio::fs::{self, OpenOptions};
use tokio::io::AsyncWriteExt;
use tracing::info;

pub mod rotator;

pub use rotator::{parse_interval_string, parse_size_string, LogRotator};

pub struct LogManager {
    log_dir: PathBuf,
}

impl LogManager {
    pub fn new() -> Result<Self> {
        // Use current directory for logs to avoid permission issues
        let current_dir = std::env::current_dir().context("Failed to get current directory")?;
        let log_dir = current_dir.join(".pm2").join("logs");

        Ok(Self { log_dir })
    }

    pub async fn ensure_log_dir(&self) -> Result<()> {
        fs::create_dir_all(&self.log_dir).await.with_context(|| {
            format!("Failed to create log directory: {}", self.log_dir.display())
        })?;
        Ok(())
    }

    pub fn get_log_path(&self, process_name: &str) -> PathBuf {
        self.log_dir.join(format!("{}-out.log", process_name))
    }

    pub fn get_error_log_path(&self, process_name: &str) -> PathBuf {
        self.log_dir.join(format!("{}-error.log", process_name))
    }

    pub async fn write_log(&self, process_name: &str, message: &str) -> Result<()> {
        self.ensure_log_dir().await?;

        let log_path = self.get_log_path(process_name);
        let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S");
        let log_line = format!("[{}] {}\n", timestamp, message);

        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&log_path)
            .await
            .with_context(|| format!("Failed to open log file: {}", log_path.display()))?;

        file.write_all(log_line.as_bytes()).await.with_context(|| {
            format!("Failed to write to log file: {}", log_path.display())
        })?;

        Ok(())
    }

    pub async fn write_error_log(&self, process_name: &str, message: &str) -> Result<()> {
        self.ensure_log_dir().await?;

        let log_path = self.get_error_log_path(process_name);
        let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S");
        let log_line = format!("[{}] {}\n", timestamp, message);

        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&log_path)
            .await
            .with_context(|| format!("Failed to open error log file: {}", log_path.display()))?;

        file.write_all(log_line.as_bytes()).await.with_context(|| {
            format!("Failed to write to error log file: {}", log_path.display())
        })?;

        Ok(())
    }

    pub async fn clear_logs(&self, process_name: &str) -> Result<()> {
        let out_log = self.get_log_path(process_name);
        let err_log = self.get_error_log_path(process_name);

        if out_log.exists() {
            fs::write(&out_log, "").await.with_context(|| {
                format!("Failed to clear log file: {}", out_log.display())
            })?;
            info!("Cleared log file: {}", out_log.display());
        }

        if err_log.exists() {
            fs::write(&err_log, "").await.with_context(|| {
                format!("Failed to clear error log file: {}", err_log.display())
            })?;
            info!("Cleared error log file: {}", err_log.display());
        }

        Ok(())
    }

    pub async fn get_log_files(&self) -> Result<Vec<PathBuf>> {
        self.ensure_log_dir().await?;

        let mut entries = fs::read_dir(&self.log_dir).await?;
        let mut files = Vec::new();

        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            if path.extension().map(|e| e == "log").unwrap_or(false) {
                files.push(path);
            }
        }

        Ok(files)
    }

    pub async fn read_last_lines(&self, file_path: &PathBuf, lines: usize) -> Result<Vec<String>> {
        use tokio::io::AsyncBufReadExt;

        let file = fs::File::open(file_path).await?;
        let reader = tokio::io::BufReader::new(file);
        let mut all_lines = Vec::new();

        let mut lines_stream = reader.lines();
        while let Some(line) = lines_stream.next_line().await? {
            all_lines.push(line);
        }

        let start = all_lines.len().saturating_sub(lines);
        Ok(all_lines[start..].to_vec())
    }

    /// 创建日志轮转器
    pub fn create_rotator(
        &self,
        process_name: &str,
        rotate_size: Option<u64>,
        rotate_count: u32,
        rotate_interval: Option<u64>,
    ) -> LogRotator {
        let log_path = self.get_log_path(process_name);
        LogRotator::new(log_path, rotate_size, rotate_count, rotate_interval)
    }

    /// 创建错误日志轮转器
    pub fn create_error_rotator(
        &self,
        process_name: &str,
        rotate_size: Option<u64>,
        rotate_count: u32,
        rotate_interval: Option<u64>,
    ) -> LogRotator {
        let log_path = self.get_error_log_path(process_name);
        LogRotator::new(log_path, rotate_size, rotate_count, rotate_interval)
    }

    /// 轮转指定进程的日志
    pub async fn rotate_logs(&self, process_name: &str) -> Result<()> {
        let out_log = self.get_log_path(process_name);
        let err_log = self.get_error_log_path(process_name);

        // 轮转标准输出日志
        if out_log.exists() {
            let metadata = fs::metadata(&out_log).await?;
            if metadata.len() > 0 {
                let mut rotator = LogRotator::new(out_log, None, 10, None);
                rotator.rotate().await?;
            }
        }

        // 轮转错误日志
        if err_log.exists() {
            let metadata = fs::metadata(&err_log).await?;
            if metadata.len() > 0 {
                let mut rotator = LogRotator::new(err_log, None, 10, None);
                rotator.rotate().await?;
            }
        }

        Ok(())
    }

    /// 获取所有轮转后的日志文件
    pub async fn get_rotated_log_files(&self, process_name: &str) -> Result<Vec<PathBuf>> {
        self.ensure_log_dir().await?;

        let mut files = Vec::new();
        let out_log = self.get_log_path(process_name);
        let err_log = self.get_error_log_path(process_name);

        // 获取当前日志文件
        if out_log.exists() {
            files.push(out_log.clone());
        }
        if err_log.exists() {
            files.push(err_log.clone());
        }

        // 获取轮转后的文件
        let mut entries = fs::read_dir(&self.log_dir).await?;
        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            let file_name = path.file_stem().unwrap_or_default().to_string_lossy();

            // 检查是否是轮转文件
            let out_stem = out_log.file_stem().unwrap_or_default().to_string_lossy();
            let err_stem = err_log.file_stem().unwrap_or_default().to_string_lossy();

            if file_name.starts_with(&format!("{}.", out_stem))
                || file_name.starts_with(&format!("{}.", err_stem))
            {
                files.push(path);
            }
        }

        // 按修改时间排序
        files.sort_by(|a, b| {
            let a_time = std::fs::metadata(a).and_then(|m| m.modified()).ok();
            let b_time = std::fs::metadata(b).and_then(|m| m.modified()).ok();
            b_time.cmp(&a_time) // 最新的在前
        });

        Ok(files)
    }

    /// 删除指定进程的所有日志文件
    pub async fn delete_logs(&self, process_name: &str) -> Result<()> {
        let files = self.get_rotated_log_files(process_name).await?;

        for file in files {
            fs::remove_file(&file)
                .await
                .with_context(|| format!("Failed to delete log file: {}", file.display()))?;
            info!("Deleted log file: {}", file.display());
        }

        Ok(())
    }
}

impl Default for LogManager {
    fn default() -> Self {
        Self::new().expect("Failed to create LogManager")
    }
}
