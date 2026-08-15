//! ŠUMU-UKIN routing — tribe_id → session resolution → fan-out to EnkiDB nodes.
//! Named for Šamaš-šuma-ukīn (𒀭𒌓𒆳𒁺), who governed Babylon under Assyrian sovereignty.
//!
//! Network I/O (added 2026-07-10): `route()` used to be a stub that always
//! reported success regardless of target reachability. It now performs
//! real `std::net::TcpStream` dispatch using the exact wire protocol
//! confirmed against `bin/enkidb-query-server/src/main.rs`:
//! `[u32 LE query_len][UTF-8 query]` request, `[u32 LE frame_len]`-prefixed
//! response frames repeated until a `frame_len == 0` DONE sentinel. Pure
//! `std::net`/`std::thread` — no tokio, no unsafe, per the ecosystem's
//! runtime laws. AllParallel spawns one thread per target and joins them;
//! AllSerial/FirstOnly are synchronous. Only EnkiDB (port 7001) has a real
//! server as of this writing — dispatch to the other six ports is real
//! code that will correctly report `targets_failed` (connection refused)
//! until those servers exist; that is honest behaviour, not a bug.

use crate::query::HeptaQuery;
use std::io::{self, Read, Write};
use std::net::TcpStream;
use std::time::Duration;

const CONNECT_TIMEOUT: Duration = Duration::from_secs(5);
const IO_TIMEOUT: Duration = Duration::from_secs(30);

/// Send one W5H2 query to a single target over the real HEPT wire protocol.
/// Returns the number of result frames received (not their contents —
/// this layer only proves reachability + protocol compliance; row
/// materialization belongs to a higher-level query executor).
fn dispatch_query(host: &str, port: u16, query_text: &str) -> Result<usize, String> {
    let addr = format!("{host}:{port}");
    let mut stream = TcpStream::connect_timeout(
        &addr.parse().map_err(|e| format!("bad address {addr}: {e}"))?,
        CONNECT_TIMEOUT,
    ).map_err(|e| format!("connect {addr}: {e}"))?;
    stream.set_read_timeout(Some(IO_TIMEOUT)).ok();
    stream.set_write_timeout(Some(IO_TIMEOUT)).ok();

    let body = query_text.as_bytes();
    stream.write_all(&(body.len() as u32).to_le_bytes())
        .map_err(|e| format!("write len to {addr}: {e}"))?;
    stream.write_all(body)
        .map_err(|e| format!("write query to {addr}: {e}"))?;

    let mut frames = 0usize;
    loop {
        let mut len_buf = [0u8; 4];
        read_exact_or(&mut stream, &mut len_buf, &addr)?;
        let frame_len = u32::from_le_bytes(len_buf);
        if frame_len == 0 {
            break; // DONE sentinel
        }
        let mut payload = vec![0u8; frame_len as usize];
        read_exact_or(&mut stream, &mut payload, &addr)?;
        if payload.starts_with(b"ERR:") {
            return Err(format!(
                "{addr} reported: {}",
                String::from_utf8_lossy(&payload)
            ));
        }
        frames += 1;
    }
    Ok(frames)
}

fn read_exact_or(stream: &mut TcpStream, buf: &mut [u8], addr: &str) -> Result<(), String> {
    match stream.read_exact(buf) {
        Ok(()) => Ok(()),
        Err(e) if e.kind() == io::ErrorKind::UnexpectedEof =>
            Err(format!("{addr} closed connection early")),
        Err(e) => Err(format!("read from {addr}: {e}")),
    }
}

/// Routing target for a single query fan-out.
#[derive(Debug, Clone)]
pub struct RoutingTarget {
    pub session_id: String,
    pub host:       String,
    pub port:       u16,
    pub tribe_id:   u32,
}

/// ŠUMU-UKIN context: holds resolved session targets and a fan-out policy.
pub struct SumuUkinContext {
    targets: Vec<RoutingTarget>,
}

/// Fan-out policy for multi-tribe queries.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FanOutPolicy {
    /// Send to all matching targets in parallel (std::thread).
    AllParallel,
    /// Send to first matching target only.
    FirstOnly,
    /// Send to all matching targets serially.
    AllSerial,
}

/// Result of a fan-out routing attempt.
#[derive(Debug)]
pub struct FanOutResult {
    pub targets_contacted: usize,
    pub targets_failed:    usize,
    pub errors:            Vec<String>,
}

impl SumuUkinContext {
    /// Build a routing context from a list of known targets.
    pub fn new(targets: Vec<RoutingTarget>) -> Self {
        Self { targets }
    }

    /// Resolve which targets should handle this query.
    /// Empty tribe filter on query → all targets.
    pub fn resolve<'a>(&'a self, query: &HeptaQuery) -> Vec<&'a RoutingTarget> {
        if query.tribes.is_empty() {
            return self.targets.iter().collect();
        }
        self.targets.iter()
            .filter(|t| query.tribes.contains(&t.tribe_id.to_string()))
            .collect()
    }

    /// Route a query to all resolved targets using the given fan-out policy.
    /// `query_text` is the original W5H2 source — the server re-parses it,
    /// so the client forwards text, not the client-side AST. Returns a
    /// FanOutResult summarising real success/failures over real TCP
    /// dispatch (see module docs — this is no longer a stub).
    pub fn route(&self, query: &HeptaQuery, query_text: &str, policy: FanOutPolicy) -> FanOutResult {
        let targets = self.resolve(query);
        match policy {
            FanOutPolicy::FirstOnly => {
                match targets.first() {
                    None => FanOutResult { targets_contacted: 0, targets_failed: 0, errors: Vec::new() },
                    Some(t) => match dispatch_query(&t.host, t.port, query_text) {
                        Ok(_frames) => FanOutResult { targets_contacted: 1, targets_failed: 0, errors: Vec::new() },
                        Err(e) => FanOutResult { targets_contacted: 0, targets_failed: 1, errors: vec![e] },
                    },
                }
            }
            FanOutPolicy::AllSerial => {
                let mut contacted = 0;
                let mut failed = 0;
                let mut errors = Vec::new();
                for t in &targets {
                    match dispatch_query(&t.host, t.port, query_text) {
                        Ok(_) => contacted += 1,
                        Err(e) => { failed += 1; errors.push(e); }
                    }
                }
                FanOutResult { targets_contacted: contacted, targets_failed: failed, errors }
            }
            FanOutPolicy::AllParallel => {
                let owned: Vec<(String, u16)> = targets.iter().map(|t| (t.host.clone(), t.port)).collect();
                let query_owned = query_text.to_string();
                let handles: Vec<_> = owned.into_iter().map(|(host, port)| {
                    let q = query_owned.clone();
                    std::thread::spawn(move || dispatch_query(&host, port, &q))
                }).collect();

                let mut contacted = 0;
                let mut failed = 0;
                let mut errors = Vec::new();
                for h in handles {
                    match h.join() {
                        Ok(Ok(_)) => contacted += 1,
                        Ok(Err(e)) => { failed += 1; errors.push(e); }
                        Err(_) => { failed += 1; errors.push("dispatch thread panicked".to_string()); }
                    }
                }
                FanOutResult { targets_contacted: contacted, targets_failed: failed, errors }
            }
        }
    }

    pub fn target_count(&self) -> usize { self.targets.len() }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parse_query;
    use std::net::TcpListener;

    fn make_targets() -> Vec<RoutingTarget> {
        vec![
            RoutingTarget { session_id: "s1".into(), host: "10.0.0.1".into(), port: 7001, tribe_id: 1 },
            RoutingTarget { session_id: "s2".into(), host: "10.0.0.2".into(), port: 7002, tribe_id: 2 },
            RoutingTarget { session_id: "s3".into(), host: "10.0.0.3".into(), port: 7003, tribe_id: 3 },
        ]
    }

    #[test]
    fn empty_tribes_routes_to_all() {
        let ctx = SumuUkinContext::new(make_targets());
        let mut q = parse_query("WHO Tribes.E").expect("parse");
        q.tribes = vec![];
        let resolved = ctx.resolve(&q);
        assert_eq!(resolved.len(), 3);
    }

    #[test]
    fn tribe_filter_routes_to_subset() {
        let ctx = SumuUkinContext::new(make_targets());
        let mut q = parse_query("WHO Tribes.E").expect("parse");
        q.tribes = vec!["1".into(), "3".into()];
        let resolved = ctx.resolve(&q);
        assert_eq!(resolved.len(), 2);
        assert!(resolved.iter().any(|t| t.tribe_id == 1));
        assert!(resolved.iter().any(|t| t.tribe_id == 3));
    }

    /// Spawns a real local server on an ephemeral port that speaks the
    /// exact HEPT wire protocol: reads one length-prefixed query, then
    /// replies with `frame_count` batch frames of `payload` followed by
    /// the u32=0 DONE sentinel. Returns the bound port.
    fn spawn_mock_server(frame_count: usize, payload: &'static str) -> u16 {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind ephemeral port");
        let port = listener.local_addr().expect("local_addr").port();
        std::thread::spawn(move || {
            if let Ok((mut stream, _)) = listener.accept() {
                let mut len_buf = [0u8; 4];
                if stream.read_exact(&mut len_buf).is_err() { return; }
                let qlen = u32::from_le_bytes(len_buf) as usize;
                let mut qbuf = vec![0u8; qlen];
                if stream.read_exact(&mut qbuf).is_err() { return; }

                for _ in 0..frame_count {
                    let bytes = payload.as_bytes();
                    let _ = stream.write_all(&(bytes.len() as u32).to_le_bytes());
                    let _ = stream.write_all(bytes);
                }
                let _ = stream.write_all(&0u32.to_le_bytes()); // DONE
            }
        });
        port
    }

    #[test]
    fn first_only_policy_contacts_one_real_server() {
        let port = spawn_mock_server(2, "[]");
        let ctx = SumuUkinContext::new(vec![
            RoutingTarget { session_id: "s1".into(), host: "127.0.0.1".into(), port, tribe_id: 1 },
        ]);
        let mut q = parse_query("WHO Tribes.E").expect("parse");
        q.tribes = vec![];
        let result = ctx.route(&q, "WHO Tribes.E", FanOutPolicy::FirstOnly);
        assert_eq!(result.targets_contacted, 1);
        assert_eq!(result.targets_failed, 0, "errors: {:?}", result.errors);
    }

    #[test]
    fn all_serial_policy_contacts_all_real_servers() {
        let p1 = spawn_mock_server(1, "[]");
        let p2 = spawn_mock_server(1, "[]");
        let p3 = spawn_mock_server(1, "[]");
        let ctx = SumuUkinContext::new(vec![
            RoutingTarget { session_id: "s1".into(), host: "127.0.0.1".into(), port: p1, tribe_id: 1 },
            RoutingTarget { session_id: "s2".into(), host: "127.0.0.1".into(), port: p2, tribe_id: 2 },
            RoutingTarget { session_id: "s3".into(), host: "127.0.0.1".into(), port: p3, tribe_id: 3 },
        ]);
        let mut q = parse_query("WHO Tribes.E").expect("parse");
        q.tribes = vec![];
        let result = ctx.route(&q, "WHO Tribes.E", FanOutPolicy::AllSerial);
        assert_eq!(result.targets_contacted, 3, "errors: {:?}", result.errors);
        assert_eq!(result.targets_failed, 0);
    }

    #[test]
    fn all_parallel_contacts_all_real_servers_concurrently() {
        let p1 = spawn_mock_server(1, "[]");
        let p2 = spawn_mock_server(1, "[]");
        let ctx = SumuUkinContext::new(vec![
            RoutingTarget { session_id: "s1".into(), host: "127.0.0.1".into(), port: p1, tribe_id: 1 },
            RoutingTarget { session_id: "s2".into(), host: "127.0.0.1".into(), port: p2, tribe_id: 2 },
        ]);
        let mut q = parse_query("WHO Tribes.E").expect("parse");
        q.tribes = vec![];
        let result = ctx.route(&q, "WHO Tribes.E", FanOutPolicy::AllParallel);
        assert_eq!(result.targets_contacted, 2, "errors: {:?}", result.errors);
        assert_eq!(result.targets_failed, 0);
    }

    #[test]
    fn unreachable_target_reports_failure_not_silent_success() {
        // Bind and immediately drop -> port is very likely closed again,
        // and definitely nothing HEPT-speaking is listening on it. This
        // is the case the old stub could never distinguish from success.
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind ephemeral port");
        let port = listener.local_addr().expect("local_addr").port();
        drop(listener);

        let ctx = SumuUkinContext::new(vec![
            RoutingTarget { session_id: "dead".into(), host: "127.0.0.1".into(), port, tribe_id: 9 },
        ]);
        let mut q = parse_query("WHO Tribes.E").expect("parse");
        q.tribes = vec![];
        let result = ctx.route(&q, "WHO Tribes.E", FanOutPolicy::FirstOnly);
        assert_eq!(result.targets_contacted, 0);
        assert_eq!(result.targets_failed, 1);
        assert!(!result.errors.is_empty());
    }
}
