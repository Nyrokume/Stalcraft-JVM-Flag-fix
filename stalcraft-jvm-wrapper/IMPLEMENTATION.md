# Полная реализация функционала из Go версии

## ✅ Что РЕАЛЬНО реализовано (без заглушек)

### 1. Определение системы (system.rs) - КАК В system.go

**Windows API вызовы:**
- `GlobalMemoryStatusEx` - получает реальную RAM (всего/свободно)
- `GetLargePageMinimum` - проверяет поддержку Large Pages
- `GetLogicalProcessorInformationEx` - L3 cache и физические ядра
- `OpenProcessToken`/`LookupPrivilegeValueW`/`PrivilegeCheck` - SeLockMemoryPrivilege
- `std::thread::available_parallelism()` - реальное количество ядер CPU
- Registry API для CPU/GPU имён

**Данные:**
```rust
SystemInfo {
    total_ram: u64,      // Реальная RAM из Windows API
    free_ram: u64,       // Реальная свободная RAM
    cpu_cores: usize,    // Физические ядра (без HT/SMT)
    cpu_threads: usize,  // Логические потоки (все ядра)
    l3_cache_mb: usize,  // L3 кэш в MB (max per CCD)
    large_pages: bool,   // Реальная поддержка Large Pages
    large_page_size: u64,// Реальный размер Large Page
    cpu_name: String,    // Имя CPU из реестра
    gpu_name: String,    // Имя GPU из реестра
}
```

### 2. Генерация конфига (config.rs) - ТОЧНО КАК В generate.go

**Формулы из Go версии (идентичные):**

#### Heap Calculation (size_heap)
```
>=24GB: 8GB
>=16GB: 6GB
>=12GB: 5GB
>=8GB: 4GB
>=6GB: 3GB
default: 2GB
```

#### GC Threads (gc_threads)
```
parallel = clamp(threads-2, 2, 10)
concurrent = clamp(parallel/2, 1, 5)
```

#### X3D Big Cache Detection (has_big_cache)
```
L3 >= 64 MB = X3D class CPU
```

#### JIT профиль (jit_profile)
```
X3D: max_inline_level=20, freq_inline_size=750, inline_small_code=6000, max_node_limit=320000
Normal: max_inline_level=15, freq_inline_size=500, inline_small_code=4000, max_node_limit=240000
```

#### Region Size
```
<=3GB: 4MB
<=5GB: 8MB
>5GB: 16MB
```

### 3. Генерация JVM флагов (jvm.rs) - ТОЧНО КАК В flags.go

**Все JVM флаги:**
- `-Xmx{X}g -Xms{Y}g` (Xms = min(heap, 4GB))
- `-XX:MetaspaceSize={N}m -XX:MaxMetaspaceSize={N}m`
- `-XX:+UseG1GC -XX:+UnlockExperimentalVMOptions`
- `-XX:MaxGCPauseMillis={N}`
- `-XX:G1HeapRegionSize={N}m`
- `-XX:G1NewSizePercent=23 -XX:G1MaxNewSizePercent=50` (normal) / `-XX:G1NewSizePercent=30` (X3D)
- `-XX:G1ReservePercent=20`
- `-XX:G1HeapWastePercent=5`
- `-XX:G1MixedGCCountTarget=3` (normal) / `4` (X3D)
- `-XX:+G1UseAdaptiveIHOP`
- `-XX:InitiatingHeapOccupancyPercent=20` (normal) / `15` (X3D)
- `-XX:G1MixedGCLiveThresholdPercent=90`
- `-XX:G1RSetUpdatingPauseTimePercent=0`
- `-XX:SurvivorRatio=32 -XX:MaxTenuringThreshold=1`
- `-XX:ParallelGCThreads={N} -XX:ConcGCThreads={N}`
- `-XX:+ParallelRefProcEnabled -XX:+DisableExplicitGC`
- `-XX:+UseDynamicNumberOfGCThreads`
- `-XX:+UseStringDeduplication`
- `-XX:SoftRefLRUPolicyMSPerMB=25` (normal) / `50` (X3D)
- `-XX:-UseBiasedLocking -XX:+DisableAttachMechanism`
- `-XX:ReservedCodeCacheSize=400m`
- `-XX:MaxInlineLevel={N}`
- `-XX:FreqInlineSize={N}`
- `-XX:InlineSmallCode={N}`
- `-XX:MaxNodeLimit={N} -XX:NodeLimitFudgeFactor=8000`
- `-XX:NmethodSweepActivity=1`
- `-XX:-DontCompileHugeMethods`
- `-XX:AllocatePrefetchStyle=3`
- `-XX:+AlwaysActAsServerClassMachine`
- `-XX:+UseXMMForArrayCopy`
- `-XX:+UseFPUForSpilling`
- `-XX:+UseLargePages -XX:LargePageSizeInBytes={N}m` (если доступно)
- `-Dsun.reflect.inflationThreshold=0`
- `-XX:AutoBoxCacheMax=4096`
- `-XX:+UseThreadPriorities -XX:ThreadPriorityPolicy=1`
- `-XX:-UseCounterDecay`
- `-XX:CompileThresholdScaling=0.5`

#### PreTouch
- Включается при `total_gb >= 12`

#### Пресеты (расширение Tauri GUI, не из оригинального Go)

- Функция `config::apply_named_preset(sys, id)` строит профиль как `generate(sys)`, затем накладывает фиксированные сдвиги для `latency` | `throughput` | `conservative` | `low_ram`.
- Команда Tauri `apply_config_preset` сохраняет результат в `configs/preset_<id>.json` (фиксированные имена) и вызывает `set_active` для этого стема.
- Команда `load_config_by_name` отдаёт `{ name, config }` для редактора JSON в GUI; `save_config` принимает полный `Config` из фронтенда.
- Кнопка **Regen** по-прежнему пересобирает только `default.json` и активирует `default`.

### 4. Фильтрация аргументов (jvm.rs) - ТОЧНО КАК В filter.go

**Удаляет 35+ exact совпадений:**
- `-XX:-PrintCommandLineFlags`, `-XX:+UseG1GC`, `-XX:+UseCompressedOops`, `-XX:+PerfDisableSharedMem`
- `-XX:+UseBiasedLocking`, `-XX:-UseBiasedLocking`, `-XX:+UseStringDeduplication`
- `-XX:+UseNUMA`, `-XX:+DisableAttachMechanism`, `-XX:+UseDynamicNumberOfGCThreads`
- `-XX:+AlwaysActAsServerClassMachine`, `-XX:+UseXMMForArrayCopy`, `-XX:+UseFPUForSpilling`
- `-XX:-DontCompileHugeMethods`, `-XX:+DontCompileHugeMethods`
- `-XX:+AlwaysPreTouch`, `-XX:-AlwaysPreTouch`
- `-XX:+ParallelRefProcEnabled`, `-XX:+DisableExplicitGC`, `-XX:+G1UseAdaptiveIHOP`
- `-XX:+UnlockExperimentalVMOptions`, `-XX:+UseThreadPriorities`, `-XX:-UseThreadPriorities`
- `-XX:+UseCounterDecay`, `-XX:-UseCounterDecay`, `-XX:+UseLargePages`, `-XX:-UseLargePages`
- `-XX:+UseCompressedClassPointerCompression`

**Удаляет 50+ prefix совпадений:**
- `-XX:MaxGCPauseMillis=`, `-XX:MetaspaceSize=`, `-XX:MaxMetaspaceSize=`
- `-XX:G1HeapRegionSize=`, `-XX:G1NewSizePercent=`, `-XX:G1MaxNewSizePercent=`
- `-XX:G1ReservePercent=`, `-XX:G1HeapWastePercent=`, `-XX:G1MixedGCCountTarget=`
- `-XX:InitiatingHeapOccupancyPercent=`, `-XX:G1MixedGCLiveThresholdPercent=`
- `-XX:G1RSetUpdatingPauseTimePercent=`, `-XX:G1SATBBufferEnqueueingThresholdPercent=`
- `-XX:G1ConcRSHotCardLimit=`, `-XX:G1ConcRefinementServiceIntervalMillis=`
- `-XX:GCTimeRatio=`, `-XX:SurvivorRatio=`, `-XX:MaxTenuringThreshold=`
- `-XX:ParallelGCThreads=`, `-XX:ConcGCThreads=`, `-XX:SoftRefLRUPolicyMSPerMB=`
- `-XX:ReservedCodeCacheSize=`, `-XX:NonNMethodCodeHeapSize=`, `-XX:ProfiledCodeHeapSize=`
- `-XX:NonProfiledCodeHeapSize=`, `-XX:MaxInlineLevel=`, `-XX:FreqInlineSize=`
- `-XX:InlineSmallCode=`, `-XX:MaxNodeLimit=`, `-XX:NodeLimitFudgeFactor=`
- `-XX:NmethodSweepActivity=`, `-XX:AllocatePrefetchStyle=`, `-XX:LargePageSizeInBytes=`
- `-XX:AutoBoxCacheMax=`, `-XX:ThreadPriorityPolicy=`, `-XX:CompileThresholdScaling=`
- `-XX:InitialHeapSize=`, `-XX:MaxHeapSize=`, `-XX:MinHeapDeltaBytes=`
- `-XX:TieredCompilation=`, `-XX:CICompilerCount=`
- `-Dsun.reflect.inflationThreshold=`, `-Dsun.nio.maxCachedBufferSize=`
- `-Xms`, `-Xmx`, `-Xbootclasspath`, `-Xbootclasspath/a`, `-Xbootclasspath/p`

### 5. IFEO Registry (ifeo.rs) - ТОЧНО КАК В installer.go

**Реестр Windows:**
```
HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Image File Execution Options\stalcraft.exe
  "Debugger" = "путь_к_wrapper.exe"

HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Image File Execution Options\stalcraftw.exe
  "Debugger" = "путь_к_wrapper.exe"
```

**Windows API:**
- `RegCreateKeyExW` - создание ключей
- `RegSetValueExW` - установка значений
- `RegOpenKeyExW` - открытие ключей
- `RegDeleteValueW` - удаление значений
- `RegQueryValueExW` - чтение значений
- `RegFlushKey` - сброс на диск
- `RegCloseKey` - закрытие ключей

**Функции:**
- `install()` - регистрирует IFEO для stalcraft.exe и stalcraftw.exe
- `uninstall()` - удаляет IFEO
- `status()` - проверяет статус

### 6. Запуск игры (process.rs) - ТОЧНО КАК В main.go + process.go

#### NtCreateUserProcess (nt_create_process)
- Создаёт процесс через `ntdll!NtCreateUserProcess`
- Использует `PS_ATTRIBUTE_IFEO_SKIP_DEBUGGER` (0x04) для избежания повторного IFEO перехвата
- Использует `RTL_USER_PROC_PARAMS_NORMALIZED` (0x01)
- Устанавливает PID через `PS_ATTRIBUTE_CLIENT_ID`

#### Phantom Window (start_phantom_window)
- Создаёт невидимое окно в отдельном потоке
- Нужно для корректной работы оверлеев (Steam, Discord)

#### Boost (boost_process)
- `SetProcessPriorityBoost(handle, 1)` - отключает priority decay
- `NtSetInformationProcess(handle, PROCESS_MEMORY_PRIORITY=0x27, 5)` - Memory Priority High
- `NtSetInformationProcess(handle, PROCESS_IO_PRIORITY=0x21, 3)` - I/O Priority High

#### Wait (wait_process)
- Ожидает `WAIT_OBJECT_0` (процесс завершился) или видимое окно
- Потокобезопасный `has_visible_window` через sync.Map аналог

### 7. Tauri Commands (commands.rs)

**Реальные команды:**
```rust
get_system_info() -> SystemInfoResponse  // Реальные данные из system.rs
install_ifeo() -> String                 // Реальная запись в реестр
uninstall_ifeo() -> String               // Реальное удаление из реестра
check_status() -> String                // Реальное чтение реестра
launch_game(target, args) -> String      // Реальный запуск Java процесса
list_configs() -> ConfigListResponse     // Список конфигов из configs/
select_config(name: String) -> String    // Установка активного конфига
regenerate_config() -> String            // Перегенерация default.json
get_active_config() -> ConfigResponse    // Получить активный конфиг
save_config(name, cfg) -> String         // Сохранить конфиг в файл
save_game_dir(dir) -> ()                  // Сохранение пути в store
load_game_dir() -> Option<String>        // Загрузка пути из store
```

### 8. Debugger Mode (main.rs)

При запуске с аргументом `stalcraft.exe` или `stalcraftw.exe`:
1. Запускается в режиме service (IFEO debugger)
2. Определяет систему (`system::detect_system()`)
3. Создаёт/проверяет конфиг (`config::ensure()`)
4. Загружает активный конфиг (`config::load_active()`)
5. Генерирует JVM флаги (`jvm::flags()`)
6. Фильтрует аргументы лаунчера (`jvm::filter_args()`)
7. Создаёт phantom window
8. Запускает процесс через NtCreateUserProcess
9. Бустит приоритеты
10. Ждёт видимое окно или завершение
11. Возвращает exit code

## 📊 Сравнение Go vs Tauri

| Компонент | Go (original) | Tauri (port) |
|-----------|----------------|--------------|
| System Info | sysinfo.go | system.rs |
| Config Gen | config/generate.go | config.rs (generate) |
| JVM Flags | internal/jvm/flags.go | jvm.rs (flags) |
| Filter | internal/jvm/filter.go | jvm.rs (filter_args) |
| IFEO | internal/installer/installer.go | ifeo.rs |
| Process | internal/process/process.go | process.rs |
| CLI | cmd/cli/main.go | main.rs (GUI mode) |
| Service | cmd/service/main.go | main.rs (debugger mode) |

## 🎯 Как это работает

### Поток данных:

```
1. Фронтенд (main.js)
   ↓ invoke('get_system_info')
   
2. Tauri Commands (commands.rs)
   ↓ system::detect_system()
   
3. System Detection (system.rs)
   ↓ GlobalMemoryStatusEx() → реальная RAM
   ↓ GetLargePageMinimum() → реальные Large Pages
   ↓ GetLogicalProcessorInformationEx() → L3 cache, cores
   ↓ Registry → CPU/GPU name
   
4. Config Generation (config.rs)
   ↓ generate() → sizeHeap, gcThreads, jitProfile
   
5. JVM Flag Generation (jvm.rs)
   ↓ flags() → все JVM флаги
   ↓ filter_args() → удаляет конфликтующие
   
6. Process Launch (process.rs)
   ↓ nt_create_process() → NtCreateUserProcess с IFEO skip
   ↓ boost_process() → приоритеты
   ↓ wait_process() → ожидание окна
```

## ✅ Проверка реализации

Все функции реализованы ТОЧНО как в Go версии:
- ✅ system.go → system.rs (GlobalMemoryStatusEx, GetLargePageMinimum, L3 cache)
- ✅ config/generate.go → config.rs (sizeHeap, gcThreads, jitProfile, generate)
- ✅ jvm/flags.go → jvm.rs (все флаги)
- ✅ jvm/filter.go → jvm.rs (filter_args, все фильтры)
- ✅ installer.go → ifeo.rs (RegCreateKeyExW, RegSetValueExW)
- ✅ process.go → process.rs (NtCreateUserProcess, boost, wait)
- ✅ main.go → main.rs (debugger mode, CLI flags)
