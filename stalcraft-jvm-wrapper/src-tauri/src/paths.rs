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
}
