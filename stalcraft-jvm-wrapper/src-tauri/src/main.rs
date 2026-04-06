// Always use windows subsystem (no console window)
#![windows_subsystem = "windows"]

mod system;
mod jvm;
mod ifeo;
mod process;
mod commands;

use std::env;
use commands::*;

fn main() {
    // Check if we're being launched by IFEO (as a debugger for stalcraft.exe)
    let args: Vec<String> = env::args().collect();

    // IFEO passes the original target executable as the first argument
    // Format: wrapper.exe stalcraft.exe [original args...]
    // This matches the Go implementation exactly
    let is_debugger_mode = args.len() >= 2 && {
        let first_arg = args[1].to_lowercase();
        first_arg.contains("stalcraft.exe") || first_arg.contains("stalcraftw.exe")
    };

    if is_debugger_mode {
        // Debugger mode: launch game with optimizations and exit
        // This matches the Go run() function
        eprintln!("[DEBUGGER] Starting in debugger mode");
        eprintln!("[DEBUGGER] Target: {}", args[1]);
        eprintln!("[DEBUGGER] Args count: {}", args.len() - 2);
        
        let result = run_as_debugger(&args);
        std::process::exit(result);
    }

    // Normal GUI mode
    eprintln!("[GUI] Starting in GUI mode");
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_store::Builder::new().build())
        .invoke_handler(tauri::generate_handler![
            get_system_info,
            install_ifeo,
            uninstall_ifeo,
            check_status,
            launch_game,
            save_game_dir,
            load_game_dir
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

/// Run as debugger - matches Go run() function exactly
fn run_as_debugger(args: &[String]) -> i32 {
    let sys = system::detect_system();
    
    eprintln!("[DEBUGGER] System detected: {} CPU, {:.2}GB RAM", sys.cpu_name, sys.total_ram_gb());

    // Calculate heap - if 0, use original args without modification
    let heap = jvm::calc_heap(&sys);
    eprintln!("[DEBUGGER] Calculated heap size: {}GB", heap);

    let final_args = if heap == 0 {
        // Use original args as-is (args[2..])
        eprintln!("[DEBUGGER] Using original arguments (heap=0)");
        args[2..].to_vec()
    } else {
        // Filter args and inject optimized flags
        let flags = jvm::generate_flags(&sys);
        eprintln!("[DEBUGGER] Generated {} JVM flags", flags.len());
        eprintln!("[DEBUGGER] Filtering arguments and injecting optimized flags");
        process::filter_args(&args[2..], &flags)
    };

    // Launch via NtCreateUserProcess (bypasses IFEO)
    let (h_process, h_thread, pid) = match process::nt_create_process(&args[1], &final_args) {
        Ok(result) => {
            eprintln!("[DEBUGGER] Process created with PID: {}", result.2);
            result
        }
        Err(e) => {
            eprintln!("[DEBUGGER ERROR] Failed to create process: {}", e);
            return 1;
        }
    };

    // Boost process (must be before cleanup)
    process::boost_process(h_process);

    // Wait for process
    let exit_code = process::wait_process(h_process, pid);
    
    // Cleanup handles (like Go's defer)
    process::cleanup_handles(h_process, h_thread);
    
    eprintln!("[DEBUGGER] Process exited with code: {}", exit_code);
    exit_code
}
