// ifeo.rs — EXBO stalcraft-jvm-optimization installer parity

use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;
use std::path::PathBuf;

use crate::{config, log, paths, system};

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
const KEY_WOW64_32KEY: u32 = 0x0200;
const KEY_READ_WRITE_32: u32 = KEY_SET_VALUE | KEY_WOW64_32KEY;
const KEY_READ_32: u32 = KEY_QUERY_VALUE | KEY_WOW64_32KEY;
const REG_SZ: u32 = 1;
const ERROR_SUCCESS: i32 = 0;
const ERROR_FILE_NOT_FOUND: i32 = 2;

const IFEO_PATH: &str =
    r"SOFTWARE\Microsoft\Windows NT\CurrentVersion\Image File Execution Options";
const SERVICE_NAME: &str = "service.exe";

/// IFEO game launchers + runtime JVM (win64/java/bin).
/// stalzone* registered ahead of stalcraft* (EXBO v1.1.2 rebrand parity).
pub const IFEO_TARGETS: &[&str] = &[
    "stalzone.exe",
    "stalzonew.exe",
    "stalcraft.exe",
    "stalcraftw.exe",
    "java.exe",
    "javaw.exe",
];

const LEGACY_TARGETS: &[&str] = &["stalart.exe", "stalartw.exe"];

pub fn should_inject_jvm(path: &str) -> bool {
    if paths::is_launcher_binary(path) {
        return true;
    }
    paths::is_game_java(path)
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

pub fn service_ready() -> bool {
    resolve_service().is_ok()
}

fn resolve_service() -> Result<PathBuf, String> {
    let service = paths::wrapper_home().join(SERVICE_NAME);
    if !service.is_file() {
        if let Ok(self_path) = std::env::current_exe() {
            if let Some(dir) = self_path.parent() {
                let alt = dir.join(SERVICE_NAME);
                if alt.is_file() {
                    return Ok(alt);
                }
            }
        }
        return Err(format!(
            "{} not found in {} (copy both exes from release)",
            SERVICE_NAME,
            paths::wrapper_home().display()
        ));
    }
    Ok(service)
}

const KEY_READ_WRITE_NATIVE: u32 = KEY_SET_VALUE;
const KEY_READ_NATIVE: u32 = KEY_QUERY_VALUE;

fn set_debugger_in_view(target: &str, debugger: &PathBuf, wow64: u32) -> Result<(), String> {
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
            KEY_ALL_ACCESS | wow64,
            std::ptr::null(),
            &mut hkey,
            std::ptr::null_mut(),
        )
    };
    if r != ERROR_SUCCESS {
        let view = if wow64 == KEY_WOW64_32KEY {
            "32-bit"
        } else {
            "64-bit"
        };
        return Err(format!("create IFEO key for {} ({view}): {}", target, r));
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
        let view = if wow64 == KEY_WOW64_32KEY {
            "32-bit"
        } else {
            "64-bit"
        };
        return Err(format!("set Debugger for {} ({view}): {}", target, r));
    }
    Ok(())
}

fn set_debugger(target: &str, debugger: &PathBuf) -> Result<(), String> {
    // ponytail: native 64-bit view = no WOW64 flag (matches EXBO Go installer)
    set_debugger_in_view(target, debugger, 0)?;
    set_debugger_in_view(target, debugger, KEY_WOW64_32KEY)?;
    Ok(())
}

fn clear_debugger_in_view(target: &str, wow64: u32) -> Result<(), String> {
    let subkey = format!(r"{}\{}", IFEO_PATH, target);
    let wide_subkey = to_wide(&subkey);
    let wide_debugger = to_wide("Debugger");
    let access = if wow64 == KEY_WOW64_32KEY {
        KEY_READ_WRITE_32
    } else {
        KEY_READ_WRITE_NATIVE
    };

    let mut hkey: isize = 0;
    let r = unsafe {
        RegOpenKeyExW(
            HKEY_LOCAL_MACHINE,
            wide_subkey.as_ptr(),
            0,
            access,
            &mut hkey,
        )
    };
    if r != ERROR_SUCCESS {
        return Ok(());
    }

    let r = unsafe { RegDeleteValueW(hkey, wide_debugger.as_ptr()) };
    unsafe { RegCloseKey(hkey) };
    if r != ERROR_SUCCESS && r != ERROR_FILE_NOT_FOUND {
        let view = if wow64 == KEY_WOW64_32KEY {
            "32-bit"
        } else {
            "64-bit"
        };
        return Err(format!("delete Debugger for {} ({view}): {}", target, r));
    }
    Ok(())
}

fn clear_debugger(target: &str) -> Result<(), String> {
    clear_debugger_in_view(target, 0)?;
    clear_debugger_in_view(target, KEY_WOW64_32KEY)?;
    Ok(())
}

fn read_debugger_in_view(target: &str, wow64: u32) -> Option<String> {
    let subkey = format!(r"{}\{}", IFEO_PATH, target);
    let wide_subkey = to_wide(&subkey);
    let wide_debugger = to_wide("Debugger");
    let access = if wow64 == KEY_WOW64_32KEY {
        KEY_READ_32
    } else {
        KEY_READ_NATIVE
    };

    let mut hkey: isize = 0;
    let r = unsafe {
        RegOpenKeyExW(
            HKEY_LOCAL_MACHINE,
            wide_subkey.as_ptr(),
            0,
            access,
            &mut hkey,
        )
    };
    if r != ERROR_SUCCESS {
        return None;
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
        return None;
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
        return None;
    }

    let wchars = (actual as usize).saturating_div(2);
    let wide_slice = unsafe { std::slice::from_raw_parts(buf.as_ptr() as *const u16, wchars) };
    let end = wide_slice.iter().position(|&c| c == 0).unwrap_or(wchars);
    String::from_utf16(&wide_slice[..end])
        .ok()
        .filter(|s| !s.is_empty())
}

fn status_for(target: &str) -> IfeoEntry {
    let debugger = read_debugger_in_view(target, 0)
        .or_else(|| read_debugger_in_view(target, KEY_WOW64_32KEY))
        .unwrap_or_default();

    IfeoEntry {
        target: target.to_string(),
        installed: !debugger.is_empty(),
        debugger,
    }
}

pub fn is_admin() -> bool {
    unsafe { IsUserAnAdmin() != 0 }
}

pub fn install(_override_path: Option<&str>) -> Result<String, String> {
    if !is_admin() {
        return Err("Administrator privileges required for IFEO install".to_string());
    }

    let service = resolve_service()?;
    let sys = system::detect_system();
    config::ensure(&sys).map_err(|e| format!("config ensure: {}", e))?;

    log::append_wrapper_log_line(&format!(
        "IFEO install_start targets={} debugger=service.exe",
        IFEO_TARGETS.len()
    ));

    for target in LEGACY_TARGETS {
        let _ = clear_debugger(target);
    }

    for target in IFEO_TARGETS {
        set_debugger(target, &service)?;
        verify_debugger_set(target, &service)?;
        log::append_wrapper_log_line(&format!(
            "IFEO target_set target={} debugger={} views=64+32",
            target,
            log::redact_path(&service.to_string_lossy())
        ));
    }

    let msg = format!(
        "IFEO installed for stalzone.exe, stalzonew.exe, stalcraft.exe, stalcraftw.exe. Debugger = \"{}\"",
        service.display()
    );
    log::append_wrapper_log_line("IFEO install_ok");
    Ok(msg)
}

pub fn uninstall() -> Result<String, String> {
    if !is_admin() {
        return Err("Administrator privileges required for IFEO uninstall".to_string());
    }

    let mut errors = Vec::new();
    for target in IFEO_TARGETS.iter().chain(LEGACY_TARGETS.iter()) {
        if let Err(e) = clear_debugger(target) {
            errors.push(format!("{}: {}", target, e));
        }
    }

    log::append_wrapper_log_line(&format!(
        "IFEO uninstall_done errors={}",
        errors.len()
    ));

    if errors.is_empty() {
        Ok("IFEO uninstalled.".to_string())
    } else {
        Err(errors.join("; "))
    }
}

fn normalize_debugger_path(s: &str) -> String {
    s.trim().trim_matches('"').to_lowercase()
}

fn verify_debugger_set(target: &str, service: &PathBuf) -> Result<(), String> {
    let want = normalize_debugger_path(&service.to_string_lossy());
    for (view, flag) in [("64-bit", 0u32), ("32-bit", KEY_WOW64_32KEY)] {
        let got = read_debugger_in_view(target, flag).unwrap_or_default();
        if normalize_debugger_path(&got) != want {
            return Err(format!(
                "IFEO verify failed for {} ({view}): {:?}",
                target, got
            ));
        }
    }
    Ok(())
}

fn debugger_points_to_service(debugger: &str) -> bool {
    normalize_debugger_path(debugger).ends_with("service.exe")
}

pub fn status() -> Result<String, String> {
    let entries: Vec<IfeoEntry> = IFEO_TARGETS.iter().map(|t| status_for(t)).collect();
    let mut lines = Vec::new();
    for e in &entries {
        if e.installed && debugger_points_to_service(&e.debugger) {
            lines.push(format!(
                "{}: ok (Debugger={})",
                e.target,
                log::redact_path(&e.debugger)
            ));
        } else if e.installed {
            lines.push(format!(
                "{}: wrong debugger (expected service.exe, got {})",
                e.target,
                log::redact_path(&e.debugger)
            ));
        } else {
            lines.push(format!("{}: not installed", e.target));
        }
    }
    if service_ready() {
        lines.push("service.exe: present".to_string());
    } else {
        lines.push("service.exe: MISSING — copy next to stalcraft-jvm-wrapper.exe".to_string());
    }
    Ok(lines.join("\n"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ifeo_targets_include_runtime_launchers() {
        for name in [
            "stalcraft.exe",
            "stalcraftw.exe",
            "stalzone.exe",
            "stalzonew.exe",
            "java.exe",
            "javaw.exe",
        ] {
            assert!(IFEO_TARGETS.contains(&name));
        }
    }

    #[test]
    fn game_java_injects() {
        let p = r"C:\Users\me\AppData\Roaming\EXBO\runtime\stalcraft\win64\java\bin\javaw.exe";
        assert!(should_inject_jvm(p));
    }

    #[test]
    fn system_java_passthrough() {
        assert!(!should_inject_jvm(r"C:\Program Files\Java\bin\java.exe"));
    }
}
