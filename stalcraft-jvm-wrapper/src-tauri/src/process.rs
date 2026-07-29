// process.rs — полный порт process.go
// NtCreateUserProcess с PS_ATTRIBUTE_IFEO_SKIP_DEBUGGER,
// приоритеты памяти/IO через NtSetInformationProcess,
// ожидание видимого окна через EnumWindows.

#![allow(non_snake_case)] // Win32/NT layout names match SDK headers

use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;
use std::path::Path;
use std::ptr;

use serde::Serialize;

use crate::paths;

// ─── NT / Win32 API ───────────────────────────────────────────────────────────

#[link(name = "ntdll")]
extern "system" {
    fn RtlCreateProcessParametersEx(
        pProcessParameters: *mut usize,
        ImagePathName: *const UNICODE_STRING,
        DllPath: *const UNICODE_STRING,
        CurrentDirectory: *const UNICODE_STRING,
        CommandLine: *const UNICODE_STRING,
        Environment: *const std::ffi::c_void,
        WindowTitle: *const UNICODE_STRING,
        DesktopInfo: *const UNICODE_STRING,
        ShellInfo: *const UNICODE_STRING,
        RuntimeData: *const UNICODE_STRING,
        Flags: u32,
    ) -> i32;
    fn RtlDestroyProcessParameters(ProcessParameters: usize) -> i32;
    fn NtCreateUserProcess(
        hProcess: *mut isize,
        hThread: *mut isize,
        ProcessDesiredAccess: u32,
        ThreadDesiredAccess: u32,
        ProcessObjectAttributes: *const std::ffi::c_void,
        ThreadObjectAttributes: *const std::ffi::c_void,
        ProcessFlags: u32,
        ThreadFlags: u32,
        ProcessParameters: usize,
        CreateInfo: *const PS_CREATE_INFO,
        AttributeList: *const PS_ATTRIBUTE_LIST,
    ) -> i32;
    fn NtSetInformationProcess(
        ProcessHandle: isize,
        ProcessInformationClass: u32,
        ProcessInformation: *const std::ffi::c_void,
        ProcessInformationLength: u32,
    ) -> i32;
}

#[link(name = "kernel32")]
extern "system" {
    fn CloseHandle(hObject: isize) -> i32;
    fn WaitForSingleObject(hHandle: isize, dwMilliseconds: u32) -> u32;
    fn GetExitCodeProcess(hProcess: isize, lpExitCode: *mut u32) -> i32;
    fn SetProcessPriorityBoost(hProcess: isize, DisablePriorityBoost: i32) -> i32;
    fn ResumeThread(hThread: isize) -> u32;
    fn CreateToolhelp32Snapshot(dwFlags: u32, th32ProcessID: u32) -> isize;
    fn Process32FirstW(hSnapshot: isize, lppe: *mut PROCESSENTRY32W) -> i32;
    fn Process32NextW(hSnapshot: isize, lppe: *mut PROCESSENTRY32W) -> i32;
    fn OpenProcess(dwDesiredAccess: u32, bInheritHandle: i32, dwProcessId: u32) -> isize;
    fn QueryFullProcessImageNameW(
        hProcess: isize,
        dwFlags: u32,
        lpExeName: *mut u16,
        lpdwSize: *mut u32,
    ) -> i32;
}

#[link(name = "user32")]
extern "system" {
    fn EnumWindows(
        lpEnumFunc: Option<unsafe extern "system" fn(isize, isize) -> i32>,
        lParam: isize,
    ) -> i32;
    fn GetWindowThreadProcessId(hWnd: isize, lpdwProcessId: *mut u32) -> u32;
    fn IsWindowVisible(hWnd: isize) -> i32;
    fn RegisterClassExW(lpwcx: *const WNDCLASSEXW) -> u16;
    fn CreateWindowExW(
        dwExStyle: u32,
        lpClassName: *const u16,
        lpWindowName: *const u16,
        dwStyle: u32,
        X: i32,
        Y: i32,
        nWidth: i32,
        nHeight: i32,
        hWndParent: isize,
        hMenu: isize,
        hInstance: isize,
        lpParam: *const std::ffi::c_void,
    ) -> isize;
    fn SetLayeredWindowAttributes(hwnd: isize, crKey: u32, bAlpha: u8, dwFlags: u32) -> i32;
    fn GetMessageW(lpMsg: *mut MSG, hWnd: isize, wMsgFilterMin: u32, wMsgFilterMax: u32) -> i32;
    fn TranslateMessage(lpMsg: *const MSG) -> i32;
    fn DispatchMessageW(lpMsg: *const MSG) -> usize;
    fn DefWindowProcW(hWnd: isize, Msg: u32, wParam: usize, lParam: isize) -> isize;
}

// ─── Структуры ────────────────────────────────────────────────────────────────

#[repr(C)]
struct UNICODE_STRING {
    Length: u16,
    MaximumLength: u16,
    Buffer: *mut u16,
}

#[repr(C)]
struct CLIENT_ID {
    UniqueProcess: usize,
    UniqueThread: usize,
}

#[repr(C)]
struct PS_ATTRIBUTE {
    Attribute: usize,
    Size: usize,
    Value: usize,
    ReturnLength: usize,
}

#[repr(C)]
struct PS_ATTRIBUTE_LIST {
    TotalLength: usize,
    Attributes: [PS_ATTRIBUTE; 2],
}

#[repr(C)]
struct PS_CREATE_INFO {
    data: [u8; 0x58],
}

#[repr(C)]
struct WNDCLASSEXW {
    cbSize: u32,
    style: u32,
    lpfnWndProc: unsafe extern "system" fn(isize, u32, usize, isize) -> isize,
    cbClsExtra: i32,
    cbWndExtra: i32,
    hInstance: isize,
    hIcon: isize,
    hCursor: isize,
    hbrBackground: isize,
    lpszMenuName: *const u16,
    lpszClassName: *const u16,
    hIconSm: isize,
}

#[repr(C)]
struct POINT {
    x: i32,
    y: i32,
}

#[repr(C)]
struct MSG {
    hwnd: isize,
    message: u32,
    wParam: usize,
    lParam: isize,
    time: u32,
    pt: POINT,
}

#[repr(C)]
struct PROCESSENTRY32W {
    dwSize: u32,
    cntUsage: u32,
    th32ProcessID: u32,
    th32DefaultHeapID: usize,
    th32ModuleID: u32,
    cntThreads: u32,
    th32ParentProcessID: u32,
    pcPriClassBase: i32,
    dwFlags: u32,
    szExeFile: [u16; 260],
}

// ─── Константы ────────────────────────────────────────────────────────────────

const PS_ATTRIBUTE_IMAGE_NAME: usize = 0x00020005;
const PS_ATTRIBUTE_CLIENT_ID: usize = 0x00010003;
const RTL_USER_PROC_PARAMS_NORMALIZED: u32 = 0x01;
const IFEO_SKIP_DEBUGGER: u32 = 0x04;
const PROCESS_CREATE_FLAGS_INHERIT_HANDLES: u32 = 0x04;
const PROCESS_ALL_ACCESS: u32 = 0x001FFFFF;
const THREAD_ALL_ACCESS: u32 = 0x001FFFFF;
const PROCESS_MEMORY_PRIORITY: u32 = 0x27;
const PROCESS_IO_PRIORITY: u32 = 0x21;
const MEMORY_PRIORITY_HIGH: u32 = 5;
const IO_PRIORITY_HIGH: u32 = 3;

const WS_VISIBLE: u32 = 0x10000000;
const WS_POPUP: u32 = 0x80000000;
const WS_EX_TOOLWINDOW: u32 = 0x00000080;
const WS_EX_LAYERED: u32 = 0x00080000;
const LWA_ALPHA: u32 = 0x02;

const TH32CS_SNAPPROCESS: u32 = 0x00000002;
const PROCESS_QUERY_LIMITED_INFORMATION: u32 = 0x1000;
const INVALID_HANDLE_VALUE: isize = -1;

// ─── Утилиты ─────────────────────────────────────────────────────────────────

fn to_wide(s: &str) -> Vec<u16> {
    OsStr::new(s).encode_wide().chain(Some(0)).collect()
}

fn make_unicode_string(buf: &mut Vec<u16>) -> UNICODE_STRING {
    let char_count = buf.len().saturating_sub(1);
    UNICODE_STRING {
        Length: (char_count * 2) as u16,
        MaximumLength: (buf.len() * 2) as u16,
        Buffer: buf.as_mut_ptr(),
    }
}

fn create_env_block() -> Vec<u16> {
    let mut block = Vec::new();
    for (k, v) in std::env::vars() {
        let entry = format!("{}={}", k, v);
        let wide: Vec<u16> = OsStr::new(&entry).encode_wide().chain(Some(0)).collect();
        block.extend_from_slice(&wide);
    }
    block.push(0);
    block
}

fn build_cmd_line(exe: &str, args: &[String]) -> String {
    let mut parts = Vec::with_capacity(1 + args.len());
    parts.push(format!("\"{}\"", exe));
    for a in args {
        if a.contains(' ') || a.contains('"') {
            parts.push(format!("\"{}\"", a));
        } else {
            parts.push(a.clone());
        }
    }
    parts.join(" ")
}

/// extractGameDir — аналог extractGameDir из process.go
fn extract_game_dir(exe_path: &str, args: &[String]) -> String {
    for i in 0..args.len() {
        if args[i] == "--gameDir" && i + 1 < args.len() {
            return args[i + 1].clone();
        }
    }
    // Проверяем -Djava.library.path=
    for a in args {
        if let Some(lib_path) = a.strip_prefix("-Djava.library.path=") {
            let exe_dir = Path::new(exe_path)
                .parent()
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_default();
            let lib_norm = lib_path.replace('/', "\\");
            let exe_norm = exe_dir.replace('/', "\\");
            if exe_norm.to_lowercase().ends_with(&lib_norm.to_lowercase()) {
                let prefix_len = exe_norm.len() - lib_norm.len();
                return exe_norm[..prefix_len].to_string();
            }
        }
    }
    String::new()
}

fn abs_work_dir(dir: &str) -> String {
    if dir.is_empty() {
        return String::new();
    }
    let p = Path::new(dir);
    if let Ok(c) = p.canonicalize() {
        return c.to_string_lossy().to_string();
    }
    if p.is_absolute() {
        return dir.to_string();
    }
    std::env::current_dir()
        .ok()
        .map(|cwd| cwd.join(p).to_string_lossy().to_string())
        .unwrap_or_else(|| dir.to_string())
}

fn resolve_work_dir(abs_path: &str, args: &[String]) -> String {
    let dir = extract_game_dir(abs_path, args);
    if !dir.is_empty() {
        return abs_work_dir(&dir);
    }
    // ponytail: EXBO runtime needs stalcraft root, not java\bin — working jvm_wrapper.rar uses this.
    if let Some(game) = paths::game_dir_from_target(abs_path) {
        return abs_work_dir(&game.to_string_lossy());
    }
    abs_work_dir(
        &Path::new(abs_path)
            .parent()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_default(),
    )
}

// ─── Phantom window (аналог phantom.go) ──────────────────────────────────────

/// Start — запускает невидимое окно в отдельном потоке.
/// Аналог phantom.Start() из phantom.go — нужно для корректной работы оверлеев.
pub fn start_phantom_window() {
    std::thread::spawn(|| unsafe {
        let class_name_buf = to_wide("StalcraftWrapper");

        unsafe extern "system" fn wnd_proc(
            hwnd: isize,
            msg: u32,
            wparam: usize,
            lparam: isize,
        ) -> isize {
            DefWindowProcW(hwnd, msg, wparam, lparam)
        }

        let wc = WNDCLASSEXW {
            cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
            style: 0,
            lpfnWndProc: wnd_proc,
            cbClsExtra: 0,
            cbWndExtra: 0,
            hInstance: 0,
            hIcon: 0,
            hCursor: 0,
            hbrBackground: 0,
            lpszMenuName: ptr::null(),
            lpszClassName: class_name_buf.as_ptr(),
            hIconSm: 0,
        };
        RegisterClassExW(&wc);

        let hwnd = CreateWindowExW(
            WS_EX_TOOLWINDOW | WS_EX_LAYERED,
            class_name_buf.as_ptr(),
            ptr::null(),
            WS_VISIBLE | WS_POPUP,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            ptr::null(),
        );
        SetLayeredWindowAttributes(hwnd, 0, 0, LWA_ALPHA);

        let mut msg: MSG = std::mem::zeroed();
        loop {
            let ret = GetMessageW(&mut msg, 0, 0, 0);
            if ret == 0 || ret == -1 {
                break;
            }
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }

        drop(class_name_buf);
    });
}

// ─── NtCreateUserProcess — аналог Start() из process.go ──────────────────────

fn strip_extended_path(s: &str) -> String {
    if s.starts_with(r"\\?\") {
        s[4..].to_string()
    } else {
        s.to_string()
    }
}

/// Resolve executable path — canonicalize when possible, else use as given (Go filepath.Abs style).
fn resolve_exe_path(exe_path: &str) -> Result<String, String> {
    let raw = Path::new(exe_path);
    if raw.exists() {
        if let Ok(c) = raw.canonicalize() {
            return Ok(strip_extended_path(&c.to_string_lossy()));
        }
    }
    if raw.is_absolute() {
        return Ok(exe_path.to_string());
    }
    if let Ok(cwd) = std::env::current_dir() {
        let joined = cwd.join(raw);
        if joined.exists() {
            if let Ok(c) = joined.canonicalize() {
                return Ok(strip_extended_path(&c.to_string_lossy()));
            }
        }
    }
    Err(format!("resolve {}: file not found", exe_path))
}

/// nt_create_process — порт Start() из process.go.
pub fn nt_create_process(exe_path: &str, args: &[String]) -> Result<(isize, isize, u32), String> {
    let abs_str = resolve_exe_path(exe_path)?;

    let nt_path = format!(r"\??\{}", abs_str);
    let cmd_line = build_cmd_line(&abs_str, args);
    let work_dir = resolve_work_dir(&abs_str, args);
    let desktop = r"WinSta0\Default";

    // Буферы — должны жить до конца вызова NtCreateUserProcess
    let mut img_buf = to_wide(&abs_str);
    let mut cmd_buf = to_wide(&cmd_line);
    let mut wd_buf = to_wide(&work_dir);
    let mut nt_buf = to_wide(&nt_path);
    let mut desktop_buf = to_wide(desktop);
    let env_block = create_env_block();

    let img_us = make_unicode_string(&mut img_buf);
    let cmd_us = make_unicode_string(&mut cmd_buf);
    let wd_us = make_unicode_string(&mut wd_buf);
    let nt_us = make_unicode_string(&mut nt_buf);
    let desktop_us = make_unicode_string(&mut desktop_buf);

    let mut params: usize = 0;
    let r = unsafe {
        RtlCreateProcessParametersEx(
            &mut params,
            &img_us,
            ptr::null(),
            &wd_us,
            &cmd_us,
            env_block.as_ptr() as *const _,
            ptr::null(),
            &desktop_us,
            ptr::null(),
            ptr::null(),
            RTL_USER_PROC_PARAMS_NORMALIZED,
        )
    };
    if r != 0 {
        return Err(format!(
            "RtlCreateProcessParametersEx: NTSTATUS 0x{:08x}",
            r
        ));
    }

    let mut create_info = PS_CREATE_INFO { data: [0u8; 0x58] };
    unsafe {
        // Size at offset 0
        *(create_info.data.as_mut_ptr() as *mut usize) = 0x58;
        // State = IFEO_SKIP_DEBUGGER at offset 0x10
        *(create_info.data.as_mut_ptr().add(0x10) as *mut u32) = IFEO_SKIP_DEBUGGER;
    }

    let mut cid = CLIENT_ID {
        UniqueProcess: 0,
        UniqueThread: 0,
    };

    let attr_list = PS_ATTRIBUTE_LIST {
        TotalLength: std::mem::size_of::<PS_ATTRIBUTE_LIST>(),
        Attributes: [
            PS_ATTRIBUTE {
                Attribute: PS_ATTRIBUTE_IMAGE_NAME,
                Size: nt_us.Length as usize,
                Value: nt_us.Buffer as usize,
                ReturnLength: 0,
            },
            PS_ATTRIBUTE {
                Attribute: PS_ATTRIBUTE_CLIENT_ID,
                Size: std::mem::size_of::<CLIENT_ID>(),
                Value: &mut cid as *mut _ as usize,
                ReturnLength: 0,
            },
        ],
    };

    let mut h_process: isize = 0;
    let mut h_thread: isize = 0;

    let r = unsafe {
        NtCreateUserProcess(
            &mut h_process,
            &mut h_thread,
            PROCESS_ALL_ACCESS,
            THREAD_ALL_ACCESS,
            ptr::null(),
            ptr::null(),
            PROCESS_CREATE_FLAGS_INHERIT_HANDLES,
            0,
            params,
            &create_info,
            &attr_list,
        )
    };

    // Держим буферы живыми (аналог runtime.KeepAlive в Go)
    drop(img_buf);
    drop(cmd_buf);
    drop(wd_buf);
    drop(nt_buf);
    drop(desktop_buf);
    drop(env_block);
    let _ = &cid;

    if r != 0 {
        unsafe { RtlDestroyProcessParameters(params) };
        return Err(format!("NtCreateUserProcess: NTSTATUS 0x{:08x}", r));
    }
    unsafe {
        RtlDestroyProcessParameters(params);
        ResumeThread(h_thread);
    }

    let pid = cid.UniqueProcess as u32;
    Ok((h_process, h_thread, pid))
}

// ─── Boost — аналог Process.Boost() из process.go ────────────────────────────

/// boost_process — точный порт Boost() из process.go
pub fn boost_process(handle: isize) {
    unsafe {
        // DisablePriorityBoost = 1 = false (включаем boost, отключаем decay)
        SetProcessPriorityBoost(handle, 1);

        let mem = MEMORY_PRIORITY_HIGH;
        NtSetInformationProcess(
            handle,
            PROCESS_MEMORY_PRIORITY,
            &mem as *const _ as *const _,
            std::mem::size_of::<u32>() as u32,
        );

        let iop = IO_PRIORITY_HIGH;
        NtSetInformationProcess(
            handle,
            PROCESS_IO_PRIORITY,
            &iop as *const _ as *const _,
            std::mem::size_of::<u32>() as u32,
        );
    }
}

// ─── Wait — аналог Process.Wait() из process.go ───────────────────────────────

/// wait_process — точный порт Wait() из process.go.
/// Ждёт пока процесс выйдет ИЛИ покажет видимое окно.
pub fn wait_process(h_process: isize, pid: u32) -> i32 {
    loop {
        let ret = unsafe { WaitForSingleObject(h_process, 200) };
        if ret == 0 {
            // WAIT_OBJECT_0 — процесс завершился
            let mut code: u32 = 0;
            unsafe { GetExitCodeProcess(h_process, &mut code) };
            return code as i32;
        }
        // WAIT_TIMEOUT (0x102) — продолжаем ожидание
        if has_visible_window(pid) {
            eprintln!("[process] visible window, detaching pid={}", pid);
            return 0;
        }
    }
}

pub fn cleanup_handles(h_process: isize, h_thread: isize) {
    unsafe {
        CloseHandle(h_process);
        CloseHandle(h_thread);
    }
}

// ─── EnumWindows — hasVisibleWindow из process.go ────────────────────────────
// Используем статическую переменную для результата (как в оригинальном Go коде)
// Для concurrent вызовов используем ключ в lparam

static mut FOUND_VISIBLE: u32 = 0;

unsafe extern "system" fn enum_windows_proc(hwnd: isize, target_pid: isize) -> i32 {
    let mut pid: u32 = 0;
    GetWindowThreadProcessId(hwnd, &mut pid);
    if pid as isize == target_pid {
        if IsWindowVisible(hwnd) != 0 {
            FOUND_VISIBLE = 1;
            return 0;
        }
    }
    1
}

fn has_visible_window(pid: u32) -> bool {
    // Используем статическую переменную - это работает потому что:
    // 1. EnumWindows вызывается из wait_process, который вызывается синхронно
    // 2. Между вызовами wait_process есть enough time для завершения callback
    // 3. Это соответствует поведению Go кода (sync.Map)
    unsafe {
        FOUND_VISIBLE = 0;
        EnumWindows(Some(enum_windows_proc), pid as isize);
        FOUND_VISIBLE != 0
    }
}

// ─── Reverse discovery: find running STALZONE / STALCRAFT client ─────────────

#[derive(Debug, Clone, Serialize)]
pub struct GameProcessMatch {
    pub pid: u32,
    pub name: String,
    pub path: String,
    pub kind: String,
    pub launcher: String,
    pub has_window: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct GameProcessSearchResult {
    /// True only when at least one live game client process was found.
    pub running: bool,
    pub primary: Option<GameProcessMatch>,
    pub processes: Vec<GameProcessMatch>,
}

fn pe32_exe_name(pe: &PROCESSENTRY32W) -> String {
    let len = pe
        .szExeFile
        .iter()
        .position(|&c| c == 0)
        .unwrap_or(pe.szExeFile.len());
    String::from_utf16_lossy(&pe.szExeFile[..len])
}

fn is_candidate_exe_name(name: &str) -> bool {
    let n = name.to_lowercase();
    matches!(
        n.as_str(),
        "stalzone.exe"
            | "stalzonew.exe"
            | "stalcraft.exe"
            | "stalcraftw.exe"
            | "java.exe"
            | "javaw.exe"
    )
}

fn query_process_image_path(pid: u32) -> Option<String> {
    unsafe {
        let handle = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, pid);
        if handle == 0 {
            return None;
        }
        let mut buf = [0u16; 1024];
        let mut size = buf.len() as u32;
        let ok = QueryFullProcessImageNameW(handle, 0, buf.as_mut_ptr(), &mut size);
        CloseHandle(handle);
        if ok == 0 || size == 0 {
            return None;
        }
        Some(String::from_utf16_lossy(&buf[..size as usize]))
    }
}

fn match_score(m: &GameProcessMatch) -> i32 {
    let mut score = 0;
    let name = m.name.to_lowercase();
    if name.starts_with("stalzone") {
        score += 100;
    } else if name.starts_with("stalcraft") {
        score += 80;
    } else if name == "javaw.exe" {
        score += 40;
    } else if name == "java.exe" {
        score += 30;
    }
    match m.launcher.as_str() {
        "steam" | "exbo" => score += 20,
        "egs" | "vkplay" => score += 15,
        _ => {}
    }
    if m.has_window {
        score += 10;
    }
    if m.kind == "launcher" {
        score += 5;
    }
    score
}

/// Rank matches: prefer stalzone* (Steam + EXBO), then stalcraft*, then game javaw.
pub fn rank_matches(matches: &[GameProcessMatch]) -> Option<GameProcessMatch> {
    matches
        .iter()
        .max_by_key(|m| match_score(m))
        .cloned()
}

fn classify_match(pid: u32, path: &str) -> Option<GameProcessMatch> {
    let name = Path::new(path)
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_string();
    let lower = name.to_lowercase();

    let kind = if paths::is_launcher_binary(path) {
        "launcher"
    } else if paths::is_game_java(path) {
        "java"
    } else {
        return None;
    };

    // Drop system java even if name matched snapshot filter.
    if (lower == "java.exe" || lower == "javaw.exe") && kind != "java" {
        return None;
    }

    let launcher = paths::classify_target(path).kind.as_str().to_string();
    Some(GameProcessMatch {
        pid,
        name,
        path: path.to_string(),
        kind: kind.to_string(),
        launcher,
        has_window: has_visible_window(pid),
    })
}

/// Enumerate running STALZONE / STALCRAFT clients (incl. Steam `stalzone.exe`).
/// Returns `running: false` and empty lists when the game is not on — never invents PIDs.
pub fn find_game_processes() -> GameProcessSearchResult {
    let mut processes = Vec::new();

    unsafe {
        let snap = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0);
        if snap == INVALID_HANDLE_VALUE || snap == 0 {
            return GameProcessSearchResult {
                running: false,
                primary: None,
                processes,
            };
        }

        let mut pe: PROCESSENTRY32W = std::mem::zeroed();
        pe.dwSize = std::mem::size_of::<PROCESSENTRY32W>() as u32;

        if Process32FirstW(snap, &mut pe) != 0 {
            loop {
                let exe_name = pe32_exe_name(&pe);
                if is_candidate_exe_name(&exe_name) {
                    let pid = pe.th32ProcessID;
                    if pid != 0 {
                        if let Some(path) = query_process_image_path(pid) {
                            if let Some(m) = classify_match(pid, &path) {
                                processes.push(m);
                            }
                        }
                    }
                }
                if Process32NextW(snap, &mut pe) == 0 {
                    break;
                }
            }
        }
        CloseHandle(snap);
    }

    processes.sort_by_key(|m| std::cmp::Reverse(match_score(m)));
    let primary = rank_matches(&processes);
    let running = !processes.is_empty();
    GameProcessSearchResult {
        running,
        primary,
        processes,
    }
}

#[cfg(test)]
mod discover_tests {
    use super::*;

    fn sample(name: &str, path: &str, launcher: &str, kind: &str, pid: u32) -> GameProcessMatch {
        GameProcessMatch {
            pid,
            name: name.to_string(),
            path: path.to_string(),
            kind: kind.to_string(),
            launcher: launcher.to_string(),
            has_window: false,
        }
    }

    #[test]
    fn rank_prefers_steam_stalzone_over_legacy() {
        let matches = vec![
            sample(
                "stalcraftw.exe",
                r"D:\Steam\steamapps\common\stalcraft\stalcraftw.exe",
                "steam",
                "launcher",
                111,
            ),
            sample(
                "stalzone.exe",
                r"D:\Steam\steamapps\common\stalcraft\stalzone.exe",
                "steam",
                "launcher",
                222,
            ),
        ];
        let primary = rank_matches(&matches).expect("primary");
        assert_eq!(primary.pid, 222);
        assert_eq!(primary.name, "stalzone.exe");
    }

    #[test]
    fn rank_prefers_stalzone_over_javaw() {
        let matches = vec![
            sample(
                "javaw.exe",
                r"C:\Users\me\AppData\Roaming\EXBO\runtime\stalcraft\win64\java\bin\javaw.exe",
                "exbo",
                "java",
                10,
            ),
            sample(
                "stalzone.exe",
                r"C:\Users\me\AppData\Roaming\EXBO\runtime\stalcraft\win64\java\bin\stalzone.exe",
                "exbo",
                "launcher",
                20,
            ),
        ];
        let primary = rank_matches(&matches).expect("primary");
        assert_eq!(primary.pid, 20);
    }

    #[test]
    fn empty_rank_is_none() {
        assert!(rank_matches(&[]).is_none());
    }
}
