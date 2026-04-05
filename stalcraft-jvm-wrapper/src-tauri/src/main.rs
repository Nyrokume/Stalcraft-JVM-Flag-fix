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
    
    // IFEO passes the original target executable as an argument
    // Format: wrapper.exe stalcraft.exe [original args...]
    let is_debugger_mode = args.len() > 1 && {
        let first_arg = args[1].to_lowercase();
        first_arg.contains("stalcraft.exe") || first_arg.contains("stalcraftw.exe")
    };

    if is_debugger_mode {
        // Debugger mode: launch game with optimizations and exit
        if let Err(e) = launch_as_debugger(&args) {
            eprintln!("[DEBUGGER] Error: {}", e);
            std::process::exit(1);
        }
        std::process::exit(0);
    }

    // Normal GUI mode
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            get_system_info,
            install_ifeo,
            uninstall_ifeo,
            check_status,
            launch_game
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

/// Launch the game when invoked as IFEO debugger
fn launch_as_debugger(args: &[String]) -> Result<(), String> {
    if args.len() < 2 {
        return Err("No target executable provided".to_string());
    }

    let target = &args[1];
    let original_args: Vec<String> = args[2..].to_vec();

    // Show ASCII art and info in console
    eprintln!();
    eprintln!(r#"   ____  _   _ _____ ____   ___  _     _    "#);
    eprintln!(r#"  / ___|| | | |_   _|  _ \ / _ \| |   | |   "#);
    eprintln!(r#"  \___ \| |_| | | | | |_) | | | | |   | |   "#);
    eprintln!(r#"   ___) |  _  | | | |  _ <| |_| | |___| |___"#);
    eprintln!(r#"  |____/|_| |_| |_| |_| \_\\___/|_____|_____|"#);
    eprintln!();
    eprintln!(r#"        ═══════════════════════════════      "#);
    eprintln!(r#"          JVM OPTIMIZATION WRAPPER v2.4.1    "#);
    eprintln!(r#"        ═══════════════════════════════      "#);
    eprintln!();
    eprintln!("[DEBUGGER] ═══════════════════════════════════════════");
    eprintln!("[DEBUGGER] IFEO Debugger Mode Activated");
    eprintln!("[DEBUGGER] ═══════════════════════════════════════════");
    eprintln!("[DEBUGGER] Target: {}", target);
    eprintln!("[DEBUGGER] Args count: {}", original_args.len());
    if !original_args.is_empty() {
        eprintln!("[DEBUGGER] Args: {:?}", original_args);
    }
    eprintln!("[DEBUGGER] ═══════════════════════════════════════════");

    // Validate target path
    if target.trim().is_empty() {
        return Err("Empty target path provided by IFEO".to_string());
    }

    // Detect system and generate optimized flags
    eprintln!("[DEBUGGER] Detecting system hardware...");
    let sys = system::detect_system();
    eprintln!("[DEBUGGER] CPU: {} ({} cores)", sys.cpu_name, sys.cpu_cores);
    eprintln!("[DEBUGGER] RAM: {:.2} GB total, {:.2} GB free", sys.total_ram_gb(), sys.free_ram_gb());

    let flags = jvm::generate_flags(&sys);
    eprintln!("[DEBUGGER] Generated {} optimized JVM flags", flags.len());

    // Show first few flags as preview
    if flags.len() > 0 {
        eprintln!("[DEBUGGER] Preview of JVM flags:");
        for (i, flag) in flags.iter().take(5).enumerate() {
            eprintln!("[DEBUGGER]   [{}] {}", i + 1, flag);
        }
        if flags.len() > 5 {
            eprintln!("[DEBUGGER]   ... and {} more", flags.len() - 5);
        }
    }

    // Launch the game with optimizations
    eprintln!("[DEBUGGER] Launching game process...");
    let result = process::launch_game(target, &original_args, &flags)?;
    eprintln!("[DEBUGGER] ✓ {}", result);
    eprintln!("[DEBUGGER] ═══════════════════════════════════════════");
    eprintln!("[DEBUGGER] Debugger mode completed successfully");
    eprintln!("[DEBUGGER] Game is running with optimized JVM parameters");
    eprintln!();

    Ok(())
}
