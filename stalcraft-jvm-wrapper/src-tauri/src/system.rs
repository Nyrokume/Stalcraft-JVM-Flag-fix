// system.rs — полный порт sysinfo.go
// Определяет RAM, ядра CPU (физические + логические), L3 кэш,
// Large Pages (SeLockMemoryPrivilege), имена CPU/GPU из реестра.

#![allow(non_snake_case)] // MEMORYSTATUSEX / privilege structs match Windows SDK

use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;

// ─── Windows API ──────────────────────────────────────────────────────────────

#[link(name = "kernel32")]
extern "system" {
    fn GlobalMemoryStatusEx(lpBuffer: *mut MEMORYSTATUSEX) -> i32;
    fn GetLargePageMinimum() -> usize;
    fn GetLogicalProcessorInformationEx(
        RelationshipType: u32,
        Buffer: *mut u8,
        ReturnedLength: *mut u32,
    ) -> i32;
}

#[link(name = "advapi32")]
extern "system" {
    fn OpenProcessToken(ProcessHandle: isize, DesiredAccess: u32, TokenHandle: *mut isize) -> i32;
    fn LookupPrivilegeValueW(
        lpSystemName: *const u16,
        lpName: *const u16,
        lpLuid: *mut LUID,
    ) -> i32;
    fn PrivilegeCheck(
        ClientToken: isize,
        RequiredPrivileges: *mut PRIVILEGE_SET,
        pfResult: *mut i32,
    ) -> i32;
    fn GetCurrentProcess() -> isize;
}

#[link(name = "advapi32")]
extern "system" {
    fn CloseHandle(hObject: isize) -> i32;
}

#[link(name = "kernel32")]
extern "system" {
    fn RegOpenKeyExW(
        hKey: isize,
        lpSubKey: *const u16,
        ulOptions: u32,
        samDesired: u32,
        phkResult: *mut isize,
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
    fn RegEnumKeyExW(
        hKey: isize,
        dwIndex: u32,
        lpName: *mut u16,
        lpcchName: *mut u32,
        lpReserved: *mut u32,
        lpClass: *mut u16,
        lpcchClass: *mut u32,
        lpftLastWriteTime: *mut u32,
    ) -> i32;
}

// ─── Структуры ────────────────────────────────────────────────────────────────

#[repr(C)]
struct MEMORYSTATUSEX {
    dwLength: u32,
    dwMemoryLoad: u32,
    ullTotalPhys: u64,
    ullAvailPhys: u64,
    ullTotalPageFile: u64,
    ullAvailPageFile: u64,
    ullTotalVirtual: u64,
    ullAvailVirtual: u64,
    ullAvailExtendedVirtual: u64,
}

#[repr(C)]
#[derive(Copy, Clone, Default)]
struct LUID {
    LowPart: u32,
    HighPart: i32,
}

#[repr(C)]
struct LUID_AND_ATTRIBUTES {
    Luid: LUID,
    Attributes: u32,
}

#[repr(C)]
struct PRIVILEGE_SET {
    PrivilegeCount: u32,
    Control: u32,
    Privilege: [LUID_AND_ATTRIBUTES; 1],
}

// ─── Константы ────────────────────────────────────────────────────────────────

const HKEY_LOCAL_MACHINE: isize = -2147483648i64 as isize; // 0x80000002
const KEY_READ: u32 = 0x20019;
const KEY_WOW64_64KEY: u32 = 0x0100;

const RELATION_PROCESSOR_CORE: u32 = 0;
const RELATION_CACHE: u32 = 2;
const CACHE_UNIFIED: u32 = 0;

// ─── Публичная структура ──────────────────────────────────────────────────────

/// Полная информация о системе — точный эквивалент sysinfo.Info из Go.
pub struct SystemInfo {
    pub total_ram: u64,
    pub free_ram: u64,
    /// Физические ядра (без HT/SMT), аналог CPUCores в Go
    pub cpu_cores: usize,
    /// Логические потоки (CPUThreads в Go) — используется для расчёта GC потоков
    pub cpu_threads: usize,
    /// Максимальный L3 кэш на один CCD в MB (аналог L3CacheMB в Go)
    pub l3_cache_mb: usize,
    pub large_pages: bool,
    pub large_page_size: u64,
    pub cpu_name: String,
    pub gpu_name: String,
}

impl SystemInfo {
    pub fn total_ram_gb(&self) -> f64 {
        self.total_ram as f64 / (1u64 << 30) as f64
    }
    pub fn free_ram_gb(&self) -> f64 {
        self.free_ram as f64 / (1u64 << 30) as f64
    }
    pub fn total_gb(&self) -> u64 {
        self.total_ram >> 30
    }
    #[allow(dead_code)]
    pub fn free_gb(&self) -> u64 {
        self.free_ram >> 30
    }
    /// X3D-класс: L3 >= 64 MB на CCD (аналог HasBigCache в Go)
    pub fn has_big_cache(&self) -> bool {
        self.l3_cache_mb >= 64
    }
    pub fn describe(&self) -> String {
        let mut s = format!(
            "{} cores, {:.1} GB RAM ({:.1} GB free)",
            self.cpu_cores,
            self.total_ram_gb(),
            self.free_ram_gb()
        );
        if self.l3_cache_mb > 0 {
            s.push_str(&format!(", L3 {} MB", self.l3_cache_mb));
        }
        if self.large_pages {
            s.push_str(", large pages available");
        }
        s
    }
}

// ─── Публичная функция обнаружения ────────────────────────────────────────────

/// Detect — точный аналог sysinfo.Detect() из Go.
/// Никогда не падает: любой сбой откатывается к безопасному значению.
pub fn detect_system() -> SystemInfo {
    let (total_ram, free_ram) = detect_memory();
    let (large_pages, large_page_size) = detect_large_pages();
    let cpu_cores = detect_physical_cores();
    let cpu_threads = detect_logical_threads();
    let l3_cache_mb = detect_l3_cache_mb();
    let cpu_name = detect_cpu_name();
    let gpu_name = detect_gpu_name();

    SystemInfo {
        total_ram,
        free_ram,
        cpu_cores,
        cpu_threads,
        l3_cache_mb,
        large_pages,
        large_page_size,
        cpu_name,
        gpu_name,
    }
}

// ─── Память ───────────────────────────────────────────────────────────────────

fn detect_memory() -> (u64, u64) {
    let mut ms = MEMORYSTATUSEX {
        dwLength: std::mem::size_of::<MEMORYSTATUSEX>() as u32,
        dwMemoryLoad: 0,
        ullTotalPhys: 0,
        ullAvailPhys: 0,
        ullTotalPageFile: 0,
        ullAvailPageFile: 0,
        ullTotalVirtual: 0,
        ullAvailVirtual: 0,
        ullAvailExtendedVirtual: 0,
    };
    if unsafe { GlobalMemoryStatusEx(&mut ms) } != 0 {
        (ms.ullTotalPhys, ms.ullAvailPhys)
    } else {
        (0, 0)
    }
}

// ─── Large Pages + SeLockMemoryPrivilege ──────────────────────────────────────

fn detect_large_pages() -> (bool, u64) {
    let size = unsafe { GetLargePageMinimum() };
    if size == 0 {
        return (false, 0);
    }
    (has_large_page_privilege(), size as u64)
}

/// Точный аналог hasLargePagePrivilege() из Go.
fn has_large_page_privilege() -> bool {
    unsafe {
        let proc = GetCurrentProcess();
        let mut token: isize = 0;
        // TOKEN_QUERY = 0x0008
        if OpenProcessToken(proc, 0x0008, &mut token) == 0 {
            return false;
        }
        let _guard = TokenGuard(token);

        let name_wide = to_wide("SeLockMemoryPrivilege");
        let mut luid = LUID::default();
        if LookupPrivilegeValueW(std::ptr::null(), name_wide.as_ptr(), &mut luid) == 0 {
            return false;
        }

        let mut ps = PRIVILEGE_SET {
            PrivilegeCount: 1,
            Control: 0,
            Privilege: [LUID_AND_ATTRIBUTES {
                Luid: luid,
                Attributes: 0x00000002, // SE_PRIVILEGE_ENABLED
            }],
        };
        let mut result: i32 = 0;
        let ret = PrivilegeCheck(token, &mut ps, &mut result);
        ret != 0 && result != 0
    }
}

struct TokenGuard(isize);
impl Drop for TokenGuard {
    fn drop(&mut self) {
        if self.0 != 0 {
            unsafe { CloseHandle(self.0) };
        }
    }
}

// ─── CPU: физические ядра + логические потоки ─────────────────────────────────

/// physicalCores() из Go — через GetLogicalProcessorInformationEx(RelationProcessorCore)
fn detect_physical_cores() -> usize {
    let buf = match get_processor_info(RELATION_PROCESSOR_CORE) {
        Some(b) => b,
        None => return std::thread::available_parallelism().map(|n| n.get()).unwrap_or(4),
    };
    let mut cores = 0usize;
    let mut off = 0u32;
    let len = buf.len() as u32;
    while off < len {
        if off + 8 > len {
            break;
        }
        let size = u32::from_le_bytes([buf[off as usize + 4], buf[off as usize + 5], buf[off as usize + 6], buf[off as usize + 7]]);
        if size == 0 {
            break;
        }
        cores += 1;
        off += size;
    }
    if cores == 0 {
        std::thread::available_parallelism().map(|n| n.get()).unwrap_or(4)
    } else {
        cores
    }
}

/// runtime.NumCPU() аналог — логические потоки ОС
fn detect_logical_threads() -> usize {
    std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(4)
}

// ─── L3 Cache (detectL3CacheMB из Go) ────────────────────────────────────────

/// detectL3CacheMB() — возвращает максимальный unified L3 на один CCD в MB.
/// На multi-CCD CPU (5950X) это per-CCD, не сумма — точно как в Go.
fn detect_l3_cache_mb() -> usize {
    let buf = match get_processor_info(RELATION_CACHE) {
        Some(b) => b,
        None => return 0,
    };

    let mut max_bytes: u64 = 0;
    let mut off = 0u32;
    let len = buf.len() as u32;

    while off < len {
        if off + 20 > len {
            break;
        }
        let base = off as usize;
        // SYSTEM_LOGICAL_PROCESSOR_INFORMATION_EX layout:
        //  [0..4]  Relationship (u32)
        //  [4..8]  Size (u32)
        //  [8]     Level (u8)  — для CACHE_RELATIONSHIP offset 8
        //  [12..16] CacheSize (u32)
        //  [16..20] Type (u32)
        let size = u32::from_le_bytes([buf[base + 4], buf[base + 5], buf[base + 6], buf[base + 7]]);
        if size == 0 || (off + size) > len {
            break;
        }

        // CACHE_RELATIONSHIP начинается с offset 8 в структуре
        // Level: byte at [base+8]
        // CacheSize: u32 at [base+12]
        // Type: u32 at [base+16]
        if base + 20 <= buf.len() {
            let level = buf[base + 8];
            let cache_size = u32::from_le_bytes([buf[base + 12], buf[base + 13], buf[base + 14], buf[base + 15]]);
            let cache_type = u32::from_le_bytes([buf[base + 16], buf[base + 17], buf[base + 18], buf[base + 19]]);

            if level == 3 && cache_type == CACHE_UNIFIED && (cache_size as u64) > max_bytes {
                max_bytes = cache_size as u64;
            }
        }

        off += size;
    }

    (max_bytes >> 20) as usize
}

/// Вызывает GetLogicalProcessorInformationEx с двойным вызовом (сначала размер, потом данные)
fn get_processor_info(relation: u32) -> Option<Vec<u8>> {
    let mut buf_len: u32 = 0;
    // Первый вызов — узнаём размер буфера
    unsafe {
        GetLogicalProcessorInformationEx(relation, std::ptr::null_mut(), &mut buf_len);
    }
    if buf_len == 0 {
        return None;
    }
    let mut buf = vec![0u8; buf_len as usize];
    let ret = unsafe {
        GetLogicalProcessorInformationEx(relation, buf.as_mut_ptr(), &mut buf_len)
    };
    if ret == 0 {
        return None;
    }
    buf.truncate(buf_len as usize);
    Some(buf)
}

// ─── CPU / GPU из реестра (устойчивее к нумерации подключей) ─────────────────

/// Перебирает `CentralProcessor\0` … `\31` — на части систем имя только не в `\0`.
fn detect_cpu_name() -> String {
    for i in 0u32..32 {
        let path = format!(r"HARDWARE\DESCRIPTION\System\CentralProcessor\{}", i);
        if let Some(s) = get_registry_string(&path, "ProcessorNameString")
            .filter(|s| !s.trim().is_empty())
        {
            return s;
        }
        if let Some(s) = get_registry_string(&path, "Identifier")
            .filter(|s| !s.trim().is_empty())
        {
            return s;
        }
    }
    "Unknown CPU".to_string()
}

const DISPLAY_CLASS_PATH: &str =
    r"SYSTEM\CurrentControlSet\Control\Class\{4d36e968-e325-11ce-bfc1-08002be10318}";

fn is_display_class_subkey(name: &str) -> bool {
    name.len() == 4 && name.chars().all(|c| c.is_ascii_hexdigit())
}

fn gpu_driver_desc_usable(s: &str) -> bool {
    let t = s.trim().to_lowercase();
    !t.is_empty()
        && !t.contains("microsoft basic render")
        && !t.contains("microsoft basic display")
}

/// Перечисляет подключи класса дисплея и читает `DriverDesc` (раньше смотрели только 0000/0001).
fn detect_gpu_name() -> String {
    let mut candidates: Vec<(u32, String)> = Vec::new();
    for sub in enumerate_subkey_names(DISPLAY_CLASS_PATH) {
        if !is_display_class_subkey(&sub) {
            continue;
        }
        let ord = u32::from_str_radix(&sub, 16).unwrap_or(0);
        let path = format!(r"{}\{}", DISPLAY_CLASS_PATH, sub);
        if let Some(desc) = get_registry_string(&path, "DriverDesc").filter(|d| gpu_driver_desc_usable(d)) {
            candidates.push((ord, desc));
        }
    }
    candidates.sort_by_key(|(ord, _)| *ord);
    if let Some((_, desc)) = candidates.last() {
        return desc.clone();
    }
    // без фильтра — хоть какое-то имя адаптера
    for sub in enumerate_subkey_names(DISPLAY_CLASS_PATH) {
        if !is_display_class_subkey(&sub) {
            continue;
        }
        let path = format!(r"{}\{}", DISPLAY_CLASS_PATH, sub);
        if let Some(desc) = get_registry_string(&path, "DriverDesc").filter(|d| !d.trim().is_empty()) {
            return desc;
        }
    }
    "Unknown GPU".to_string()
}

/// Имена подключей первого уровня под `HKLM\path` (для класса видео).
fn enumerate_subkey_names(key_path: &str) -> Vec<String> {
    let wide_path = to_wide(key_path);
    let key_read_64 = KEY_READ | KEY_WOW64_64KEY;
    let mut hkey: isize = 0;
    let open = unsafe {
        RegOpenKeyExW(
            HKEY_LOCAL_MACHINE,
            wide_path.as_ptr(),
            0,
            key_read_64,
            &mut hkey,
        )
    };
    if open != 0 {
        return Vec::new();
    }

    let mut out = Vec::new();
    let mut idx = 0u32;
    loop {
        let mut name_buf = vec![0u16; 256];
        let mut name_chars = name_buf.len() as u32;
        let r = unsafe {
            RegEnumKeyExW(
                hkey,
                idx,
                name_buf.as_mut_ptr(),
                &mut name_chars,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                std::ptr::null_mut(),
            )
        };
        if r != 0 {
            break;
        }
        let end = name_buf
            .iter()
            .position(|&c| c == 0)
            .unwrap_or(name_chars as usize);
        let s = String::from_utf16_lossy(&name_buf[..end]);
        if !s.is_empty() {
            out.push(s.to_string());
        }
        idx += 1;
        if idx > 256 {
            break;
        }
    }
    unsafe { RegCloseKey(hkey) };
    out
}

// ─── Утилиты реестра ─────────────────────────────────────────────────────────

fn to_wide(s: &str) -> Vec<u16> {
    OsStr::new(s).encode_wide().chain(Some(0)).collect()
}

pub fn get_registry_string(key_path: &str, value_name: &str) -> Option<String> {
    let wide_path = to_wide(key_path);
    let wide_value = to_wide(value_name);
    let key_read_64 = KEY_READ | KEY_WOW64_64KEY;

    let mut hkey: isize = 0;
    let result = unsafe {
        RegOpenKeyExW(HKEY_LOCAL_MACHINE, wide_path.as_ptr(), 0, key_read_64, &mut hkey)
    };
    if result != 0 {
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

    let mut buf: Vec<u8> = vec![0u8; buf_len as usize + 2];
    let mut actual_len = buf_len;
    let q = unsafe {
        RegQueryValueExW(
            hkey,
            wide_value.as_ptr(),
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            buf.as_mut_ptr(),
            &mut actual_len,
        )
    };
    unsafe { RegCloseKey(hkey) };
    if q != 0 {
        return None;
    }

    let wchars = actual_len as usize / 2;
    let wide_slice: Vec<u16> = (0..wchars)
        .map(|i| u16::from_le_bytes([buf[i * 2], buf[i * 2 + 1]]))
        .collect();

    // убираем нулевой терминатор
    let end = wide_slice.iter().position(|&c| c == 0).unwrap_or(wchars);
    String::from_utf16(&wide_slice[..end])
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}
