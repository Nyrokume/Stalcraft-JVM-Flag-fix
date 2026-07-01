// ifeo.rs — порт installer.go (stalart* + stalcraft* client targets)

use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;
use std::path::PathBuf;

use crate::log;

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
    fn RegDeleteValueW(hKey: isize, lpValueName: *const u16) -> i32;
    fn RegCloseKey(hKey: isize) -> i32;
}

#[link(name = "shell32")]
extern "system" {
    fn IsUserAnAdmin() -> i32;
}

const HKEY_LOCAL_MACHINE: isize = -2147483648i64 as isize;
const KEY_ALL_ACCESS: u32 = 0xF003F;
const KEY_SET_VALUE: u32 = 0x0002;
const KEY_QUERY_VALUE: u32 = 0x0001;
const KEY_WOW64_64KEY: u32 = 0x0100;
const KEY_READ_WRITE: u32 = KEY_SET_VALUE | KEY_WOW64_64KEY;
const KEY_READ_64: u32 = KEY_QUERY_VALUE | KEY_WOW64_64KEY;
const REG_SZ: u32 = 1;
const ERROR_SUCCESS: i32 = 0;
const ERROR_FILE_NOT_FOUND: i32 = 2;

const IFEO_PATH: &str =
    r"SOFTWARE\Microsoft\Windows NT\CurrentVersion\Image File Execution Options";

pub const IFEO_TARGETS: &[&str] = &[
    "stalart.exe",
    "stalartw.exe",
    "stalcraft.exe",
    "stalcraftw.exe",
];

fn path_ends_with_executable(path_lower: &str, name: &str) -> bool {
    if path_lower.len() < name.len() || !path_lower.ends_with(name) {
        return false;
    }
    if path_lower.len() == name.len() {
        return true;
    }
    matches!(
        path_lower.as_bytes()[path_lower.len() - name.len() - 1],
        b'\\' | b'/'
    )
}

pub fn is_ifeo_debugger_invocation(image_path: &str) -> bool {
    let lower = image_path.to_lowercase();
    IFEO_TARGETS
        .iter()
        .any(|&exe| path_ends_with_executable(&lower, exe))
}

#[derive(Debug, Clone)]
pub struct IfeoEntry {
    pub target: String,
    pub installed: bool,
    pub debugger: String,
}

fn to_wide(s: &str) -> Vec<u16> {
    OsStr::new(s).encode_wide().chain(Some(0)).collect()
}

fn resolve_wrapper() -> Result<PathBuf, String> {
    let self_path = std::env::current_exe().map_err(|e| format!("resolve self: {}", e))?;
    if !self_path.is_file() {
        return Err("wrapper executable path invalid".to_string());
    }
    Ok(self_path)
}

fn set_debugger(target: &str, debugger: &PathBuf) -> Result<(), String> {
    let subkey = format!(r"{}\{}", IFEO_PATH, target);
    let wide_subkey = to_wide(&subkey);
    let wide_debugger = to_wide("Debugger");
    let debugger_str = format!("\"{}\"", debugger.display());
    let wide_value = to_wide(&debugger_str);

    let mut hkey: isize = 0;
    let r = unsafe {
        RegCreateKeyExW(
            HKEY_LOCAL_MACHINE,
            wide_subkey.as_ptr(),
            0,
            std::ptr::null(),
            0,
            KEY_ALL_ACCESS | KEY_WOW64_64KEY,
            std::ptr::null(),
            &mut hkey,
            std::ptr::null_mut(),
        )
    };
    if r != ERROR_SUCCESS {
        return Err(format!("create IFEO key for {}: {}", target, r));
    }

    let data = unsafe {
        std::slice::from_raw_parts(wide_value.as_ptr() as *const u8, wide_value.len() * 2)
    };
    let r = unsafe {
        RegSetValueExW(
            hkey,
            wide_debugger.as_ptr(),
            0,
            REG_SZ,
            data.as_ptr(),
            data.len() as u32,
        )
    };
    unsafe { RegCloseKey(hkey) };
    if r != ERROR_SUCCESS {
        return Err(format!("set Debugger for {}: {}", target, r));
    }
    Ok(())
}

fn clear_debugger(target: &str) -> Result<(), String> {
    let subkey = format!(r"{}\{}", IFEO_PATH, target);
    let wide_subkey = to_wide(&subkey);
    let wide_debugger = to_wide("Debugger");

    let mut hkey: isize = 0;
    let r = unsafe {
        RegOpenKeyExW(
            HKEY_LOCAL_MACHINE,
            wide_subkey.as_ptr(),
            0,
            KEY_READ_WRITE,
            &mut hkey,
        )
    };
    if r != ERROR_SUCCESS {
        return Err(format!("open IFEO key for {}: {}", target, r));
    }

    let r = unsafe { RegDeleteValueW(hkey, wide_debugger.as_ptr()) };
    unsafe { RegCloseKey(hkey) };
    if r != ERROR_SUCCESS && r != ERROR_FILE_NOT_FOUND {
        return Err(format!("delete Debugger for {}: {}", target, r));
    }
    Ok(())
}

fn status_for(target: &str) -> IfeoEntry {
    let subkey = format!(r"{}\{}", IFEO_PATH, target);
    let wide_subkey = to_wide(&subkey);
    let wide_debugger = to_wide("Debugger");

    let mut hkey: isize = 0;
    let r = unsafe {
        RegOpenKeyExW(
            HKEY_LOCAL_MACHINE,
            wide_subkey.as_ptr(),
            0,
            KEY_READ_64,
            &mut hkey,
        )
    };
    if r != ERROR_SUCCESS {
        return IfeoEntry {
            target: target.to_string(),
            installed: false,
            debugger: String::new(),
        };
    }

    let mut buf_len: u32 = 0;
    let q = unsafe {
        RegQueryValueExW(
            hkey,
            wide_debugger.as_ptr(),
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            &mut buf_len,
        )
    };
    if q != ERROR_SUCCESS || buf_len == 0 {
        unsafe { RegCloseKey(hkey) };
        return IfeoEntry {
            target: target.to_string(),
            installed: false,
            debugger: String::new(),
        };
    }

    let mut buf = vec![0u8; buf_len as usize + 2];
    let mut actual = buf_len;
    let q = unsafe {
        RegQueryValueExW(
            hkey,
            wide_debugger.as_ptr(),
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            buf.as_mut_ptr(),
            &mut actual,
        )
    };
    unsafe { RegCloseKey(hkey) };
    if q != ERROR_SUCCESS {
        return IfeoEntry {
            target: target.to_string(),
            installed: false,
            debugger: String::new(),
        };
    }

    let wchars = actual as usize / 2;
    let wide_slice: Vec<u16> = (0..wchars)
        .map(|i| u16::from_le_bytes([buf[i * 2], buf[i * 2 + 1]]))
        .collect();
    let end = wide_slice.iter().position(|&c| c == 0).unwrap_or(wchars);
    let val = String::from_utf16(&wide_slice[..end]).unwrap_or_default();

    IfeoEntry {
        target: target.to_string(),
        installed: !val.is_empty(),
        debugger: val,
    }
}

pub fn is_admin() -> bool {
    unsafe { IsUserAnAdmin() != 0 }
}

/// Install IFEO for all client targets. `override_path` ignored (kept for CLI compat).
pub fn install(_override_path: Option<&str>) -> Result<String, String> {
    if !is_admin() {
        return Err("Administrator privileges required for IFEO install".to_string());
    }

    let wrapper = resolve_wrapper()?;
    log::append_wrapper_log_line(&format!(
        "IFEO install_start targets={}",
        IFEO_TARGETS.len()
    ));

    for target in IFEO_TARGETS {
        set_debugger(target, &wrapper)?;
        log::append_wrapper_log_line(&format!(
            "IFEO target_set target={} debugger={}",
            target,
            log::redact_path(&wrapper.to_string_lossy())
        ));
    }

    let msg = format!(
        "IFEO installed for {} targets. Debugger = \"{}\"",
        IFEO_TARGETS.len(),
        wrapper.display()
    );
    log::append_wrapper_log_line("IFEO install_ok");
    Ok(msg)
}

pub fn uninstall() -> Result<String, String> {
    if !is_admin() {
        return Err("Administrator privileges required for IFEO uninstall".to_string());
    }

    let mut errors = Vec::new();
    for target in IFEO_TARGETS {
        if let Err(e) = clear_debugger(target) {
            errors.push(format!("{}: {}", target, e));
        }
    }

    log::append_wrapper_log_line(&format!(
        "IFEO uninstall_done errors={}",
        errors.len()
    ));

    if errors.is_empty() {
        Ok("IFEO uninstalled for all targets.".to_string())
    } else {
        Err(errors.join("; "))
    }
}

pub fn status() -> Result<String, String> {
    let entries: Vec<IfeoEntry> = IFEO_TARGETS.iter().map(|t| status_for(t)).collect();
    let mut lines = Vec::new();
    for e in &entries {
        if e.installed {
            lines.push(format!(
                "{}: installed (Debugger={})",
                e.target,
                log::redact_path(&e.debugger)
            ));
        } else {
            lines.push(format!("{}: not installed", e.target));
        }
    }
    Ok(lines.join("\n"))
}

#[allow(dead_code)]
pub fn status_entries() -> Vec<IfeoEntry> {
    IFEO_TARGETS.iter().map(|t| status_for(t)).collect()
}
