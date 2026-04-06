use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;
use std::ptr;

// NT API procedures
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
}

#[link(name = "user32")]
extern "system" {
    fn EnumWindows(lpEnumFunc: Option<unsafe extern "system" fn(isize, isize) -> i32>, lParam: isize) -> i32;
    fn GetWindowThreadProcessId(hWnd: isize, lpdwProcessId: *mut u32) -> u32;
    fn IsWindowVisible(hWnd: isize) -> i32;
    fn RegisterClassExW(lpwcx: *const WNDCLASSEXW) -> u16;
    fn CreateWindowExW(
        dwExStyle: u32,
        lpClassName: *const u16,
        lpWindowName: *const u16,
        dwStyle: u32,
        X: i32, Y: i32, nWidth: i32, nHeight: i32,
        hWndParent: isize, hMenu: isize, hInstance: isize, lpParam: *const std::ffi::c_void,
    ) -> isize;
    fn SetLayeredWindowAttributes(hwnd: isize, crKey: u32, bAlpha: u8, dwFlags: u32) -> i32;
    fn GetMessageW(lpMsg: *mut MSG, hWnd: isize, wMsgFilterMin: u32, wMsgFilterMax: u32) -> i32;
    fn TranslateMessage(lpMsg: *const MSG) -> i32;
    fn DispatchMessageW(lpMsg: *const MSG) -> usize;
    fn DefWindowProcW(hWnd: isize, Msg: u32, wParam: usize, lParam: isize) -> isize;
}

// NT structures
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

// Window structures for phantom window
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

// NT constants
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

// Window constants
const WS_VISIBLE: u32 = 0x10000000;
const WS_POPUP: u32 = 0x80000000;
const WS_EX_TOOLWINDOW: u32 = 0x00000080;
const WS_EX_LAYERED: u32 = 0x00080000;
const LWA_ALPHA: u32 = 0x02;

fn to_unicode_string(s: &str) -> (UNICODE_STRING, Vec<u16>) {
    let mut buf: Vec<u16> = OsStr::new(s).encode_wide().chain(Some(0)).collect();
    let len = ((buf.len() - 1) * 2) as u16;
    let maximum = (buf.len() * 2) as u16;
    let us = UNICODE_STRING {
        Length: len,
        MaximumLength: maximum,
        Buffer: buf.as_mut_ptr(),
    };
    (us, buf)
}

fn create_env_block() -> Vec<u16> {
    let mut block = Vec::new();
    for e in std::env::vars() {
        let entry = format!("{}={}", e.0, e.1);
        let wide: Vec<u16> = OsStr::new(&entry).encode_wide().chain(Some(0)).collect();
        block.extend_from_slice(&wide);
    }
    block.push(0); // Double null terminator
    block
}

fn build_cmd_line(exe: &str, args: &[String]) -> String {
    let mut parts = Vec::new();
    parts.push(format!("\"{}\"", exe));
    for a in args {
        parts.push(a.clone());
    }
    parts.join(" ")
}

fn extract_game_dir(args: &[String]) -> String {
    for i in 0..args.len() {
        if args[i] == "--gameDir" && i + 1 < args.len() {
            return args[i + 1].clone();
        }
    }
    String::new()
}

/// Create a phantom window to keep the wrapper alive when launched by IFEO.
/// This mimics the Go version's createPhantomWindow().
pub fn create_phantom_window() {
    use std::thread;

    thread::spawn(|| {
        unsafe {
            let class_name_str = "StalcraftWrapper";
            let class_name: Vec<u16> = OsStr::new(&class_name_str).encode_wide().chain(Some(0)).collect();

            unsafe extern "system" fn wnd_proc(hwnd: isize, msg: u32, wParam: usize, lParam: isize) -> isize {
                DefWindowProcW(hwnd, msg, wParam, lParam)
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
                lpszClassName: class_name.as_ptr(),
                hIconSm: 0,
            };

            RegisterClassExW(&wc);

            let hwnd = CreateWindowExW(
                WS_EX_TOOLWINDOW | WS_EX_LAYERED,
                class_name.as_ptr(),
                ptr::null(),
                WS_VISIBLE | WS_POPUP,
                0, 0, 0, 0,
                0, 0, 0, ptr::null(),
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
        }
    });
}

/// nt_create_process creates a process via NtCreateUserProcess, bypassing kernel32 IFEO check.
/// This matches the Go implementation exactly.
pub fn nt_create_process(exe_path: &str, args: &[String]) -> Result<(isize, isize, u32), String> {
    use std::path::Path;

    // Get absolute path - matching Go's filepath.Abs()
    // IMPORTANT: We use dunce::canonicalize or strip \\?\ prefix because
    // std::path::canonicalize() adds \\?\ prefix on Windows which breaks NT paths
    let abs_path = Path::new(exe_path)
        .canonicalize()
        .map_err(|e| format!("Failed to resolve absolute path: {}", e))?;
    
    let mut abs_path_str = abs_path.to_string_lossy().to_string();
    
    // Strip the \\?\ prefix that canonicalize() adds on Windows
    // This is critical - NT API expects \??\C:\... not \??\\?\C:\...
    if abs_path_str.starts_with(r"\\?\") {
        abs_path_str = abs_path_str[4..].to_string();
        eprintln!("[NT] Stripped \\\\?\\ prefix from path");
    }
    
    // NT API requires \??\ prefix (matches Go exactly)
    let nt_path = format!(r"\??\{}", abs_path_str);
    eprintln!("[NT] Image path: {}", abs_path_str);
    eprintln!("[NT] NT path: {}", nt_path);

    let cmd_line = build_cmd_line(&abs_path_str, args);

    let work_dir = if !extract_game_dir(args).is_empty() {
        extract_game_dir(args)
    } else {
        abs_path.parent()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_default()
    };

    let (img_us, img_buf) = to_unicode_string(&abs_path_str);
    let (cmd_us, cmd_buf) = to_unicode_string(&cmd_line);
    let (wd_us, wd_buf) = to_unicode_string(&work_dir);
    let (nt_us, nt_buf) = to_unicode_string(&nt_path);
    let env_block = create_env_block();
    let (desktop_us, desktop_buf) = to_unicode_string(r"WinSta0\Default");

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
        return Err(format!("RtlCreateProcessParametersEx: 0x{:08x}", r));
    }

    let mut create_info: PS_CREATE_INFO = PS_CREATE_INFO { data: [0; 0x58] };
    // Set Size at offset 0
    unsafe {
        *(create_info.data.as_mut_ptr() as *mut usize) = 0x58;
        // Set State at offset 0x10 to IFEO_SKIP_DEBUGGER
        let state_ptr = create_info.data.as_mut_ptr().add(0x10);
        *(state_ptr as *mut u32) = IFEO_SKIP_DEBUGGER;
    }

    let mut cid: CLIENT_ID = CLIENT_ID {
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

    // Keep buffers alive (like runtime.KeepAlive in Go)
    drop(img_buf);
    drop(cmd_buf);
    drop(wd_buf);
    drop(nt_buf);
    drop(env_block);
    drop(desktop_buf);

    if r != 0 {
        unsafe { RtlDestroyProcessParameters(params) };
        return Err(format!("NtCreateUserProcess: 0x{:08x}", r));
    }

    unsafe { RtlDestroyProcessParameters(params) };

    let pid = cid.UniqueProcess as u32;
    Ok((h_process, h_thread, pid))
}

/// boost_process sets high memory/IO priority and disables priority decay.
pub fn boost_process(handle: isize) {
    unsafe {
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

static mut FOUND_VISIBLE: u32 = 0;

unsafe extern "system" fn enum_windows_proc(hwnd: isize, target_pid: isize) -> i32 {
    let mut pid: u32 = 0;
    GetWindowThreadProcessId(hwnd, &mut pid);

    if pid as isize == target_pid {
        if IsWindowVisible(hwnd) != 0 {
            FOUND_VISIBLE = 1;
            return 0; // Stop enumeration
        }
    }
    1 // Continue enumeration
}

fn has_visible_window(pid: u32) -> bool {
    unsafe {
        FOUND_VISIBLE = 0;
        EnumWindows(Some(enum_windows_proc), pid as isize);
        FOUND_VISIBLE != 0
    }
}

/// wait_process waits for the process to either exit or show a visible window.
pub fn wait_process(h_process: isize, pid: u32) -> i32 {
    loop {
        let ret = unsafe { WaitForSingleObject(h_process, 200) };
        if ret == 0 { // WAIT_OBJECT_0
            let mut exit_code: u32 = 0;
            unsafe {
                GetExitCodeProcess(h_process, &mut exit_code);
            }
            return exit_code as i32;
        }

        if has_visible_window(pid) {
            return 0;
        }
    }
}

/// Cleanup handles after process is launched.
pub fn cleanup_handles(h_process: isize, h_thread: isize) {
    unsafe {
        CloseHandle(h_process);
        CloseHandle(h_thread);
    }
}

// Constants for argument filtering
const EXACT_REMOVE: [&str; 2] = [
    "-XX:-PrintCommandLineFlags",
    "-XX:+UseG1GC",
];

const PREFIX_REMOVE: [&str; 26] = [
    "-XX:MaxGCPauseMillis=",
    "-XX:MetaspaceSize=",
    "-XX:MaxMetaspaceSize=",
    "-XX:G1HeapRegionSize=",
    "-XX:G1NewSizePercent=",
    "-XX:G1MaxNewSizePercent=",
    "-XX:G1ReservePercent=",
    "-XX:G1HeapWastePercent=",
    "-XX:G1MixedGCCountTarget=",
    "-XX:InitiatingHeapOccupancyPercent=",
    "-XX:G1MixedGCLiveThresholdPercent=",
    "-XX:G1RSetUpdatingPauseTimePercent=",
    "-XX:SurvivorRatio=",
    "-XX:MaxTenuringThreshold=",
    "-XX:ParallelGCThreads=",
    "-XX:ConcGCThreads=",
    "-XX:SoftRefLRUPolicyMSPerMB=",
    "-XX:ReservedCodeCacheSize=",
    "-XX:NonNMethodCodeHeapSize=",
    "-XX:ProfiledCodeHeapSize=",
    "-XX:NonProfiledCodeHeapSize=",
    "-XX:MaxInlineLevel=",
    "-XX:FreqInlineSize=",
    "-XX:LargePageSizeInBytes=",
    "-Xms",
    "-Xmx",
];

fn should_remove(arg: &str) -> bool {
    if EXACT_REMOVE.contains(&arg) {
        return true;
    }
    PREFIX_REMOVE.iter().any(|p| arg.starts_with(p))
}

fn split_args(args: &[String]) -> (Vec<String>, String, Vec<String>) {
    let mut jvm = Vec::new();
    let mut main_class = String::new();
    let mut app = Vec::new();

    let mut i = 0;
    while i < args.len() {
        let a = &args[i];
        if a == "-classpath" || a == "-cp" || a == "-jar" {
            jvm.push(a.clone());
            i += 1;
            if i < args.len() {
                jvm.push(args[i].clone());
            }
            i += 1;
            continue;
        }
        if a.starts_with('-') {
            jvm.push(a.clone());
            i += 1;
            continue;
        }
        main_class = a.clone();
        if i + 1 < args.len() {
            app = args[i+1..].to_vec();
        }
        return (jvm, main_class, app);
    }
    (jvm, main_class, app)
}

pub fn filter_args(orig: &[String], injected: &[String]) -> Vec<String> {
    let (jvm, main_class, app) = split_args(orig);

    let filtered: Vec<String> = jvm.into_iter()
        .filter(|a| !should_remove(a))
        .collect();

    let mut result = Vec::with_capacity(filtered.len() + injected.len() + 1 + app.len());
    result.extend(filtered);
    result.extend_from_slice(injected);
    if !main_class.is_empty() {
        result.push(main_class);
    }
    result.extend(app);
    result
}

/// Launch game - matches Go implementation exactly
/// The key insight: we launch stalcraft.exe directly, NOT java.exe
/// NtCreateUserProcess bypasses IFEO so there's no infinite loop
pub fn launch_game(target: &str, args: &[String], injected_flags: &[String]) -> Result<String, String> {
    if target.is_empty() {
        return Err("Failed to start game: Empty target path".to_string());
    }

    // Validate that target is a file, not a directory
    let target_path = std::path::Path::new(target);
    if target_path.is_dir() {
        return Err(format!(
            "❌ Target is a directory, not an executable: {}\n\nPlease select the actual game executable file (e.g., stalcraft.exe)",
            target
        ));
    }

    if !target_path.exists() {
        return Err(format!("❌ Target executable does not exist: {}", target));
    }

    let file_size = std::fs::metadata(target).map(|m| m.len()).unwrap_or(0);
    eprintln!("[LAUNCH] ========== GAME LAUNCH START ==========");
    eprintln!("[LAUNCH] Target: {}", target);
    eprintln!("[LAUNCH] File size: {} bytes", file_size);
    eprintln!("[LAUNCH] Original args count: {}", args.len());
    eprintln!("[LAUNCH] Injected flags count: {}", injected_flags.len());
    eprintln!("[LAUNCH] Working directory: {}", target_path.parent().unwrap_or_else(|| std::path::Path::new(".")).display());

    // Filter args - remove old JVM flags and add optimized ones
    let final_args = if args.is_empty() {
        injected_flags.to_vec()
    } else {
        filter_args(args, injected_flags)
    };

    eprintln!("[LAUNCH] Final args count: {}", final_args.len());

    // Create phantom window to keep message pump alive
    create_phantom_window();

    // Launch via NtCreateUserProcess to bypass IFEO
    let (h_process, h_thread, pid) = match nt_create_process(target, &final_args) {
        Ok(result) => {
            eprintln!("[LAUNCH] Process created via NtCreateUserProcess with PID: {}", result.2);
            result
        }
        Err(e) => {
            eprintln!("[LAUNCH ERROR] NtCreateUserProcess failed: {}", e);
            return Err(format!("Failed to start game: {}", e));
        }
    };

    // Boost process priority
    boost_process(h_process);

    // Wait for game to show window or exit
    let exit_code = wait_process(h_process, pid);

    // Cleanup
    cleanup_handles(h_process, h_thread);

    // Exit code 0 means SUCCESS (process launched and exited cleanly, likely spawned child)
    // This matches Go behavior where run() returns waitProcess() directly
    if exit_code == 0 {
        eprintln!("[LAUNCH] Game process {} completed successfully (exit code: 0)", pid);
        eprintln!("[LAUNCH] If game didn't appear, it may have spawned a child process");
        Ok(format!("Game launched successfully (PID {}, exit code: 0)", pid))
    } else {
        eprintln!("[LAUNCH ERROR] Game exited with error code {}", exit_code);
        Err(format!("Game process {} exited before window appeared: exit code: {}", pid, exit_code))
    }
}
