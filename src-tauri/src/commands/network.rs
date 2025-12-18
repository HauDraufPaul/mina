use crate::providers::NetworkProvider;
use std::sync::Mutex;
use tauri::State;

#[tauri::command]
pub fn get_network_interfaces(provider: State<'_, Mutex<NetworkProvider>>) -> Result<Vec<crate::providers::network::NetworkInterface>, String> {
    let mut provider_guard = provider.lock().map_err(|e| format!("Provider lock error: {}", e))?;
    provider_guard.refresh();
    Ok(provider_guard.get_interfaces())
}

#[tauri::command]
pub fn get_network_connections(provider: State<'_, Mutex<NetworkProvider>>) -> Result<Vec<crate::providers::network::NetworkConnection>, String> {
    let provider_guard = provider.lock().map_err(|e| format!("Provider lock error: {}", e))?;
    Ok(provider_guard.get_connections())
}

