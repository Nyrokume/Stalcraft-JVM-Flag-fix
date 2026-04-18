// config.rs — порт config.go + generate.go
// Модель профиля JVM-тюнинга: персистентность на диске (configs/*.json),
// указатель "active" в HKCU реестре, автогенерация по железу.

use serde::{Deserialize, Serialize};
use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;
use std::path::PathBuf;

use crate::system::SystemInfo;

// ─── Windows Registry API ─────────────────────────────────────────────────────

#[link(name = "advapi32")]
extern "system" {
    fn RegCreateKeyExW(
        hKey: isize,
        lpSubKey: *const u16,
        Reserved: u32,
        lpClass: *const u16,
        dwOptions: u32,
        samDesired: u32,
        lpSecurityAttributes: *const std::ffi::c_void,
        phkResult: *mut isize,
        lpdwDisposition: *mut u32,
    ) -> i32;
    fn RegOpenKeyExW(
        hKey: isize,
        lpSubKey: *const u16,
        ulOptions: u32,
        samDesired: u32,
        phkResult: *mut isize,
    ) -> i32;
    fn RegSetValueExW(
        hKey: isize,
        lpValueName: *const u16,
        Reserved: u32,
        dwType: u32,
        lpData: *const u8,
        cbData: u32,
    ) -> i32;
    fn RegQueryValueExW(
        hKey: isize,
        lpValueName: *const u16,
        lpReserved: *mut u32,
        lpType: *mut u32,
        lpData: *mut u8,
        lpcbData: *mut u32,
    ) -> i32;
    fn RegCloseKey(hKey: isize) -> i32;
}

const HKEY_CURRENT_USER: isize = -2147483647i64 as isize; // 0x80000001
const KEY_SET_VALUE: u32 = 0x0002;
const KEY_QUERY_VALUE: u32 = 0x0001;
const REG_SZ: u32 = 1;
const REGISTRY_PATH: &str = r"Software\StalcraftWrapper";

// ─── Config структура (точный порт config.Config из Go) ──────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub heap_size_gb: u64,
    pub pre_touch: bool,
    pub metaspace_mb: u64,

    pub max_gc_pause_millis: u64,
    pub g1_heap_region_size_mb: u64,
    pub g1_new_size_percent: u64,
    pub g1_max_new_size_percent: u64,
    pub g1_reserve_percent: u64,
    pub g1_heap_waste_percent: u64,
    pub g1_mixed_gc_count_target: u64,
    pub initiating_heap_occupancy_percent: u64,
    pub g1_mixed_gc_live_threshold_percent: u64,
    pub g1_rset_updating_pause_time_percent: u64,
    pub survivor_ratio: u64,
    pub max_tenuring_threshold: u64,

    pub g1_satb_buffer_enqueuing_threshold_percent: u64,
    pub g1_conc_rs_hot_card_limit: u64,
    pub g1_conc_refinement_service_interval_millis: u64,
    pub gc_time_ratio: u64,
    pub use_dynamic_number_of_gc_threads: bool,
    pub use_string_deduplication: bool,

    pub parallel_gc_threads: u64,
    pub conc_gc_threads: u64,
    pub soft_ref_lru_policy_ms_per_mb: u64,

    pub reserved_code_cache_size_mb: u64,
    pub max_inline_level: u64,
    pub freq_inline_size: u64,
    pub inline_small_code: u64,
    pub max_node_limit: u64,
    pub node_limit_fudge_factor: u64,
    pub nmethod_sweep_activity: u64,
    pub dont_compile_huge_methods: bool,
    pub allocate_prefetch_style: u64,
    pub always_act_as_server_class: bool,
    pub use_xmm_for_array_copy: bool,
    pub use_fpu_for_spilling: bool,

    pub use_large_pages: bool,
    pub large_page_size_mb: u64,

    // Java 9 специфика
    pub reflection_inflation_threshold: i64,
    pub auto_box_cache_max: u64,
    pub use_thread_priorities: bool,
    pub thread_priority_policy: u64,
    pub use_counter_decay: bool,
    pub compile_threshold_scaling: f64,
}

// ─── Генерация конфига по железу (generate.go) ───────────────────────────────

/// generate() — точный порт Generate() из generate.go
pub fn generate(sys: &SystemInfo) -> Config {
    let heap = size_heap(sys.total_gb());
    let (parallel, mut concurrent) = gc_threads(sys.cpu_threads);
    let jit = jit_profile(sys);

    // Throughput-first defaults для обычных CPU
    let mut ihop: u64 = 20;
    let mut pause_ms: u64 = 50;
    let mut new_size_percent: u64 = 23;
    let mut mixed_count_target: u64 = 3;
    let mut soft_ref_ms: u64 = 25;

    if sys.has_big_cache() {
        // X3D-класс — тугой pause budget, более ранний IHOP
        ihop = 15;
        pause_ms = 25;
        new_size_percent = 30;
        mixed_count_target = 4;
        soft_ref_ms = 50;
        // Дополнительный concurrent worker только при 16+ потоках
        if sys.cpu_threads >= 16 {
            concurrent += 1;
        }
    }

    Config {
        heap_size_gb: heap,
        pre_touch: sys.total_gb() >= 12,
        metaspace_mb: 512,

        max_gc_pause_millis: pause_ms,
        g1_heap_region_size_mb: region_size(heap),
        g1_new_size_percent: new_size_percent,
        g1_max_new_size_percent: 50,
        g1_reserve_percent: 20,
        g1_heap_waste_percent: 5,
        g1_mixed_gc_count_target: mixed_count_target,
        initiating_heap_occupancy_percent: ihop,
        g1_mixed_gc_live_threshold_percent: 90,
        g1_rset_updating_pause_time_percent: 0,
        survivor_ratio: 32,
        max_tenuring_threshold: 1,

        g1_satb_buffer_enqueuing_threshold_percent: 30,
        g1_conc_rs_hot_card_limit: 16,
        g1_conc_refinement_service_interval_millis: 150,
        gc_time_ratio: 99,
        use_dynamic_number_of_gc_threads: true,
        use_string_deduplication: true,

        parallel_gc_threads: parallel as u64,
        conc_gc_threads: concurrent as u64,
        soft_ref_lru_policy_ms_per_mb: soft_ref_ms,

        reserved_code_cache_size_mb: 400,
        max_inline_level: jit.max_inline_level,
        freq_inline_size: jit.freq_inline_size,
        inline_small_code: jit.inline_small_code,
        max_node_limit: jit.max_node_limit,
        node_limit_fudge_factor: 8000,
        nmethod_sweep_activity: 1,
        dont_compile_huge_methods: false,
        allocate_prefetch_style: 3,
        always_act_as_server_class: true,
        use_xmm_for_array_copy: true,
        use_fpu_for_spilling: true,

        use_large_pages: sys.large_pages,
        large_page_size_mb: sys.large_page_size / (1024 * 1024),

        reflection_inflation_threshold: 0,
        auto_box_cache_max: 4096,
        use_thread_priorities: true,
        thread_priority_policy: 1,
        use_counter_decay: false,
        compile_threshold_scaling: 0.5,
    }
}

/// Пресет поверх `generate(sys)`: сохраняется как `preset_<stem>.json`.
///
/// Идентификаторы: `latency`, `throughput`, `conservative`, `low_ram`, `balanced`,
/// `streaming`, `nursery`, `power` (алиасы: `hardware`/`auto` → balanced, `lowram`).
pub fn apply_named_preset(sys: &SystemInfo, id: &str) -> Result<(Config, String), String> {
    let id = id.trim().to_ascii_lowercase();
    let (stem, key) = match id.as_str() {
        "latency" => ("preset_latency", "latency"),
        "throughput" => ("preset_throughput", "throughput"),
        "conservative" => ("preset_conservative", "conservative"),
        "low_ram" | "lowram" => ("preset_low_ram", "low_ram"),
        "balanced" | "hardware" | "auto" => ("preset_balanced", "balanced"),
        "streaming" => ("preset_streaming", "streaming"),
        "nursery" => ("preset_nursery", "nursery"),
        "power" => ("preset_power", "power"),
        _ => {
            return Err(format!(
                "Unknown preset '{}'. Valid: latency, throughput, conservative, low_ram, balanced, streaming, nursery, power",
                id
            ));
        }
    };

    let mut cfg = generate(sys);

    match key {
        "balanced" => {}
        "latency" => {
            cfg.max_gc_pause_millis = (cfg.max_gc_pause_millis * 7 / 10).max(22);
            cfg.initiating_heap_occupancy_percent = cfg.initiating_heap_occupancy_percent.saturating_sub(4).max(12);
            cfg.g1_new_size_percent = (cfg.g1_new_size_percent + 4).min(42);
            cfg.g1_mixed_gc_count_target = (cfg.g1_mixed_gc_count_target + 1).min(6);
            cfg.soft_ref_lru_policy_ms_per_mb = (cfg.soft_ref_lru_policy_ms_per_mb + 12).min(85);
        }
        "throughput" => {
            cfg.max_gc_pause_millis = (cfg.max_gc_pause_millis * 13 / 10).min(95);
            cfg.initiating_heap_occupancy_percent = (cfg.initiating_heap_occupancy_percent + 5).min(32);
            cfg.g1_mixed_gc_count_target = cfg.g1_mixed_gc_count_target.saturating_sub(1).max(2);
            cfg.soft_ref_lru_policy_ms_per_mb = cfg.soft_ref_lru_policy_ms_per_mb.saturating_sub(8).max(15);
        }
        "conservative" => {
            cfg.use_large_pages = false;
            cfg.pre_touch = false;
            cfg.max_inline_level = cfg.max_inline_level.saturating_sub(4).max(9);
            cfg.max_node_limit = cfg.max_node_limit.saturating_sub(50_000).max(120_000);
            cfg.freq_inline_size = cfg.freq_inline_size.saturating_sub(100).max(250);
            cfg.compile_threshold_scaling = 0.65;
            cfg.dont_compile_huge_methods = true;
        }
        "low_ram" => {
            cfg.heap_size_gb = cfg.heap_size_gb.saturating_sub(1).max(2);
            cfg.g1_heap_region_size_mb = region_size(cfg.heap_size_gb);
            if sys.total_gb() < 12 {
                cfg.pre_touch = false;
            }
        }
        "streaming" => {
            cfg.heap_size_gb = cfg.heap_size_gb.saturating_sub(1).max(2);
            cfg.g1_heap_region_size_mb = region_size(cfg.heap_size_gb);
            cfg.pre_touch = false;
            cfg.max_gc_pause_millis = (cfg.max_gc_pause_millis * 11 / 10).min(88);
            cfg.soft_ref_lru_policy_ms_per_mb = (cfg.soft_ref_lru_policy_ms_per_mb + 18).min(95);
            cfg.metaspace_mb = 640;
        }
        "nursery" => {
            cfg.g1_new_size_percent = (cfg.g1_new_size_percent + 8).min(45);
            cfg.initiating_heap_occupancy_percent = (cfg.initiating_heap_occupancy_percent + 3).min(30);
            cfg.g1_mixed_gc_count_target = (cfg.g1_mixed_gc_count_target + 1).min(6);
            cfg.max_gc_pause_millis = (cfg.max_gc_pause_millis * 9 / 10).max(24);
        }
        "power" => {
            cfg.parallel_gc_threads = (cfg.parallel_gc_threads + 1).min(10);
            cfg.conc_gc_threads = (cfg.conc_gc_threads + 1).min(5);
            cfg.reserved_code_cache_size_mb = (cfg.reserved_code_cache_size_mb + 80).min(600);
        }
        _ => unreachable!(),
    }

    Ok((cfg, stem.to_string()))
}

// ─── Вспомогательные функции генерации ───────────────────────────────────────

struct JitLimits {
    max_inline_level: u64,
    freq_inline_size: u64,
    inline_small_code: u64,
    max_node_limit: u64,
}

/// jitProfile() — масштабирование по L3 кэшу (X3D получает глубокий inlining)
fn jit_profile(sys: &SystemInfo) -> JitLimits {
    if sys.has_big_cache() {
        JitLimits {
            max_inline_level: 20,
            freq_inline_size: 750,
            inline_small_code: 6000,
            max_node_limit: 320000,
        }
    } else {
        JitLimits {
            max_inline_level: 15,
            freq_inline_size: 500,
            inline_small_code: 4000,
            max_node_limit: 240000,
        }
    }
}

/// sizeHeap() — от 2 до 8 GB по total RAM, точный порт из generate.go
fn size_heap(total_gb: u64) -> u64 {
    match total_gb {
        t if t >= 24 => 8,
        t if t >= 16 => 6,
        t if t >= 12 => 5,
        t if t >= 8 => 4,
        t if t >= 6 => 3,
        _ => 2,
    }
}

/// gcThreads() — порт из generate.go: threads−2, capped [2..10] / [1..5]
fn gc_threads(threads: usize) -> (usize, usize) {
    let parallel = clamp(threads.saturating_sub(2), 2, 10);
    let concurrent = clamp(parallel / 2, 1, 5);
    (parallel, concurrent)
}

/// regionSize() — размер G1 региона по heap size
fn region_size(heap_gb: u64) -> u64 {
    match heap_gb {
        h if h <= 3 => 4,
        h if h <= 5 => 8,
        _ => 16,
    }
}

fn clamp(v: usize, lo: usize, hi: usize) -> usize {
    v.max(lo).min(hi)
}

// ─── Персистентность: configs/*.json ─────────────────────────────────────────

/// Директория конфигов рядом с exe (аналог Dir() в Go)
pub fn config_dir() -> PathBuf {
    if let Ok(exe) = std::env::current_exe() {
        if let Some(parent) = exe.parent() {
            return parent.join("configs");
        }
    }
    PathBuf::from("configs")
}

/// Save — записывает cfg в configs/<name>.json (аналог Config.Save() в Go)
pub fn save(cfg: &Config, name: &str) -> Result<(), String> {
    let dir = config_dir();
    std::fs::create_dir_all(&dir).map_err(|e| format!("create configs dir: {}", e))?;
    let path = dir.join(format!("{}.json", name));
    let data = serde_json::to_string_pretty(cfg).map_err(|e| format!("marshal config: {}", e))?;
    std::fs::write(&path, data).map_err(|e| format!("write {}: {}", path.display(), e))?;
    Ok(())
}

/// Load — читает configs/<name>.json (аналог Load() в Go)
pub fn load(name: &str) -> Result<Config, String> {
    let path = config_dir().join(format!("{}.json", name));
    let data =
        std::fs::read_to_string(&path).map_err(|e| format!("read {}: {}", path.display(), e))?;
    serde_json::from_str::<Config>(&data).map_err(|e| format!("parse {}: {}", path.display(), e))
}

/// list() — возвращает имена всех конфигов на диске без расширения .json
pub fn list() -> Result<Vec<String>, String> {
    let dir = config_dir();
    if !dir.exists() {
        return Ok(vec![]);
    }
    let entries = std::fs::read_dir(&dir).map_err(|e| format!("scan configs: {}", e))?;
    let mut names = Vec::new();
    for entry in entries.flatten() {
        let p = entry.path();
        if p.extension().and_then(|e| e.to_str()) == Some("json") {
            if let Some(stem) = p.file_stem().and_then(|s| s.to_str()) {
                names.push(stem.to_string());
            }
        }
    }
    names.sort();
    Ok(names)
}

/// ensure() — создаёт configs/ и default.json если пусто, выставляет active (Ensure из Go)
pub fn ensure(sys: &SystemInfo) -> Result<(), String> {
    let dir = config_dir();
    std::fs::create_dir_all(&dir).map_err(|e| format!("create configs dir: {}", e))?;

    let entries = list()?;
    if entries.is_empty() {
        let cfg = generate(sys);
        save(&cfg, "default")?;
    }

    if active_name().is_none() {
        set_active("default")?;
    }
    Ok(())
}

/// load_active() — LoadActive() из Go: активный конфиг с fallback на default
pub fn load_active() -> Result<(Config, String), String> {
    let requested = active_name().unwrap_or_else(|| "default".to_string());
    match load(&requested) {
        Ok(cfg) => Ok((cfg, requested)),
        Err(e) if requested != "default" => {
            // Fallback на default как в Go
            match load("default") {
                Ok(cfg) => Ok((cfg, "default".to_string())),
                Err(_) => Err(e),
            }
        }
        Err(e) => Err(e),
    }
}

/// active_exists() — ActiveExists() из Go
pub fn active_exists() -> bool {
    match active_name() {
        None => false,
        Some(name) => config_dir().join(format!("{}.json", name)).exists(),
    }
}

// ─── Реестр: HKCU\Software\StalcraftWrapper\ActiveConfig ─────────────────────

fn to_wide(s: &str) -> Vec<u16> {
    OsStr::new(s).encode_wide().chain(Some(0)).collect()
}

/// set_active() — SetActive() из Go: пишет имя активного конфига в HKCU
pub fn set_active(name: &str) -> Result<(), String> {
    let wide_path = to_wide(REGISTRY_PATH);
    let wide_value = to_wide("ActiveConfig");
    let wide_name = to_wide(name);

    let mut hkey: isize = 0;
    let r = unsafe {
        RegCreateKeyExW(
            HKEY_CURRENT_USER,
            wide_path.as_ptr(),
            0,
            std::ptr::null(),
            0,
            KEY_SET_VALUE,
            std::ptr::null(),
            &mut hkey,
            std::ptr::null_mut(),
        )
    };
    if r != 0 {
        return Err(format!("RegCreateKeyEx: {}", r));
    }

    let data_bytes =
        unsafe { std::slice::from_raw_parts(wide_name.as_ptr() as *const u8, wide_name.len() * 2) };
    let r = unsafe {
        RegSetValueExW(
            hkey,
            wide_value.as_ptr(),
            0,
            REG_SZ,
            data_bytes.as_ptr(),
            data_bytes.len() as u32,
        )
    };
    unsafe { RegCloseKey(hkey) };

    if r != 0 {
        return Err(format!("RegSetValueEx: {}", r));
    }
    Ok(())
}

/// active_name() — ActiveName() из Go: читает ActiveConfig из HKCU
pub fn active_name() -> Option<String> {
    let wide_path = to_wide(REGISTRY_PATH);
    let wide_value = to_wide("ActiveConfig");

    let mut hkey: isize = 0;
    let r = unsafe {
        RegOpenKeyExW(
            HKEY_CURRENT_USER,
            wide_path.as_ptr(),
            0,
            KEY_QUERY_VALUE,
            &mut hkey,
        )
    };
    if r != 0 {
        return None;
    }

    let mut buf_len: u32 = 0;
    let q = unsafe {
        RegQueryValueExW(
            hkey,
            wide_value.as_ptr(),
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            &mut buf_len,
        )
    };
    if q != 0 || buf_len == 0 {
        unsafe { RegCloseKey(hkey) };
        return None;
    }

    let mut buf = vec![0u8; buf_len as usize + 2];
    let mut actual = buf_len;
    let q = unsafe {
        RegQueryValueExW(
            hkey,
            wide_value.as_ptr(),
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            buf.as_mut_ptr(),
            &mut actual,
        )
    };
    unsafe { RegCloseKey(hkey) };
    if q != 0 {
        return None;
    }

    let wchars = actual as usize / 2;
    let wide_slice: Vec<u16> = (0..wchars)
        .map(|i| u16::from_le_bytes([buf[i * 2], buf[i * 2 + 1]]))
        .collect();
    let end = wide_slice.iter().position(|&c| c == 0).unwrap_or(wchars);
    String::from_utf16(&wide_slice[..end])
        .ok()
        .filter(|s| !s.is_empty())
}
