use crate::providers::homebrew::HomebrewProvider;
use tokio::sync::Mutex;
use tauri::State;

#[tauri::command]
pub fn is_homebrew_available() -> bool {
    HomebrewProvider::is_available()
}

#[tauri::command]
pub async fn list_installed_packages(provider: State<'_, Mutex<HomebrewProvider>>) -> Result<Vec<crate::providers::homebrew::HomebrewPackage>, String> {
    let provider_guard = provider.lock().await;
    provider_guard.list_installed().await
}

#[tauri::command]
pub async fn list_outdated_packages(provider: State<'_, Mutex<HomebrewProvider>>) -> Result<Vec<String>, String> {
    let provider_guard = provider.lock().await;
    provider_guard.list_outdated().await
}

#[tauri::command]
pub async fn get_package_dependencies(package: String, provider: State<'_, Mutex<HomebrewProvider>>) -> Result<Vec<String>, String> {
    let provider_guard = provider.lock().await;
    provider_guard.get_dependencies(&package).await
}

#[tauri::command]
pub async fn list_services(provider: State<'_, Mutex<HomebrewProvider>>) -> Result<Vec<crate::providers::homebrew::HomebrewService>, String> {
    let provider_guard = provider.lock().await;
    provider_guard.list_services().await
}

#[tauri::command]
pub async fn start_service(service: String, provider: State<'_, Mutex<HomebrewProvider>>) -> Result<(), String> {
    let provider_guard = provider.lock().await;
    provider_guard.start_service(&service).await
}

#[tauri::command]
pub async fn stop_service(service: String, provider: State<'_, Mutex<HomebrewProvider>>) -> Result<(), String> {
    let provider_guard = provider.lock().await;
    provider_guard.stop_service(&service).await
}

#[tauri::command]
pub async fn get_cache_size(provider: State<'_, Mutex<HomebrewProvider>>) -> Result<u64, String> {
    let provider_guard = provider.lock().await;
    provider_guard.get_cache_size().await
}

