// ifeo.rs — полный порт installer.go
// IFEO (Image File Execution Options) — установка/удаление/статус перехвата.
// Debugger = "\"<path to service.exe>\"" как в setDebugger() из Go.

use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;

// ─── Windows Registry API ─────────────────────────────────────────────────────

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
    fn RegFlushKey(hKey: isize) -> i32;
    fn RegCloseKey(hKey: isize) -> i32;
}

#[link(name = "shell32")]
extern "system" {
    fn IsUserAnAdmin() -> i32;
}

// ─── Константы ────────────────────────────────────────────────────────────────

const HKEY_LOCAL_MACHINE: isize = -2147483648i64 as isize;
const KEY_ALL_ACCESS: u32 = 0xF003F;
const KEY_SET_VALUE: u32 = 0x0002;
const KEY_QUERY_VALUE: u32 = 0x0001;
const KEY_WOW64_64KEY: u32 = 0x0100;
const KEY_READ_WRITE: u32 = KEY_SET_VALUE | KEY_WOW64_64KEY;
const KEY_READ_64: u32 = KEY_QUERY_VALUE | KEY_WOW64_64KEY;
const REG_SZ: u32 = 1;

const IFEO_PATH: &str =
    r"SOFTWARE\Microsoft\Windows NT\CurrentVersion\Image File Execution Options";

/// Таргеты — точный аналог Targets из installer.go
const TARGETS: &[&str] = &["stalcraft.exe", "stalcraftw.exe"];

// ─── Утилиты ─────────────────────────────────────────────────────────────────

fn to_wide(s: &str) -> Vec<u16> {
    OsStr::new(s).encode_wide().chain(Some(0)).collect()
}

fn is_admin() -> bool {
    unsafe { IsUserAnAdmin() != 0 }
}

/// resolveService — путь к текущему exe (Tauri single exe acting as both CLI and service)
/// В режиме Tauri один exe работает и как GUI, и как debugger (service).
fn resolve_service() -> Result<String, String> {
    let self_path = std::env::current_exe().map_err(|e| format!("resolve self: {}", e))?;
    Ok(self_path.to_string_lossy().to_string())
}

// ─── setDebugger — точный порт setDebugger() из installer.go ─────────────────

fn set_debugger(target: &str, debugger: &str) -> Result<(), String> {
    let key_path = format!("{}\\{}", IFEO_PATH, target);
    let wide_path = to_wide(&key_path);
    // В Go: key.SetStringValue("Debugger", `"` + debugger + `"`)
    // Т.е. значение обёрнуто в кавычки
    let debugger_quoted = format!("\"{}\"", debugger);
    let wide_debugger = to_wide(&debugger_quoted);

    let mut hkey: isize = 0;
    let r = unsafe {
        RegCreateKeyExW(
            HKEY_LOCAL_MACHINE,
            wide_path.as_ptr(),
            0,
            std::ptr::null(),
            0,
            KEY_ALL_ACCESS | KEY_WOW64_64KEY,
            std::ptr::null(),
            &mut hkey,
            std::ptr::null_mut(),
        )
    };
    if r != 0 {
        return Err(format!("create IFEO key for {} (error: {})", target, r));
    }

    let wide_value = to_wide("Debugger");
    // Данные: REG_SZ без нулевого терминатора (как Go: len-1)
    let data_bytes = unsafe {
        std::slice::from_raw_parts(
            wide_debugger.as_ptr() as *const u8,
            (wide_debugger.len() - 1) * 2,
        )
    };
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
    unsafe {
        RegFlushKey(hkey);
        RegCloseKey(hkey);
    }

    if r != 0 {
        return Err(format!("set Debugger for {} (error: {})", target, r));
    }
    Ok(())
}

// ─── clearDebugger — порт clearDebugger() из installer.go ────────────────────

fn clear_debugger(target: &str) -> Result<(), String> {
    let key_path = format!("{}\\{}", IFEO_PATH, target);
    let wide_path = to_wide(&key_path);

    let mut hkey: isize = 0;
    let open = unsafe {
        RegOpenKeyExW(
            HKEY_LOCAL_MACHINE,
            wide_path.as_ptr(),
            0,
            KEY_READ_WRITE,
            &mut hkey,
        )
    };

    if open != 0 {
        return Err(format!("open IFEO key for {}: {}", target, open));
    }

    let wide_value = to_wide("Debugger");
    let del = unsafe { RegDeleteValueW(hkey, wide_value.as_ptr()) };
    if del == 0 {
        unsafe { RegFlushKey(hkey) };
    }
    unsafe { RegCloseKey(hkey) };

    if del != 0 {
        return Err(format!("delete Debugger for {}: {}", target, del));
    }
    Ok(())
}

// ─── Публичные функции ────────────────────────────────────────────────────────

/// install() — точный порт Install() из installer.go
pub fn install() -> Result<String, String> {
    if !is_admin() {
        return Err("Administrator privileges required. Please run as Administrator.".to_string());
    }

    let service = resolve_service()?;

    for target in TARGETS {
        set_debugger(target, &service)
            .map_err(|e| format!("installer target failed ({}): {}", target, e))?;
        eprintln!("[installer] set {} -> {}", target, service);
    }
    eprintln!("[installer] done");
    Ok(format!(
        "IFEO registered for all targets. Debugger = \"{}\"",
        service
    ))
}

/// uninstall() — точный порт Uninstall() из installer.go
pub fn uninstall() -> Result<String, String> {
    if !is_admin() {
        return Err("Administrator privileges required. Please run as Administrator.".to_string());
    }

    let mut results = Vec::new();
    let mut any_removed = false;
    let mut errs = Vec::new();

    for target in TARGETS {
        match clear_debugger(target) {
            Ok(()) => {
                results.push(format!("IFEO removed for {}", target));
                any_removed = true;
                eprintln!("[installer] cleared {}", target);
            }
            Err(e) => {
                results.push(format!("{}: not installed or failed ({})", target, e));
                errs.push(e);
            }
        }
    }

    if !errs.is_empty() && !any_removed {
        return Ok("Not installed".to_string());
    }
    Ok(results.join("\n"))
}

/// status() — точный порт Status() из installer.go
pub fn status() -> Result<String, String> {
    let mut results = Vec::new();
    let mut found = false;

    for target in TARGETS {
        let key_path = format!("{}\\{}", IFEO_PATH, target);
        let wide_path = to_wide(&key_path);
        let wide_value = to_wide("Debugger");

        let mut hkey: isize = 0;
        let r = unsafe {
            RegOpenKeyExW(
                HKEY_LOCAL_MACHINE,
                wide_path.as_ptr(),
                0,
                KEY_READ_64,
                &mut hkey,
            )
        };
        if r != 0 {
            results.push(format!("{}: not installed", target));
            continue;
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
            results.push(format!("{}: not installed", target));
            continue;
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
            results.push(format!("{}: not installed", target));
            continue;
        }

        let wchars = actual as usize / 2;
        let wide_slice: Vec<u16> = (0..wchars)
            .map(|i| u16::from_le_bytes([buf[i * 2], buf[i * 2 + 1]]))
            .collect();
        let end = wide_slice.iter().position(|&c| c == 0).unwrap_or(wchars);
        match String::from_utf16(&wide_slice[..end]) {
            Ok(val) => {
                let val = val.trim().to_string();
                if val.is_empty() {
                    results.push(format!("{}: not installed", target));
                } else {
                    results.push(format!("{} -> {}", target, val));
                    found = true;
                }
            }
            Err(_) => {
                results.push(format!("{}: not installed", target));
            }
        }
    }

    if !found {
        results.push("Not installed".to_string());
    }
    Ok(results.join("\n"))
}
