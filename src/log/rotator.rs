use anyhow::{Context, Result};
use chrono::{Local, NaiveDateTime};
use std::path::PathBuf;
use tokio::fs;
use tracing::info;

/// 日志轮转器
pub struct LogRotator {
    /// 日志文件路径
    log_path: PathBuf,
    /// 轮转大小限制（字节）
    rotate_size: Option<u64>,
    /// 保留的轮转文件数量
    rotate_count: u32,
    /// 轮转时间间隔（秒）
    rotate_interval: Option<u64>,
    /// 上次轮转时间
    last_rotation: Option<NaiveDateTime>,
}

impl LogRotator {
    /// 创建新的日志轮转器
    pub fn new(
        log_path: PathBuf,
        rotate_size: Option<u64>,
        rotate_count: u32,
        rotate_interval: Option<u64>,
    ) -> Self {
        Self {
            log_path,
            rotate_size,
            rotate_count,
            rotate_interval,
            last_rotation: None,
        }
    }

    pub async fn should_rotate(&mut self) -> Result<bool> {
        if !self.log_path.exists() {
            return Ok(false);
        }

        if let Some(max_size) = self.rotate_size {
            let metadata = fs::metadata(&self.log_path).await?;
            if metadata.len() >= max_size {
                info!(
                    "Log file {} reached size limit ({} >= {} bytes)",
                    self.log_path.display(),
                    metadata.len(),
                    max_size
                );
                return Ok(true);
            }
        }

        if let Some(interval) = self.rotate_interval {
            let now = Local::now().naive_local();
            if let Some(last) = self.last_rotation {
                let elapsed = now.signed_duration_since(last).num_seconds() as u64;
                if elapsed >= interval {
                    info!(
                        "Log file {} reached time interval ({} >= {} seconds)",
                        self.log_path.display(),
                        elapsed,
                        interval
                    );
                    return Ok(true);
                }
            } else {
                self.last_rotation = Some(now);
            }
        }

        Ok(false)
    }

    pub async fn rotate(&mut self) -> Result<()> {
        if !self.log_path.exists() {
            return Ok(());
        }

        info!("Rotating log file: {}", self.log_path.display());

        let oldest = self.get_rotated_path(self.rotate_count);
        if oldest.exists() {
            fs::remove_file(&oldest).await.with_context(|| {
                format!("Failed to remove old log file: {}", oldest.display())
            })?;
            info!("Removed old log file: {}", oldest.display());
        }

        for i in (1..self.rotate_count).rev() {
            let from = self.get_rotated_path(i);
            let to = self.get_rotated_path(i + 1);

            if from.exists() {
                fs::rename(&from, &to).await.with_context(|| {
                    format!(
                        "Failed to rename log file from {} to {}",
                        from.display(),
                        to.display()
                    )
                })?;
            }
        }

        let rotated = self.get_rotated_path(1);
        fs::rename(&self.log_path, &rotated).await.with_context(|| {
            format!(
                "Failed to rotate log file from {} to {}",
                self.log_path.display(),
                rotated.display()
            )
        })?;

        self.last_rotation = Some(Local::now().naive_local());

        info!(
            "Log file rotated: {} -> {}",
            self.log_path.display(),
            rotated.display()
        );

        Ok(())
    }

    /// 获取轮转后的文件路径
    fn get_rotated_path(&self, index: u32) -> PathBuf {
        let file_name = self.log_path.file_stem().unwrap_or_default();
        let extension = self.log_path.extension().unwrap_or_default();

        let rotated_name = if extension.is_empty() {
            format!("{}.{}.log", file_name.to_string_lossy(), index)
        } else {
            format!(
                "{}.{}.log.{}",
                file_name.to_string_lossy(),
                index,
                extension.to_string_lossy()
            )
        };

        self.log_path.with_file_name(rotated_name)
    }

    #[allow(dead_code)]
    pub async fn check_and_rotate(&mut self) -> Result<bool> {
        if self.should_rotate().await? {
            self.rotate().await?;
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

/// 解析大小字符串（如 "10M", "1G"）
pub fn parse_size_string(s: &str) -> Option<u64> {
    let s = s.trim().to_uppercase();

    if s.ends_with("GB") || s.ends_with("G") {
        s.trim_end_matches("GB")
            .trim_end_matches("G")
            .trim()
            .parse::<u64>()
            .ok()
            .map(|v| v * 1024 * 1024 * 1024)
    } else if s.ends_with("MB") || s.ends_with("M") {
        s.trim_end_matches("MB")
            .trim_end_matches("M")
            .trim()
            .parse::<u64>()
            .ok()
            .map(|v| v * 1024 * 1024)
    } else if s.ends_with("KB") || s.ends_with("K") {
        s.trim_end_matches("KB")
            .trim_end_matches("K")
            .trim()
            .parse::<u64>()
            .ok()
            .map(|v| v * 1024)
    } else if s.ends_with("B") {
        s.trim_end_matches("B")
            .trim()
            .parse::<u64>()
            .ok()
    } else {
        s.parse::<u64>().ok()
    }
}

/// 解析时间间隔字符串（如 "1d", "12h", "30m"）
pub fn parse_interval_string(s: &str) -> Option<u64> {
    let s = s.trim().to_lowercase();

    if s.ends_with("d") || s.ends_with("day") || s.ends_with("days") {
        s.trim_end_matches("days")
            .trim_end_matches("day")
            .trim_end_matches("d")
            .trim()
            .parse::<u64>()
            .ok()
            .map(|v| v * 24 * 60 * 60)
    } else if s.ends_with("h") || s.ends_with("hour") || s.ends_with("hours") {
        s.trim_end_matches("hours")
            .trim_end_matches("hour")
            .trim_end_matches("h")
            .trim()
            .parse::<u64>()
            .ok()
            .map(|v| v * 60 * 60)
    } else if s.ends_with("m") || s.ends_with("min") || s.ends_with("minute") || s.ends_with("minutes") {
        s.trim_end_matches("minutes")
            .trim_end_matches("minute")
            .trim_end_matches("min")
            .trim_end_matches("m")
            .trim()
            .parse::<u64>()
            .ok()
            .map(|v| v * 60)
    } else if s.ends_with("s") || s.ends_with("sec") || s.ends_with("second") || s.ends_with("seconds") {
        s.trim_end_matches("seconds")
            .trim_end_matches("second")
            .trim_end_matches("sec")
            .trim_end_matches("s")
            .trim()
            .parse::<u64>()
            .ok()
    } else {
        s.parse::<u64>().ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_size_string() {
        assert_eq!(parse_size_string("10B"), Some(10));
        assert_eq!(parse_size_string("10K"), Some(10 * 1024));
        assert_eq!(parse_size_string("10KB"), Some(10 * 1024));
        assert_eq!(parse_size_string("10M"), Some(10 * 1024 * 1024));
        assert_eq!(parse_size_string("10MB"), Some(10 * 1024 * 1024));
        assert_eq!(parse_size_string("1G"), Some(1 * 1024 * 1024 * 1024));
        assert_eq!(parse_size_string("1GB"), Some(1 * 1024 * 1024 * 1024));
        assert_eq!(parse_size_string("100"), Some(100));
    }

    #[test]
    fn test_parse_interval_string() {
        assert_eq!(parse_interval_string("30s"), Some(30));
        assert_eq!(parse_interval_string("30sec"), Some(30));
        assert_eq!(parse_interval_string("5m"), Some(5 * 60));
        assert_eq!(parse_interval_string("5min"), Some(5 * 60));
        assert_eq!(parse_interval_string("2h"), Some(2 * 60 * 60));
        assert_eq!(parse_interval_string("2hour"), Some(2 * 60 * 60));
        assert_eq!(parse_interval_string("1d"), Some(1 * 24 * 60 * 60));
        assert_eq!(parse_interval_string("1day"), Some(1 * 24 * 60 * 60));
        assert_eq!(parse_interval_string("3600"), Some(3600));
    }
}
