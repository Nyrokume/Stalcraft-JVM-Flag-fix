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
    fn GetSystemFirmwareTable(
        FirmwareTableProviderSignature: u32,
        FirmwareTableID: u32,
        pFirmwareTableBuffer: *mut u8,
        BufferSize: u32,
    ) -> u32;
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
const KEY_WOW64_32KEY: u32 = 0x0200;

const RELATION_PROCESSOR_CORE: u32 = 0;
const RELATION_CACHE: u32 = 2;
const CACHE_UNIFIED: u32 = 0;

const SMBIOS_PROVIDER_RSMB: u32 = 0x5253_4D42; // 'RSMB'
const DMI_MEMORY_DEVICE: u8 = 17;
const DMI_END_OF_TABLE: u8 = 127;

/// Memory bandwidth tier for G1 tuning (Go MemTier).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MemTier {
    Slow,
    Mid,
}

impl MemTier {
    pub fn as_str(self) -> &'static str {
        match self {
            MemTier::Slow => "slow",
            MemTier::Mid => "mid",
        }
    }
}

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
    /// Highest ConfiguredMemoryClockSpeed from SMBIOS Type 17, MT/s (0 = unknown)
    pub mem_speed_mts: usize,
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
    /// X3D-класс: L3 >= 64 MB на CCD (GUI only; generate() no longer keys off this)
    pub fn has_big_cache(&self) -> bool {
        self.l3_cache_mb >= 64
    }

    /// MemTier — slow (≤2933 MT/s) vs mid (everything else / unknown).
    pub fn mem_tier(&self) -> MemTier {
        if self.mem_speed_mts > 0 && self.mem_speed_mts <= 2933 {
            MemTier::Slow
        } else {
            MemTier::Mid
        }
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
            if self.large_page_size > 0 {
                s.push_str(&format!(", large pages ({} MB)", self.large_page_size >> 20));
            } else {
                s.push_str(", large pages available");
            }
        }
        if self.mem_speed_mts > 0 {
            s.push_str(&format!(", mem {} MT/s ({})", self.mem_speed_mts, self.mem_tier().as_str()));
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
    let mem_speed_mts = detect_mem_speed_mts();
    let cpu_name = detect_cpu_name();
    let gpu_name = detect_gpu_name();

    SystemInfo {
        total_ram,
        free_ram,
        cpu_cores,
        cpu_threads,
        l3_cache_mb,
        mem_speed_mts,
        large_pages,
        large_page_size,
        cpu_name,
        gpu_name,
    }
}

// ─── SMBIOS memory speed (mem.go) ───────────────────────────────────────────

fn detect_mem_speed_mts() -> usize {
    let size = unsafe {
        GetSystemFirmwareTable(SMBIOS_PROVIDER_RSMB, 0, std::ptr::null_mut(), 0)
    };
    if size == 0 {
        return 0;
    }
    let mut buf = vec![0u8; size as usize];
    let got = unsafe {
        GetSystemFirmwareTable(
            SMBIOS_PROVIDER_RSMB,
            0,
            buf.as_mut_ptr(),
            size,
        )
    };
    if got == 0 {
        return 0;
    }
    if buf.len() < 8 {
        return 0;
    }
    let table_len = u32::from_le_bytes([buf[4], buf[5], buf[6], buf[7]]) as usize;
    if table_len == 0 || 8 + table_len > buf.len() {
        return 0;
    }
    let table = &buf[8..8 + table_len];
    let mut best = 0usize;
    let mut off = 0usize;
    while off + 4 <= table.len() {
        let typ = table[off];
        let length = table[off + 1] as usize;
        if length < 4 || off + length > table.len() {
            break;
        }
        if typ == DMI_MEMORY_DEVICE {
            if let Some(s) = mem_device_speed_mts(&table[off..off + length]) {
                if s > best {
                    best = s;
                }
            }
        }
        off += length;
        while off + 1 < table.len() && !(table[off] == 0 && table[off + 1] == 0) {
            off += 1;
        }
        off += 2;
        if typ == DMI_END_OF_TABLE {
            break;
        }
    }
    best
}

fn mem_device_speed_mts(rec: &[u8]) -> Option<usize> {
    if rec.len() < 0x0E {
        return None;
    }
    let size = u16::from_le_bytes([rec[0x0C], rec[0x0D]]);
    if size == 0 {
        return None;
    }
    if rec.len() >= 0x22 {
        let v = u16::from_le_bytes([rec[0x20], rec[0x21]]);
        if v != 0 && v != 0xFFFF {
            return Some(v as usize);
        }
        if rec.len() >= 0x5C && v == 0xFFFF {
            let ext = u32::from_le_bytes([rec[0x58], rec[0x59], rec[0x5A], rec[0x5B]]);
            if ext != 0 {
                return Some(ext as usize);
            }
        }
    }
    if rec.len() >= 0x17 {
        let v = u16::from_le_bytes([rec[0x15], rec[0x16]]);
        if v != 0 && v != 0xFFFF {
            return Some(v as usize);
        }
        if rec.len() >= 0x58 && v == 0xFFFF {
            let ext = u32::from_le_bytes([rec[0x54], rec[0x55], rec[0x56], rec[0x57]]);
            if ext != 0 {
                return Some(ext as usize);
            }
        }
    }
    None
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

// ─── CPU / GPU из реестра + CPUID + WMI ─────────────────────────────────────

fn clean_hardware_name(s: &str) -> String {
    let mut t = s.trim().to_string();
    while t.contains("  ") {
        t = t.replace("  ", " ");
    }
    t
}

fn is_generic_cpu_identifier(s: &str) -> bool {
    let t = s.trim();
    t.starts_with("Intel64 Family")
        || t.starts_with("AMD64 Family")
        || t.starts_with("x86 Family")
        || t.eq_ignore_ascii_case("unknown")
}

#[cfg(target_arch = "x86_64")]
fn detect_cpu_brand_cpuid() -> Option<String> {
    use std::arch::x86_64::{__cpuid, __cpuid_count};

    let ext = unsafe { __cpuid(0x8000_0000) };
    if ext.eax < 0x8000_0004 {
        return None;
    }

    let mut brand = [0u32; 12];
    for (i, chunk) in brand.chunks_mut(4).enumerate() {
        let leaf = unsafe { __cpuid_count(0x8000_0002 + i as u32, 0) };
        chunk[0] = leaf.eax;
        chunk[1] = leaf.ebx;
        chunk[2] = leaf.ecx;
        chunk[3] = leaf.edx;
    }

    let bytes: Vec<u8> = brand.iter().flat_map(|w| w.to_le_bytes()).collect();
    let s = String::from_utf8_lossy(&bytes)
        .trim_matches('\0')
        .trim()
        .to_string();
    if s.is_empty() || is_generic_cpu_identifier(&s) {
        return None;
    }
    Some(clean_hardware_name(&s))
}

#[cfg(not(target_arch = "x86_64"))]
fn detect_cpu_brand_cpuid() -> Option<String> {
    None
}

#[cfg(windows)]
fn run_hidden_powershell(script: &str) -> Option<String> {
    use std::os::windows::process::CommandExt;
    const CREATE_NO_WINDOW: u32 = 0x0800_0000;

    let output = std::process::Command::new("powershell")
        .args(["-NoProfile", "-NonInteractive", "-Command", script])
        .creation_flags(CREATE_NO_WINDOW)
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let s = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if s.is_empty() {
        return None;
    }
    Some(clean_hardware_name(&s))
}

#[cfg(not(windows))]
fn run_hidden_powershell(_script: &str) -> Option<String> {
    None
}

fn detect_cpu_name_wmi() -> Option<String> {
    run_hidden_powershell(
        "(Get-CimInstance Win32_Processor | Select-Object -ExpandProperty Name -First 1)",
    )
    .filter(|s| !is_generic_cpu_identifier(s))
}

fn detect_gpu_name_wmi() -> Option<String> {
    run_hidden_powershell(
        "Get-CimInstance Win32_VideoController | Where-Object { $_.AdapterRAM -gt 0 -and $_.Name -notmatch 'Microsoft Basic' } | Sort-Object AdapterRAM -Descending | Select-Object -ExpandProperty Name -First 1",
    )
    .filter(|s| gpu_driver_desc_usable(s))
}

/// `ProcessorNameString` из любого подключа `CentralProcessor` (нумерация на разных ПК разная).
fn detect_cpu_name() -> String {
    const CPU_BASE: &str = r"HARDWARE\DESCRIPTION\System\CentralProcessor";
    for sub in enumerate_subkey_names(CPU_BASE) {
        let path = format!(r"{}\{}", CPU_BASE, sub);
        if let Some(s) = get_registry_string(&path, "ProcessorNameString")
            .filter(|s| !s.trim().is_empty() && !is_generic_cpu_identifier(s))
        {
            return clean_hardware_name(&s);
        }
    }
    for i in 0u32..32 {
        let path = format!(r"{}\{}", CPU_BASE, i);
        if let Some(s) = get_registry_string(&path, "ProcessorNameString")
            .filter(|s| !s.trim().is_empty() && !is_generic_cpu_identifier(s))
        {
            return clean_hardware_name(&s);
        }
    }
    if let Some(s) = detect_cpu_brand_cpuid() {
        return s;
    }
    if let Some(s) = detect_cpu_name_wmi() {
        return s;
    }
    for sub in enumerate_subkey_names(CPU_BASE) {
        let path = format!(r"{}\{}", CPU_BASE, sub);
        if let Some(s) = get_registry_string(&path, "ProcessorNameString")
            .filter(|s| !s.trim().is_empty())
        {
            return clean_hardware_name(&s);
        }
        if let Some(s) = get_registry_string(&path, "Identifier")
            .filter(|s| !s.trim().is_empty())
        {
            return clean_hardware_name(&s);
        }
    }
    if let Ok(v) = std::env::var("PROCESSOR_IDENTIFIER") {
        let t = v.trim().to_string();
        if !t.is_empty() {
            return clean_hardware_name(&t);
        }
    }
    detect_cpu_brand_cpuid().unwrap_or_else(|| "Unknown CPU".to_string())
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
        return clean_hardware_name(desc);
    }
    // без фильтра — хоть какое-то имя адаптера
    for sub in enumerate_subkey_names(DISPLAY_CLASS_PATH) {
        if !is_display_class_subkey(&sub) {
            continue;
        }
        let path = format!(r"{}\{}", DISPLAY_CLASS_PATH, sub);
        if let Some(desc) = get_registry_string(&path, "DriverDesc").filter(|d| !d.trim().is_empty()) {
            return clean_hardware_name(&desc);
        }
    }
    if let Some(s) = detect_gpu_name_wmi() {
        return s;
    }
    "Unknown GPU".to_string()
}

/// Имена подключей первого уровня под `HKLM\path` (для класса видео).
fn enumerate_subkey_names(key_path: &str) -> Vec<String> {
    for flags in [KEY_READ, KEY_READ | KEY_WOW64_64KEY, KEY_READ | KEY_WOW64_32KEY] {
        let names = enumerate_subkey_names_with_flags(key_path, flags);
        if !names.is_empty() {
            return names;
        }
    }
    Vec::new()
}

fn enumerate_subkey_names_with_flags(key_path: &str, sam_desired: u32) -> Vec<String> {
    let wide_path = to_wide(key_path);
    let mut hkey: isize = 0;
    let open = unsafe {
        RegOpenKeyExW(
            HKEY_LOCAL_MACHINE,
            wide_path.as_ptr(),
            0,
            sam_desired,
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
    for flags in [KEY_READ, KEY_READ | KEY_WOW64_64KEY, KEY_READ | KEY_WOW64_32KEY] {
        if let Some(s) = get_registry_string_with_flags(key_path, value_name, flags) {
            return Some(s);
        }
    }
    None
}

fn get_registry_string_with_flags(key_path: &str, value_name: &str, sam_desired: u32) -> Option<String> {
    let wide_path = to_wide(key_path);
    let wide_value = to_wide(value_name);

    let mut hkey: isize = 0;
    let result = unsafe {
        RegOpenKeyExW(HKEY_LOCAL_MACHINE, wide_path.as_ptr(), 0, sam_desired, &mut hkey)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mem_device_speed_reads_configured_clock() {
        let mut rec = vec![0u8; 0x22];
        rec[0x0C] = 8;
        rec[0x0D] = 0; // size != 0
        rec[0x20] = 0x40;
        rec[0x21] = 0x06; // 1600 MT/s
        assert_eq!(mem_device_speed_mts(&rec), Some(1600));
    }

    #[test]
    fn mem_tier_slow_at_2933() {
        let sys = SystemInfo {
            total_ram: 0,
            free_ram: 0,
            cpu_cores: 8,
            cpu_threads: 16,
            l3_cache_mb: 32,
            mem_speed_mts: 2933,
            large_pages: false,
            large_page_size: 0,
            cpu_name: String::new(),
            gpu_name: String::new(),
        };
        assert_eq!(sys.mem_tier(), MemTier::Slow);
    }

    #[test]
    fn mem_tier_mid_when_unknown() {
        let sys = SystemInfo {
            total_ram: 0,
            free_ram: 0,
            cpu_cores: 8,
            cpu_threads: 16,
            l3_cache_mb: 32,
            mem_speed_mts: 0,
            large_pages: false,
            large_page_size: 0,
            cpu_name: String::new(),
            gpu_name: String::new(),
        };
        assert_eq!(sys.mem_tier(), MemTier::Mid);
    }
}
