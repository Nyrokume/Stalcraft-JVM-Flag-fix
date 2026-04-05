use windows_sys::Win32::System::Registry::{
    RegCreateKeyExW, RegDeleteValueW, RegOpenKeyExW, RegQueryValueExW, RegCloseKey, RegSetValueExW,
    HKEY_LOCAL_MACHINE, KEY_WRITE, KEY_READ, REG_SZ, HKEY,
};
use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;

const IFEO_PATH: &str = r"SOFTWARE\Microsoft\Windows NT\CurrentVersion\Image File Execution Options";
const TARGET_EXES: [&str; 2] = ["stalcraft.exe", "stalcraftw.exe"];

fn to_wide_string(s: &str) -> Vec<u16> {
    OsStr::new(s).encode_wide().chain(Some(0)).collect()
}

pub fn install() -> Result<String, String> {
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
                KEY_WRITE,
                std::ptr::null_mut(),
                &mut hkey,
                std::ptr::null_mut(),
            )
        };

        if result != 0 {
            return Err(format!("Failed to create registry key for {} (run as admin)", target));
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

        unsafe { RegCloseKey(hkey) };

        if result != 0 {
            return Err(format!("Failed to set Debugger value for {}", target));
        }
    }

    Ok(format!("IFEO registered for all targets. Debugger = {}", self_path_str))
}

pub fn uninstall() -> Result<String, String> {
    let mut results = Vec::new();

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
                KEY_WRITE,
                &mut hkey,
            )
        };

        if result != 0 {
            results.push(format!("{}: not installed", target));
            continue;
        }

        let del_result = unsafe { RegDeleteValueW(hkey, wide_value_name.as_ptr()) };
        unsafe { RegCloseKey(hkey) };

        if del_result == 0 {
            results.push(format!("IFEO removed for {}", target));
        } else {
            results.push(format!("{}: not installed", target));
        }
    }

    Ok(results.join("\n"))
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
                KEY_READ,
                &mut hkey,
            )
        };

        if result != 0 {
            results.push(format!("{}: not installed", target));
            continue;
        }

        let mut buffer: [u16; 1024] = [0; 1024];
        let mut buffer_len = (buffer.len() * 2) as u32;
        let query_result = unsafe {
            RegQueryValueExW(
                hkey,
                wide_value_name.as_ptr(),
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                buffer.as_mut_ptr() as *mut u8,
                &mut buffer_len,
            )
        };

        unsafe { RegCloseKey(hkey) };

        if query_result != 0 {
            results.push(format!("{}: not installed", target));
            continue;
        }

        // Convert wide string to Rust string
        let len = buffer_len as usize / 2;
        let wide_slice = &buffer[..len];
        if let Ok(val) = String::from_utf16(wide_slice) {
            results.push(format!("{} -> {}", target, val));
            found = true;
        }
    }

    if !found {
        results.push("Not installed".to_string());
    }

    Ok(results.join("\n"))
}
