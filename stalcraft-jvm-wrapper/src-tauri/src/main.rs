// GUI + installer (EXBO cli.exe parity)

#![windows_subsystem = "windows"]

use stalcraft_jvm_wrapper::commands::*;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if let Some(flag) = args.get(1) {
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
