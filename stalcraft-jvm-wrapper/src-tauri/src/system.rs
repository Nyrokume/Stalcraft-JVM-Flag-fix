use windows_sys::Win32::System::SystemInformation::{GlobalMemoryStatusEx, MEMORYSTATUSEX};
use windows_sys::Win32::System::Registry::{RegOpenKeyExW, RegQueryValueExW, RegCloseKey, HKEY_LOCAL_MACHINE, KEY_READ, KEY_WOW64_64KEY};
use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;

const KEY_READ_64: u32 = KEY_READ | KEY_WOW64_64KEY;

pub struct SystemInfo {
    pub cpu_name: String,
    pub gpu_name: String,
    pub total_ram: u64,
    pub free_ram: u64,
    pub cpu_cores: usize,
    pub large_pages: bool,
    pub large_page_size: u64,
}

// GetLargePageMinimum is exported without the W suffix
#[link(name = "kernel32")]
extern "system" {
    fn GetLargePageMinimum() -> usize;
}

fn to_wide_string(s: &str) -> Vec<u16> {
    OsStr::new(s).encode_wide().chain(Some(0)).collect()
}

fn get_registry_string(key_path: &str, value_name: &str) -> Option<String> {
    let wide_path = to_wide_string(key_path);
    let wide_value = to_wide_string(value_name);

    let mut hkey = std::ptr::null_mut();
    let result = unsafe {
        RegOpenKeyExW(
            HKEY_LOCAL_MACHINE,
            wide_path.as_ptr(),
            0,
            KEY_READ_64,
            &mut hkey,
        )
    };

    if result != 0 {
        return None;
    }

    // First, get required buffer size
    let mut buffer_len: u32 = 0;
    let query_result = unsafe {
        RegQueryValueExW(
            hkey,
            wide_value.as_ptr(),
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            &mut buffer_len,
        )
    };

    if query_result != 0 || buffer_len == 0 {
        unsafe { RegCloseKey(hkey) };
        return None;
    }

    // Allocate buffer and read value
    let mut buffer: Vec<u16> = vec![0; (buffer_len as usize / 2) + 1];
    let mut actual_len = buffer_len;
    let query_result = unsafe {
        RegQueryValueExW(
            hkey,
            wide_value.as_ptr(),
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            buffer.as_mut_ptr() as *mut u8,
            &mut actual_len,
        )
    };

    unsafe { RegCloseKey(hkey) };

    if query_result != 0 {
        return None;
    }

    // Convert wide string (exclude null terminator)
    let len = (actual_len as usize / 2).saturating_sub(1);
    if len == 0 {
        return None;
    }
    
    String::from_utf16(&buffer[..len])
        .map(|s| s.trim().to_string())
        .ok()
        .filter(|s| !s.is_empty())
}

fn get_cpu_name() -> String {
    get_registry_string(
        r"HARDWARE\DESCRIPTION\System\CentralProcessor\0",
        "ProcessorNameString"
    ).unwrap_or_else(|| "Unknown CPU".to_string())
}

fn get_gpu_name() -> String {
    // Try primary GPU adapter
    if let Some(gpu) = get_registry_string(
        r"SYSTEM\CurrentControlSet\Control\Class\{4d36e968-e325-11ce-bfc1-08002be10318}\0000",
        "DriverDesc"
    ) {
        return gpu;
    }

    // Try alternate path
    if let Some(gpu) = get_registry_string(
        r"SYSTEM\CurrentControlSet\Control\Video",
        ""
    ) {
        return gpu;
    }

    "Unknown GPU".to_string()
}

pub fn detect_system() -> SystemInfo {
    let cpu_cores = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(4);

    let cpu_name = get_cpu_name();
    let gpu_name = get_gpu_name();

    let mut mem_status: MEMORYSTATUSEX = unsafe { std::mem::zeroed() };
    mem_status.dwLength = std::mem::size_of::<MEMORYSTATUSEX>() as u32;

    let (total_ram, free_ram) = unsafe {
        if GlobalMemoryStatusEx(&mut mem_status) != 0 {
            (mem_status.ullTotalPhys, mem_status.ullAvailPhys)
        } else {
            (0, 0)
        }
    };

    // Check large pages support
    let large_page_size = unsafe { GetLargePageMinimum() };
    let large_pages = large_page_size > 0;

    SystemInfo {
        cpu_name,
        gpu_name,
        total_ram,
        free_ram,
        cpu_cores,
        large_pages,
        large_page_size: large_page_size as u64,
    }
}

impl SystemInfo {
    pub fn total_ram_gb(&self) -> f64 {
        self.total_ram as f64 / (1u64 << 30) as f64
    }

    pub fn free_ram_gb(&self) -> f64 {
        self.free_ram as f64 / (1u64 << 30) as f64
    }

    pub fn bytes_to_gb(&self, bytes: u64) -> u64 {
        bytes >> 30
    }
}
