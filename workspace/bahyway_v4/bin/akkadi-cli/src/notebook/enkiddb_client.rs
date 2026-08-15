//! A real, pure-std TCP client for EnkiDDB's wire protocol -- the same
//! length-prefixed u32 LE frame `enkiddb-write-server`/`enkiddb-read-server`
//! speak (see those crates' own module docs), the same shape this
//! session's own `frame_client.py` verification harness used against a
//! live deployment.
//!
//! This is the real gap `AkkadiKernel`'s stub `run_sql_query` never
//! closed (`Ok(format!("[SqlQuery -> EnkiDB] {src}"))` -- a formatted
//! string, no network I/O at all). This client actually connects,
//! actually sends the query, and actually returns what the server said
//! -- including a real `ERR:...` response, which is exactly what a
//! client debug-capture session needs to preserve.

use std::io::{self, Read, Write};
use std::net::TcpStream;
use std::time::{Duration, Instant};

const MAX_FRAME: u32 = 16 * 1024 * 1024;

/// The result of sending one request frame: what came back, and how
/// long the round trip took. `response` is the raw server text --
/// `OK:...`/`ERR:...` for the write-node protocol, a JSON array for
/// `QUERY:`/`SEARCH:` against a read node -- callers interpret it, this
/// client never parses server semantics, only the wire framing.
pub struct EnkiddbResponse {
    pub response: String,
    pub elapsed_ms: u128,
}

/// Send one request frame to `host:port` and return its response frame,
/// timed. A connection failure, a timeout, or a malformed frame are all
/// reported as `Err(String)` -- for a debug-capture tool, "the server was
/// unreachable" is itself a real, useful thing to record, not a panic.
pub fn send_query(host: &str, port: u16, payload: &str) -> Result<EnkiddbResponse, String> {
    let start = Instant::now();
    let addr = format!("{host}:{port}");
    let mut stream = TcpStream::connect(&addr).map_err(|e| format!("connect {addr}: {e}"))?;
    stream
        .set_read_timeout(Some(Duration::from_secs(30)))
        .map_err(|e| format!("set_read_timeout: {e}"))?;
    stream
        .set_write_timeout(Some(Duration::from_secs(30)))
        .map_err(|e| format!("set_write_timeout: {e}"))?;

    write_frame(&mut stream, payload).map_err(|e| format!("write request: {e}"))?;
    let response = read_frame(&mut stream).map_err(|e| format!("read response: {e}"))?;

    Ok(EnkiddbResponse { response, elapsed_ms: start.elapsed().as_millis() })
}

fn write_frame(stream: &mut TcpStream, payload: &str) -> io::Result<()> {
    let bytes = payload.as_bytes();
    stream.write_all(&(bytes.len() as u32).to_le_bytes())?;
    stream.write_all(bytes)?;
    stream.flush()
}

fn read_frame(stream: &mut TcpStream) -> io::Result<String> {
    let mut len_buf = [0u8; 4];
    stream.read_exact(&mut len_buf)?;
    let len = u32::from_le_bytes(len_buf);
    if len == 0 {
        return Ok(String::new());
    }
    if len > MAX_FRAME {
        return Err(io::Error::new(io::ErrorKind::InvalidData, format!("frame too large: {len}")));
    }
    let mut buf = vec![0u8; len as usize];
    stream.read_exact(&mut buf)?;
    String::from_utf8(buf).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
}
