use crate::system::SystemInfo;

pub fn generate_flags(sys: &SystemInfo) -> Vec<String> {
    let heap = calc_heap(sys);
    let (parallel, conc) = calc_gc_threads(sys);
    let region = calc_region_size(heap);
    let meta = calc_metaspace(heap);
    let cc = calc_code_cache(heap);
    let (surv, tenure) = calc_survivor(sys);
    let soft_ref = calc_soft_ref(heap);

    let mut flags = vec![
        format!("-Xmx{}g", heap),
        format!("-Xms{}g", heap),
        "-XX:+AlwaysPreTouch".to_string(),
        format!("-XX:MetaspaceSize={}m", meta),
        format!("-XX:MaxMetaspaceSize={}m", meta),
        "-XX:+UseG1GC".to_string(),
        "-XX:+UnlockExperimentalVMOptions".to_string(),
        "-XX:MaxGCPauseMillis=50".to_string(),
        format!("-XX:G1HeapRegionSize={}m", region),
        "-XX:G1NewSizePercent=30".to_string(),
        "-XX:G1MaxNewSizePercent=40".to_string(),
        "-XX:G1ReservePercent=15".to_string(),
        "-XX:G1HeapWastePercent=5".to_string(),
        "-XX:G1MixedGCCountTarget=4".to_string(),
        "-XX:+G1UseAdaptiveIHOP".to_string(),
        "-XX:InitiatingHeapOccupancyPercent=35".to_string(),
        "-XX:G1MixedGCLiveThresholdPercent=90".to_string(),
        "-XX:G1RSetUpdatingPauseTimePercent=5".to_string(),
        format!("-XX:SurvivorRatio={}", surv),
        format!("-XX:MaxTenuringThreshold={}", tenure),
        format!("-XX:ParallelGCThreads={}", parallel),
        format!("-XX:ConcGCThreads={}", conc),
        "-XX:+ParallelRefProcEnabled".to_string(),
        "-XX:+DisableExplicitGC".to_string(),
        format!("-XX:SoftRefLRUPolicyMSPerMB={}", soft_ref),
        "-XX:+UseCompressedOops".to_string(),
        format!("-XX:ReservedCodeCacheSize={}m", cc),
        format!("-XX:NonNMethodCodeHeapSize={}m", calc_non_method(cc)),
        format!("-XX:ProfiledCodeHeapSize={}m", calc_profiled(cc)),
        format!("-XX:NonProfiledCodeHeapSize={}m", calc_non_profiled(cc)),
        "-XX:MaxInlineLevel=15".to_string(),
        "-XX:FreqInlineSize=500".to_string(),
        "-XX:+PerfDisableSharedMem".to_string(),
        "-Djdk.nio.maxCachedBufferSize=131072".to_string(),
    ];

    if sys.large_pages {
        flags.push("-XX:+UseLargePages".to_string());
        flags.push(format!(
            "-XX:LargePageSizeInBytes={}m",
            sys.large_page_size / (1024 * 1024)
        ));
    }

    flags
}

pub fn calc_heap(sys: &SystemInfo) -> u64 {
    let free = sys.bytes_to_gb(sys.free_ram);
    let total = sys.bytes_to_gb(sys.total_ram);

    if total <= 8 {
        return 0;
    }

    let mut heap = free / 2;

    let mut floor = total / 4;
    if floor < 6 {
        floor = 6;
    }
    let mut cap = total * 3 / 4;
    if cap > 16 {
        cap = 16;
    }

    if heap < floor {
        heap = floor;
    }
    if heap > cap {
        heap = cap;
    }
    if heap < 6 {
        heap = 6;
    }
    heap
}

fn calc_gc_threads(sys: &SystemInfo) -> (u64, u64) {
    let mut parallel = sys.cpu_cores as u64 - 2;
    if parallel < 2 {
        parallel = 2;
    }
    let mut concurrent = parallel / 4;
    if concurrent < 1 {
        concurrent = 1;
    }
    (parallel, concurrent)
}

fn calc_region_size(heap_gb: u64) -> u64 {
    match heap_gb {
        0..=4 => 4,
        5..=8 => 8,
        9..=16 => 16,
        _ => 32,
    }
}

fn calc_metaspace(heap_gb: u64) -> u64 {
    match heap_gb {
        0..=4 => 128,
        5..=8 => 256,
        _ => 512,
    }
}

fn calc_code_cache(heap_gb: u64) -> u64 {
    let mut cc = heap_gb * 1024 / 16;
    if cc < 128 {
        cc = 128;
    }
    if cc > 512 {
        cc = 512;
    }
    cc
}

fn calc_non_method(cc: u64) -> u64 {
    cc * 5 / 100
}

fn calc_profiled(cc: u64) -> u64 {
    cc * 38 / 100
}

fn calc_non_profiled(cc: u64) -> u64 {
    cc - calc_non_method(cc) - calc_profiled(cc)
}

fn calc_survivor(sys: &SystemInfo) -> (u64, u64) {
    if sys.cpu_cores <= 4 {
        (32, 1)
    } else {
        (8, 4)
    }
}

fn calc_soft_ref(heap_gb: u64) -> u64 {
    match heap_gb {
        0..=4 => 10,
        5..=8 => 25,
        _ => 50,
    }
}
