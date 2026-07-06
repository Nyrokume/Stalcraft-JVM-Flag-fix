// server_block.rs — outbound firewall rules for Stalcraft tunnel IPs (ports 29450–29460)

use std::net::Ipv4Addr;
use std::process::{Command, Stdio};

use crate::log;

const RULE_PREFIX: &str = "STALZONE-SB-";

pub fn is_valid_ipv4(ip: &str) -> bool {
    ip.parse::<Ipv4Addr>().is_ok()
}

pub fn rule_name(protocol: &str, ip: &str) -> String {
    format!("{}{}-{}", RULE_PREFIX, protocol, ip.replace('.', "-"))
}

/// Deduplicated valid IPv4 addresses, preserving first-seen order.
pub fn unique_valid_ipv4(ips: &[String]) -> Vec<String> {
    let mut unique = Vec::new();
    for ip in ips {
        let trimmed = ip.trim();
        if trimmed.is_empty() || !is_valid_ipv4(trimmed) {
            continue;
        }
        if !unique.iter().any(|v| v == trimmed) {
            unique.push(trimmed.to_string());
        }
    }
    unique
}

fn run_powershell(script: &str) -> Result<String, String> {
    let mut cmd = Command::new("powershell");
    cmd.args([
        "-NoProfile",
        "-NoLogo",
        "-NonInteractive",
        "-WindowStyle",
        "Hidden",
        "-ExecutionPolicy",
        "Bypass",
        "-Command",
        script,
    ])
    .stdin(Stdio::null())
    .stdout(Stdio::piped())
    .stderr(Stdio::piped());

    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        cmd.creation_flags(CREATE_NO_WINDOW);
    }

    let output = cmd
        .output()
        .map_err(|e| format!("powershell failed: {}", e))?;

    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
    if !output.status.success() {
        if stderr.is_empty() {
            return Err(format!(
                "powershell exit {}",
                output.status.code().unwrap_or(-1)
            ));
        }
        return Err(stderr);
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

pub fn clear_rules() -> Result<u32, String> {
    let script = format!(
        r#"$n = 0; Get-NetFirewallRule -ErrorAction SilentlyContinue | Where-Object {{ $_.DisplayName -like '{prefix}*' }} | ForEach-Object {{ Remove-NetFirewallRule -Name $_.Name -ErrorAction SilentlyContinue; $n++ }}; Write-Output $n"#,
        prefix = RULE_PREFIX
    );
    let out = run_powershell(&script)?;
    out.parse::<u32>()
        .map_err(|_| format!("unexpected output: {}", out))
}

pub fn apply_blocks(ips: &[String]) -> Result<String, String> {
    let unique = unique_valid_ipv4(ips);
    if unique.is_empty() {
        return Err("No valid IPv4 addresses to block".to_string());
    }

    if let Err(e) = clear_rules() {
        log::append_wrapper_log_line(&format!("server_block clear before apply warn: {}", e));
    }

    let mut rule_lines = Vec::with_capacity(unique.len() * 2);
    for ip in &unique {
        for (proto, flag) in [("TCP", "TCP"), ("UDP", "UDP")] {
            let name = rule_name(proto, ip);
            rule_lines.push(format!(
                "New-NetFirewallRule -DisplayName '{name}' -Name '{name}' -Direction Outbound -RemoteAddress '{ip}' -Action Block -Protocol {flag} -RemotePort 29450-29460 -Enabled True -Profile Any -ErrorAction Stop"
            ));
        }
    }

    let script = rule_lines.join("; ");
    if let Err(e) = run_powershell(&script) {
        if let Err(clear_err) = clear_rules() {
            log::append_wrapper_log_line(&format!(
                "server_block rollback clear failed: {}",
                clear_err
            ));
        }
        return Err(format!("Failed to apply firewall rules: {}", e));
    }

    let msg = format!(
        "Blocked {} IP address(es) on ports 29450–29460",
        unique.len()
    );
    log::append_wrapper_log_line(&format!("server_block apply ok: {}", msg));
    Ok(msg)
}

pub struct BlockStatus {
    pub active: bool,
    pub rule_count: u32,
}

pub fn blocking_status() -> Result<BlockStatus, String> {
    let script = format!(
        r#"$r = @(Get-NetFirewallRule -ErrorAction SilentlyContinue | Where-Object {{ $_.DisplayName -like '{prefix}*' -and $_.Enabled -eq 'True' }}); Write-Output $r.Count"#,
        prefix = RULE_PREFIX
    );
    let out = run_powershell(&script)?;
    let count = out.parse::<u32>().unwrap_or(0);
    Ok(BlockStatus {
        active: count > 0,
        rule_count: count,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validates_ipv4() {
        assert!(is_valid_ipv4("1.2.3.4"));
        assert!(!is_valid_ipv4("not-an-ip"));
        assert!(!is_valid_ipv4("::1"));
    }

    #[test]
    fn rule_name_formats_protocol_and_ip() {
        assert_eq!(
            rule_name("TCP", "1.2.3.4"),
            "STALZONE-SB-TCP-1-2-3-4"
        );
    }

    #[test]
    fn unique_valid_ipv4_dedups_and_skips_invalid() {
        let ips = vec![
            "1.1.1.1".into(),
            "1.1.1.1".into(),
            "bad".into(),
            " 2.2.2.2 ".into(),
            "".into(),
        ];
        assert_eq!(
            unique_valid_ipv4(&ips),
            vec!["1.1.1.1".to_string(), "2.2.2.2".to_string()]
        );
    }

    #[test]
    fn unique_valid_ipv4_empty_for_all_invalid() {
        assert!(unique_valid_ipv4(&["x".into(), "::1".into()]).is_empty());
    }

    #[test]
    fn apply_blocks_rejects_empty_ip_list() {
        let err = apply_blocks(&[]).unwrap_err();
        assert!(err.contains("No valid IPv4"));
    }
}
