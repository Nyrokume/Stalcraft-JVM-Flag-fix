// paths.rs — typed launcher roots (EXBO / Steam / EGS / VK) + jvm_wrapper home

use std::path::{Path, PathBuf};

pub const JVM_WRAPPER_DIR: &str = "jvm_wrapper";
const RUNTIME_STALCRAFT: &str = r"runtime\stalcraft";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LauncherKind {
    Exbo,
    Steam,
    Egs,
    VkPlay,
    Unknown,
}

impl LauncherKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Exbo => "exbo",
            Self::Steam => "steam",
            Self::Egs => "egs",
            Self::VkPlay => "vkplay",
            Self::Unknown => "unknown",
        }
    }
}

#[derive(Debug, Clone)]
pub struct LauncherLayout {
    pub kind: LauncherKind,
    pub root: PathBuf,
    pub wrapper: PathBuf,
    pub runtime_stalcraft: Option<PathBuf>,
}

impl LauncherLayout {
    pub fn exbo() -> Option<Self> {
        exbo_roaming_root().map(|root| Self {
            kind: LauncherKind::Exbo,
            wrapper: root.join(JVM_WRAPPER_DIR),
            runtime_stalcraft: Some(root.join(RUNTIME_STALCRAFT)),
            root,
        })
    }
}

pub fn exbo_roaming_root() -> Option<PathBuf> {
    std::env::var_os("APPDATA").map(|appdata| PathBuf::from(appdata).join("EXBO"))
}

/// Canonical jvm_wrapper directory: exe dir → EXBO\jvm_wrapper → fallback.
pub fn wrapper_home() -> PathBuf {
    if let Ok(exe) = std::env::current_exe() {
        if let Some(parent) = exe.parent() {
            if parent
                .file_name()
                .is_some_and(|n| n.eq_ignore_ascii_case(JVM_WRAPPER_DIR))
            {
                return parent.to_path_buf();
            }
            if parent.join("service.exe").is_file() {
                return parent.to_path_buf();
            }
        }
    }
    if let Some(layout) = LauncherLayout::exbo() {
        if layout.wrapper.is_dir() {
            return layout.wrapper;
        }
        if layout.root.is_dir() {
            return layout.wrapper;
        }
    }
    std::env::current_exe()
        .ok()
        .and_then(|e| e.parent().map(|p| p.to_path_buf()))
        .unwrap_or_else(|| PathBuf::from(JVM_WRAPPER_DIR))
}

pub fn configs_dir() -> PathBuf {
    wrapper_home().join("configs")
}

/// Example JVM presets shipped beside the app (or project `examples/` in dev).
pub fn examples_dir() -> PathBuf {
    let beside = wrapper_home().join("examples");
    if beside.is_dir() {
        return beside;
    }
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../examples")
}

pub fn logs_dir() -> PathBuf {
    wrapper_home().join("logs")
}

fn norm_lower(path: &str) -> String {
    path.replace('/', "\\").to_lowercase()
}

fn slice_dir(path: &str, marker: &str) -> Option<PathBuf> {
    let lower = norm_lower(path);
    let idx = lower.find(&marker.to_lowercase())?;
    Some(PathBuf::from(&path[..idx + marker.len()]))
}

/// Classify a running image path into a launcher layout.
pub fn classify_target(path: &str) -> LauncherLayout {
    let lower = norm_lower(path);

    if let Some(layout) = LauncherLayout::exbo() {
        let root = layout.root.to_string_lossy().to_lowercase();
        if lower.contains(&*root) || lower.contains(r"\roaming\exbo\") {
            return layout;
        }
    }

    if let Some(root) = slice_dir(path, r"\steamapps\common\stalcraft") {
        return LauncherLayout {
            kind: LauncherKind::Steam,
            wrapper: root.join(JVM_WRAPPER_DIR),
            runtime_stalcraft: None,
            root,
        };
    }

    if let Some(root) = slice_dir(path, r"\epic games\stalcraft") {
        return LauncherLayout {
            kind: LauncherKind::Egs,
            wrapper: root.join(JVM_WRAPPER_DIR),
            runtime_stalcraft: None,
            root,
        };
    }

    if let Some(root) = slice_dir(path, r"\vkplay\stalcraft") {
        return LauncherLayout {
            kind: LauncherKind::VkPlay,
            wrapper: root.join(JVM_WRAPPER_DIR),
            runtime_stalcraft: None,
            root,
        };
    }

    LauncherLayout {
        kind: LauncherKind::Unknown,
        root: PathBuf::new(),
        wrapper: wrapper_home(),
        runtime_stalcraft: None,
    }
}

pub fn is_launcher_binary(path: &str) -> bool {
    let name = Path::new(path)
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_lowercase();
    matches!(
        name.as_str(),
        "stalzone.exe" | "stalzonew.exe" | "stalcraft.exe" | "stalcraftw.exe"
    )
}

pub fn is_runtime_java_bin(path: &str) -> bool {
    let lower = norm_lower(path);
    lower.contains(r"\runtime\stalcraft\") && lower.contains(r"\java\bin\")
}

pub fn is_game_java(path: &str) -> bool {
    let name = Path::new(path)
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_lowercase();
    if name != "java.exe" && name != "javaw.exe" {
        return false;
    }

    let layout = classify_target(path);
    if layout.kind != LauncherKind::Unknown {
        let root = layout.root.to_string_lossy().to_lowercase();
        if !root.is_empty() && norm_lower(path).contains(&root) {
            return true;
        }
    }

    const MARKERS: &[&str] = &[
        r"\runtime\stalcraft",
        r"\stalcraft\",
        r"\exbo\",
        r"\steamapps\common\stalcraft",
    ];
    let lower = norm_lower(path);
    MARKERS.iter().any(|m| lower.contains(&m.to_lowercase()))
}

pub fn is_game_scoped(path: &str) -> bool {
    classify_target(path).kind != LauncherKind::Unknown
}

pub fn target_kind(path: &str) -> &'static str {
    if is_game_java(path) {
        "java"
    } else if is_launcher_binary(path) {
        "launcher"
    } else {
        "other"
    }
}

pub fn scope_label(path: &str) -> &'static str {
    if is_game_scoped(path) {
        "game"
    } else {
        "unknown"
    }
}

/// Inject JVM flags for IFEO launcher shims and game java/javaw (EXBO service parity).
/// Launcher binaries are always injected when IFEO fires — path-independent.
/// java/javaw stay game-scoped to avoid system JDK false positives.
pub fn should_inject_jvm(path: &str) -> bool {
    if is_launcher_binary(path) {
        return true;
    }
    is_game_java(path)
}

/// Working / game directory inferred from image path when args lack --gameDir.
pub fn game_dir_from_target(path: &str) -> Option<PathBuf> {
    let layout = classify_target(path);
    match layout.kind {
        LauncherKind::Exbo => {
            if let Some(dir) = slice_dir(path, RUNTIME_STALCRAFT) {
                return Some(dir);
            }
            if layout.root.is_dir() {
                return Some(layout.root);
            }
            None
        }
        LauncherKind::Steam | LauncherKind::Egs | LauncherKind::VkPlay => {
            if layout.root.is_dir() {
                Some(layout.root)
            } else {
                None
            }
        }
        LauncherKind::Unknown => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exbo_java_path_classifies() {
        let p = r"C:\Users\me\AppData\Roaming\EXBO\runtime\stalcraft\win64\java\bin\javaw.exe";
        let layout = classify_target(p);
        assert_eq!(layout.kind, LauncherKind::Exbo);
        assert!(is_game_java(p));
    }

    #[test]
    fn system_java_rejected() {
        assert!(!is_game_java(r"C:\Program Files\Java\bin\java.exe"));
    }

    #[test]
    fn exbo_game_dir_from_java() {
        let p = r"C:\Users\me\AppData\Roaming\EXBO\runtime\stalcraft\win64\java\bin\javaw.exe";
        let dir = game_dir_from_target(p).unwrap();
        assert!(dir.to_string_lossy().ends_with("stalcraft"));
    }

    #[test]
    fn runtime_java_bin_detected() {
        assert!(is_runtime_java_bin(
            r"C:\Users\me\AppData\Roaming\EXBO\runtime\stalcraft\win64\java\bin\stalzone.exe"
        ));
        assert!(!is_runtime_java_bin(
            r"C:\Users\me\AppData\Roaming\EXBO\stalzone.exe"
        ));
    }

    #[test]
    fn exbo_runtime_stalzone_injects() {
        let p = r"C:\Users\me\AppData\Roaming\EXBO\runtime\stalcraft\win64\java\bin\stalzone.exe";
        assert!(should_inject_jvm(p));
        assert_eq!(target_kind(p), "launcher");
        assert_eq!(scope_label(p), "game");
    }

    #[test]
    fn exbo_root_stalzone_injects() {
        let p = r"C:\Users\me\AppData\Roaming\EXBO\stalzone.exe";
        assert!(should_inject_jvm(p));
    }

    #[test]
    fn steam_stalcraftw_injects() {
        let p = r"D:\Steam\steamapps\common\stalcraft\stalcraftw.exe";
        let layout = classify_target(p);
        assert_eq!(layout.kind, LauncherKind::Steam);
        assert!(should_inject_jvm(p));
    }

    #[test]
    fn steam_game_javaw_injects() {
        let p = r"D:\Steam\steamapps\common\stalcraft\java\bin\javaw.exe";
        assert!(should_inject_jvm(p));
        assert_eq!(target_kind(p), "java");
    }

    #[test]
    fn custom_path_stalzone_injects() {
        let p = r"E:\Portable\STALCRAFT\stalzone.exe";
        assert!(should_inject_jvm(p));
        assert_eq!(scope_label(p), "unknown");
    }

    #[test]
    fn system_java_no_inject() {
        assert!(!should_inject_jvm(r"C:\Program Files\Java\bin\java.exe"));
        assert_eq!(scope_label(r"C:\Program Files\Java\bin\java.exe"), "unknown");
    }

    #[test]
    fn cmd_exe_no_inject() {
        assert!(!should_inject_jvm(r"C:\Windows\System32\cmd.exe"));
        assert_eq!(target_kind(r"C:\Windows\System32\cmd.exe"), "other");
    }
}
