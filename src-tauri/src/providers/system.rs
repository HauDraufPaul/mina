use sysinfo::System;
use std::sync::{Arc, Mutex};
use std::time::Instant;

pub struct SystemProvider {
    system: Arc<Mutex<System>>,
    last_network_check: Arc<Mutex<(Instant, u64, u64)>>,
}

impl SystemProvider {
    pub fn new() -> Self {
        let mut system = System::new_all();
        system.refresh_all();
        
        SystemProvider {
            system: Arc::new(Mutex::new(system)),
            last_network_check: Arc::new(Mutex::new((Instant::now(), 0, 0))),
        }
    }

    pub fn refresh(&self) {
        let mut system = self.system.lock().unwrap();
        system.refresh_all();
    }

    pub fn get_cpu_usage(&self) -> f64 {
        let system = self.system.lock().unwrap();
        system.global_cpu_info().cpu_usage() as f64
    }

    pub fn get_cpu_count(&self) -> usize {
        let system = self.system.lock().unwrap();
        system.cpus().len()
    }

    pub fn get_cpu_frequency(&self) -> u64 {
        let system = self.system.lock().unwrap();
        system.global_cpu_info().frequency()
    }

    pub fn get_memory_total(&self) -> u64 {
        let system = self.system.lock().unwrap();
        system.total_memory()
    }

    pub fn get_memory_used(&self) -> u64 {
        let system = self.system.lock().unwrap();
        system.used_memory()
    }

    pub fn get_memory_free(&self) -> u64 {
        let system = self.system.lock().unwrap();
        system.free_memory()
    }

    pub fn get_disk_metrics(&self) -> (u64, u64, u64) {
        use sysinfo::Disks;
        let disks = Disks::new_with_refreshed_list();
        for disk in disks.list() {
            let total = disk.total_space();
            let free = disk.available_space();
            let used = total - free;
            return (total, used, free);
        }
        (0, 0, 0)
    }

    pub fn get_network_speeds(&self) -> (f64, f64) {
        use sysinfo::Networks;
        let networks = Networks::new_with_refreshed_list();
        let mut total_rx = 0u64;
        let mut total_tx = 0u64;
        
        for (_, network) in networks.list() {
            total_rx += network.received();
            total_tx += network.transmitted();
        }

        let mut last_check = self.last_network_check.lock().unwrap();
        let now = Instant::now();
        let elapsed = now.duration_since(last_check.0);
        
        if elapsed.as_secs() > 0 {
            let rx_speed = if total_rx > last_check.1 {
                (total_rx - last_check.1) as f64 / elapsed.as_secs() as f64
            } else {
                0.0
            };
            
            let tx_speed = if total_tx > last_check.2 {
                (total_tx - last_check.2) as f64 / elapsed.as_secs() as f64
            } else {
                0.0
            };
            
            *last_check = (now, total_rx, total_tx);
            (rx_speed, tx_speed)
        } else {
            (0.0, 0.0)
        }
    }

    pub fn get_network_total(&self) -> (u64, u64) {
        use sysinfo::Networks;
        let networks = Networks::new_with_refreshed_list();
        let mut total_rx = 0u64;
        let mut total_tx = 0u64;
        
        for (_, network) in networks.list() {
            total_rx += network.received();
            total_tx += network.transmitted();
        }
        
        (total_rx, total_tx)
    }
}

impl Default for SystemProvider {
    fn default() -> Self {
        Self::new()
    }
}

