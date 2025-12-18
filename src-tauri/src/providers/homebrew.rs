use serde::{Deserialize, Serialize};
use std::process::Command;
use std::str;

#[derive(Debug, Serialize, Deserialize)]
pub struct HomebrewPackage {
    pub name: String,
    pub version: String,
    pub installed: bool,
    pub outdated: bool,
    pub dependencies: Vec<String>,
    pub description: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HomebrewService {
    pub name: String,
    pub status: String,
    pub running: bool,
}

pub struct HomebrewProvider;

impl HomebrewProvider {
    pub fn new() -> Self {
        HomebrewProvider
    }

    pub fn is_available() -> bool {
        Command::new("brew")
            .arg("--version")
            .output()
            .is_ok()
    }

    pub fn list_installed(&self) -> Result<Vec<HomebrewPackage>, String> {
        if !Self::is_available() {
            return Err("Homebrew is not installed".to_string());
        }

        let output = Command::new("brew")
            .args(&["list", "--versions"])
            .output()
            .map_err(|e| format!("Failed to run brew list: {}", e))?;

        if !output.status.success() {
            return Err("brew list command failed".to_string());
        }

        let stdout = str::from_utf8(&output.stdout)
            .map_err(|e| format!("Invalid UTF-8: {}", e))?;

        let mut packages = Vec::new();
        for line in stdout.lines() {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 {
                packages.push(HomebrewPackage {
                    name: parts[0].to_string(),
                    version: parts[1].to_string(),
                    installed: true,
                    outdated: false,
                    dependencies: Vec::new(),
                    description: None,
                });
            }
        }

        Ok(packages)
    }

    pub fn list_outdated(&self) -> Result<Vec<String>, String> {
        if !Self::is_available() {
            return Err("Homebrew is not installed".to_string());
        }

        let output = Command::new("brew")
            .args(&["outdated"])
            .output()
            .map_err(|e| format!("Failed to run brew outdated: {}", e))?;

        if !output.status.success() {
            return Err("brew outdated command failed".to_string());
        }

        let stdout = str::from_utf8(&output.stdout)
            .map_err(|e| format!("Invalid UTF-8: {}", e))?;

        Ok(stdout.lines().map(|s| s.to_string()).collect())
    }

    pub fn get_dependencies(&self, package: &str) -> Result<Vec<String>, String> {
        if !Self::is_available() {
            return Err("Homebrew is not installed".to_string());
        }

        let output = Command::new("brew")
            .args(&["deps", "--installed", package])
            .output()
            .map_err(|e| format!("Failed to run brew deps: {}", e))?;

        if !output.status.success() {
            return Err("brew deps command failed".to_string());
        }

        let stdout = str::from_utf8(&output.stdout)
            .map_err(|e| format!("Invalid UTF-8: {}", e))?;

        Ok(stdout.lines().map(|s| s.to_string()).collect())
    }

    pub fn list_services(&self) -> Result<Vec<HomebrewService>, String> {
        if !Self::is_available() {
            return Err("Homebrew is not installed".to_string());
        }

        let output = Command::new("brew")
            .args(&["services", "list"])
            .output()
            .map_err(|e| format!("Failed to run brew services: {}", e))?;

        if !output.status.success() {
            return Err("brew services command failed".to_string());
        }

        let stdout = str::from_utf8(&output.stdout)
            .map_err(|e| format!("Invalid UTF-8: {}", e))?;

        let mut services = Vec::new();
        for line in stdout.lines().skip(1) {
            // Skip header line
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 {
                let status = parts[1].to_string();
                services.push(HomebrewService {
                    name: parts[0].to_string(),
                    status: status.clone(),
                    running: status == "started",
                });
            }
        }

        Ok(services)
    }

    pub fn start_service(&self, service: &str) -> Result<(), String> {
        if !Self::is_available() {
            return Err("Homebrew is not installed".to_string());
        }

        let output = Command::new("brew")
            .args(&["services", "start", service])
            .output()
            .map_err(|e| format!("Failed to start service: {}", e))?;

        if !output.status.success() {
            return Err(format!("Failed to start service: {}", service));
        }

        Ok(())
    }

    pub fn stop_service(&self, service: &str) -> Result<(), String> {
        if !Self::is_available() {
            return Err("Homebrew is not installed".to_string());
        }

        let output = Command::new("brew")
            .args(&["services", "stop", service])
            .output()
            .map_err(|e| format!("Failed to stop service: {}", e))?;

        if !output.status.success() {
            return Err(format!("Failed to stop service: {}", service));
        }

        Ok(())
    }

    pub fn get_cache_size(&self) -> Result<u64, String> {
        if !Self::is_available() {
            return Err("Homebrew is not installed".to_string());
        }

        let output = Command::new("brew")
            .args(&["--cache"])
            .output()
            .map_err(|e| format!("Failed to get cache path: {}", e))?;

        if !output.status.success() {
            return Err("Failed to get cache path".to_string());
        }

        // This is a simplified version - in production, you'd calculate actual cache size
        Ok(0)
    }
}

impl Default for HomebrewProvider {
    fn default() -> Self {
        Self::new()
    }
}

