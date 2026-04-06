use windows_sys::Win32::System::Registry::{
    RegCreateKeyExW, RegDeleteKeyExW, RegDeleteValueW, RegOpenKeyExW, RegQueryValueExW, RegCloseKey, RegSetValueExW, RegFlushKey,
    HKEY_LOCAL_MACHINE, KEY_WRITE, KEY_READ, KEY_WOW64_64KEY, REG_SZ, HKEY,
};
use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;
use std::thread;
use std::time::Duration;

const IFEO_PATH: &str = r"SOFTWARE\Microsoft\Windows NT\CurrentVersion\Image File Execution Options";
const TARGET_EXES: [&str; 2] = ["stalcraft.exe", "stalcraftw.exe"];
const KEY_WRITE_64: u32 = KEY_WRITE | KEY_WOW64_64KEY;
const KEY_READ_64: u32 = KEY_READ | KEY_WOW64_64KEY;

#[link(name = "shell32")]
extern "system" {
    fn IsUserAnAdmin() -> i32;
}

fn is_admin() -> bool {
    unsafe { IsUserAnAdmin() != 0 }
}

fn to_wide_string(s: &str) -> Vec<u16> {
    OsStr::new(s).encode_wide().chain(Some(0)).collect()
}

fn registry_flush() {
    thread::sleep(Duration::from_millis(300));
}

pub fn install() -> Result<String, String> {
    if !is_admin() {
        return Err("Administrator privileges required. Please run as Administrator.".to_string());
    }

    let self_path = std::env::current_exe()
        .map_err(|e| format!("Failed to get executable path: {}", e))?;
    let self_path_str = self_path.to_string_lossy();

    for target in TARGET_EXES.iter() {
        let key_path = format!("{}\\{}", IFEO_PATH, target);
        let wide_path = to_wide_string(&key_path);
        let wide_debugger = to_wide_string(&self_path_str);

        let mut hkey: HKEY = std::ptr::null_mut();
        let result = unsafe {
            RegCreateKeyExW(
                HKEY_LOCAL_MACHINE,
                wide_path.as_ptr(),
                0,
                std::ptr::null_mut(),
                0,
                KEY_WRITE_64,
                std::ptr::null_mut(),
                &mut hkey,
                std::ptr::null_mut(),
            )
        };

        if result != 0 {
            return Err(format!("Failed to create registry key for {} (error: {})", target, result));
        }

        let wide_value_name = to_wide_string("Debugger");
        let result = unsafe {
            RegSetValueExW(
                hkey,
                wide_value_name.as_ptr(),
                0,
                REG_SZ,
                wide_debugger.as_ptr() as *const u8,
                ((wide_debugger.len() - 1) * 2) as u32,
            )
        };

        unsafe { RegFlushKey(hkey) };
        unsafe { RegCloseKey(hkey) };

        if result != 0 {
            return Err(format!("Failed to set Debugger value for {} (error: {})", target, result));
        }
    }

    registry_flush();

    Ok(format!("IFEO registered for all targets. Debugger = {}", self_path_str))
}

pub fn uninstall() -> Result<String, String> {
    if !is_admin() {
        return Err("Administrator privileges required. Please run as Administrator.".to_string());
    }

    let mut results = Vec::new();
    let mut any_removed = false;

    for target in TARGET_EXES.iter() {
        let key_path = format!("{}\\{}", IFEO_PATH, target);
        let wide_path = to_wide_string(&key_path);

        // First, try to delete the Debugger value
        let mut hkey: HKEY = std::ptr::null_mut();
        let open_result = unsafe {
            RegOpenKeyExW(
                HKEY_LOCAL_MACHINE,
                wide_path.as_ptr(),
                0,
                KEY_WRITE_64,
                &mut hkey,
            )
        };

        if open_result == 0 {
            // Key exists, delete the Debugger value
            let wide_value_name = to_wide_string("Debugger");
            let del_result = unsafe { RegDeleteValueW(hkey, wide_value_name.as_ptr()) };
            
            if del_result == 0 {
                unsafe { RegFlushKey(hkey) };
            }
            unsafe { RegCloseKey(hkey) };

            if del_result == 0 {
                // Value deleted successfully
                results.push(format!("IFEO removed for {}", target));
                any_removed = true;
            } else if del_result == 2 {
                // ERROR_FILE_NOT_FOUND - value doesn't exist, delete the entire key
                let del_key_result = unsafe {
                    RegDeleteKeyExW(
                        HKEY_LOCAL_MACHINE,
                        wide_path.as_ptr(),
                        KEY_WOW64_64KEY,
                        0,
                    )
                };

                if del_key_result == 0 {
                    results.push(format!("IFEO removed for {}", target));
                    any_removed = true;
                } else {
                    results.push(format!("{}: not installed", target));
                }
            } else {
                // Other error
                results.push(format!("{}: delete failed (error {})", target, del_result));
            }
        } else {
            // Key doesn't exist
            results.push(format!("{}: not installed", target));
        }
    }

    if any_removed {
        registry_flush();
    }

    if !any_removed {
        Ok("Not installed".to_string())
    } else {
        Ok(results.join("\n"))
    }
}

pub fn status() -> Result<String, String> {
    let mut results = Vec::new();
    let mut found = false;

    for target in TARGET_EXES.iter() {
        let key_path = format!("{}\\{}", IFEO_PATH, target);
        let wide_path = to_wide_string(&key_path);
        let wide_value_name = to_wide_string("Debugger");

        let mut hkey: HKEY = std::ptr::null_mut();
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
            results.push(format!("{}: not installed", target));
            continue;
        }

        // First, get required buffer size
        let mut buffer_len: u32 = 0;
        let query_result = unsafe {
            RegQueryValueExW(
                hkey,
                wide_value_name.as_ptr(),
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                &mut buffer_len,
            )
        };

        if query_result != 0 || buffer_len == 0 {
            unsafe { RegCloseKey(hkey) };
            results.push(format!("{}: not installed", target));
            continue;
        }

        // Now allocate proper buffer and read value
        let mut buffer: Vec<u16> = vec![0; (buffer_len as usize / 2) + 1];
        let mut actual_len = buffer_len;
        let query_result = unsafe {
            RegQueryValueExW(
                hkey,
                wide_value_name.as_ptr(),
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                buffer.as_mut_ptr() as *mut u8,
                &mut actual_len,
            )
        };

        unsafe { RegCloseKey(hkey) };

        if query_result != 0 {
            results.push(format!("{}: not installed", target));
            continue;
        }

        // Convert wide string to Rust string (exclude null terminator)
        let len = (actual_len as usize / 2).saturating_sub(1);
        let wide_slice = &buffer[..len];
        if let Ok(val) = String::from_utf16(wide_slice) {
            let val = val.trim().to_string();
            if !val.is_empty() {
                results.push(format!("{} -> {}", target, val));
                found = true;
            } else {
                results.push(format!("{}: not installed", target));
            }
        } else {
            results.push(format!("{}: not installed", target));
        }
    }

    if !found {
        results.push("Not installed".to_string());
    }

    Ok(results.join("\n"))
}
