use crate::providers::SystemProvider;
use crate::{SystemMetrics, CpuMetrics, MemoryMetrics, DiskMetrics, NetworkMetrics};
use std::sync::Mutex;
use tauri::State;

#[tauri::command]
pub fn get_system_metrics(provider: State<'_, Mutex<SystemProvider>>) -> Result<SystemMetrics, String> {
    let mut provider_guard = provider.lock().map_err(|e| format!("Provider lock error: {}", e))?;
    provider_guard.refresh();
    
    // CPU Metrics
    let cpu_usage = provider_guard.get_cpu_usage();
    let cpu_count = provider_guard.get_cpu_count();
    let cpu_frequency = provider_guard.get_cpu_frequency();

    // Memory Metrics
    let total_memory = provider_guard.get_memory_total();
    let used_memory = provider_guard.get_memory_used();
    let free_memory = provider_guard.get_memory_free();
    let memory_usage = if total_memory > 0 {
        (used_memory as f64 / total_memory as f64) * 100.0
    } else {
        0.0
    };

    // Disk Metrics
    let (disk_total, disk_used, disk_free) = provider_guard.get_disk_metrics();
    let disk_usage = if disk_total > 0 {
        (disk_used as f64 / disk_total as f64) * 100.0
    } else {
        0.0
    };

    // Network Metrics
    let (total_rx, total_tx) = provider_guard.get_network_total();
    let (rx_speed, tx_speed) = provider_guard.get_network_speeds();

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

