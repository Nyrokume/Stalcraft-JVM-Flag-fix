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
            select_config,
            regenerate_config,
            get_active_config,
            read_wrapper_log_tail,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
