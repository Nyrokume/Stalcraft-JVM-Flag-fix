use tauri::command;
use tauri_plugin_store::StoreExt;
use serde::Serialize;
use crate::system;
use crate::jvm;
use crate::ifeo;
use crate::process;

#[derive(Serialize)]
pub struct SystemInfoResponse {
    pub cpu_name: String,
    pub gpu_name: String,
    pub total_ram_gb: f64,
    pub free_ram_gb: f64,
    pub cpu_cores: usize,
    pub large_pages: bool,
    pub large_page_size_mb: u64,
    pub suggested_heap_gb: u64,
}

#[command]
pub fn get_system_info() -> Result<SystemInfoResponse, String> {
    let sys = system::detect_system();
    // Use the same calc_heap function that launch_game uses to avoid divergence
    let heap = jvm::calc_heap(&sys);

    Ok(SystemInfoResponse {
        cpu_name: sys.cpu_name.clone(),
        gpu_name: sys.gpu_name.clone(),
        total_ram_gb: sys.total_ram_gb(),
        free_ram_gb: sys.free_ram_gb(),
        cpu_cores: sys.cpu_cores,
        large_pages: sys.large_pages,
        large_page_size_mb: sys.large_page_size / (1024 * 1024),
        suggested_heap_gb: heap,
    })
}

#[command]
pub fn install_ifeo() -> Result<String, String> {
    ifeo::install()
}

#[command]
pub fn uninstall_ifeo() -> Result<String, String> {
    ifeo::uninstall()
}

#[command]
pub fn check_status() -> Result<String, String> {
    ifeo::status()
}

#[command]
pub fn launch_game(target: String, args: Vec<String>) -> Result<String, String> {
    let sys = system::detect_system();
    let flags = jvm::generate_flags(&sys);
    process::launch_game(&target, &args, &flags)
}

#[command]
pub fn save_game_dir(app: tauri::AppHandle, game_dir: String) -> Result<(), String> {
    let store = app.store("settings.json").map_err(|e| e.to_string())?;
    store.set("game_dir", serde_json::json!(game_dir));
    store.save().map_err(|e| e.to_string())?;
    Ok(())
}

#[command]
pub fn load_game_dir(app: tauri::AppHandle) -> Result<Option<String>, String> {
    let store = app.store("settings.json").map_err(|e| e.to_string())?;
    let dir = store.get("game_dir").and_then(|v| v.as_str().map(|s| s.to_string()));
    Ok(dir)
}
