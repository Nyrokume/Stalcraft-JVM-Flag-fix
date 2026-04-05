# Полная реализация функционала из Go версии

## ✅ Что РЕАЛЬНО реализовано (без заглушек)

### 1. Определение системы (system.rs) - КАК В system.go

**Windows API вызовы:**
- `GlobalMemoryStatusEx` - получает реальную RAM (всего/свободно)
- `GetLargePageMinimum` - проверяет поддержку Large Pages
- `std::thread::available_parallelism()` - реальное количество ядер CPU

**Данные:**
```rust
SystemInfo {
    total_ram: u64,      // Реальная RAM из Windows API
    free_ram: u64,       // Реальная свободная RAM
    cpu_cores: usize,    // Реальное количество ядер
    large_pages: bool,   // Реальная поддержка Large Pages
    large_page_size: u64 // Реальный размер Large Page
}
```

### 2. Генерация JVM флагов (jvm.rs) - ТОЧНО КАК В jvm.go

**Формулы из Go версии (идентичные):**

#### Heap Calculation (calc_heap)
```
if total_ram <= 8GB: return 0 (использовать стандартные)
heap = free_ram / 2
floor = total_ram / 4 (мин 6GB)
cap = total_ram * 3/4 (макс 16GB)
heap = clamp(heap, floor, cap)
if heap < 6: heap = 6
```

#### GC Threads (calc_gc_threads)
```
parallel = cpu_cores - 2 (мин 2)
concurrent = parallel / 4 (мин 1)
```

#### Region Size
```
<=4GB: 4MB
<=8GB: 8MB
<=16GB: 16MB
>16GB: 32MB
```

#### Metaspace
```
<=4GB: 128MB
<=8GB: 256MB
>8GB: 512MB
```

#### Code Cache
```
cc = heap * 1024 / 16
clamp(cc, 128, 512)
```

#### Survivor Ratio & Tenuring
```
cpu_cores <= 4: ratio=32, tenuring=1
cpu_cores > 4: ratio=8, tenuring=4
```

#### Soft Reference
```
<=4GB: 10ms/MB
<=8GB: 25ms/MB
>8GB: 50ms/MB
```

**Все JVM флаги:**
- `-Xmx{N}g -Xms{N}g` (динамический heap)
- `-XX:+AlwaysPreTouch`
- `-XX:MetaspaceSize={N}m -XX:MaxMetaspaceSize={N}m`
- `-XX:+UseG1GC -XX:+UnlockExperimentalVMOptions`
- `-XX:MaxGCPauseMillis=50`
- `-XX:G1HeapRegionSize={N}m`
- `-XX:G1NewSizePercent=30 -XX:G1MaxNewSizePercent=40`
- `-XX:G1ReservePercent=15`
- `-XX:G1HeapWastePercent=5`
- `-XX:G1MixedGCCountTarget=4`
- `-XX:+G1UseAdaptiveIHOP`
- `-XX:InitiatingHeapOccupancyPercent=35`
- `-XX:G1MixedGCLiveThresholdPercent=90`
- `-XX:G1RSetUpdatingPauseTimePercent=5`
- `-XX:SurvivorRatio={N}`
- `-XX:MaxTenuringThreshold={N}`
- `-XX:ParallelGCThreads={N} -XX:ConcGCThreads={N}`
- `-XX:+ParallelRefProcEnabled`
- `-XX:+DisableExplicitGC`
- `-XX:SoftRefLRUPolicyMSPerMB={N}`
- `-XX:+UseCompressedOops`
- `-XX:ReservedCodeCacheSize={N}m`
- `-XX:NonNMethodCodeHeapSize={N}m`
- `-XX:ProfiledCodeHeapSize={N}m`
- `-XX:NonProfiledCodeHeapSize={N}m`
- `-XX:MaxInlineLevel=15`
- `-XX:FreqInlineSize=500`
- `-XX:+PerfDisableSharedMem`
- `-Djdk.nio.maxCachedBufferSize=131072`
- `-XX:+UseLargePages -XX:LargePageSizeInBytes={N}m` (если доступно)

### 3. IFEO Registry (ifeo.rs) - ТОЧНО КАК В install.go

**Реестр Windows:**
```
HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Image File Execution Options\stalcraft.exe
  "Debugger" = "путь_к_обёртке.exe"

HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Image File Execution Options\stalcraftw.exe
  "Debugger" = "путь_к_обёртке.exe"
```

**Windows API:**
- `RegCreateKeyExW` - создание ключей
- `RegSetValueExW` - установка значений
- `RegOpenKeyExW` - открытие ключей
- `RegDeleteValueW` - удаление значений
- `RegQueryValueExW` - чтение значений
- `RegCloseKey` - закрытие ключей

**Функции:**
- `install()` - регистрирует IFEO для stalcraft.exe и stalcraftw.exe
- `uninstall()` - удаляет IFEO
- `status()` - проверяет статус

### 4. Запуск игры (process.rs) - ТОЧНО КАК В main.go

#### resolve_target (как в Go)
```rust
if target == "stalcraftw.exe":
    java_exe = "javaw.exe"
else:
    java_exe = "java.exe"

if dir/java_exe exists:
    return dir/java_exe
else:
    return target
```

#### filter_args (как в Go)
**Удаляет 26+ префиксов JVM флагов:**
- `-XX:MaxGCPauseMillis=`
- `-XX:MetaspaceSize=`
- `-XX:MaxMetaspaceSize=`
- `-XX:G1HeapRegionSize=`
- ... и все остальные

**Удаляет 2 точных совпадения:**
- `-XX:-PrintCommandLineFlags`
- `-XX:+UseG1GC`

**Заменяет на оптимизированные флаги**

#### boost_process (как в Go)
**Windows API вызовы:**
- `OpenProcess` - открывает процесс
- `SetProcessPriorityBoost` - включает boost
- `NtSetInformationProcess` (PROCESS_MEMORY_PRIORITY) - устанавливает Memory Priority = 5 (Normal)
- `NtSetInformationProcess` (PROCESS_IO_PRIORITY) - устанавливает I/O Priority = 3 (High)

**Creation Flags:**
- `HIGH_PRIORITY_CLASS` (0x00000080)

### 5. Tauri Commands (commands.rs)

**Реальные команды:**
```rust
get_system_info() -> SystemInfoResponse  // Реальные данные из system.rs
install_ifeo() -> String                 // Реальная запись в реестр
uninstall_ifeo() -> String               // Реальное удаление из реестра
check_status() -> String                 // Реальное чтение реестра
launch_game(target, args) -> String      // Реальный запуск Java процесса
```

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
   ↓ available_parallelism() → реальные ядра
   
4. JVM Flag Generation (jvm.rs)
   ↓ calc_heap() → реальный heap (6-16GB)
   ↓ calc_gc_threads() → реальные GC потоки
   ↓ generate_flags() → все JVM флаги
   
5. Process Launch (process.rs)
   ↓ resolve_target() → находит java.exe
   ↓ filter_args() → удаляет старые флаги
   ↓ Command::spawn() → запускает с HIGH_PRIORITY
   ↓ boost_process() → повышает приоритеты
```

## 📊 Реальные данные vs Заглушки

### ✅ РЕАЛЬНЫЕ данные (из Windows API):
- Total RAM (GlobalMemoryStatusEx)
- Free RAM (GlobalMemoryStatusEx)
- CPU Cores (available_parallelism)
- Large Pages (GetLargePageMinimum)
- Heap calculation (calc_heap)
- GC threads (calc_gc_threads)
- All JVM flags (generate_flags)
- IFEO registry (Reg* API)
- Process launching (CreateProcess)
- Process boosting (NtSetInformationProcess)

### ❌ Заглушки (ТОЛЬКО для browser mode):
- Когда запускаешь через `npm run dev` без Tauri
- Показывает ошибку "Tauri command not available in browser mode"
- НЕ используется в реальном приложении

## 🚀 Использование

### Запуск в Tauri (РЕАЛЬНЫЕ данные):
```bash
npm run tauri dev
```

### Запуск в браузере (заглушки):
```bash
npm run dev
```

## ⚠️ Требования

1. **Windows 10/11** - использует Windows API
2. **Administrator rights** - для IFEO (запись в HKLM)
3. **Java установлена** - для запуска stalcraft.exe
4. **STALCRAFT установлен** - для выбора папки

## 📝 Формулы (идентичны Go версии)

| Параметр | Формула | Мин | Макс |
|----------|---------|-----|------|
| Heap | free/2, clamp(total/4, total*3/4) | 6GB | 16GB |
| ParallelGCThreads | cores - 2 | 2 | - |
| ConcGCThreads | parallel / 4 | 1 | - |
| G1RegionSize | heap <= 4: 4, <= 8: 8, <= 16: 16, > 16: 32 | 4MB | 32MB |
| Metaspace | heap <= 4: 128, <= 8: 256, > 8: 512 | 128MB | 512MB |
| CodeCache | heap * 1024 / 16 | 128MB | 512MB |
| SurvivorRatio | cores <= 4: 32, > 4: 8 | 8 | 32 |
| MaxTenuringThreshold | cores <= 4: 1, > 4: 4 | 1 | 4 |
| SoftRefLRUPolicyMSPerMB | heap <= 4: 10, <= 8: 25, > 8: 50 | 10 | 50 |

## ✅ Проверка реализации

Все функции реализованы ТОЧНО как в Go версии:
- ✅ system.go → system.rs (GlobalMemoryStatusEx, GetLargePageMinimum)
- ✅ jvm.go → jvm.rs (calc_heap, generate_flags)
- ✅ install.go → ifeo.rs (RegCreateKeyExW, RegSetValueExW)
- ✅ main.go → process.rs (resolveTarget, filterArgs, boostProcess)
- ✅ menu.go → GUI (HTML/CSS/JS вместо CLI меню)
