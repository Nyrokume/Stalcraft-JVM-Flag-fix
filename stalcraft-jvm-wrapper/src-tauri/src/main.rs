// main.rs — полный порт cmd/service/main.go + cmd/cli/main.go
// Два режима: GUI (обычный запуск) и Debugger (IFEO перехват).
// В debugger mode: загружает активный конфиг, фильтрует аргументы,
// запускает игру через NtCreateUserProcess, бустит приоритеты.

#![windows_subsystem = "windows"]

mod system;
mod config;
mod jvm;
mod ifeo;
mod process;
mod commands;

use std::env;
use commands::*;

fn main() {
    let args: Vec<String> = env::args().collect();

    // ─── Debugger mode (IFEO перехват) ───────────────────────────────────────
    // Windows запускает: wrapper.exe stalcraft.exe [оригинальные аргументы...]
    let is_debugger_mode = args.len() >= 2 && {
        let first = args[1].to_lowercase();
        first.contains("stalcraft.exe") || first.contains("stalcraftw.exe")
    };

    if is_debugger_mode {
        eprintln!("[service] startup, args={}", args.len() - 1);
        eprintln!("[service] target={}", args[1]);
        let code = run_as_debugger(&args);
        std::process::exit(code);
    }

    // ─── CLI флаги (--install, --uninstall, --status) ─────────────────────────
    if let Some(flag) = args.get(1) {
        match flag.as_str() {
            "--install" => {
                match ifeo::install() {
                    Ok(msg) => {
                        eprintln!("[install] {}", msg);
                        std::process::exit(0);
                    }
                    Err(e) => {
                        eprintln!("[install] failed: {}", e);
                        std::process::exit(1);
                    }
                }
            }
            "--uninstall" => {
                match ifeo::uninstall() {
                    Ok(msg) => {
                        eprintln!("[uninstall] {}", msg);
                        std::process::exit(0);
                    }
                    Err(e) => {
                        eprintln!("[uninstall] failed: {}", e);
                        std::process::exit(1);
                    }
                }
            }
            "--status" => {
                match ifeo::status() {
                    Ok(s) => eprintln!("[status] {}", s),
                    Err(e) => eprintln!("[status] error: {}", e),
                }
                std::process::exit(0);
            }
            _ => {}
        }
    }

    // ─── GUI mode (Tauri) ─────────────────────────────────────────────────────
    eprintln!("[gui] starting");
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
            list_configs,
            select_config,
            regenerate_config,
            apply_config_preset,
            get_active_config,
            load_config_by_name,
            save_config,
            save_game_dir,
            load_game_dir,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

/// run_as_debugger — точный порт функции launch() из cmd/service/main.go
fn run_as_debugger(args: &[String]) -> i32 {
    let target = &args[1];
    let orig_args = &args[2..];

    let sys = system::detect_system();
    eprintln!(
        "[service] system: {} cores ({} threads), L3={}MB, {:.2}GB RAM ({:.2}GB free), big_cache={}",
        sys.cpu_cores, sys.cpu_threads, sys.l3_cache_mb,
        sys.total_ram_gb(), sys.free_ram_gb(),
        sys.has_big_cache()
    );

    // ensure() создаёт configs/ и default.json если нужно
    if let Err(e) = config::ensure(&sys) {
        eprintln!("[service] config ensure failed: {}", e);
    }

    // Загружаем активный конфиг (LoadActive из Go)
    let final_args = match config::load_active() {
        Err(e) => {
            eprintln!("[service] config load failed, keeping original args: {}", e);
            orig_args.to_vec()
        }
        Ok((cfg, loaded_name)) => {
            if cfg.heap_size_gb == 0 {
                eprintln!("[service] heap=0, skipping flag injection (config: {})", loaded_name);
                orig_args.to_vec()
            } else {
                let flags = jvm::flags(&cfg);
                eprintln!(
                    "[service] config={}, heap={}GB, GC={}/{}, l3={}MB, big_cache={}, flags={}",
                    loaded_name, cfg.heap_size_gb,
                    cfg.parallel_gc_threads, cfg.conc_gc_threads,
                    sys.l3_cache_mb, sys.has_big_cache(),
                    flags.len()
                );
                jvm::filter_args(orig_args, &flags)
            }
        }
    };

    eprintln!("[service] starting process, exe={}, arg_count={}", target, final_args.len());

    // phantom window — как в Go
    process::start_phantom_window();

    // NtCreateUserProcess (Start() из process.go)
    let (h_process, h_thread, pid) = match process::nt_create_process(target, &final_args) {
        Ok(r) => {
            eprintln!("[service] process started, pid={}", r.2);
            r
        }
        Err(e) => {
            eprintln!("[service] process start failed: {}", e);
            return 1;
        }
    };

    // Boost (Process.Boost() из process.go)
    process::boost_process(h_process);

    // Wait (Process.Wait() из process.go)
    let exit_code = process::wait_process(h_process, pid);
    process::cleanup_handles(h_process, h_thread);

    eprintln!("[service] exit, code={}", exit_code);
    exit_code
}
