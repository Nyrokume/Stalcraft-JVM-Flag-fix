use std::collections::HashMap;
use std::io::ErrorKind;
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4, TcpStream};
use std::thread;
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};

pub const PING_CONCURRENCY: usize = 10;
pub const DEFAULT_TIMEOUT_MS: u64 = 1200;

#[derive(Debug, Clone, Deserialize)]
pub struct PingTarget {
    pub id: String,
    pub host: String,
    pub port: u16,
}

#[derive(Debug, Clone, Serialize)]
pub struct PingResult {
    pub id: String,
    pub ms: Option<u32>,
}

pub fn clamp_timeout_ms(timeout_ms: u64) -> u64 {
    timeout_ms.clamp(100, 10_000)
}

/// Parse IPv4 host:port without DNS (catalog uses literal IPs).
pub fn socket_addr_v4(host: &str, port: u16) -> Option<SocketAddr> {
    let ip: Ipv4Addr = host.trim().parse().ok()?;
    Some(SocketAddr::V4(SocketAddrV4::new(ip, port)))
}

fn elapsed_ms(start: Instant) -> u32 {
    start.elapsed().as_millis().min(u128::from(u32::MAX)) as u32
}

/// TCP connect latency. Connection refused/reset still counts — host is reachable.
pub fn tcp_ping_ms(host: &str, port: u16, timeout_ms: u64) -> Option<u32> {
    let socket_addr = socket_addr_v4(host, port)?;
    let timeout = Duration::from_millis(clamp_timeout_ms(timeout_ms));
    let start = Instant::now();

    match TcpStream::connect_timeout(&socket_addr, timeout) {
        Ok(_) => Some(elapsed_ms(start)),
        Err(e) => {
            if e.kind() == ErrorKind::TimedOut {
                None
            } else {
                Some(elapsed_ms(start))
            }
        }
    }
}

pub fn ping_targets(targets: &[PingTarget], timeout_ms: u64) -> Vec<PingResult> {
    if targets.is_empty() {
        return Vec::new();
    }

    let timeout = clamp_timeout_ms(timeout_ms);
    let mut by_id = HashMap::with_capacity(targets.len());

    thread::scope(|scope| {
        for chunk in targets.chunks(PING_CONCURRENCY) {
            let mut handles = Vec::with_capacity(chunk.len());
            for t in chunk {
                let target = t.clone();
                handles.push(scope.spawn(move || PingResult {
                    id: target.id.clone(),
                    ms: tcp_ping_ms(&target.host, target.port, timeout),
                }));
            }
            for handle in handles {
                if let Ok(result) = handle.join() {
                    by_id.insert(result.id.clone(), result);
                }
            }
        }
    });

    targets
        .iter()
        .map(|t| {
            by_id.get(&t.id).cloned().unwrap_or(PingResult {
                id: t.id.clone(),
                ms: None,
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ping_result_serializes_none() {
        let r = PingResult {
            id: "a".into(),
            ms: None,
        };
        let j = serde_json::to_string(&r).unwrap();
        assert!(j.contains("\"ms\":null"));
    }

    #[test]
    fn clamp_timeout_ms_bounds() {
        assert_eq!(clamp_timeout_ms(50), 100);
        assert_eq!(clamp_timeout_ms(1500), 1500);
        assert_eq!(clamp_timeout_ms(99_999), 10_000);
    }

    #[test]
    fn socket_addr_v4_parses_literal_ip() {
        let addr = socket_addr_v4("79.127.241.67", 29450).unwrap();
        assert_eq!(addr.port(), 29450);
    }

    #[test]
    fn socket_addr_v4_rejects_hostname() {
        assert!(socket_addr_v4("example.com", 29450).is_none());
    }

    #[test]
    fn ping_targets_empty() {
        assert!(ping_targets(&[], 1500).is_empty());
    }

    #[test]
    fn ping_targets_preserves_ids() {
        let targets = vec![
            PingTarget {
                id: "a".into(),
                host: "127.0.0.1".into(),
                port: 1,
            },
            PingTarget {
                id: "b".into(),
                host: "127.0.0.1".into(),
                port: 2,
            },
        ];
        let results = ping_targets(&targets, 100);
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].id, "a");
        assert_eq!(results[1].id, "b");
    }

    #[test]
    fn ping_targets_closed_port_returns_latency() {
        let targets = vec![PingTarget {
            id: "local".into(),
            host: "127.0.0.1".into(),
            port: 1,
        }];
        let results = ping_targets(&targets, 2000);
        assert_eq!(results.len(), 1);
        assert!(results[0].ms.is_some(), "refused port should still yield RTT");
    }
}
