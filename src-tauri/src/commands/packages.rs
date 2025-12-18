use crate::providers::homebrew::HomebrewProvider;
use std::sync::Mutex;
use tauri::AppHandle;

#[tauri::command]
pub fn is_homebrew_available() -> bool {
    HomebrewProvider::is_available()
}

#[tauri::command]
pub fn list_installed_packages(app: AppHandle) -> Result<Vec<crate::providers::homebrew::HomebrewPackage>, String> {
    let provider = app.try_state::<Mutex<HomebrewProvider>>()
        .ok_or("HomebrewProvider not found")?;
    let provider = provider.lock().map_err(|e| format!("Provider lock error: {}", e))?;
    provider.list_installed()
}

#[tauri::command]
pub fn list_outdated_packages(app: AppHandle) -> Result<Vec<String>, String> {
    let provider = app.try_state::<Mutex<HomebrewProvider>>()
        .ok_or("HomebrewProvider not found")?;
    let provider = provider.lock().map_err(|e| format!("Provider lock error: {}", e))?;
    provider.list_outdated()
}

#[tauri::command]
pub fn get_package_dependencies(package: String, app: AppHandle) -> Result<Vec<String>, String> {
    let provider = app.try_state::<Mutex<HomebrewProvider>>()
        .ok_or("HomebrewProvider not found")?;
    let provider = provider.lock().map_err(|e| format!("Provider lock error: {}", e))?;
    provider.get_dependencies(&package)
}

#[tauri::command]
pub fn list_services(app: AppHandle) -> Result<Vec<crate::providers::homebrew::HomebrewService>, String> {
    let provider = app.try_state::<Mutex<HomebrewProvider>>()
        .ok_or("HomebrewProvider not found")?;
    let provider = provider.lock().map_err(|e| format!("Provider lock error: {}", e))?;
    provider.list_services()
}

#[tauri::command]
pub fn start_service(service: String, app: AppHandle) -> Result<(), String> {
    let provider = app.try_state::<Mutex<HomebrewProvider>>()
        .ok_or("HomebrewProvider not found")?;
    let provider = provider.lock().map_err(|e| format!("Provider lock error: {}", e))?;
    provider.start_service(&service)
}

#[tauri::command]
pub fn stop_service(service: String, app: AppHandle) -> Result<(), String> {
    let provider = app.try_state::<Mutex<HomebrewProvider>>()
        .ok_or("HomebrewProvider not found")?;
    let provider = provider.lock().map_err(|e| format!("Provider lock error: {}", e))?;
    provider.stop_service(&service)
}

#[tauri::command]
pub fn get_cache_size(app: AppHandle) -> Result<u64, String> {
    let provider = app.try_state::<Mutex<HomebrewProvider>>()
        .ok_or("HomebrewProvider not found")?;
    let provider = provider.lock().map_err(|e| format!("Provider lock error: {}", e))?;
    provider.get_cache_size()
}

