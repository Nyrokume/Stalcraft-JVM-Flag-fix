use windows_sys::Win32::System::SystemInformation::{GlobalMemoryStatusEx, MEMORYSTATUSEX};
use windows_sys::Win32::System::Registry::{RegOpenKeyExW, RegQueryValueExW, RegCloseKey, HKEY_LOCAL_MACHINE, KEY_READ};
use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;

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

fn get_cpu_name() -> String {
    let key_path = to_wide_string(r"HARDWARE\DESCRIPTION\System\CentralProcessor\0");
    let value_name = to_wide_string("ProcessorNameString");

    let mut hkey = std::ptr::null_mut();
    let result = unsafe {
        RegOpenKeyExW(
            HKEY_LOCAL_MACHINE,
            key_path.as_ptr(),
            0,
            KEY_READ,
            &mut hkey,
        )
    };

    if result != 0 {
        return "Unknown CPU".to_string();
    }

    let mut buffer: [u16; 256] = [0; 256];
    let mut buffer_len = (buffer.len() * 2) as u32;
    let query_result = unsafe {
        RegQueryValueExW(
            hkey,
            value_name.as_ptr(),
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            buffer.as_mut_ptr() as *mut u8,
            &mut buffer_len,
        )
    };

    unsafe { RegCloseKey(hkey) };

    if query_result != 0 {
        return "Unknown CPU".to_string();
    }

    // Convert wide string to Rust string
    let len = buffer_len as usize / 2;
    let wide_slice = &buffer[..len];
    String::from_utf16(wide_slice)
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|_| "Unknown CPU".to_string())
}

fn get_gpu_name() -> String {
    // Try to get GPU name from registry
    // Check Display adapters
    let key_path = to_wide_string(r"SYSTEM\CurrentControlSet\Control\Class\{4d36e968-e325-11ce-bfc1-08002be10318}\0000");
    let value_name = to_wide_string("DriverDesc");

    let mut hkey = std::ptr::null_mut();
    let result = unsafe {
        RegOpenKeyExW(
            HKEY_LOCAL_MACHINE,
            key_path.as_ptr(),
            0,
            KEY_READ,
            &mut hkey,
        )
    };

    if result == 0 {
        let mut buffer: [u16; 256] = [0; 256];
        let mut buffer_len = (buffer.len() * 2) as u32;
        let query_result = unsafe {
            RegQueryValueExW(
                hkey,
                value_name.as_ptr(),
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                buffer.as_mut_ptr() as *mut u8,
                &mut buffer_len,
            )
        };

        unsafe { RegCloseKey(hkey) };

        if query_result == 0 {
            let len = buffer_len as usize / 2;
            let wide_slice = &buffer[..len];
            if let Ok(gpu_name) = String::from_utf16(wide_slice) {
                let gpu_name = gpu_name.trim().to_string();
                if !gpu_name.is_empty() {
                    return gpu_name;
                }
            }
        }
    }

    // Fallback: try WMI or other methods
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
