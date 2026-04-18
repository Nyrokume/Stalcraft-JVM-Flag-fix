// commands.rs — Tauri IPC команды
// Полный набор команд включая управление конфигами (config.go логика).

use serde::Serialize;
use tauri::command;
use tauri_plugin_store::StoreExt;

use crate::{config, ifeo, jvm, process, system};

// ─── SystemInfo response ──────────────────────────────────────────────────────

#[derive(Serialize)]
pub struct SystemInfoResponse {
    pub cpu_name: String,
    pub gpu_name: String,
    pub total_ram_gb: f64,
    pub free_ram_gb: f64,
    pub cpu_cores: usize,
    pub cpu_threads: usize,
    pub l3_cache_mb: usize,
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
    // ensure() создаёт default.json если отсутствует
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
        has_big_cache: sys.has_big_cache(),
        large_pages: sys.large_pages,
        large_page_size_mb: sys.large_page_size / (1024 * 1024),
        suggested_heap_gb: heap,
        active_config: config::active_name(),
        active_config_exists: config::active_exists(),
    })
}

// ─── IFEO commands ────────────────────────────────────────────────────────────

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

// ─── Game launch ──────────────────────────────────────────────────────────────

/// launch_game — загружает активный конфиг и запускает игру
#[command]
pub fn launch_game(target: String, args: Vec<String>) -> Result<String, String> {
    if target.is_empty() {
        return Err("Empty target path".to_string());
    }

    let target_path = std::path::Path::new(&target);
    if !target_path.exists() {
        return Err(format!("Target does not exist: {}", target));
    }
    if target_path.is_dir() {
        return Err(format!(
            "Target is a directory, not an executable: {}",
            target
        ));
    }

    let sys = system::detect_system();
    let _ = config::ensure(&sys);

    // Загружаем активный конфиг (аналог loadActive в main.go)
    let (cfg, loaded_name) = match config::load_active() {
        Ok(pair) => pair,
        Err(e) => {
            eprintln!(
                "[launch] config load failed: {} — using generated defaults",
                e
            );
            let generated = config::generate(&sys);
            (generated, "generated".to_string())
        }
    };

    eprintln!(
        "[launch] config: {}, heap: {}GB, GC threads: {}/{}",
        loaded_name, cfg.heap_size_gb, cfg.parallel_gc_threads, cfg.conc_gc_threads
    );

    let flags = if cfg.heap_size_gb == 0 {
        eprintln!("[launch] heap=0, keeping original args");
        args.clone()
    } else {
        let injected = jvm::flags(&cfg);
        eprintln!("[launch] injecting {} JVM flags", injected.len());
        jvm::filter_args(&args, &injected)
    };

    eprintln!("[launch] target={}, arg_count={}", target, flags.len());

    // Создаём phantom window (аналог phantom.Start())
    process::start_phantom_window();

    let (h_process, h_thread, pid) = process::nt_create_process(&target, &flags)
        .map_err(|e| format!("Failed to start game: {}", e))?;

    eprintln!("[launch] process started, pid={}", pid);

    process::boost_process(h_process);

    let exit_code = process::wait_process(h_process, pid);
    process::cleanup_handles(h_process, h_thread);

    if exit_code == 0 {
        eprintln!("[launch] pid={} exited/detached, code=0", pid);
        Ok(format!(
            "Game launched (PID {}, config: {})",
            pid, loaded_name
        ))
    } else {
        Err(format!(
            "Game process {} exited with code {} before window appeared",
            pid, exit_code
        ))
    }
}

// ─── Config management ────────────────────────────────────────────────────────

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
        "Regenerated default config.\nSystem: {}\nHeap: {}GB, GC: {}/{} threads",
        desc, cfg.heap_size_gb, cfg.parallel_gc_threads, cfg.conc_gc_threads
    ))
}

#[command]
pub fn apply_config_preset(preset: String) -> Result<String, String> {
    let sys = system::detect_system();
    let (cfg, stem) = config::apply_named_preset(&sys, &preset)?;
    config::save(&cfg, &stem)?;
    config::set_active(&stem)?;
    let desc = sys.describe();
    Ok(format!(
        "Preset applied: {} (active)\nSystem: {}\nHeap: {}GB, GC: {}/{} threads",
        stem,
        desc,
        cfg.heap_size_gb,
        cfg.parallel_gc_threads,
        cfg.conc_gc_threads
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

// ─── Settings persistence (game dir) ─────────────────────────────────────────

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
    let dir = store
        .get("game_dir")
        .and_then(|v| v.as_str().map(|s| s.to_string()));
    Ok(dir)
}
