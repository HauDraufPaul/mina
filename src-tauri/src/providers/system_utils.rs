use serde::{Deserialize, Serialize};
use sysinfo::Disks;
use std::process::Command;

#[derive(Debug, Serialize, Deserialize)]
pub struct DiskInfo {
    pub total: u64,
    pub used: u64,
    pub free: u64,
    pub usage_percent: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SystemInfo {
    pub os: String,
    pub architecture: String,
    pub kernel: String,
    pub uptime: u64,
}

pub struct SystemUtilsProvider;

impl SystemUtilsProvider {
    pub fn new() -> Self {
        SystemUtilsProvider
    }

    pub fn get_disk_info(&self) -> Result<DiskInfo, String> {
        let disks = Disks::new_with_refreshed_list();
        let mut total = 0u64;
        let mut used = 0u64;
        let mut free = 0u64;

        for disk in disks.list() {
            total += disk.total_space();
            free += disk.available_space();
        }

        used = total - free;
        let usage_percent = if total > 0 {
            (used as f64 / total as f64) * 100.0
        } else {
            0.0
        };

        Ok(DiskInfo {
            total,
            used,
            free,
            usage_percent,
        })
    }

    pub fn get_system_info(&self) -> Result<SystemInfo, String> {
        let os = if cfg!(target_os = "macos") {
            "macOS".to_string()
        } else if cfg!(target_os = "windows") {
            "Windows".to_string()
        } else {
            "Linux".to_string()
        };

        let architecture = if cfg!(target_arch = "x86_64") {
            "x86_64".to_string()
        } else if cfg!(target_arch = "aarch64") || cfg!(target_arch = "arm64") {
            "ARM64".to_string()
        } else {
            "Unknown".to_string()
        };

        let kernel = if cfg!(target_os = "macos") {
            "Darwin".to_string()
        } else if cfg!(target_os = "linux") {
            "Linux".to_string()
        } else {
            "Unknown".to_string()
        };

        // Get uptime (simplified - in production, use system APIs)
        // Note: sysinfo doesn't have direct uptime, using a placeholder
        // In production, you'd use platform-specific APIs
        let uptime = 86400; // Placeholder: 24 hours

        Ok(SystemInfo {
            os,
            architecture,
            kernel,
            uptime,
        })
    }

    pub fn prevent_sleep(&self) -> Result<(), String> {
        if cfg!(target_os = "macos") {
            Command::new("caffeinate")
                .arg("-d")
                .spawn()
                .map_err(|e| format!("Failed to prevent sleep: {}", e))?;
            Ok(())
        } else {
            Err("Sleep prevention not implemented for this platform".to_string())
        }
    }
}

impl Default for SystemUtilsProvider {
    fn default() -> Self {
        Self::new()
    }
}

