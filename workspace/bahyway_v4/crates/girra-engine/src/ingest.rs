//! Minimal sovereign wire: "metric_name value unix_millis\n" over TCP.
//! Engines emit; girra-dashboard listens. Port 7100 is deliberately
//! outside the EnkiDB-Types range (7001-7007).

use crate::metrics::Shared;
use std::io::{BufRead, BufReader};
use std::net::TcpListener;
use std::thread;

pub const DEFAULT_PORT: u16 = 7100;

pub fn spawn_listener(registry: Shared, port: u16) {
    thread::spawn(move || {
        let listener = TcpListener::bind(("0.0.0.0", port)).expect("girra ingest bind failed");
        for stream in listener.incoming().flatten() {
            let reg = registry.clone();
            thread::spawn(move || {
                let reader = BufReader::new(stream);
                for line in reader.lines().map_while(Result::ok) {
                    if let Some(sample) = parse_line(&line) {
                        if let Ok(mut r) = reg.lock() {
                            r.push(&sample.0, sample.1, sample.2);
                        }
                    }
                }
            });
        }
    });
}

/// Parses one "metric_name value unix_millis" line. Split out from
/// `spawn_listener` so the wire format itself is unit-testable without a
/// real socket.
fn parse_line(line: &str) -> Option<(String, f64, u64)> {
    let mut it = line.split_whitespace();
    let name = it.next()?;
    let v: f64 = it.next()?.parse().ok()?;
    let ts: u64 = it.next()?.parse().ok()?;
    Some((name.to_string(), v, ts))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_a_well_formed_line() {
        let parsed = parse_line("cqrs_lag_ms 42.5 1700000000000");
        assert_eq!(
            parsed,
            Some(("cqrs_lag_ms".to_string(), 42.5, 1_700_000_000_000))
        );
    }

    #[test]
    fn rejects_malformed_lines() {
        assert!(parse_line("").is_none());
        assert!(parse_line("only_name").is_none());
        assert!(parse_line("name not_a_number 123").is_none());
        assert!(parse_line("name 1.0 not_a_timestamp").is_none());
    }
}
