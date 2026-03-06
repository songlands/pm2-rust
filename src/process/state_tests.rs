#[cfg(test)]
mod tests {
    use super::super::*;
    use super::super::state::ProcessState;
    use std::collections::HashMap;

    fn create_test_process(name: &str) -> ProcessInfo {
        ProcessInfo::new(
            name.to_string(),
            "/path/to/script.js".to_string(),
            1,
            HashMap::new(),
            None,
            None,
            None,
            false,
            false,
        )
    }

    #[test]
    fn test_process_state_new() {
        let state = ProcessState::new();
        assert!(state.processes.is_empty());
        assert_eq!(state.version, env!("CARGO_PKG_VERSION"));
    }

    #[test]
    fn test_add_process() {
        let mut state = ProcessState::new();
        let process = create_test_process("test-process");
        let name = process.name.clone();

        state.add_process(process);

        assert_eq!(state.processes.len(), 1);
        assert!(state.processes.contains_key(&name));
    }

    #[test]
    fn test_remove_process() {
        let mut state = ProcessState::new();
        let process = create_test_process("test-process");
        let name = process.name.clone();

        state.add_process(process);
        let removed = state.remove_process(&name);

        assert!(removed.is_some());
        assert_eq!(removed.unwrap().name, name);
        assert!(state.processes.is_empty());
    }

    #[test]
    fn test_remove_nonexistent_process() {
        let mut state = ProcessState::new();
        let removed = state.remove_process("nonexistent");
        assert!(removed.is_none());
    }

    #[test]
    fn test_get_process() {
        let mut state = ProcessState::new();
        let process = create_test_process("test-process");
        let name = process.name.clone();

        state.add_process(process);
        let retrieved = state.get_process(&name);

        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().name, name);
    }

    #[test]
    fn test_get_nonexistent_process() {
        let state = ProcessState::new();
        let retrieved = state.get_process("nonexistent");
        assert!(retrieved.is_none());
    }

    #[test]
    fn test_find_by_id_exact_match() {
        let mut state = ProcessState::new();
        let process = create_test_process("test-process");
        let id = process.id.clone();

        state.add_process(process);
        let found = state.find_by_id(&id);

        assert!(found.is_some());
        assert_eq!(found.unwrap().id, id);
    }

    #[test]
    fn test_find_by_id_prefix_match() {
        let mut state = ProcessState::new();
        let process = create_test_process("test-process");
        let id = process.id.clone();
        let prefix = &id[..8];

        state.add_process(process);
        let found = state.find_by_id(prefix);

        assert!(found.is_some());
        assert!(found.unwrap().id.starts_with(prefix));
    }

    #[test]
    fn test_find_by_id_not_found() {
        let state = ProcessState::new();
        let found = state.find_by_id("nonexistent-id");
        assert!(found.is_none());
    }

    #[test]
    fn test_list_processes() {
        let mut state = ProcessState::new();
        let process1 = create_test_process("process-1");
        let process2 = create_test_process("process-2");

        state.add_process(process1);
        state.add_process(process2);

        let list = state.list_processes();
        assert_eq!(list.len(), 2);
    }

    #[test]
    fn test_update_process_status() {
        let mut state = ProcessState::new();
        let process = create_test_process("test-process");
        let name = process.name.clone();

        state.add_process(process);
        state.update_process_status(&name, ProcessStatus::Online);

        let updated = state.get_process(&name).unwrap();
        assert_eq!(updated.status, ProcessStatus::Online);
    }

    #[test]
    fn test_update_process_status_nonexistent() {
        let mut state = ProcessState::new();
        state.update_process_status("nonexistent", ProcessStatus::Online);
    }

    #[test]
    fn test_update_process_pid() {
        let mut state = ProcessState::new();
        let process = create_test_process("test-process");
        let name = process.name.clone();

        state.add_process(process);
        state.update_process_pid(&name, Some(12345));

        let updated = state.get_process(&name).unwrap();
        assert_eq!(updated.pid, Some(12345));
    }

    #[test]
    fn test_update_process_pid_to_none() {
        let mut state = ProcessState::new();
        let mut process = create_test_process("test-process");
        process.pid = Some(12345);
        let name = process.name.clone();

        state.add_process(process);
        state.update_process_pid(&name, None);

        let updated = state.get_process(&name).unwrap();
        assert!(updated.pid.is_none());
    }

    #[test]
    fn test_increment_restart_count() {
        let mut state = ProcessState::new();
        let process = create_test_process("test-process");
        let name = process.name.clone();

        state.add_process(process);
        assert_eq!(state.get_process(&name).unwrap().restart_count, 0);

        state.increment_restart_count(&name);
        assert_eq!(state.get_process(&name).unwrap().restart_count, 1);

        state.increment_restart_count(&name);
        assert_eq!(state.get_process(&name).unwrap().restart_count, 2);
    }

    #[test]
    fn test_find_by_pid() {
        let mut state = ProcessState::new();
        let mut process = create_test_process("test-process");
        process.pid = Some(12345);

        state.add_process(process);
        let found = state.find_by_pid(12345);

        assert!(found.is_some());
        assert_eq!(found.unwrap().pid, Some(12345));
    }

    #[test]
    fn test_find_by_pid_not_found() {
        let state = ProcessState::new();
        let found = state.find_by_pid(99999);
        assert!(found.is_none());
    }

    #[test]
    fn test_update_metrics() {
        let mut state = ProcessState::new();
        let process = create_test_process("test-process");
        let name = process.name.clone();

        state.add_process(process);
        state.update_metrics(&name, 50.5, 1024.5, 3600);

        let updated = state.get_process(&name).unwrap();
        assert_eq!(updated.cpu_percent, 50.5);
        assert_eq!(updated.memory_mb, 1024.5);
        assert_eq!(updated.uptime_seconds, 3600);
    }
}
