// log.rs — file logger next to wrapper exe (logs/wrapper.log)

use std::io::Write;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::config;

const MAX_LOG_BYTES: u64 = 2 * 1024 * 1024;

fn wrapper_log_path() -> PathBuf {
    config::config_dir()
        .parent()
        .map(|p| p.join("logs").join("wrapper.log"))
        .unwrap_or_else(|| PathBuf::from("logs").join("wrapper.log"))
}

fn unix_ts() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

/// Redact Windows user profile segment: `\Users\Vasya\` → `\Users\<user>\`
pub fn redact_path(p: &str) -> String {
    if p.is_empty() {
        return p.to_string();
    }
    const MARKER: &str = r"\users\";
    let lower = p.to_lowercase();
    let Some(idx) = lower.find(MARKER) else {
        return p.to_string();
    };
    let start = idx + MARKER.len();
    if start >= p.len() {
        return p.to_string();
    }
    if let Some(rest) = p[start..].find('\\') {
        format!("{}<user>{}", &p[..start], &p[start + rest..])
    } else {
        format!("{}<user>", &p[..start])
    }
}

fn maybe_truncate_log(path: &PathBuf) {
    if let Ok(meta) = std::fs::metadata(path) {
        if meta.len() > MAX_LOG_BYTES {
            let _ = std::fs::write(path, []);
        }
    }
}

pub fn append_wrapper_log_line(line: &str) {
    let _ = append_inner(line);
}

fn append_inner(line: &str) -> std::io::Result<()> {
    let path = wrapper_log_path();
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir)?;
    }
    maybe_truncate_log(&path);
    let mut f = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)?;
    writeln!(f, "[{}] {}", unix_ts(), line)?;
    Ok(())
}

pub fn read_wrapper_log_tail(max_lines: usize) -> Result<String, String> {
    let path = wrapper_log_path();
    if !path.exists() {
        return Ok(String::new());
    }
    let data = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
    let lines: Vec<&str> = data.lines().filter(|l| !l.is_empty()).collect();
    let take = max_lines.max(1).min(500);
    let start = lines.len().saturating_sub(take);
    Ok(lines[start..].join("\n"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn redact_user_path() {
        let p = r"C:\Users\Vasya\Games\stalcraft\stalcraft.exe";
        assert_eq!(
            redact_path(p),
            r"C:\Users\<user>\Games\stalcraft\stalcraft.exe"
        );
    }
}
