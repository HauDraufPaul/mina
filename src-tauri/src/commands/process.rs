use crate::providers::ProcessProvider;
use std::sync::Mutex;
use tauri::State;

#[tauri::command]
pub fn get_processes(mut provider: State<'_, Mutex<ProcessProvider>>) -> Result<Vec<crate::providers::process::ProcessInfo>, String> {
    let mut provider_guard = provider.lock().map_err(|e| format!("Provider lock error: {}", e))?;
    provider_guard.refresh();
    Ok(provider_guard.get_processes())
}

#[tauri::command]
pub fn get_process(pid: u32, provider: State<'_, Mutex<ProcessProvider>>) -> Result<Option<crate::providers::process::ProcessInfo>, String> {
    let provider_guard = provider.lock().map_err(|e| format!("Provider lock error: {}", e))?;
    Ok(provider_guard.get_process(pid))
}

#[tauri::command]
pub fn kill_process(pid: u32, mut provider: State<'_, Mutex<ProcessProvider>>) -> Result<(), String> {
    let mut provider_guard = provider.lock().map_err(|e| format!("Provider lock error: {}", e))?;
    provider_guard.kill_process(pid)
}

