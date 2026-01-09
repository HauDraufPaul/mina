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
        
        // On macOS, find the main boot disk (usually mounted at "/")
        // On other systems, use the root filesystem
        let main_disk = if cfg!(target_os = "macos") {
            disks.list()
                .iter()
                .find(|disk| {
                    let mount_point = disk.mount_point().to_string_lossy();
                    mount_point == "/" || mount_point == "/System/Volumes/Data"
                })
                .or_else(|| disks.list().first())
        } else {
            // Linux/Windows: find root filesystem
            disks.list()
                .iter()
                .find(|disk| {
                    let mount_point = disk.mount_point().to_string_lossy();
                    mount_point == "/" || mount_point == "C:\\"
                })
                .or_else(|| disks.list().first())
        };

        if let Some(disk) = main_disk {
            let total = disk.total_space();
            let free = disk.available_space();
            let used = total.saturating_sub(free);
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
        } else {
            // Fallback: sum all disks if we can't find the main one
            let mut total = 0u64;
            let mut free = 0u64;

            for disk in disks.list() {
                total += disk.total_space();
                free += disk.available_space();
            }

            let used = total.saturating_sub(free);
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
        } else if cfg!(target_arch = "aarch64") {
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

        // Get real system uptime
        let uptime = if cfg!(target_os = "macos") {
            // Use sysctl to get boot time on macOS
            match Command::new("sysctl")
                .arg("-n")
                .arg("kern.boottime")
                .output()
            {
                Ok(output) => {
                    let output_str = String::from_utf8_lossy(&output.stdout);
                    // Parse: { sec = 1234567890, usec = 0 }
                    if let Some(sec_start) = output_str.find("sec = ") {
                        let sec_str = &output_str[sec_start + 6..];
                        if let Some(sec_end) = sec_str.find(',') {
                            if let Ok(boot_time) = sec_str[..sec_end].trim().parse::<i64>() {
                                let now = std::time::SystemTime::now()
                                    .duration_since(std::time::UNIX_EPOCH)
                                    .map_err(|e| {
                                        eprintln!("Failed to get system time: {}", e);
                                        return 0;
                                    })
                                    .unwrap_or_else(|_| {
                                        eprintln!("System time error");
                                        return std::time::Duration::from_secs(0);
                                    })
                                    .as_secs() as i64;
                                (now - boot_time).max(0) as u64
                            } else {
                                0
                            }
                        } else {
                            0
                        }
                    } else {
                        0
                    }
                }
                Err(_) => 0,
            }
        } else if cfg!(target_os = "linux") {
            // Read /proc/uptime on Linux
            match std::fs::read_to_string("/proc/uptime") {
                Ok(content) => {
                    if let Some(first) = content.split_whitespace().next() {
                        first.parse::<f64>().map(|s| s as u64).unwrap_or(0)
                    } else {
                        0
                    }
                }
                Err(_) => 0,
            }
        } else {
            // Windows or other - try to get uptime via system command
            // For now, return 0 if we can't determine it
            0
        };

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

