// ifeo.rs — EXBO stalcraft-jvm-optimization installer parity
// Dual-view IFEO (64+32) + fail-closed verify so hooks don't silently drift.

use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;
use std::path::{Path, PathBuf};

use serde::Serialize;

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
    paths::should_inject_jvm(path)
}

#[derive(Debug, Clone, Serialize)]
pub struct IfeoTargetHealth {
    pub target: String,
    pub native64: Option<String>,
    pub wow32: Option<String>,
    pub ok: bool,
    pub detail: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct IfeoHealth {
    pub service_path: Option<String>,
    pub service_present: bool,
    pub targets: Vec<IfeoTargetHealth>,
    pub all_ok: bool,
    pub summary: String,
}

fn to_wide(s: &str) -> Vec<u16> {
    OsStr::new(s).encode_wide().chain(Some(0)).collect()
}

/// Normalize a Debugger / filesystem path for equality checks.
pub fn normalize_path_key(s: &str) -> String {
    let mut t = s.trim().trim_matches('"').to_string();
    if let Some(rest) = t.strip_prefix(r"\\?\") {
        t = rest.to_string();
    } else if let Some(rest) = t.strip_prefix("//?/") {
        t = rest.to_string();
    }
    t = t.replace('/', r"\");
    while t.contains(r"\\") {
        t = t.replace(r"\\", r"\");
    }
    t.to_lowercase()
}

fn strip_extended_prefix(p: PathBuf) -> PathBuf {
    let s = p.to_string_lossy();
    if let Some(rest) = s.strip_prefix(r"\\?\") {
        PathBuf::from(rest)
    } else {
        p
    }
}

/// Absolute, existing path suitable for writing into IFEO Debugger.
fn canonical_service_path(p: &Path) -> Result<PathBuf, String> {
    if !p.is_file() {
        return Err(format!("{} is not a file: {}", SERVICE_NAME, p.display()));
    }
    let meta = std::fs::metadata(p).map_err(|e| format!("stat {}: {}", p.display(), e))?;
    if meta.len() == 0 {
        return Err(format!("{} is empty (0 bytes): {}", SERVICE_NAME, p.display()));
    }
    let canon = std::fs::canonicalize(p)
        .unwrap_or_else(|_| p.to_path_buf());
    Ok(strip_extended_prefix(canon))
}

pub fn service_ready() -> bool {
    resolve_service().is_ok()
}

fn resolve_service() -> Result<PathBuf, String> {
    // Prefer exe-adjacent service.exe (distribution root = wherever the zip was unpacked).
    if let Ok(self_path) = std::env::current_exe() {
        if let Some(dir) = self_path.parent() {
            let beside = dir.join(SERVICE_NAME);
            if beside.is_file() {
                return canonical_service_path(&beside);
            }
        }
    }
    let service = paths::wrapper_home().join(SERVICE_NAME);
    if service.is_file() {
        return canonical_service_path(&service);
    }
    Err(format!(
        "{} not found next to this exe ({}) — keep both exes in the same folder after unpacking wrapper.zip",
        SERVICE_NAME,
        paths::wrapper_home().display()
    ))
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

/// True when registry Debugger value points at the expected service.exe path.
pub fn debugger_matches(expected: &Path, got: &str) -> bool {
    if got.trim().is_empty() {
        return false;
    }
    let got_key = normalize_path_key(got);
    if !got_key.ends_with("service.exe") {
        return false;
    }
    got_key == normalize_path_key(&expected.to_string_lossy())
}

fn target_health(target: &str, expected: Option<&Path>) -> IfeoTargetHealth {
    let native64 = read_debugger_in_view(target, 0);
    let wow32 = read_debugger_in_view(target, KEY_WOW64_32KEY);

    let detail = match expected {
        None => {
            if native64.is_none() && wow32.is_none() {
                "not_installed".to_string()
            } else {
                "service_missing".to_string()
            }
        }
        Some(svc) => {
            let n_ok = native64.as_ref().is_some_and(|d| debugger_matches(svc, d));
            let w_ok = wow32.as_ref().is_some_and(|d| debugger_matches(svc, d));
            if n_ok && w_ok {
                "ok".to_string()
            } else if native64.is_none() && wow32.is_none() {
                "not_installed".to_string()
            } else if native64.is_some() != wow32.is_some() || n_ok != w_ok {
                if !n_ok && !w_ok {
                    let sample = native64.as_deref().or(wow32.as_deref()).unwrap_or("");
                    if !normalize_path_key(sample).ends_with("service.exe") {
                        "wrong_debugger".to_string()
                    } else {
                        "path_mismatch".to_string()
                    }
                } else {
                    "view_split".to_string()
                }
            } else if !n_ok {
                let sample = native64.as_deref().unwrap_or("");
                if !normalize_path_key(sample).ends_with("service.exe") {
                    "wrong_debugger".to_string()
                } else {
                    "path_mismatch".to_string()
                }
            } else {
                "not_ok".to_string()
            }
        }
    };

    let ok = detail == "ok";
    IfeoTargetHealth {
        target: target.to_string(),
        native64,
        wow32,
        ok,
        detail,
    }
}

/// Full dual-view health check against live adjacent service.exe.
pub fn health() -> IfeoHealth {
    let service = resolve_service().ok();
    let service_present = service.is_some();
    let expected = service.as_deref();

    let targets: Vec<IfeoTargetHealth> = IFEO_TARGETS
        .iter()
        .map(|t| target_health(t, expected))
        .collect();

    let all_ok = service_present && targets.iter().all(|t| t.ok);

    let mut lines = Vec::new();
    for t in &targets {
        let sample = t
            .native64
            .as_deref()
            .or(t.wow32.as_deref())
            .unwrap_or("");
        let redacted = if sample.is_empty() {
            String::new()
        } else {
            log::redact_path(sample)
        };
        match t.detail.as_str() {
            "ok" => lines.push(format!("{}: ok (Debugger={})", t.target, redacted)),
            "not_installed" => lines.push(format!("{}: not installed", t.target)),
            "wrong_debugger" => lines.push(format!(
                "{}: wrong debugger (expected service.exe, got {})",
                t.target, redacted
            )),
            "path_mismatch" => lines.push(format!(
                "{}: installed but path moved — reinstall/repair IFEO (Debugger={})",
                t.target, redacted
            )),
            "view_split" => lines.push(format!(
                "{}: view split (64-bit/32-bit Debugger mismatch) — reinstall/repair",
                t.target
            )),
            "service_missing" => lines.push(format!(
                "{}: Debugger set but service.exe MISSING locally",
                t.target
            )),
            other => lines.push(format!("{}: {}", t.target, other)),
        }
    }

    if let Some(ref path) = service {
        lines.push(format!(
            "service.exe: present ({})",
            log::redact_path(&path.to_string_lossy())
        ));
    } else {
        lines.push(
            "service.exe: MISSING — keep service.exe next to stalcraft-jvm-wrapper.exe after unpack"
                .to_string(),
        );
    }

    if all_ok {
        lines.push("verify: all_ok (6 targets × 2 registry views)".to_string());
    } else {
        lines.push("verify: FAILED — run INSTALL or REPAIR".to_string());
    }

    IfeoHealth {
        service_path: service.map(|p| p.to_string_lossy().into_owned()),
        service_present,
        targets,
        all_ok,
        summary: lines.join("\n"),
    }
}

/// Mandatory verification — Err when IFEO is not fully healthy.
pub fn verify() -> Result<IfeoHealth, String> {
    let h = health();
    if h.all_ok {
        Ok(h)
    } else {
        Err(h.summary)
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
        "IFEO install_start targets={} debugger={}",
        IFEO_TARGETS.len(),
        log::redact_path(&service.to_string_lossy())
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

    let h = health();
    if !h.all_ok {
        log::append_wrapper_log_line("IFEO install_verify_failed");
        return Err(format!(
            "IFEO install wrote keys but mandatory verify failed:\n{}",
            h.summary
        ));
    }

    let msg = format!(
        "IFEO installed for {} (64-bit + 32-bit views). Debugger = \"{}\"",
        IFEO_TARGETS.join(", "),
        service.display()
    );
    log::append_wrapper_log_line("IFEO install_ok");
    Ok(msg)
}

/// Reinstall IFEO if health check fails (path moved / view split / missing).
pub fn repair() -> Result<String, String> {
    if !is_admin() {
        return Err("Administrator privileges required for IFEO repair".to_string());
    }
    let h = health();
    if h.all_ok {
        log::append_wrapper_log_line("IFEO repair_skip already_ok");
        return Ok(format!("IFEO already healthy.\n{}", h.summary));
    }
    log::append_wrapper_log_line("IFEO repair_start");
    let msg = install(None)?;
    log::append_wrapper_log_line("IFEO repair_ok");
    Ok(format!("IFEO repaired.\n{msg}"))
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

fn verify_debugger_set(target: &str, service: &PathBuf) -> Result<(), String> {
    for (view, flag) in [("64-bit", 0u32), ("32-bit", KEY_WOW64_32KEY)] {
        let got = read_debugger_in_view(target, flag).unwrap_or_default();
        if !debugger_matches(service, &got) {
            return Err(format!(
                "IFEO verify failed for {} ({view}): {:?}",
                target, got
            ));
        }
    }
    Ok(())
}

pub fn status() -> Result<String, String> {
    Ok(health().summary)
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
    fn normalize_strips_quotes_slashes_and_extended() {
        assert_eq!(
            normalize_path_key(r#""C:\Foo\service.exe""#),
            r"c:\foo\service.exe"
        );
        assert_eq!(
            normalize_path_key(r"\\?\C:\Foo/service.exe"),
            r"c:\foo\service.exe"
        );
        assert_eq!(
            normalize_path_key(r"C:/Foo\\service.exe"),
            r"c:\foo\service.exe"
        );
    }

    #[test]
    fn debugger_matches_quoted_and_cased() {
        let p = PathBuf::from(r"C:\Apps\wrapper\service.exe");
        assert!(debugger_matches(
            &p,
            r#""C:\Apps\wrapper\service.exe""#
        ));
        assert!(debugger_matches(
            &p,
            r"C:\APPS\WRAPPER\SERVICE.EXE"
        ));
        assert!(!debugger_matches(
            &p,
            r#""C:\Other\service.exe""#
        ));
        assert!(!debugger_matches(&p, r"C:\Apps\wrapper\other.exe"));
        assert!(!debugger_matches(&p, ""));
    }

    #[test]
    fn game_java_injects() {
        let p = r"C:\Users\me\AppData\Roaming\EXBO\runtime\stalcraft\win64\java\bin\javaw.exe";
        assert!(should_inject_jvm(p));
    }

    #[test]
    fn runtime_stalzone_injects() {
        let p = r"C:\Users\me\AppData\Roaming\EXBO\runtime\stalcraft\win64\java\bin\stalzone.exe";
        assert!(should_inject_jvm(p));
    }

    #[test]
    fn root_stalzone_injects() {
        let p = r"C:\Users\me\AppData\Roaming\EXBO\stalzone.exe";
        assert!(should_inject_jvm(p));
    }

    #[test]
    fn steam_stalcraftw_injects() {
        let p = r"D:\Steam\steamapps\common\stalcraft\stalcraftw.exe";
        assert!(should_inject_jvm(p));
    }

    #[test]
    fn system_java_passthrough() {
        assert!(!should_inject_jvm(r"C:\Program Files\Java\bin\java.exe"));
    }
}
