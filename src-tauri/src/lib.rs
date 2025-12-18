use serde::{Deserialize, Serialize};
use sysinfo::{System, Cpu, Disks, Networks};

#[derive(Debug, Serialize, Deserialize)]
pub struct SystemMetrics {
    pub cpu: CpuMetrics,
    pub memory: MemoryMetrics,
    pub disk: DiskMetrics,
    pub network: NetworkMetrics,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CpuMetrics {
    pub usage: f64,
    pub cores: usize,
    pub frequency: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MemoryMetrics {
    pub total: u64,
    pub used: u64,
    pub free: u64,
    pub usage: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DiskMetrics {
    pub total: u64,
    pub used: u64,
    pub free: u64,
    pub usage: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NetworkMetrics {
    pub rx: u64,
    pub tx: u64,
    #[serde(rename = "rxSpeed")]
    pub rx_speed: f64,
    #[serde(rename = "txSpeed")]
    pub tx_speed: f64,
}

#[tauri::command]
fn get_system_metrics() -> Result<SystemMetrics, String> {
    let mut system = System::new_all();
    system.refresh_all();

    // CPU Metrics
    let cpu_usage = system.global_cpu_info().cpu_usage() as f64;
    let cpu_count = system.cpus().len();
    let cpu_frequency = system.global_cpu_info().frequency();

    // Memory Metrics
    let total_memory = system.total_memory();
    let used_memory = system.used_memory();
    let free_memory = system.free_memory();
    let memory_usage = if total_memory > 0 {
        (used_memory as f64 / total_memory as f64) * 100.0
    } else {
        0.0
    };

    // Disk Metrics (using first disk)
    let mut disk_total = 0u64;
    let mut disk_used = 0u64;
    let mut disk_free = 0u64;
    
    for disk in system.disks() {
        disk_total = disk.total_space();
        disk_free = disk.available_space();
        disk_used = disk_total - disk_free;
        break; // Use first disk
    }
    
    let disk_usage = if disk_total > 0 {
        (disk_used as f64 / disk_total as f64) * 100.0
    } else {
        0.0
    };

    // Network Metrics
    let mut total_rx = 0u64;
    let mut total_tx = 0u64;
    
    for (_, network) in system.networks() {
        total_rx += network.received();
        total_tx += network.transmitted();
    }

    // Calculate speeds (simplified - in real implementation, track previous values)
    let rx_speed = 0.0; // Would need to track previous values
    let tx_speed = 0.0; // Would need to track previous values

    Ok(SystemMetrics {
        cpu: CpuMetrics {
            usage: cpu_usage,
            cores: cpu_count,
            frequency: cpu_frequency,
        },
        memory: MemoryMetrics {
            total: total_memory,
            used: used_memory,
            free: free_memory,
            usage: memory_usage,
        },
        disk: DiskMetrics {
            total: disk_total,
            used: disk_used,
            free: disk_free,
            usage: disk_usage,
        },
        network: NetworkMetrics {
            rx: total_rx,
            tx: total_tx,
            rx_speed,
            tx_speed,
        },
    })
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![get_system_metrics])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

