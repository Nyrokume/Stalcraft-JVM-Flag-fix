// service.exe — IFEO debugger binary (EXBO cmd/service/main.go)
#![windows_subsystem = "windows"]

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        stalcraft_jvm_wrapper::log::append_wrapper_log_line("service_error missing_target");
        std::process::exit(1);
    }

    stalcraft_jvm_wrapper::log::append_wrapper_log_line(&format!(
        "service_invoked argc={} target={} inject={}",
        args.len() - 1,
        stalcraft_jvm_wrapper::log::redact_path(&args[1]),
        stalcraft_jvm_wrapper::ifeo::should_inject_jvm(&args[1])
    ));

    std::process::exit(stalcraft_jvm_wrapper::run_service_mode(&args));
}
