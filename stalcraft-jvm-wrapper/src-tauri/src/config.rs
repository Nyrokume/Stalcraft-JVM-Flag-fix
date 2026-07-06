// config.rs — порт config.go + generate.go

use serde::{Deserialize, Serialize};
use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;
use std::path::PathBuf;

use crate::system::{MemTier, SystemInfo};

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

const HKEY_CURRENT_USER: isize = -2147483647i64 as isize;
const KEY_SET_VALUE: u32 = 0x0002;
const KEY_QUERY_VALUE: u32 = 0x0001;
const REG_SZ: u32 = 1;
const REGISTRY_PATH: &str = r"Software\StalcraftWrapper";
const LEGACY_REGISTRY_PATH: &str = r"Software\StalartJvmWrapper";

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

    pub reflection_inflation_threshold: i64,
    pub auto_box_cache_max: u64,
    pub use_thread_priorities: bool,
    pub thread_priority_policy: u64,
    pub use_counter_decay: bool,
    pub compile_threshold_scaling: f64,
}

pub fn generate(sys: &SystemInfo) -> Config {
    let heap = size_heap(sys.total_gb());
    let (parallel, concurrent) = gc_threads(sys.cpu_threads);

    let (pause_ms, mixed_count_target, rset_updating_pct, new_size_percent) = match sys.mem_tier() {
        MemTier::Slow => (150u64, 4u64, 5u64, 30u64),
        MemTier::Mid => (100, 6, 8, 33),
    };

    let ihop = 25u64;
    let soft_ref_ms = 50u64;
    let tenuring = 3u64;
    let survivor_ratio = 12u64;

    Config {
        heap_size_gb: heap,
        pre_touch: sys.total_gb() >= 12,
        metaspace_mb: 512,

        max_gc_pause_millis: pause_ms,
        g1_heap_region_size_mb: region_size(heap),
        g1_new_size_percent: new_size_percent,
        g1_max_new_size_percent: 50,
        g1_reserve_percent: 15,
        g1_heap_waste_percent: 10,
        g1_mixed_gc_count_target: mixed_count_target,
        initiating_heap_occupancy_percent: ihop,
        g1_mixed_gc_live_threshold_percent: 85,
        g1_rset_updating_pause_time_percent: rset_updating_pct,
        survivor_ratio,
        max_tenuring_threshold: tenuring,

        g1_satb_buffer_enqueuing_threshold_percent: 30,
        g1_conc_rs_hot_card_limit: 16,
        g1_conc_refinement_service_interval_millis: 150,
        gc_time_ratio: 99,
        use_dynamic_number_of_gc_threads: true,
        use_string_deduplication: false,

        parallel_gc_threads: parallel as u64,
        conc_gc_threads: concurrent as u64,
        soft_ref_lru_policy_ms_per_mb: soft_ref_ms,

        reserved_code_cache_size_mb: 400,
        max_inline_level: 15,
        freq_inline_size: 500,
        inline_small_code: 4000,
        max_node_limit: 240_000,
        node_limit_fudge_factor: 8000,
        nmethod_sweep_activity: 1,
        dont_compile_huge_methods: false,
        allocate_prefetch_style: 3,
        always_act_as_server_class: true,
        use_xmm_for_array_copy: true,
        use_fpu_for_spilling: true,

        use_large_pages: sys.large_pages,

        reflection_inflation_threshold: 0,
        auto_box_cache_max: 4096,
        use_thread_priorities: true,
        thread_priority_policy: 1,
        use_counter_decay: false,
        compile_threshold_scaling: 0.5,
    }
}

fn size_heap(total_gb: u64) -> u64 {
    match total_gb {
        t if t >= 16 => 6,
        t if t >= 12 => 5,
        t if t >= 8 => 4,
        t if t >= 6 => 3,
        _ => 2,
    }
}

fn gc_threads(threads: usize) -> (usize, usize) {
    let parallel = clamp(threads.saturating_sub(2), 2, 10);
    let concurrent = clamp(parallel / 2, 1, 5);
    (parallel, concurrent)
}

fn region_size(heap_gb: u64) -> u64 {
    if heap_gb <= 3 {
        4
    } else {
        8
    }
}

fn clamp(v: usize, lo: usize, hi: usize) -> usize {
    v.max(lo).min(hi)
}

pub fn config_dir() -> PathBuf {
    crate::paths::configs_dir()
}

pub fn save(cfg: &Config, name: &str) -> Result<(), String> {
    let dir = config_dir();
    std::fs::create_dir_all(&dir).map_err(|e| format!("create configs dir: {}", e))?;
    let path = dir.join(format!("{}.json", name));
    let data = serde_json::to_string_pretty(cfg).map_err(|e| format!("marshal config: {}", e))?;
    std::fs::write(&path, data).map_err(|e| format!("write {}: {}", path.display(), e))?;
    Ok(())
}

pub fn load(name: &str) -> Result<Config, String> {
    let path = config_dir().join(format!("{}.json", name));
    let data =
        std::fs::read_to_string(&path).map_err(|e| format!("read {}: {}", path.display(), e))?;
    serde_json::from_str::<Config>(&data).map_err(|e| format!("parse {}: {}", path.display(), e))
}

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

pub fn load_active() -> Result<(Config, String), String> {
    let requested = active_name().unwrap_or_else(|| "default".to_string());
    match load(&requested) {
        Ok(cfg) => Ok((cfg, requested)),
        Err(e) if requested != "default" => match load("default") {
            Ok(cfg) => Ok((cfg, "default".to_string())),
            Err(_) => Err(e),
        },
        Err(e) => Err(e),
    }
}

pub fn active_exists() -> bool {
    match active_name() {
        None => false,
        Some(name) => config_dir().join(format!("{}.json", name)).exists(),
    }
}

fn to_wide(s: &str) -> Vec<u16> {
    OsStr::new(s).encode_wide().chain(Some(0)).collect()
}

fn read_active_from_path(registry_path: &str) -> Option<String> {
    let wide_path = to_wide(registry_path);
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

/// ActiveName with ponytail migration from legacy StalartJvmWrapper key.
pub fn active_name() -> Option<String> {
    if let Some(name) = read_active_from_path(REGISTRY_PATH) {
        return Some(name);
    }
    if let Some(legacy) = read_active_from_path(LEGACY_REGISTRY_PATH) {
        let _ = set_active(&legacy);
        return Some(legacy);
    }
    None
}
