// main.rs — GUI + IFEO debugger service mode

#![windows_subsystem = "windows"]

mod system;
mod config;
mod jvm;
mod ifeo;
mod log;
mod process;
mod commands;

use std::env;
use commands::*;

fn main() {
    let args: Vec<String> = env::args().collect();

    let is_debugger_mode = args.len() >= 2 && ifeo::is_ifeo_debugger_invocation(&args[1]);

    if is_debugger_mode {
        eprintln!("[service] startup, args={}", args.len() - 1);
        eprintln!("[service] target={}", log::redact_path(&args[1]));
        let code = run_as_debugger(&args);
        std::process::exit(code);
    }

    if let Some(flag) = args.get(1) {
        match flag.as_str() {
            "--install" => {
                match ifeo::install(None) {
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

    eprintln!("[gui] starting");
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            get_system_info,
            install_ifeo,
            uninstall_ifeo,
            check_status,
            list_configs,
            select_config,
            regenerate_config,
            get_active_config,
            load_config_by_name,
            save_config,
            read_wrapper_log_tail,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn run_as_debugger(args: &[String]) -> i32 {
    let target = &args[1];
    let orig_args = &args[2..];
    let target_redacted = log::redact_path(target);

    let sys = system::detect_system();
    eprintln!(
        "[service] system: {} cores ({} threads), L3={}MB, mem={}MT/s ({}), {:.2}GB RAM, large_pages={}",
        sys.cpu_cores,
        sys.cpu_threads,
        sys.l3_cache_mb,
        sys.mem_speed_mts,
        sys.mem_tier().as_str(),
        sys.total_ram_gb(),
        sys.large_pages
    );
    log::append_wrapper_log_line(&format!(
        "service_startup cores={} threads={} ram_gb={} mem_mts={} mem_tier={} l3_mb={} large_pages={} target={}",
        sys.cpu_cores,
        sys.cpu_threads,
        sys.total_gb(),
        sys.mem_speed_mts,
        sys.mem_tier().as_str(),
        sys.l3_cache_mb,
        sys.large_pages,
        target_redacted
    ));

    if let Err(e) = config::ensure(&sys) {
        eprintln!("[service] config ensure failed: {}", e);
        log::append_wrapper_log_line(&format!("config_ensure_warn err={}", e));
    }

    let mut final_args = orig_args.to_vec();
    let jvm_mode;

    match config::load_active() {
        Err(e) => {
            eprintln!("[service] config load failed, keeping original args: {}", e);
            log::append_wrapper_log_line(&format!(
                "jvm_mode=CONFIG_ERROR err={} target={}",
                e, target_redacted
            ));
            jvm_mode = "CONFIG_ERROR";
        }
        Ok((cfg, loaded_name)) => {
            if cfg.heap_size_gb == 0 {
                eprintln!("[service] heap=0, skipping flag injection (config: {})", loaded_name);
                log::append_wrapper_log_line(&format!(
                    "jvm_mode=NO_HEAP profile={} target={}",
                    loaded_name, target_redacted
                ));
                jvm_mode = "NO_HEAP";
            } else {
                let flags = jvm::flags(&cfg);
                let n = flags.len();
                eprintln!(
                    "[service] config={}, heap={}GB, GC={}/{}, flags={}",
                    loaded_name,
                    cfg.heap_size_gb,
                    cfg.parallel_gc_threads,
                    cfg.conc_gc_threads,
                    n
                );
                log::append_wrapper_log_line(&format!(
                    "jvm_mode=INJECTED profile={} heap_gb={} parallel_gc={} conc_gc={} region_mb={} pause_ms={} flags_count={} target={}",
                    loaded_name,
                    cfg.heap_size_gb,
                    cfg.parallel_gc_threads,
                    cfg.conc_gc_threads,
                    cfg.g1_heap_region_size_mb,
                    cfg.max_gc_pause_millis,
                    n,
                    target_redacted
                ));
                final_args = jvm::filter_args(orig_args, &flags);
                jvm_mode = "INJECTED";
            }
        }
    }

    eprintln!(
        "[service] starting process, exe={}, arg_count={}",
        target_redacted,
        final_args.len()
    );

    process::start_phantom_window();

    let (h_process, h_thread, pid) = match process::nt_create_process(target, &final_args) {
        Ok(r) => {
            eprintln!("[service] process started, pid={}", r.2);
            log::append_wrapper_log_line(&format!(
                "spawn_ok pid={} jvm_mode={} arg_count={}",
                r.2, jvm_mode, final_args.len()
            ));
            r
        }
        Err(e) => {
            eprintln!("[service] process start failed: {}", e);
            log::append_wrapper_log_line(&format!(
                "spawn_fail jvm_mode={} err={} target={}",
                jvm_mode, e, target_redacted
            ));
            return 1;
        }
    };

    process::boost_process(h_process);

    let exit_code = process::wait_process(h_process, pid);
    process::cleanup_handles(h_process, h_thread);

    log::append_wrapper_log_line(&format!(
        "child_exit pid={} jvm_mode={} exit_code={}",
        pid, jvm_mode, exit_code
    ));
    eprintln!("[service] exit, code={}", exit_code);
    exit_code
}
