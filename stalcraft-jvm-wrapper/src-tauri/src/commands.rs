// commands.rs — Tauri IPC

use serde::Serialize;
use tauri::command;

use crate::{config, elevate, ifeo, log, system};

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
    if !ifeo::service_ready() {
        return Err(
            "service.exe not found next to this app — copy both files from release/".to_string(),
        );
    }
    if ifeo::is_admin() {
        return ifeo::install(None);
    }
    let code = elevate::run_as_admin("--install")?;
    if code != 0 {
        return Err(format!("Install failed (exit {})", code));
    }
    ifeo::status()
}

#[command]
pub fn uninstall_ifeo() -> Result<String, String> {
    if ifeo::is_admin() {
        return ifeo::uninstall();
    }
    let code = elevate::run_as_admin("--uninstall")?;
    if code != 0 {
        return Err(format!("Uninstall failed (exit {})", code));
    }
    Ok("IFEO uninstalled.".to_string())
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

#[derive(Serialize)]
pub struct ExampleListResponse {
    pub names: Vec<String>,
}

#[command]
pub fn list_examples() -> Result<ExampleListResponse, String> {
    Ok(ExampleListResponse {
        names: config::list_examples()?,
    })
}

#[command]
pub fn import_example_config(name: String) -> Result<String, String> {
    config::import_example(&name)?;
    Ok(format!("Imported preset: {name}.json"))
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
pub async fn ping_servers(
    targets: Vec<crate::ping::PingTarget>,
    timeout_ms: Option<u64>,
) -> Result<Vec<crate::ping::PingResult>, String> {
    let timeout_ms = timeout_ms.unwrap_or(crate::ping::DEFAULT_TIMEOUT_MS);
    tauri::async_runtime::spawn_blocking(move || crate::ping::ping_targets(&targets, timeout_ms))
        .await
        .map_err(|e| format!("ping task join failed: {e}"))
}

fn write_sb_ips_temp(ips: &[String]) -> Result<std::path::PathBuf, String> {
    let path = std::env::temp_dir().join(format!("stalzone-sb-{}.json", std::process::id()));
    let json = serde_json::to_string(ips).map_err(|e| e.to_string())?;
    std::fs::write(&path, json).map_err(|e| e.to_string())?;
    Ok(path)
}

#[command]
pub fn start_server_blocking(ips: Vec<String>) -> Result<String, String> {
    if ifeo::is_admin() {
        return crate::server_block::apply_blocks(&ips);
    }
    let path = write_sb_ips_temp(&ips)?;
    let arg = format!("--sb-apply \"{}\"", path.display());
    let code = elevate::run_as_admin(&arg)?;
    let _ = std::fs::remove_file(&path);
    if code != 0 {
        return Err(format!("Blocking failed (exit {})", code));
    }
    Ok("Firewall rules applied.".to_string())
}

#[command]
pub fn stop_server_blocking() -> Result<String, String> {
    if ifeo::is_admin() {
        let removed = crate::server_block::clear_rules()?;
        return Ok(format!("Removed {} firewall rule(s)", removed));
    }
    let code = elevate::run_as_admin("--sb-clear")?;
    if code != 0 {
        return Err(format!("Unblock failed (exit {})", code));
    }
    Ok("Firewall rules removed.".to_string())
}

#[derive(Serialize)]
pub struct ServerBlockStatus {
    pub active: bool,
    pub rule_count: u32,
}

#[command]
pub fn server_blocking_active() -> Result<ServerBlockStatus, String> {
    let status = crate::server_block::blocking_status()?;
    Ok(ServerBlockStatus {
        active: status.active,
        rule_count: status.rule_count,
    })
}
