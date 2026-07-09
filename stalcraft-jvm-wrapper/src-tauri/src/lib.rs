// Shared library — service.exe + GUI binary (EXBO cli/service split)

pub mod config;
pub mod elevate;
pub mod ifeo;
pub mod jvm;
pub mod log;
pub mod paths;
pub mod ping;
pub mod process;
pub mod server_block;
pub mod system;

#[cfg(feature = "gui")]
pub mod commands;

/// IFEO service entry — port of cmd/service/main.go
pub fn run_service_mode(args: &[String]) -> i32 {
    process::start_phantom_window();
    launch(&args[1], &args[2..])
}

/// EXBO launch(): filter JVM flags when config is valid, then NtCreateUserProcess.
fn launch(target: &str, orig_args: &[String]) -> i32 {
    let target_redacted = log::redact_path(target);
    let inject = paths::should_inject_jvm(target);

    log::append_wrapper_log_line(&format!(
        "service_startup argc={} target={} inject={}",
        orig_args.len() + 1,
        target_redacted,
        inject
    ));

    let sys = system::detect_system();
    log::append_wrapper_log_line(&format!(
        "system cores={} threads={} ram_gb={} mem_mts={} mem_tier={} l3_mb={} large_pages={}",
        sys.cpu_cores,
        sys.cpu_threads,
        sys.total_gb(),
        sys.mem_speed_mts,
        sys.mem_tier().as_str(),
        sys.l3_cache_mb,
        sys.large_pages
    ));

    if let Err(e) = config::ensure(&sys) {
        log::append_wrapper_log_line(&format!("config_ensure_warn err={}", e));
    }

    let mut final_args = orig_args.to_vec();
    let jvm_mode;

    if !inject {
        log::append_wrapper_log_line(&format!(
            "jvm_mode=PASSTHROUGH reason=non_game_java target={} arg_count={}",
            target_redacted,
            orig_args.len()
        ));
        jvm_mode = "PASSTHROUGH";
    } else {
        let requested = config::active_name();
        match config::load_active() {
            Err(e) => {
                log::append_wrapper_log_line(&format!(
                    "jvm_mode=CONFIG_ERROR err={} target={}",
                    e, target_redacted
                ));
                jvm_mode = "CONFIG_ERROR";
            }
            Ok((cfg, loaded_name)) => {
                if let Some(ref req) = requested {
                    if req != &loaded_name {
                        log::append_wrapper_log_line(&format!(
                            "config_fallback requested={} loaded={}",
                            req, loaded_name
                        ));
                    }
                }
                if cfg.heap_size_gb == 0 {
                    log::append_wrapper_log_line(&format!(
                        "jvm_mode=NO_HEAP profile={} target={}",
                        loaded_name, target_redacted
                    ));
                    jvm_mode = "NO_HEAP";
                } else {
                    let flags = jvm::flags(&cfg);
                    log::append_wrapper_log_line(&format!(
                        "jvm_mode=INJECTED profile={} heap_gb={} parallel_gc={} conc_gc={} region_mb={} pause_ms={} flags_count={} target={}",
                        loaded_name,
                        cfg.heap_size_gb,
                        cfg.parallel_gc_threads,
                        cfg.conc_gc_threads,
                        cfg.g1_heap_region_size_mb,
                        cfg.max_gc_pause_millis,
                        flags.len(),
                        target_redacted
                    ));
                    final_args = jvm::filter_args(orig_args, &flags);
                    jvm_mode = "INJECTED";
                }
            }
        }
    }

    spawn_child(target, &final_args, jvm_mode)
}

fn spawn_child(target: &str, args: &[String], jvm_mode: &str) -> i32 {
    let target_redacted = log::redact_path(target);

    let (h_process, h_thread, pid) = match process::nt_create_process(target, args) {
        Ok(r) => {
            log::append_wrapper_log_line(&format!(
                "spawn_ok pid={} jvm_mode={} arg_count={}",
                r.2,
                jvm_mode,
                args.len()
            ));
            r
        }
        Err(e) => {
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
    exit_code
}
