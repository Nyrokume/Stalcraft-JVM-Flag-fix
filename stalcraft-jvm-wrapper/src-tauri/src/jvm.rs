// jvm.rs — порт flags.go + filter.go

use crate::config::Config;

pub fn flags(cfg: &Config) -> Vec<String> {
    let cc = if cfg.reserved_code_cache_size_mb == 0 {
        256
    } else {
        cfg.reserved_code_cache_size_mb
    };

    let mut f: Vec<String> = vec![
        format!("-Xmx{}g", cfg.heap_size_gb),
        format!("-Xms{}g", cfg.heap_size_gb),
        format!("-XX:MetaspaceSize={}m", cfg.metaspace_mb),
        format!("-XX:MaxMetaspaceSize={}m", cfg.metaspace_mb),
        "-XX:+UseG1GC".to_string(),
        "-XX:+UnlockExperimentalVMOptions".to_string(),
        format!("-XX:MaxGCPauseMillis={}", cfg.max_gc_pause_millis),
        format!("-XX:G1HeapRegionSize={}m", cfg.g1_heap_region_size_mb),
        format!("-XX:G1NewSizePercent={}", cfg.g1_new_size_percent),
        format!("-XX:G1MaxNewSizePercent={}", cfg.g1_max_new_size_percent),
        format!("-XX:G1ReservePercent={}", cfg.g1_reserve_percent),
        format!("-XX:G1HeapWastePercent={}", cfg.g1_heap_waste_percent),
        format!(
            "-XX:G1MixedGCCountTarget={}",
            cfg.g1_mixed_gc_count_target
        ),
        "-XX:+G1UseAdaptiveIHOP".to_string(),
        format!(
            "-XX:InitiatingHeapOccupancyPercent={}",
            cfg.initiating_heap_occupancy_percent
        ),
        format!(
            "-XX:G1MixedGCLiveThresholdPercent={}",
            cfg.g1_mixed_gc_live_threshold_percent
        ),
        format!(
            "-XX:G1RSetUpdatingPauseTimePercent={}",
            cfg.g1_rset_updating_pause_time_percent
        ),
        format!("-XX:SurvivorRatio={}", cfg.survivor_ratio),
        format!("-XX:MaxTenuringThreshold={}", cfg.max_tenuring_threshold),
        format!("-XX:ParallelGCThreads={}", cfg.parallel_gc_threads),
        format!("-XX:ConcGCThreads={}", cfg.conc_gc_threads),
        "-XX:+ParallelRefProcEnabled".to_string(),
        "-XX:+DisableExplicitGC".to_string(),
        format!(
            "-XX:SoftRefLRUPolicyMSPerMB={}",
            cfg.soft_ref_lru_policy_ms_per_mb
        ),
        "-XX:-UseBiasedLocking".to_string(),
        "-XX:+DisableAttachMechanism".to_string(),
        format!("-XX:ReservedCodeCacheSize={}m", cc),
        format!("-XX:NonNMethodCodeHeapSize={}m", cc * 5 / 100),
        format!("-XX:ProfiledCodeHeapSize={}m", cc * 48 / 100),
        format!(
            "-XX:NonProfiledCodeHeapSize={}m",
            cc - cc * 5 / 100 - cc * 48 / 100
        ),
        format!("-XX:MaxInlineLevel={}", cfg.max_inline_level),
        format!("-XX:FreqInlineSize={}", cfg.freq_inline_size),
        "-Djdk.nio.maxCachedBufferSize=131072".to_string(),
    ];

    if cfg.pre_touch {
        f.push("-XX:+AlwaysPreTouch".to_string());
    }
    if cfg.g1_satb_buffer_enqueuing_threshold_percent > 0 {
        f.push(format!(
            "-XX:G1SATBBufferEnqueueingThresholdPercent={}",
            cfg.g1_satb_buffer_enqueuing_threshold_percent
        ));
    }
    if cfg.g1_conc_rs_hot_card_limit > 0 {
        f.push(format!(
            "-XX:G1ConcRSHotCardLimit={}",
            cfg.g1_conc_rs_hot_card_limit
        ));
    }
    if cfg.g1_conc_refinement_service_interval_millis > 0 {
        f.push(format!(
            "-XX:G1ConcRefinementServiceIntervalMillis={}",
            cfg.g1_conc_refinement_service_interval_millis
        ));
    }
    if cfg.gc_time_ratio > 0 {
        f.push(format!("-XX:GCTimeRatio={}", cfg.gc_time_ratio));
    }
    if cfg.use_dynamic_number_of_gc_threads {
        f.push("-XX:+UseDynamicNumberOfGCThreads".to_string());
    }
    if cfg.use_string_deduplication {
        f.push("-XX:+UseStringDeduplication".to_string());
    }
    if cfg.inline_small_code > 0 {
        f.push(format!("-XX:InlineSmallCode={}", cfg.inline_small_code));
    }
    if cfg.max_node_limit > 0 && cfg.node_limit_fudge_factor > 0 {
        f.push(format!(
            "-XX:NodeLimitFudgeFactor={}",
            cfg.node_limit_fudge_factor
        ));
        f.push(format!("-XX:MaxNodeLimit={}", cfg.max_node_limit));
    }
    if cfg.nmethod_sweep_activity > 0 {
        f.push(format!(
            "-XX:NmethodSweepActivity={}",
            cfg.nmethod_sweep_activity
        ));
    }
    if !cfg.dont_compile_huge_methods {
        f.push("-XX:-DontCompileHugeMethods".to_string());
    }
    if cfg.allocate_prefetch_style > 0 {
        f.push(format!(
            "-XX:AllocatePrefetchStyle={}",
            cfg.allocate_prefetch_style
        ));
    }
    if cfg.always_act_as_server_class {
        f.push("-XX:+AlwaysActAsServerClassMachine".to_string());
    }
    if cfg.use_xmm_for_array_copy {
        f.push("-XX:+UseXMMForArrayCopy".to_string());
    }
    if cfg.use_fpu_for_spilling {
        f.push("-XX:+UseFPUForSpilling".to_string());
    }
    if cfg.use_large_pages {
        f.push("-XX:+UseLargePages".to_string());
    }

    f.push(format!(
        "-Dsun.reflect.inflationThreshold={}",
        cfg.reflection_inflation_threshold
    ));

    if cfg.auto_box_cache_max > 0 {
        f.push(format!("-XX:AutoBoxCacheMax={}", cfg.auto_box_cache_max));
    }
    if cfg.use_thread_priorities {
        f.push("-XX:+UseThreadPriorities".to_string());
        if cfg.thread_priority_policy > 0 {
            f.push(format!(
                "-XX:ThreadPriorityPolicy={}",
                cfg.thread_priority_policy
            ));
        }
    }
    if !cfg.use_counter_decay {
        f.push("-XX:-UseCounterDecay".to_string());
    }
    if cfg.compile_threshold_scaling > 0.0 && (cfg.compile_threshold_scaling - 1.0).abs() > 1e-9 {
        f.push(format!(
            "-XX:CompileThresholdScaling={}",
            cfg.compile_threshold_scaling
        ));
    }

    f
}

static EXACT_REMOVE: &[&str] = &[
    "-XX:-PrintCommandLineFlags",
    "-XX:+UseG1GC",
    "-XX:+UseCompressedOops",
    "-XX:+PerfDisableSharedMem",
    "-XX:+UseBiasedLocking",
    "-XX:-UseBiasedLocking",
    "-XX:+UseStringDeduplication",
    "-XX:+UseNUMA",
    "-XX:+DisableAttachMechanism",
    "-XX:+UseDynamicNumberOfGCThreads",
    "-XX:+AlwaysActAsServerClassMachine",
    "-XX:+UseXMMForArrayCopy",
    "-XX:+UseFPUForSpilling",
    "-XX:-DontCompileHugeMethods",
    "-XX:+AlwaysPreTouch",
    "-XX:+ParallelRefProcEnabled",
    "-XX:+DisableExplicitGC",
    "-XX:+G1UseAdaptiveIHOP",
    "-XX:+UnlockExperimentalVMOptions",
    "-XX:+UseThreadPriorities",
    "-XX:-UseThreadPriorities",
    "-XX:+UseCounterDecay",
    "-XX:-UseCounterDecay",
    "-XX:+UseLargePages",
    "-XX:-UseLargePages",
];

static PREFIX_REMOVE: &[&str] = &[
    "-XX:MaxGCPauseMillis=",
    "-XX:MetaspaceSize=",
    "-XX:MaxMetaspaceSize=",
    "-XX:G1HeapRegionSize=",
    "-XX:G1NewSizePercent=",
    "-XX:G1MaxNewSizePercent=",
    "-XX:G1ReservePercent=",
    "-XX:G1HeapWastePercent=",
    "-XX:G1MixedGCCountTarget=",
    "-XX:InitiatingHeapOccupancyPercent=",
    "-XX:G1MixedGCLiveThresholdPercent=",
    "-XX:G1RSetUpdatingPauseTimePercent=",
    "-XX:G1SATBBufferEnqueueingThresholdPercent=",
    "-XX:G1ConcRSHotCardLimit=",
    "-XX:G1ConcRefinementServiceIntervalMillis=",
    "-XX:GCTimeRatio=",
    "-XX:SurvivorRatio=",
    "-XX:MaxTenuringThreshold=",
    "-XX:ParallelGCThreads=",
    "-XX:ConcGCThreads=",
    "-XX:SoftRefLRUPolicyMSPerMB=",
    "-XX:ReservedCodeCacheSize=",
    "-XX:NonNMethodCodeHeapSize=",
    "-XX:ProfiledCodeHeapSize=",
    "-XX:NonProfiledCodeHeapSize=",
    "-XX:MaxInlineLevel=",
    "-XX:FreqInlineSize=",
    "-XX:InlineSmallCode=",
    "-XX:MaxNodeLimit=",
    "-XX:NodeLimitFudgeFactor=",
    "-XX:NmethodSweepActivity=",
    "-XX:AllocatePrefetchStyle=",
    "-XX:LargePageSizeInBytes=",
    "-XX:AutoBoxCacheMax=",
    "-XX:ThreadPriorityPolicy=",
    "-XX:CompileThresholdScaling=",
    "-Dsun.reflect.inflationThreshold=",
    "-Xms",
    "-Xmx",
];

fn should_remove(arg: &str) -> bool {
    EXACT_REMOVE.contains(&arg) || PREFIX_REMOVE.iter().any(|p| arg.starts_with(p))
}

fn split_args(args: &[String]) -> (Vec<String>, String, Vec<String>) {
    let mut jvm = Vec::new();
    let mut main_class = String::new();
    let mut app = Vec::new();
    let mut i = 0;
    while i < args.len() {
        let a = &args[i];
        if a == "-classpath" || a == "-cp" || a == "-jar" {
            jvm.push(a.clone());
            i += 1;
            if i < args.len() {
                jvm.push(args[i].clone());
            }
            i += 1;
            continue;
        }
        if a.starts_with('-') {
            jvm.push(a.clone());
            i += 1;
            continue;
        }
        main_class = a.clone();
        if i + 1 < args.len() {
            app = args[i + 1..].to_vec();
        }
        return (jvm, main_class, app);
    }
    (jvm, main_class, app)
}

pub fn filter_args(orig: &[String], injected: &[String]) -> Vec<String> {
    let (jvm_args, main_class, app) = split_args(orig);
    let filtered: Vec<String> = jvm_args.into_iter().filter(|a| !should_remove(a)).collect();
    let mut result = Vec::with_capacity(filtered.len() + injected.len() + 1 + app.len());
    result.extend(filtered);
    result.extend_from_slice(injected);
    if !main_class.is_empty() {
        result.push(main_class);
    }
    result.extend(app);
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{generate, Config};
    use crate::system::SystemInfo;

    #[test]
    fn filter_strips_heap_flags() {
        let orig = vec![
            "-Xmx2g".to_string(),
            "-Xms1g".to_string(),
            "Main".to_string(),
        ];
        let injected = vec!["-Xmx6g".to_string()];
        let out = filter_args(&orig, &injected);
        assert!(out.contains(&"-Xmx6g".to_string()));
        assert!(!out.iter().any(|a| a == "-Xmx2g"));
        assert!(out.contains(&"Main".to_string()));
    }

    #[test]
    fn flags_includes_heap_and_g1_for_default_config() {
        let sys = SystemInfo {
            total_ram: 32 << 30,
            free_ram: 16 << 30,
            cpu_cores: 8,
            cpu_threads: 16,
            l3_cache_mb: 32,
            mem_speed_mts: 3200,
            large_pages: false,
            large_page_size: 0,
            cpu_name: "test".into(),
            gpu_name: "test".into(),
        };
        let cfg: Config = generate(&sys);
        let f = flags(&cfg);
        assert!(f.iter().any(|a| a.starts_with("-Xmx")));
        assert!(f.iter().any(|a| a == "-XX:+UseG1GC"));
        assert!(f.len() > 20);
    }
}
