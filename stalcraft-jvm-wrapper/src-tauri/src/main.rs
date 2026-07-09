// GUI + installer (EXBO cli.exe parity)

#![windows_subsystem = "windows"]

use stalcraft_jvm_wrapper::commands::*;

fn attach_parent_console() {
    #[link(name = "kernel32")]
    extern "system" {
        fn AttachConsole(process_id: u32) -> i32;
        fn AllocConsole() -> i32;
    }
    const ATTACH_PARENT_PROCESS: u32 = 0xFFFF_FFFF;
    unsafe {
        if AttachConsole(ATTACH_PARENT_PROCESS) == 0 {
            AllocConsole();
        }
    }
}

fn cli_print(msg: &str) {
    use std::io::Write;
    let _ = writeln!(std::io::stdout(), "{msg}");
    let _ = std::io::stdout().flush();
}

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if let Some(flag) = args.get(1) {
        if flag.starts_with("--") {
            attach_parent_console();
        }
        match flag.as_str() {
            "--install" => {
                match stalcraft_jvm_wrapper::ifeo::install(None) {
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
                match stalcraft_jvm_wrapper::ifeo::uninstall() {
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
                match stalcraft_jvm_wrapper::ifeo::status() {
                    Ok(s) => eprintln!("[status] {}", s),
                    Err(e) => eprintln!("[status] error: {}", e),
                }
                std::process::exit(0);
            }
            "--sb-apply" => {
                let path = match args.get(2) {
                    Some(p) => p,
                    None => {
                        eprintln!("[sb-apply] missing path");
                        std::process::exit(1);
                    }
                };
                let data = match std::fs::read_to_string(path) {
                    Ok(d) => d,
                    Err(e) => {
                        eprintln!("[sb-apply] read failed: {}", e);
                        std::process::exit(1);
                    }
                };
                let ips: Vec<String> = match serde_json::from_str(&data) {
                    Ok(v) => v,
                    Err(e) => {
                        eprintln!("[sb-apply] parse failed: {}", e);
                        std::process::exit(1);
                    }
                };
                match stalcraft_jvm_wrapper::server_block::apply_blocks(&ips) {
                    Ok(msg) => {
                        eprintln!("[sb-apply] {}", msg);
                        std::process::exit(0);
                    }
                    Err(e) => {
                        eprintln!("[sb-apply] failed: {}", e);
                        std::process::exit(1);
                    }
                }
            }
            "--sb-clear" => {
                match stalcraft_jvm_wrapper::server_block::clear_rules() {
                    Ok(n) => {
                        eprintln!("[sb-clear] removed {} rule(s)", n);
                        std::process::exit(0);
                    }
                    Err(e) => {
                        eprintln!("[sb-clear] failed: {}", e);
                        std::process::exit(1);
                    }
                }
            }
            "--probe-path" => {
                let path = match args.get(2) {
                    Some(p) => p,
                    None => {
                        cli_print("[probe-path] missing path");
                        std::process::exit(1);
                    }
                };
                let layout = stalcraft_jvm_wrapper::paths::classify_target(path);
                let lines = [
                    format!("[probe-path] path={path}"),
                    format!("[probe-path] launcher={}", layout.kind.as_str()),
                    format!(
                        "[probe-path] scope={}",
                        stalcraft_jvm_wrapper::paths::scope_label(path)
                    ),
                    format!(
                        "[probe-path] target_kind={}",
                        stalcraft_jvm_wrapper::paths::target_kind(path)
                    ),
                    format!(
                        "[probe-path] should_inject={}",
                        stalcraft_jvm_wrapper::paths::should_inject_jvm(path)
                    ),
                    format!(
                        "[probe-path] wrapper_home={}",
                        stalcraft_jvm_wrapper::paths::wrapper_home().display()
                    ),
                ];
                for line in &lines {
                    cli_print(line);
                }
                let log_dir = stalcraft_jvm_wrapper::paths::logs_dir();
                let _ = std::fs::create_dir_all(&log_dir);
                let _ = std::fs::write(log_dir.join("probe-last.txt"), lines.join("\n"));
                std::process::exit(0);
            }
            _ => {}
        }
    }

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            get_system_info,
            install_ifeo,
            uninstall_ifeo,
            check_status,
            list_configs,
            list_examples,
            import_example_config,
            select_config,
            regenerate_config,
            get_active_config,
            read_wrapper_log_tail,
            ping_servers,
            start_server_blocking,
            stop_server_blocking,
            server_blocking_active,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
