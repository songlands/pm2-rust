#[cfg(test)]
mod tests {
    use super::super::*;
    use std::collections::HashMap;

    #[test]
    fn test_process_info_new() {
        let env_vars = HashMap::new();
        let process = ProcessInfo::new(
            "test-process".to_string(),
            "/path/to/script.js".to_string(),
            1,
            env_vars,
            None,
            None,
            None,
            false,
            false,
        );

        assert_eq!(process.name, "test-process");
        assert_eq!(process.script, "/path/to/script.js");
        assert_eq!(process.instances, 1);
        assert_eq!(process.status, ProcessStatus::Stopped);
        assert_eq!(process.restart_count, 0);
        assert!(process.pid.is_none());
        assert!(!process.cluster_mode);
        assert!(!process.watch);
        assert_eq!(process.log_rotate_count, 10);
    }

    #[test]
    fn test_process_info_new_with_instances() {
        let env_vars = HashMap::new();
        let process = ProcessInfo::new(
            "test-process".to_string(),
            "/path/to/script.js".to_string(),
            4,
            env_vars,
            None,
            None,
            None,
            false,
            true,
        );

        assert_eq!(process.instances, 4);
        assert!(process.cluster_mode);
    }

    #[test]
    fn test_process_info_update_status() {
        let env_vars = HashMap::new();
        let mut process = ProcessInfo::new(
            "test-process".to_string(),
            "/path/to/script.js".to_string(),
            1,
            env_vars,
            None,
            None,
            None,
            false,
            false,
        );

        let old_updated_at = process.updated_at;
        std::thread::sleep(std::time::Duration::from_millis(10));

        process.update_status(ProcessStatus::Online);

        assert_eq!(process.status, ProcessStatus::Online);
        assert!(process.updated_at > old_updated_at);
    }

    #[test]
    fn test_process_status_display() {
        assert_eq!(ProcessStatus::Online.to_string(), "online");
        assert_eq!(ProcessStatus::Stopped.to_string(), "stopped");
        assert_eq!(ProcessStatus::Stopping.to_string(), "stopping");
        assert_eq!(ProcessStatus::Launching.to_string(), "launching");
        assert_eq!(ProcessStatus::Errored.to_string(), "errored");
        assert_eq!(ProcessStatus::OneLaunchStatus.to_string(), "one-launch-status");
    }

    #[test]
    fn test_process_status_equality() {
        assert_eq!(ProcessStatus::Online, ProcessStatus::Online);
        assert_ne!(ProcessStatus::Online, ProcessStatus::Stopped);
    }

    #[test]
    fn test_process_info_with_env_vars() {
        let mut env_vars = HashMap::new();
        env_vars.insert("NODE_ENV".to_string(), "production".to_string());
        env_vars.insert("PORT".to_string(), "3000".to_string());

        let process = ProcessInfo::new(
            "test-process".to_string(),
            "/path/to/script.js".to_string(),
            1,
            env_vars,
            None,
            None,
            None,
            false,
            false,
        );

        assert_eq!(process.env_vars.get("NODE_ENV"), Some(&"production".to_string()));
        assert_eq!(process.env_vars.get("PORT"), Some(&"3000".to_string()));
    }

    #[test]
    fn test_process_info_with_log_config() {
        let env_vars = HashMap::new();
        let process = ProcessInfo::new(
            "test-process".to_string(),
            "/path/to/script.js".to_string(),
            1,
            env_vars,
            Some("/var/log/out.log".to_string()),
            Some("/var/log/err.log".to_string()),
            Some(512),
            true,
            false,
        );

        assert_eq!(process.log_file, Some("/var/log/out.log".to_string()));
        assert_eq!(process.error_log_file, Some("/var/log/err.log".to_string()));
        assert_eq!(process.max_memory_restart, Some(512));
        assert!(process.watch);
    }

    #[test]
    fn test_default_log_rotate_count() {
        assert_eq!(default_log_rotate_count(), 10);
    }

    #[test]
    fn test_process_info_id_is_uuid() {
        let env_vars = HashMap::new();
        let process = ProcessInfo::new(
            "test-process".to_string(),
            "/path/to/script.js".to_string(),
            1,
            env_vars,
            None,
            None,
            None,
            false,
            false,
        );

        assert!(!process.id.is_empty());
        assert!(process.id.contains('-'))
    }

    #[test]
    fn test_process_info_id_unique() {
        let env_vars = HashMap::new();
        let process1 = ProcessInfo::new(
            "test-process-1".to_string(),
            "/path/to/script.js".to_string(),
            1,
            env_vars.clone(),
            None,
            None,
            None,
            false,
            false,
        );

        let process2 = ProcessInfo::new(
            "test-process-2".to_string(),
            "/path/to/script.js".to_string(),
            1,
            env_vars,
            None,
            None,
            None,
            false,
            false,
        );

        assert_ne!(process1.id, process2.id);
    }
}
