use anyhow::{Context, Result};
use std::path::PathBuf;
use tokio::fs;

pub mod rotator;

#[cfg(test)]
mod tests;

pub use rotator::{parse_interval_string, parse_size_string, LogRotator};

fn get_pm2_home() -> PathBuf {
    if let Ok(home) = std::env::var("PM2_HOME") {
        PathBuf::from(home)
    } else if let Some(home_dir) = dirs::home_dir() {
        home_dir.join(".pm2")
    } else {
        let current_dir = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        current_dir.join(".pm2")
    }
}

pub struct LogManager {
    log_dir: PathBuf,
}

impl LogManager {
    pub fn new() -> Result<Self> {
        let log_dir = get_pm2_home().join("logs");

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

    pub async fn rotate_logs(&self, process_name: &str) -> Result<()> {
        let out_log = self.get_log_path(process_name);
        let err_log = self.get_error_log_path(process_name);

        if out_log.exists() {
            let metadata = fs::metadata(&out_log).await?;
            if metadata.len() > 0 {
                let mut rotator = LogRotator::new(out_log, None, 10, None);
                rotator.rotate().await?;
            }
        }

        if err_log.exists() {
            let metadata = fs::metadata(&err_log).await?;
            if metadata.len() > 0 {
                let mut rotator = LogRotator::new(err_log, None, 10, None);
                rotator.rotate().await?;
            }
        }

        Ok(())
    }

    pub async fn get_rotated_log_files(&self, process_name: &str) -> Result<Vec<PathBuf>> {
        self.ensure_log_dir().await?;

        let mut files = Vec::new();
        let out_log = self.get_log_path(process_name);
        let err_log = self.get_error_log_path(process_name);

        if out_log.exists() {
            files.push(out_log.clone());
        }
        if err_log.exists() {
            files.push(err_log.clone());
        }

        let mut entries = fs::read_dir(&self.log_dir).await?;
        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            let file_name = path.file_stem().unwrap_or_default().to_string_lossy();

            let out_stem = out_log.file_stem().unwrap_or_default().to_string_lossy();
            let err_stem = err_log.file_stem().unwrap_or_default().to_string_lossy();

            if file_name.starts_with(&format!("{}.", out_stem))
                || file_name.starts_with(&format!("{}.", err_stem))
            {
                files.push(path);
            }
        }

        files.sort_by(|a, b| {
            let a_time = std::fs::metadata(a).and_then(|m| m.modified()).ok();
            let b_time = std::fs::metadata(b).and_then(|m| m.modified()).ok();
            b_time.cmp(&a_time)
        });

        Ok(files)
    }
}

impl Default for LogManager {
    fn default() -> Self {
        Self::new().expect("Failed to create LogManager")
    }
}
