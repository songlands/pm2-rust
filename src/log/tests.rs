#[cfg(test)]
mod tests {
    use super::super::*;

    #[test]
    fn test_parse_size_string_gb() {
        assert_eq!(parse_size_string("1G"), Some(1024 * 1024 * 1024));
        assert_eq!(parse_size_string("1GB"), Some(1024 * 1024 * 1024));
        assert_eq!(parse_size_string("2g"), Some(2 * 1024 * 1024 * 1024));
    }

    #[test]
    fn test_parse_size_string_mb() {
        assert_eq!(parse_size_string("10M"), Some(10 * 1024 * 1024));
        assert_eq!(parse_size_string("10MB"), Some(10 * 1024 * 1024));
        assert_eq!(parse_size_string("5m"), Some(5 * 1024 * 1024));
    }

    #[test]
    fn test_parse_size_string_kb() {
        assert_eq!(parse_size_string("100K"), Some(100 * 1024));
        assert_eq!(parse_size_string("100KB"), Some(100 * 1024));
        assert_eq!(parse_size_string("50k"), Some(50 * 1024));
    }

    #[test]
    fn test_parse_size_string_bytes() {
        assert_eq!(parse_size_string("1024B"), Some(1024));
        assert_eq!(parse_size_string("1024"), Some(1024));
    }

    #[test]
    fn test_parse_size_string_invalid() {
        assert_eq!(parse_size_string("invalid"), None);
        assert_eq!(parse_size_string(""), None);
    }

    #[test]
    fn test_parse_interval_string_days() {
        assert_eq!(parse_interval_string("1d"), Some(24 * 60 * 60));
        assert_eq!(parse_interval_string("1day"), Some(24 * 60 * 60));
        assert_eq!(parse_interval_string("2days"), Some(2 * 24 * 60 * 60));
    }

    #[test]
    fn test_parse_interval_string_hours() {
        assert_eq!(parse_interval_string("1h"), Some(60 * 60));
        assert_eq!(parse_interval_string("1hour"), Some(60 * 60));
        assert_eq!(parse_interval_string("12hours"), Some(12 * 60 * 60));
    }

    #[test]
    fn test_parse_interval_string_minutes() {
        assert_eq!(parse_interval_string("1m"), Some(60));
        assert_eq!(parse_interval_string("1min"), Some(60));
        assert_eq!(parse_interval_string("30minutes"), Some(30 * 60));
    }

    #[test]
    fn test_parse_interval_string_seconds() {
        assert_eq!(parse_interval_string("30s"), Some(30));
        assert_eq!(parse_interval_string("30sec"), Some(30));
        assert_eq!(parse_interval_string("30seconds"), Some(30));
    }

    #[test]
    fn test_parse_interval_string_invalid() {
        assert_eq!(parse_interval_string("invalid"), None);
        assert_eq!(parse_interval_string(""), None);
    }

    #[test]
    fn test_log_manager_new() {
        let manager = LogManager::new();
        assert!(manager.is_ok());
    }

    #[test]
    fn test_log_manager_default() {
        let manager = LogManager::default();
        assert!(manager.get_log_path("test").to_string_lossy().contains("test-out.log"));
    }

    #[test]
    fn test_log_manager_get_log_path() {
        let manager = LogManager::new().unwrap();
        let path = manager.get_log_path("my-process");
        let path_str = path.to_string_lossy();
        assert!(path_str.contains("my-process-out.log"));
    }

    #[test]
    fn test_log_manager_get_error_log_path() {
        let manager = LogManager::new().unwrap();
        let path = manager.get_error_log_path("my-process");
        let path_str = path.to_string_lossy();
        assert!(path_str.contains("my-process-error.log"));
    }

}
