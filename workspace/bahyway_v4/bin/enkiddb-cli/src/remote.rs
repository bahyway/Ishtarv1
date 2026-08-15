//! Minimal client for `enkiddb-write-server`'s wire protocol (length-
//! prefixed u32 LE frame, one request/response per connection) — reused
//! here so `enkiddb-cli ingest --remote host:port` can run the
//! Architect's git-authorship check on the VDI itself (this host's own
//! real git checkout, correct ownership, git already installed) and send
//! only already-verified documents over the wire, never asking the
//! container to touch a bind-mounted repo at all. This is what actually
//! retires the need for `INGEST_DIR` + in-container git for this
//! workflow (PB-201/PB-202's fix stays in the image for local/dev use of
//! `INGEST_DIR`, it just stops being load-bearing for client-facing
//! ingestion).

use std::io::{self, Read, Write};
use std::net::TcpStream;
use std::time::Duration;

const MAX_FRAME: u32 = 16 * 1024 * 1024;
const TIMEOUT_SECS: u64 = 30;

/// Sends one document (`<collection>\n<markdown>`) to a running
/// `enkiddb-write-server` and returns its raw response text verbatim
/// (`OK:<kaki_hex>:<section_count>` or `ERR:<reason>`) -- the caller
/// decides what counts as success, this function never interprets it.
pub fn send_document(host: &str, port: u16, collection: &str, body: &str) -> Result<String, String> {
    let mut stream = TcpStream::connect((host, port)).map_err(|e| e.to_string())?;
    stream.set_read_timeout(Some(Duration::from_secs(TIMEOUT_SECS))).ok();
    stream.set_write_timeout(Some(Duration::from_secs(TIMEOUT_SECS))).ok();

    let payload = format!("{collection}\n{body}");
    write_frame(&mut stream, &payload).map_err(|e| e.to_string())?;
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
        return Err(io::Error::new(io::ErrorKind::InvalidData, format!("frame too large: {len}")));
    }
    let mut buf = vec![0u8; len as usize];
    s.read_exact(&mut buf)?;
    String::from_utf8(buf).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
}
