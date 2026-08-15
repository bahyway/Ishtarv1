//! enkiddb-asset-server — Central Shared Asset Storage.
//!
//! The third leg of the "3 Podman + VDI" topology (Write, Read, and this
//! one): a single, dedicated container that owns the host directory
//! holding physical assets (markdown, scripts, images, SVGs) and serves
//! them read-only over plain HTTP GET. Write and Read nodes never
//! bind-mount that host path themselves — they fetch by pointer path
//! from this server instead, which is what actually closes PB-200's
//! class of bug (two containers independently claiming `:Z` on the same
//! shared host mount): with this server as the sole owner of the host
//! path, no other container ever touches it at all, so there is no
//! second claimant to conflict with.
//!
//! Deliberately does not use MinIO or NFS: this job (serve a known file
//! by path, read-only) is small enough for this ecosystem to own outright
//! in pure Rust rather than adopt an external service, matching the
//! zero-external-dependency axiom the rest of the ecosystem holds to.
//! Hand-rolled HTTP/1.1 subset (GET only, no persistent connections, no
//! range requests) — enough for curl, a browser, Godot's HTTPRequest
//! node, and Ansible health checks, without pulling in an HTTP crate.
#![forbid(unsafe_code)]

use std::env;
use std::fs;
use std::io::{self, BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::path::{Component, Path, PathBuf};
use std::time::Duration;

const READ_TIMEOUT: u64 = 30;
const WRITE_TIMEOUT: u64 = 30;
const MAX_HEADER_BYTES: usize = 8 * 1024;

const ALLOWED_EXTENSIONS: &[&str] =
    &["md", "sh", "py", "rs", "yml", "yaml", "png", "svg", "jpg", "jpeg", "json", "txt"];

fn bind_addr() -> String {
    env::var("BIND_ADDR").unwrap_or_else(|_| "0.0.0.0:7301".to_string())
}
fn asset_root() -> PathBuf {
    PathBuf::from(env::var("ASSET_ROOT").unwrap_or_else(|_| "/data/assets".to_string()))
}

fn main() {
    eprintln!("𒁾 enkiddb-asset-server — Central Shared Asset Storage");

    let root = asset_root();
    if let Err(e) = fs::create_dir_all(&root) {
        eprintln!("FATAL: cannot create ASSET_ROOT {}: {e}", root.display());
        std::process::exit(1);
    }
    let canonical_root = fs::canonicalize(&root).unwrap_or_else(|e| {
        eprintln!("FATAL: cannot canonicalize ASSET_ROOT {}: {e}", root.display());
        std::process::exit(1);
    });

    let addr = bind_addr();
    let listener = TcpListener::bind(&addr).unwrap_or_else(|e| {
        eprintln!("FATAL: bind {addr}: {e}");
        std::process::exit(1);
    });
    eprintln!("  asset_root = {}", canonical_root.display());
    eprintln!("𒁾 Listening on {addr}");

    std::thread::scope(|scope| {
        for stream in listener.incoming() {
            match stream {
                Ok(s) => {
                    let root_ref = &canonical_root;
                    scope.spawn(move || {
                        if let Err(e) = handle(s, root_ref) {
                            eprintln!("[conn] {e}");
                        }
                    });
                }
                Err(e) => eprintln!("[accept] {e}"),
            }
        }
    });
}

fn handle(mut stream: TcpStream, root: &Path) -> io::Result<()> {
    stream.set_read_timeout(Some(Duration::from_secs(READ_TIMEOUT)))?;
    stream.set_write_timeout(Some(Duration::from_secs(WRITE_TIMEOUT)))?;

    let mut reader = BufReader::new(stream.try_clone()?);
    let mut request_line = String::new();
    reader.read_line(&mut request_line)?;
    let Some((method, req_path)) = parse_request_line(request_line.trim_end()) else {
        return respond(&mut stream, 400, "Bad Request", "text/plain", b"malformed request line");
    };
    let method = method.to_string();
    let req_path = req_path.to_string();

    // Drain and discard headers -- never inspected, but still have to be
    // read off the wire before a response can be written.
    let mut total = request_line.len();
    loop {
        let mut line = String::new();
        let n = reader.read_line(&mut line)?;
        total += n;
        if n == 0 || line == "\r\n" || line == "\n" || total > MAX_HEADER_BYTES {
            break;
        }
    }

    if method != "GET" {
        return respond(&mut stream, 405, "Method Not Allowed", "text/plain", b"only GET is served");
    }

    if req_path == "/" {
        let listing = list_root(root).unwrap_or_default();
        return respond(&mut stream, 200, "OK", "text/plain; charset=utf-8", listing.as_bytes());
    }

    match resolve_safe_path(root, &req_path) {
        Ok(file_path) if file_path.is_file() => match fs::read(&file_path) {
            Ok(bytes) => respond(&mut stream, 200, "OK", content_type_for(&file_path), &bytes),
            Err(_) => respond(&mut stream, 404, "Not Found", "text/plain", b"not found"),
        },
        Ok(_) => respond(&mut stream, 404, "Not Found", "text/plain", b"not found"),
        Err(ServeError::Traversal) => respond(&mut stream, 403, "Forbidden", "text/plain", b"path escapes asset root"),
        Err(ServeError::DisallowedExtension) => {
            respond(&mut stream, 403, "Forbidden", "text/plain", b"extension not served")
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
enum ServeError {
    Traversal,
    DisallowedExtension,
}

/// Parses `"GET /path HTTP/1.1"` into `(method, path)`. No query string or
/// fragment handling -- assets are addressed by plain path only.
fn parse_request_line(line: &str) -> Option<(&str, &str)> {
    let mut parts = line.split_whitespace();
    let method = parts.next()?;
    let path = parts.next()?;
    parts.next()?; // HTTP version, required but unchecked
    Some((method, path))
}

/// Resolves a request path against `root`, rejecting anything that would
/// escape it (`..`/root-anchored components, checked before touching the
/// filesystem) or that doesn't carry a known extension. Fails closed: any
/// ambiguity is a rejection, not a serve -- the same discipline
/// `enkiddb::authorship::check_authorship` uses for the analogous
/// git-authorship gate.
fn resolve_safe_path(root: &Path, req_path: &str) -> Result<PathBuf, ServeError> {
    let relative = req_path.trim_start_matches('/');
    let candidate = root.join(relative);

    let has_allowed_ext = candidate
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| ALLOWED_EXTENSIONS.contains(&e.to_lowercase().as_str()))
        .unwrap_or(false);
    if !has_allowed_ext {
        return Err(ServeError::DisallowedExtension);
    }

    if Path::new(relative)
        .components()
        .any(|c| matches!(c, Component::ParentDir | Component::RootDir | Component::Prefix(_)))
    {
        return Err(ServeError::Traversal);
    }

    match fs::canonicalize(&candidate) {
        Ok(resolved) if resolved.starts_with(root) => Ok(resolved),
        Ok(_) => Err(ServeError::Traversal),
        // Doesn't exist yet -- let the caller's is_file() check turn this
        // into a plain 404 rather than a permission error.
        Err(_) => Ok(candidate),
    }
}

fn content_type_for(path: &Path) -> &'static str {
    match path.extension().and_then(|e| e.to_str()).map(str::to_lowercase).as_deref() {
        Some("md") => "text/markdown; charset=utf-8",
        Some("svg") => "image/svg+xml",
        Some("png") => "image/png",
        Some("jpg") | Some("jpeg") => "image/jpeg",
        Some("py") => "text/x-python; charset=utf-8",
        Some("sh") => "text/x-shellscript; charset=utf-8",
        Some("rs") => "text/x-rust; charset=utf-8",
        Some("yml") | Some("yaml") => "application/yaml; charset=utf-8",
        Some("json") => "application/json; charset=utf-8",
        _ => "text/plain; charset=utf-8",
    }
}

/// A flat, one-per-line relative listing of every file under `root` --
/// the closest thing this server has to a directory index. Health checks
/// and a human with `curl` are the intended audience, not programmatic
/// discovery: Write/Read nodes address assets by pointer path, they never
/// need to browse.
fn list_root(root: &Path) -> io::Result<String> {
    let mut out = String::new();
    let mut stack = vec![root.to_path_buf()];
    let mut entries = Vec::new();
    while let Some(dir) = stack.pop() {
        for entry in fs::read_dir(&dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
            } else if let Ok(rel) = path.strip_prefix(root) {
                entries.push(rel.to_string_lossy().to_string());
            }
        }
    }
    entries.sort();
    for e in entries {
        out.push_str(&e);
        out.push('\n');
    }
    Ok(out)
}

fn respond(stream: &mut TcpStream, code: u16, reason: &str, content_type: &str, body: &[u8]) -> io::Result<()> {
    let header = format!(
        "HTTP/1.1 {code} {reason}\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        body.len()
    );
    stream.write_all(header.as_bytes())?;
    stream.write_all(body)?;
    stream.flush()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_request_line_extracts_method_and_path() {
        assert_eq!(parse_request_line("GET /docs/foo.md HTTP/1.1"), Some(("GET", "/docs/foo.md")));
        assert_eq!(parse_request_line("POST / HTTP/1.1"), Some(("POST", "/")));
        assert_eq!(parse_request_line(""), None);
        assert_eq!(parse_request_line("GET"), None);
    }

    #[test]
    fn content_type_for_maps_known_extensions() {
        assert_eq!(content_type_for(Path::new("a.md")), "text/markdown; charset=utf-8");
        assert_eq!(content_type_for(Path::new("a.svg")), "image/svg+xml");
        assert_eq!(content_type_for(Path::new("a.png")), "image/png");
        assert_eq!(content_type_for(Path::new("a.bin")), "text/plain; charset=utf-8");
    }

    fn scratch_dir(name: &str) -> PathBuf {
        let dir = env::temp_dir().join(format!("enkiddb_asset_server_test_{name}"));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        fs::canonicalize(&dir).unwrap()
    }

    #[test]
    fn resolve_safe_path_serves_a_real_file_with_allowed_extension() {
        let root = scratch_dir("ok");
        fs::write(root.join("readme.md"), "# hi\n").unwrap();
        let resolved = resolve_safe_path(&root, "/readme.md").unwrap();
        assert_eq!(resolved, root.join("readme.md"));
    }

    #[test]
    fn resolve_safe_path_serves_a_nested_file() {
        let root = scratch_dir("nested");
        fs::create_dir_all(root.join("docs/sub")).unwrap();
        fs::write(root.join("docs/sub/page.md"), "# hi\n").unwrap();
        let resolved = resolve_safe_path(&root, "/docs/sub/page.md").unwrap();
        assert_eq!(resolved, root.join("docs/sub/page.md"));
    }

    #[test]
    fn resolve_safe_path_rejects_parent_dir_traversal() {
        let root = scratch_dir("traversal");
        let err = resolve_safe_path(&root, "/../etc/passwd.md").unwrap_err();
        assert_eq!(err, ServeError::Traversal);
    }

    #[test]
    fn resolve_safe_path_rejects_disallowed_extension() {
        let root = scratch_dir("ext");
        fs::write(root.join("secrets.env"), "SECRET=1\n").unwrap();
        let err = resolve_safe_path(&root, "/secrets.env").unwrap_err();
        assert_eq!(err, ServeError::DisallowedExtension);
    }

    #[test]
    fn resolve_safe_path_allows_a_not_yet_existing_path_through_to_a_404() {
        let root = scratch_dir("missing");
        let resolved = resolve_safe_path(&root, "/nope.md").unwrap();
        assert_eq!(resolved, root.join("nope.md"));
        assert!(!resolved.is_file());
    }

    #[test]
    fn list_root_lists_nested_files_relative_to_root_sorted() {
        let root = scratch_dir("listing");
        fs::create_dir_all(root.join("sub")).unwrap();
        fs::write(root.join("b.md"), "b").unwrap();
        fs::write(root.join("sub/a.md"), "a").unwrap();
        let listing = list_root(&root).unwrap();
        let mut lines: Vec<&str> = listing.lines().collect();
        lines.sort();
        assert_eq!(lines, vec!["b.md", "sub/a.md"]);
    }
}
