use std::process::Command;
use std::path::PathBuf;
use std::os::windows::process::CommandExt;

const HIGH_PRIORITY_CLASS: u32 = 0x00000080;
const CREATE_NO_WINDOW: u32 = 0x08000000;
const PROCESS_MEMORY_PRIORITY: u32 = 0x27;
const PROCESS_IO_PRIORITY: u32 = 0x21;
const MEMORY_PRIORITY_NORMAL: u32 = 5;
const IO_PRIORITY_HIGH: u32 = 3;

// Manual DLL imports for functions not in windows-sys
mod winapi {
    #[link(name = "kernel32")]
    extern "system" {
        pub fn SetProcessPriorityBoost(hprocess: isize, disabled: i32) -> i32;
        pub fn OpenProcess(desired_access: u32, inherit_handle: i32, process_id: u32) -> isize;
    }
    
    #[link(name = "ntdll")]
    extern "system" {
        pub fn NtSetInformationProcess(
            process_handle: isize,
            process_information_class: u32,
            process_information: *const std::ffi::c_void,
            process_information_length: u32,
        ) -> i32;
    }
}

const PROCESS_SET_INFORMATION: u32 = 0x0200;
const PROCESS_QUERY_INFORMATION: u32 = 0x0400;

pub fn resolve_target(target: &str) -> String {
    eprintln!("[resolve_target] Input: {}", target);
    
    // Validate path is not empty
    if target.trim().is_empty() {
        eprintln!("[resolve_target] Empty target provided");
        return target.to_string();
    }
    
    let path = PathBuf::from(target);
    
    // Get the base filename
    let base = path.file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("")
        .to_lowercase();

    let dir = path.parent().unwrap_or_else(|| std::path::Path::new(""));

    eprintln!("[resolve_target] Base executable: {}", base);
    eprintln!("[resolve_target] Directory: {}", dir.display());

    // Always use javaw.exe to avoid console window
    let java_exe = if base == "stalcraftw.exe" || base == "stalcraft.exe" {
        "javaw.exe"
    } else {
        // Not a stalcraft executable, use as-is
        eprintln!("[resolve_target] No substitution needed for: {}", base);
        return target.to_string();
    };

    let java_path = dir.join(java_exe);
    eprintln!("[resolve_target] Looking for Java executable at: {}", java_path.display());
    
    if java_path.exists() {
        let java_path_str = java_path.to_string_lossy().to_string();
        eprintln!("[resolve_target] ✓ Found Java executable: {}", java_path_str);
        java_path_str
    } else {
        eprintln!("[resolve_target] ✗ Java executable not found, using original target");
        target.to_string()
    }
}

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

fn filter_args(orig: &[String], injected: &[String]) -> Vec<String> {
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

pub fn boost_process(pid: u32) {
    unsafe {
        let handle = winapi::OpenProcess(
            PROCESS_SET_INFORMATION | PROCESS_QUERY_INFORMATION,
            0,
            pid,
        );
        
        if handle == 0 {
            return;
        }
        
        winapi::SetProcessPriorityBoost(handle, 1);
        
        let mem = MEMORY_PRIORITY_NORMAL;
        winapi::NtSetInformationProcess(
            handle,
            PROCESS_MEMORY_PRIORITY,
            &mem as *const _ as *const _,
            std::mem::size_of::<u32>() as u32,
        );
        
        let iop = IO_PRIORITY_HIGH;
        winapi::NtSetInformationProcess(
            handle,
            PROCESS_IO_PRIORITY,
            &iop as *const _ as *const _,
            std::mem::size_of::<u32>() as u32,
        );
    }
}

pub fn launch_game(target: &str, args: &[String], injected_flags: &[String]) -> Result<String, String> {
    // Validate target path
    if target.is_empty() {
        return Err("Failed to start game: Empty target path".to_string());
    }

    // Log the target for debugging
    eprintln!("[launch_game] Target: {}", target);
    
    let resolved_target = resolve_target(target);
    eprintln!("[launch_game] Resolved target: {}", resolved_target);
    
    // Validate resolved target
    if resolved_target.is_empty() {
        return Err("Failed to start game: Resolved target is empty".to_string());
    }

    let final_args = if args.is_empty() {
        injected_flags.iter().map(|s| s.clone()).collect()
    } else {
        filter_args(args, injected_flags)
    };

    eprintln!("[launch_game] Args: {:?}", final_args);

    let mut cmd = Command::new(&resolved_target);
    cmd.args(&final_args);
    // Combine flags: HIGH_PRIORITY_CLASS | CREATE_NO_WINDOW
    cmd.creation_flags(HIGH_PRIORITY_CLASS | CREATE_NO_WINDOW);

    let child = cmd.spawn()
        .map_err(|e| {
            let error_msg = format!("Failed to start game: {} (target: {})", e, resolved_target);
            eprintln!("[launch_game] Error: {}", error_msg);
            error_msg
        })?;

    let pid = child.id();
    boost_process(pid);

    Ok(format!("Game started with PID {}", pid))
}
