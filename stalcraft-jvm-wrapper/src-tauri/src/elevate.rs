// elevate.rs — UAC re-launch (EXBO internal/elevate/elevate.go)

#![allow(non_snake_case)]

use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;

#[link(name = "shell32")]
extern "system" {
    fn ShellExecuteExW(pExecInfo: *mut SHELLEXECUTEINFOW) -> i32;
}

#[link(name = "kernel32")]
extern "system" {
    fn WaitForSingleObject(hHandle: isize, dwMilliseconds: u32) -> u32;
    fn GetExitCodeProcess(hProcess: isize, lpExitCode: *mut u32) -> i32;
    fn CloseHandle(hObject: isize) -> i32;
}

const SEE_MASK_NOCLOSEPROCESS: u32 = 0x00000040;
const SEE_MASK_NOASYNC: u32 = 0x00000100;
const INFINITE: u32 = 0xFFFFFFFF;

#[repr(C)]
struct SHELLEXECUTEINFOW {
    cbSize: u32,
    fMask: u32,
    hwnd: isize,
    lpVerb: *const u16,
    lpFile: *const u16,
    lpParameters: *const u16,
    lpDirectory: *const u16,
    nShow: i32,
    hInstApp: usize,
    lpIDList: *const std::ffi::c_void,
    lpClass: *const u16,
    hkeyClass: isize,
    dwHotKey: u32,
    hIcon: isize,
    hProcess: isize,
}

fn to_wide(s: &str) -> Vec<u16> {
    OsStr::new(s).encode_wide().chain(Some(0)).collect()
}

/// Re-run current exe with args under elevated token; wait and return exit code.
pub fn run_as_admin(args: &str) -> Result<i32, String> {
    let exe = std::env::current_exe().map_err(|e| format!("resolve self: {}", e))?;
    let verb = to_wide("runas");
    let file = to_wide(&exe.to_string_lossy());
    let params = to_wide(args);

    let mut sei = SHELLEXECUTEINFOW {
        cbSize: std::mem::size_of::<SHELLEXECUTEINFOW>() as u32,
        fMask: SEE_MASK_NOCLOSEPROCESS | SEE_MASK_NOASYNC,
        hwnd: 0,
        lpVerb: verb.as_ptr(),
        lpFile: file.as_ptr(),
        lpParameters: params.as_ptr(),
        lpDirectory: std::ptr::null(),
        nShow: 0,
        hInstApp: 0,
        lpIDList: std::ptr::null(),
        lpClass: std::ptr::null(),
        hkeyClass: 0,
        dwHotKey: 0,
        hIcon: 0,
        hProcess: 0,
    };

    if unsafe { ShellExecuteExW(&mut sei) } == 0 {
        return Err("UAC elevation cancelled or failed".to_string());
    }

    unsafe {
        WaitForSingleObject(sei.hProcess, INFINITE);
        let mut code: u32 = 1;
        GetExitCodeProcess(sei.hProcess, &mut code);
        CloseHandle(sei.hProcess);
        Ok(code as i32)
    }
}
