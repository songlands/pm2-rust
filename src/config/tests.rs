#[cfg(test)]
mod tests {
    use super::super::*;

    #[test]
    fn test_parse_memory_string() {
        assert_eq!(parse_memory_string("100M"), Some(100));
        assert_eq!(parse_memory_string("100MB"), Some(100));
        assert_eq!(parse_memory_string("1G"), Some(1024));
        assert_eq!(parse_memory_string("1GB"), Some(1024));
        assert_eq!(parse_memory_string("512"), Some(512));
    }

    #[test]
    fn test_parse_memory_string_kb() {
        assert_eq!(parse_memory_string("1024K"), Some(1));
        assert_eq!(parse_memory_string("1024KB"), Some(1));
    }

    #[test]
    fn test_parse_memory_string_invalid() {
        assert_eq!(parse_memory_string("invalid"), None);
        assert_eq!(parse_memory_string(""), None);
    }

    #[test]
    fn test_app_config_parse_memory_limit() {
        let mut config = AppConfig {
            name: "test".to_string(),
            script: "app.js".to_string(),
            instances: 1,
            exec_mode: ExecMode::Fork,
            watch: false,
            ignore_watch: vec![],
            max_memory_restart: Some("512M".to_string()),
            log_file: None,
            error_file: None,
            out_file: None,
            pid_file: None,
            env: std::collections::HashMap::new(),
            env_production: std::collections::HashMap::new(),
            env_development: std::collections::HashMap::new(),
            args: vec![],
            node_args: vec![],
            cwd: ".".to_string(),
            autorestart: false,
            max_restarts: 15,
            min_uptime: None,
            listen_timeout: None,
            kill_timeout: None,
            cron_restart: None,
            merge_logs: false,
            log_date_format: None,
            log_rotate_size: None,
            log_rotate_count: 10,
            log_rotate_interval: None,
        };

        assert_eq!(config.parse_memory_limit(), Some(512));

        config.max_memory_restart = Some("1G".to_string());
        assert_eq!(config.parse_memory_limit(), Some(1024));

        config.max_memory_restart = None;
        assert_eq!(config.parse_memory_limit(), None);
    }

    #[test]
    fn test_app_config_defaults() {
        let config = AppConfig {
            name: "test".to_string(),
            script: "app.js".to_string(),
            instances: 1,
            exec_mode: ExecMode::Fork,
            watch: false,
            ignore_watch: vec![],
            max_memory_restart: None,
            log_file: None,
            error_file: None,
            out_file: None,
            pid_file: None,
            env: std::collections::HashMap::new(),
            env_production: std::collections::HashMap::new(),
            env_development: std::collections::HashMap::new(),
            args: vec![],
            node_args: vec![],
            cwd: ".".to_string(),
            autorestart: false,
            max_restarts: 15,
            min_uptime: None,
            listen_timeout: None,
            kill_timeout: None,
            cron_restart: None,
            merge_logs: false,
            log_date_format: None,
            log_rotate_size: None,
            log_rotate_count: 10,
            log_rotate_interval: None,
        };

        assert_eq!(config.instances, 1);
        assert_eq!(config.exec_mode, ExecMode::Fork);
        assert!(!config.watch);
        assert_eq!(config.cwd, ".");
        assert_eq!(config.max_restarts, 15);
        assert_eq!(config.log_rotate_count, 10);
    }

    #[test]
    fn test_exec_mode_default() {
        assert_eq!(ExecMode::default(), ExecMode::Fork);
    }

    #[test]
    fn test_exec_mode_equality() {
        assert_eq!(ExecMode::Fork, ExecMode::Fork);
        assert_eq!(ExecMode::Cluster, ExecMode::Cluster);
        assert_ne!(ExecMode::Fork, ExecMode::Cluster);
    }

    #[test]
    fn test_parse_env_vars() {
        use super::super::parser::ConfigParser;
        
        let env_list = vec![
            "NODE_ENV=production".to_string(),
            "PORT=3000".to_string(),
            "DEBUG=true".to_string(),
        ];

        let env_map = ConfigParser::parse_env_vars(&env_list);
        
        assert_eq!(env_map.get("NODE_ENV"), Some(&"production".to_string()));
        assert_eq!(env_map.get("PORT"), Some(&"3000".to_string()));
        assert_eq!(env_map.get("DEBUG"), Some(&"true".to_string()));
    }

    #[test]
    fn test_parse_env_vars_empty() {
        use super::super::parser::ConfigParser;
        
        let env_list: Vec<String> = vec![];
        let env_map = ConfigParser::parse_env_vars(&env_list);
        
        assert!(env_map.is_empty());
    }

    #[test]
    fn test_parse_env_vars_invalid() {
        use super::super::parser::ConfigParser;
        
        let env_list = vec!["INVALID_VAR".to_string()];
        let env_map = ConfigParser::parse_env_vars(&env_list);
        
        assert!(env_map.is_empty());
    }
}
