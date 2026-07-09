// service.exe — IFEO debugger binary (EXBO cmd/service/main.go)
#![windows_subsystem = "windows"]

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        stalcraft_jvm_wrapper::log::append_wrapper_log_line("service_error missing_target");
        std::process::exit(1);
    }

    let target = &args[1];
    stalcraft_jvm_wrapper::log::append_wrapper_log_line(&format!(
        "service_invoked argc={} target={} launcher={} scope={} inject={} target_kind={}",
        args.len() - 1,
        stalcraft_jvm_wrapper::log::redact_path(target),
        stalcraft_jvm_wrapper::paths::classify_target(target).kind.as_str(),
        stalcraft_jvm_wrapper::paths::scope_label(target),
        stalcraft_jvm_wrapper::paths::should_inject_jvm(target),
        stalcraft_jvm_wrapper::paths::target_kind(target),
    ));

    std::process::exit(stalcraft_jvm_wrapper::run_service_mode(&args));
}
