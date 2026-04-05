# IFEO Debugger Mode Implementation

## Problem
When IFEO (Image File Execution Options) intercepts a game launch, it launches the wrapper as a "debugger". Previously, this caused:
1. A second GUI window to open
2. The second instance would fail with error 0x80070057 (Invalid parameter)
3. The game wouldn't launch

## Solution
Implemented dual-mode architecture in `main.rs`:

### Mode Detection
The wrapper now detects which mode it's running in by checking command-line arguments:
- **GUI Mode**: Launched normally (no args or non-game args)
- **Debugger Mode**: Launched by IFEO with game executable as first argument

### Debugger Mode Flow
1. IFEO intercepts `stalcraft.exe` launch
2. Windows launches: `wrapper.exe stalcraft.exe [original args...]`
3. Wrapper detects debugger mode (first arg contains "stalcraft")
4. **No GUI is created** - runs as console application
5. Detects system hardware
6. Generates optimized JVM flags
7. Launches the actual game with optimizations
8. Exits cleanly

### GUI Mode Flow
1. User opens wrapper normally
2. GUI shows system info, IFEO status, etc.
3. User clicks "Launch Game"
4. Wrapper invokes Tauri command to launch `stalcraft.exe`
5. IFEO intercepts and triggers debugger mode (above)
6. GUI shows success message

## Technical Details

### Path Handling
- All paths are validated before use
- Unicode paths (including Cyrillic) are supported
- Path separators are normalized for Windows

### Error Handling
- Debugger mode logs to stderr with `[DEBUGGER]` prefix
- GUI mode shows user-friendly messages
- Both modes provide detailed error information

### Process Launch
- Game process is spawned with `HIGH_PRIORITY_CLASS`
- Process boosting applies memory and I/O priority
- PID is tracked for monitoring

## Testing
1. Install IFEO registry hook
2. Launch game via wrapper GUI
3. Verify:
   - Only ONE wrapper window is visible (GUI mode)
   - Game launches with optimized JVM parameters
   - No error dialogs appear
   - System log shows successful launch

## Debugging
If issues occur, check:
1. Console output for `[DEBUGGER]` messages
2. Event Viewer for Windows errors
3. IFEO registry keys are correct
4. Wrapper has admin privileges for IFEO

## Files Modified
- `src-tauri/src/main.rs` - Dual-mode architecture
- `src-tauri/src/process.rs` - Enhanced error logging
- `src/main.js` - Path normalization and handling
