//! enkiddb-rag-client — the one real TCP client both chat agents share
//! for querying EnkiDDB's RAG (`SEARCH:<top_k>:<query>`).
//!
//! Originally written once, inline, inside `enkidullm-chat` (TamuzAI's
//! `enkiddb_bridge.rs`) when TamuzAI became the first agent wired to
//! EnkiDDB. Extracted here when EaAgent needed the identical wire client
//! a second time -- this session already found, live, what happens when
//! the same procedure gets hand-copied into a second place instead of
//! shared (EnkiDDB_MANUAL.md's sync guidance drifting behind
//! playbook_208, see PB-211): one real client, used by every agent that
//! needs to search EnkiDDB, not a second copy quietly rotting.
//!
//! Speaks the exact same length-prefixed wire protocol every other
//! client in this ecosystem uses against `enkiddb-read-server`
//! (`bin/enkiddb-read-server/src/main.rs`'s `send()`/`read_frame`) --
//! matching `bin/enkiddb-cli/src/remote.rs` and
//! `bin/akkadi-cli/src/notebook/enkiddb_client.rs`'s identical framing.
#![forbid(unsafe_code)]

use std::io::{self, Read, Write};
use std::net::TcpStream;
use std::time::Duration;

const MAX_FRAME: u32 = 16 * 1024 * 1024;
const TIMEOUT_SECS: u64 = 30;

/// The default EnkiDDB Read Node every client in this ecosystem defaults
/// to unless overridden -- `uruk-node-read`'s real IP (updated
/// 2026-08-26: the bare-metal migration renamed and re-addressed the
/// CQRS nodes -- `enkidb-node-read` at .107 was the pre-rename address;
/// `uruk-node-read` at .112 is the real, current one, confirmed live
/// against `ansible/inventory.ini`). `TamuzAI`/`EaAgent` run as part of
/// DubSar IDE on the bare-metal host, where `127.0.0.1` would not reach
/// the Read Node, which lives on its own dedicated VM.
pub const DEFAULT_HOST: &str = "192.168.122.112";
pub const DEFAULT_PORT: u16 = 7102;

/// One real search hit, parsed from `enkiddb-read-server`'s actual JSON
/// response shape (`{"kaki":..,"score":..,"text":..}` --
/// `bin/enkiddb-read-server/src/main.rs::search_hits_to_json`). Parsed by
/// hand (no `serde_json` dependency in this crate) since the shape is
/// small and fixed.
#[derive(Debug, Clone, PartialEq)]
pub struct EnkiddbHit {
    pub kaki: String,
    pub score: f32,
    pub text: String,
}

/// Runs `SEARCH:<top_k>:<query>` against a live EnkiDDB Read Node and
/// returns the parsed hits, ranked (as the server already ranks them).
/// A transport failure or a server-side `ERR:...` both surface as `Err`
/// with the real message -- never a silently empty result standing in
/// for a real failure.
pub fn search(host: &str, port: u16, top_k: usize, query: &str) -> Result<Vec<EnkiddbHit>, String> {
    let payload = format!("SEARCH:{top_k}:{query}");
    let response = send_frame(host, port, &payload)?;
    if let Some(reason) = response.strip_prefix("ERR:") {
        return Err(reason.to_string());
    }
    parse_hits(&response)
}

fn send_frame(host: &str, port: u16, payload: &str) -> Result<String, String> {
    let mut stream =
        TcpStream::connect((host, port)).map_err(|e| format!("connect {host}:{port}: {e}"))?;
    stream
        .set_read_timeout(Some(Duration::from_secs(TIMEOUT_SECS)))
        .ok();
    stream
        .set_write_timeout(Some(Duration::from_secs(TIMEOUT_SECS)))
        .ok();
    write_frame(&mut stream, payload).map_err(|e| e.to_string())?;
    read_frame(&mut stream).map_err(|e| e.to_string())
}

fn write_frame(s: &mut TcpStream, payload: &str) -> io::Result<()> {
    let b = payload.as_bytes();
    s.write_all(&(b.len() as u32).to_le_bytes())?;
    s.write_all(b)?;
    s.flush()
}

fn read_frame(s: &mut TcpStream) -> io::Result<String> {
    let mut lb = [0u8; 4];
    s.read_exact(&mut lb)?;
    let len = u32::from_le_bytes(lb);
    if len == 0 {
        return Ok(String::new());
    }
    if len > MAX_FRAME {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("frame too large: {len}"),
        ));
    }
    let mut buf = vec![0u8; len as usize];
    s.read_exact(&mut buf)?;
    String::from_utf8(buf).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
}

/// Hand-rolled parse of `search_hits_to_json`'s exact real output shape:
/// `[{"kaki":"...","score":0.1234,"text":"..."},...]`. Deliberately not a
/// general JSON parser -- this crate has no `serde_json` dependency and
/// the server's own output shape is small, fixed, and controlled by this
/// same workspace (`bin/enkiddb-read-server/src/main.rs::json_safe`
/// already escapes `"`/`\`/`\n` on the way out, so a straightforward
/// scan for those three escapes on the way back in is sufficient).
fn parse_hits(json: &str) -> Result<Vec<EnkiddbHit>, String> {
    let trimmed = json.trim();
    if trimmed == "[]" || trimmed.is_empty() {
        return Ok(Vec::new());
    }
    let mut hits = Vec::new();
    let mut rest = trimmed.trim_start_matches('[').trim_end_matches(']');
    while !rest.is_empty() {
        let obj_end = rest
            .find('}')
            .ok_or_else(|| "malformed hit: no closing brace".to_string())?;
        let obj = &rest[..=obj_end];
        hits.push(parse_one_hit(obj)?);
        rest = rest[obj_end + 1..].trim_start_matches(',').trim_start();
    }
    Ok(hits)
}

fn parse_one_hit(obj: &str) -> Result<EnkiddbHit, String> {
    let kaki = extract_string_field(obj, "kaki").ok_or("missing \"kaki\" field")?;
    let score: f32 = extract_number_field(obj, "score").ok_or("missing \"score\" field")?;
    let text = extract_string_field(obj, "text").ok_or("missing \"text\" field")?;
    Ok(EnkiddbHit {
        kaki,
        score,
        text: unescape(&text),
    })
}

fn extract_string_field(obj: &str, field: &str) -> Option<String> {
    let needle = format!("\"{field}\":\"");
    let start = obj.find(&needle)? + needle.len();
    let mut end = start;
    let bytes = obj.as_bytes();
    while end < bytes.len() {
        if bytes[end] == b'"' && bytes[end - 1] != b'\\' {
            break;
        }
        end += 1;
    }
    Some(obj[start..end].to_string())
}

fn extract_number_field(obj: &str, field: &str) -> Option<f32> {
    let needle = format!("\"{field}\":");
    let start = obj.find(&needle)? + needle.len();
    let rest = &obj[start..];
    let end = rest.find([',', '}']).unwrap_or(rest.len());
    rest[..end].trim().parse().ok()
}

fn unescape(s: &str) -> String {
    s.replace("\\n", "\n")
        .replace("\\\"", "\"")
        .replace("\\\\", "\\")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_hits_empty_array() {
        assert_eq!(parse_hits("[]").unwrap(), Vec::new());
    }

    #[test]
    fn parse_hits_single_real_shape() {
        let json = r#"[{"kaki":"ab12cd34","score":0.8421,"text":"A real widget guide."}]"#;
        let hits = parse_hits(json).unwrap();
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].kaki, "ab12cd34");
        assert!((hits[0].score - 0.8421).abs() < 1e-6);
        assert_eq!(hits[0].text, "A real widget guide.");
    }

    #[test]
    fn parse_hits_multiple() {
        let json = r#"[{"kaki":"a1","score":0.9,"text":"first"},{"kaki":"b2","score":0.5,"text":"second"}]"#;
        let hits = parse_hits(json).unwrap();
        assert_eq!(hits.len(), 2);
        assert_eq!(hits[0].kaki, "a1");
        assert_eq!(hits[1].kaki, "b2");
    }

    #[test]
    fn parse_hits_unescapes_quotes_and_newlines() {
        let json = r#"[{"kaki":"a1","score":0.5,"text":"line one\nline \"two\""}]"#;
        let hits = parse_hits(json).unwrap();
        assert_eq!(hits[0].text, "line one\nline \"two\"");
    }

    #[test]
    fn parse_hits_rejects_malformed_input() {
        assert!(parse_hits("[{\"kaki\":\"a1\"").is_err());
    }
}
