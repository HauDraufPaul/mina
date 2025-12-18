use serde::{Deserialize, Serialize};
use sysinfo::{System, Pid, Process};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProcessInfo {
    pub pid: u32,
    pub name: String,
    pub cpu_usage: f32,
    pub memory_usage: u64,
    pub status: String,
    pub parent_pid: Option<u32>,
}

pub struct ProcessProvider {
    system: System,
}

impl ProcessProvider {
    pub fn new() -> Self {
        let mut system = System::new_all();
        system.refresh_all();
        ProcessProvider { system }
    }

    pub fn refresh(&mut self) {
        self.system.refresh_all();
    }

    pub fn get_processes(&self) -> Vec<ProcessInfo> {
        let mut processes = Vec::new();
        
        for (pid, process) in self.system.processes() {
            processes.push(ProcessInfo {
                pid: pid.as_u32(),
                name: process.name().to_string(),
                cpu_usage: process.cpu_usage(),
                memory_usage: process.memory(),
                status: format!("{:?}", process.status()),
                parent_pid: process.parent().map(|p| p.as_u32()),
            });
        }
        
        // Sort by CPU usage descending
        processes.sort_by(|a, b| b.cpu_usage.partial_cmp(&a.cpu_usage).unwrap());
        
        processes
    }

    pub fn get_process(&self, pid: u32) -> Option<ProcessInfo> {
        let pid = Pid::from_u32(pid);
        self.system.process(pid).map(|process| {
            ProcessInfo {
                pid: pid.as_u32(),
                name: process.name().to_string(),
                cpu_usage: process.cpu_usage(),
                memory_usage: process.memory(),
                status: format!("{:?}", process.status()),
                parent_pid: process.parent().map(|p| p.as_u32()),
            }
        })
    }

    pub fn kill_process(&mut self, pid: u32) -> Result<(), String> {
        let pid = Pid::from_u32(pid);
        if let Some(process) = self.system.process(pid) {
            process.kill();
            Ok(())
        } else {
            Err(format!("Process {} not found", pid.as_u32()))
        }
    }
}

impl Default for ProcessProvider {
    fn default() -> Self {
        Self::new()
    }
}

