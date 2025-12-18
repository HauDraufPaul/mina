use crate::providers::system_utils::SystemUtilsProvider;
use std::sync::Mutex;
use tauri::State;

#[tauri::command]
pub fn get_disk_info(
    provider: State<'_, Mutex<SystemUtilsProvider>>,
) -> Result<crate::providers::system_utils::DiskInfo, String> {
    let provider_guard = provider.lock().map_err(|e| format!("Provider lock error: {}", e))?;
    provider_guard.get_disk_info()
        .map_err(|e| format!("Failed to get disk info: {}", e))
}

#[tauri::command]
pub fn get_system_info(
    provider: State<'_, Mutex<SystemUtilsProvider>>,
) -> Result<crate::providers::system_utils::SystemInfo, String> {
    let provider_guard = provider.lock().map_err(|e| format!("Provider lock error: {}", e))?;
    provider_guard.get_system_info()
        .map_err(|e| format!("Failed to get system info: {}", e))
}

#[tauri::command]
pub fn prevent_sleep(
    provider: State<'_, Mutex<SystemUtilsProvider>>,
) -> Result<(), String> {
    let provider_guard = provider.lock().map_err(|e| format!("Provider lock error: {}", e))?;
    provider_guard.prevent_sleep()
        .map_err(|e| format!("Failed to prevent sleep: {}", e))
}

