// commands.rs — Tauri IPC

use serde::Serialize;
use tauri::command;

use crate::{config, ifeo, log, system};

#[derive(Serialize)]
pub struct SystemInfoResponse {
    pub cpu_name: String,
    pub gpu_name: String,
    pub total_ram_gb: f64,
    pub free_ram_gb: f64,
    pub cpu_cores: usize,
    pub cpu_threads: usize,
    pub l3_cache_mb: usize,
    pub mem_speed_mts: usize,
    pub mem_tier: String,
    pub has_big_cache: bool,
    pub large_pages: bool,
    pub large_page_size_mb: u64,
    pub suggested_heap_gb: u64,
    pub active_config: Option<String>,
    pub active_config_exists: bool,
}

#[command]
pub fn get_system_info() -> Result<SystemInfoResponse, String> {
    let sys = system::detect_system();
    let _ = config::ensure(&sys);
    let heap = config::load_active()
        .map(|(cfg, _)| cfg.heap_size_gb)
        .unwrap_or_else(|_| config::generate(&sys).heap_size_gb);

    Ok(SystemInfoResponse {
        cpu_name: sys.cpu_name.clone(),
        gpu_name: sys.gpu_name.clone(),
        total_ram_gb: sys.total_ram_gb(),
        free_ram_gb: sys.free_ram_gb(),
        cpu_cores: sys.cpu_cores,
        cpu_threads: sys.cpu_threads,
        l3_cache_mb: sys.l3_cache_mb,
        mem_speed_mts: sys.mem_speed_mts,
        mem_tier: sys.mem_tier().as_str().to_string(),
        has_big_cache: sys.has_big_cache(),
        large_pages: sys.large_pages,
        large_page_size_mb: sys.large_page_size >> 20,
        suggested_heap_gb: heap,
        active_config: config::active_name(),
        active_config_exists: config::active_exists(),
    })
}

#[command]
pub fn install_ifeo() -> Result<String, String> {
    ifeo::install(None)
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
pub fn read_wrapper_log_tail(max_lines: u32) -> Result<String, String> {
    let n = max_lines.max(1).min(500) as usize;
    log::read_wrapper_log_tail(n)
}

#[derive(Serialize)]
pub struct ConfigListResponse {
    pub names: Vec<String>,
    pub active: Option<String>,
    pub active_exists: bool,
}

#[command]
pub fn list_configs() -> Result<ConfigListResponse, String> {
    let names = config::list()?;
    Ok(ConfigListResponse {
        names,
        active: config::active_name(),
        active_exists: config::active_exists(),
    })
}

#[command]
pub fn select_config(name: String) -> Result<String, String> {
    config::set_active(&name)?;
    Ok(format!("Active config set to: {}", name))
}

#[command]
pub fn regenerate_config() -> Result<String, String> {
    let sys = system::detect_system();
    let cfg = config::generate(&sys);
    let desc = sys.describe();
    config::save(&cfg, "default")?;
    config::set_active("default")?;
    Ok(format!(
        "Regenerated default config.\nSystem: {}\nHeap: {}GB, GC threads: {}/{}",
        desc, cfg.heap_size_gb, cfg.parallel_gc_threads, cfg.conc_gc_threads
    ))
}

#[derive(Serialize)]
pub struct ConfigResponse {
    pub name: String,
    pub config: config::Config,
}

#[command]
pub fn get_active_config() -> Result<ConfigResponse, String> {
    let (cfg, name) = config::load_active()?;
    Ok(ConfigResponse { name, config: cfg })
}

#[command]
pub fn load_config_by_name(name: String) -> Result<ConfigResponse, String> {
    let cfg = config::load(&name)?;
    Ok(ConfigResponse { name, config: cfg })
}

#[command]
pub fn save_config(name: String, cfg: config::Config) -> Result<String, String> {
    config::save(&cfg, &name)?;
    Ok(format!("Saved config: {}", name))
}
