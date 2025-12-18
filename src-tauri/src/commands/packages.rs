use crate::providers::homebrew::HomebrewProvider;
use std::sync::Mutex;
use tauri::State;

#[tauri::command]
pub fn is_homebrew_available() -> bool {
    HomebrewProvider::is_available()
}

#[tauri::command]
pub fn list_installed_packages(provider: State<'_, Mutex<HomebrewProvider>>) -> Result<Vec<crate::providers::homebrew::HomebrewPackage>, String> {
    let provider_guard = provider.lock().map_err(|e| format!("Provider lock error: {}", e))?;
    provider_guard.list_installed()
}

#[tauri::command]
pub fn list_outdated_packages(provider: State<'_, Mutex<HomebrewProvider>>) -> Result<Vec<String>, String> {
    let provider_guard = provider.lock().map_err(|e| format!("Provider lock error: {}", e))?;
    provider_guard.list_outdated()
}

#[tauri::command]
pub fn get_package_dependencies(package: String, provider: State<'_, Mutex<HomebrewProvider>>) -> Result<Vec<String>, String> {
    let provider_guard = provider.lock().map_err(|e| format!("Provider lock error: {}", e))?;
    provider_guard.get_dependencies(&package)
}

#[tauri::command]
pub fn list_services(provider: State<'_, Mutex<HomebrewProvider>>) -> Result<Vec<crate::providers::homebrew::HomebrewService>, String> {
    let provider_guard = provider.lock().map_err(|e| format!("Provider lock error: {}", e))?;
    provider_guard.list_services()
}

#[tauri::command]
pub fn start_service(service: String, provider: State<'_, Mutex<HomebrewProvider>>) -> Result<(), String> {
    let provider_guard = provider.lock().map_err(|e| format!("Provider lock error: {}", e))?;
    provider_guard.start_service(&service)
}

#[tauri::command]
pub fn stop_service(service: String, provider: State<'_, Mutex<HomebrewProvider>>) -> Result<(), String> {
    let provider_guard = provider.lock().map_err(|e| format!("Provider lock error: {}", e))?;
    provider_guard.stop_service(&service)
}

#[tauri::command]
pub fn get_cache_size(provider: State<'_, Mutex<HomebrewProvider>>) -> Result<u64, String> {
    let provider_guard = provider.lock().map_err(|e| format!("Provider lock error: {}", e))?;
    provider_guard.get_cache_size()
}

