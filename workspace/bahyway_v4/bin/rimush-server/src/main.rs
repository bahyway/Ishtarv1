//! Rimush (𒌷𒈬𒍑) -- BahyWay.Ecosystem's Entrance GUI + monitoring dashboard.
//!
//! WHAT THIS IS: the one front door on the bare-metal host (`uruk`) that
//! (a) one-click launches the three real IDEs (Girsu, DubSar PDM, DubSar
//! Theater) plus the two passport tools (Sargon, Gilgamesh) -- four of
//! the five run the exact same playbook their own desktop launcher
//! already runs (PB-226/230/285, see PB-288's own `.desktop` `Exec=`
//! lines), never a second, drifting launch mechanism; Girsu has no
//! launch playbook at all yet (see LAUNCH_TARGETS' own comment) so it
//! spawns `codium` directly -- and (b) shows whether the real
//! infrastructure (the two CQRS VM nodes' EnkiDB Type ports, the vault
//! VM's SSH) is actually up, right now.
//!
//! WHY BARE-METAL, NOT THE VAULT VM (Architect gave "Vault VM or Host"):
//! launching Girsu/DubSar PDM/DubSar Theater means spawning real GUI
//! processes with direct GPU access -- that only works from a process
//! running ON the machine with the GPU. The vault VM has no GPU and no
//! display; it could show a dashboard but could never launch these tools.
//! One binary that can't do half its job isn't the entrance. The vault
//! VM's own role (holding Sargon/Gilgamesh secrets) is unaffected --
//! Rimush never touches vault content, it only launches the already-
//! independently-gated tools that do (each has its own passphrase
//! prompt, confirmed live in the Architect's own screenshots).
//!
//! WHY NO AUTH LAYER: Rimush binds 127.0.0.1 ONLY (see bind_addr()) --
//! never reachable from anywhere but a browser on this same host. The
//! security boundary is the loopback bind, not a login screen. Adding a
//! second auth layer on top of that would be pure scope creep: every
//! tool it launches already has its own real gate (Sargon Passport
//! Manager's "Unlock Sargon Vault" prompt, DubSar IDE's own Sovereign
//! Login) -- Rimush is a menu in front of already-secured doors, not a
//! new door of its own.
//!
//! WHY ZERO DEPENDENCIES: matches this ecosystem's own standing rule,
//! already proven at enkiddb-asset-server (hand-rolled HTTP/1.1 subset,
//! no HTTP crate) and eridu_build_dag.rs (std only, zero crates) --
//! "serve a known small set of pages and poll a few TCP ports" doesn't
//! need Grafana, Prometheus, or an HTTP framework. No [dependencies] in
//! this crate's Cargo.toml at all.
#![forbid(unsafe_code)]

use std::env;
use std::fs::OpenOptions;
use std::io::{self, BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::time::Duration;

const READ_TIMEOUT: u64 = 10;
const WRITE_TIMEOUT: u64 = 10;
const MAX_HEADER_BYTES: usize = 8 * 1024;
const TCP_CHECK_TIMEOUT_MS: u64 = 800;

fn bind_addr() -> String {
    // 127.0.0.1, not 0.0.0.0 -- see the module doc's WHY NO AUTH LAYER.
    env::var("BIND_ADDR").unwrap_or_else(|_| "127.0.0.1:7900".to_string())
}

fn home_dir() -> String {
    env::var("HOME").unwrap_or_else(|_| "/root".to_string())
}

fn repo_root() -> PathBuf {
    match env::var("ENKIDB_REPO_ROOT") {
        Ok(v) => PathBuf::from(v),
        Err(_) => PathBuf::from(home_dir()).join("Forge/EnkiDB"),
    }
}

fn godot_bin() -> String {
    env::var("GODOT_BIN").unwrap_or_else(|_| format!("{}/.local/bin/godot4", home_dir()))
}

fn write_host() -> String {
    env::var("WRITE_HOST").unwrap_or_else(|_| "192.168.122.111".to_string())
}
fn read_host() -> String {
    env::var("READ_HOST").unwrap_or_else(|_| "192.168.122.112".to_string())
}
fn vault_host() -> String {
    env::var("VAULT_HOST").unwrap_or_else(|_| "192.168.122.113".to_string())
}

/// Godot's own headless `--import` priming pass (PB-226/230/285's shared
/// pattern) has NO self-healing check of its own -- confirmed by reading
/// all three: the task is gated purely by `when: not skip_prime`, so
/// passing neither flag makes it run in full on EVERY single click,
/// forever, not just the first time. That was the real bug behind
/// "still very very slow" after the first fix (which correctly stopped
/// hardcoding skip_prime=true, but left nothing in its place -- trading
/// "broken on a fresh host" for "always slow"). Godot's own import cache
/// under `<project>/.godot/` is the real signal for whether a prime is
/// actually still needed: if it doesn't exist yet, this project has
/// never been primed. If it does exist, priming is still needed only if
/// the GDExtension bridge itself is NEWER than that cache (rebuilt since
/// the last prime) -- matching PB-285's own comment on why priming is
/// needed again "one after the bridge .so actually changed", just
/// finally checked instead of left to a human to know and pass by hand.
fn needs_priming(project_dir: &std::path::Path, bridge_files: &[&str]) -> bool {
    let godot_cache = project_dir.join(".godot");
    let cache_mtime = match std::fs::metadata(&godot_cache).and_then(|m| m.modified()) {
        Ok(t) => t,
        Err(_) => return true, // never primed at all
    };
    for bridge in bridge_files {
        if let Ok(bridge_mtime) =
            std::fs::metadata(project_dir.join("bin").join(bridge)).and_then(|m| m.modified())
        {
            if bridge_mtime > cache_mtime {
                return true; // rebuilt since the last prime
            }
        }
    }
    false
}

fn godot_project_dir(name: &str) -> PathBuf {
    repo_root().join("workspace/bahyway_v4/godot").join(name)
}

/// `-e godot_bin=...` always (PB-288's own proven fix for a non-
/// interactive-shell PATH not finding it), plus `-e skip_prime=true`
/// ONLY when `needs_priming` says the cache is already current for
/// THIS project+bridge set -- never hardcoded either way.
fn godot_launch_args(project_name: &str, bridge_files: &[&str]) -> Vec<String> {
    let mut args = vec!["-e".to_string(), format!("godot_bin={}", godot_bin())];
    let project_dir = godot_project_dir(project_name);
    if !needs_priming(&project_dir, bridge_files) {
        args.push("-e".to_string());
        args.push("skip_prime=true".to_string());
    }
    args
}

/// One launchable tool. `playbook` + `extra_args` reproduce EXACTLY the
/// same `ansible-playbook` invocation PB-288's own desktop `.desktop`
/// launchers use for DubSar Theater/Sargon/Gilgamesh, and PB-230's own
/// documented usage for DubSar PDM -- never a second, drifting mechanism
/// for "how do I launch this tool."
struct LaunchTarget {
    id: &'static str,
    name: &'static str,
    hue: &'static str, // CSS accent color -- distinct per tile, same intent as PB-288's per-tool hue tint
    playbook: &'static str,
    extra_args: fn() -> Vec<String>,
}

// FIXED 2026-08-21, found live: every non-Girsu target originally
// hardcoded -e skip_build=true -e skip_prime=true, copied straight from
// PB-288's own .desktop launchers. Those flags are only correct there
// because PB-288's own DONE message explicitly documents relying on a
// prior MANUAL first run ("run PB-226/285 directly, without those
// flags, first if this is the very first launch on this host") --
// exactly the kind of hidden human prerequisite Rimush exists to not
// need. On this real, freshly-bootstrapped uruk host that broke two
// different ways: DubSar PDM's own pdm-gdext bridge had never been
// built, so skip_build=true left it missing entirely (confirmed: the
// tile did nothing); Theater/Sargon/Gilgamesh's shared kupru-gdext
// bridge WAS already built from earlier session work, so those "worked"
// -- but skip_prime=true skipped the headless import-priming pass that
// had never actually run, so Godot did that same scan inline during the
// interactive launch instead, showing up as slow startup rather than a
// failure. Every one of PB-226/230/285's own build/copy/prime steps is
// already self-healing (checks whether the real destination file exists
// before deciding to skip, confirmed by reading each playbook) -- so
// the correct fix is to stop overriding that with a blanket assumption
// copied from a different context, not to try to duplicate their
// self-healing logic here.
//
// FIXED AGAIN 2026-08-21, found live right after the fix above shipped:
// removing skip_prime entirely made every click "still very very slow",
// not just the first one -- because unlike build/copy, the priming task
// in all three playbooks has NO self-healing check of its own (`when:
// not skip_prime`, nothing else). See needs_priming()/godot_launch_args()
// above: Rimush now checks each project's own `.godot/` import cache
// against its bridge file(s)' mtimes itself and only passes
// skip_prime=true when a fresh prime is provably still valid --
// self-healing on Rimush's side since the playbooks don't do it
// themselves. First click per tool is still a real build + a real prime
// (unavoidable, same as any genuine first run); every click after that,
// where nothing was rebuilt, is fast.
const LAUNCH_TARGETS: &[LaunchTarget] = &[
    LaunchTarget {
        id: "girsu",
        name: "Girsu IDE",
        hue: "#3ba55c",
        // FIXED 2026-08-21, found live: PB-542/PB-194 is pure toolchain
        // PROVISIONING (dnf package installs under become: true, which
        // hangs waiting for a sudo password with no TTY when spawned
        // from Rimush) -- it never opens VSCodium at all, so the "Girsu
        // IDE" tile did nothing. PB-542's own vars name the real launch
        // command directly (`codium_cmd: "codium"`) and the real
        // workspace it opens (`study_root:
        // ~/bahyway/study/vulkan`) -- there is no dedicated launch
        // playbook for Girsu (its real desktop shortcut was hand-built
        // once, outside any playbook, per PB-288's own comment, and was
        // never committed), so this spawns `codium` directly rather than
        // inventing or guessing at a playbook that doesn't exist.
        playbook: "",
        extra_args: || vec![],
    },
    LaunchTarget {
        id: "dubsar-pdm",
        name: "DubSar PDM IDE",
        hue: "#d99a2b",
        playbook: "playbook_230_build_and_launch_dubsar_pdm.yml",
        // PB-230's own reported bridges: "Bridge (login): .../bin/libkupru_gdext.so",
        // "Bridge (math): .../bin/libpdm_gdext.so".
        extra_args: || godot_launch_args("dubsar-pdm", &["libkupru_gdext.so", "libpdm_gdext.so"]),
    },
    LaunchTarget {
        id: "dubsar-theater",
        name: "DubSar Theater IDE",
        hue: "#6f8fd6",
        playbook: "playbook_226_launch_dubsar_godot_ide.yml",
        // PB-226's own reported bridge set, verbatim from its own DONE message.
        extra_args: || {
            godot_launch_args(
                "dubsar-theater",
                &[
                    "libkupru_gdext.so",
                    "libmarduk_gdext.so",
                    "libnaming_registry_gd.so",
                    "libnavi_translate_gdext.so",
                    "libdubsar_gridnav_gd.so",
                    "libenkimdb_registry_gd.so",
                ],
            )
        },
    },
    LaunchTarget {
        id: "sargon",
        name: "Sargon Passport Manager",
        hue: "#c266b3",
        playbook: "playbook_285_launch_sargon_or_gilgamesh_key_tool.yml",
        extra_args: || {
            let mut args = vec!["-e".to_string(), "tool=sargon".to_string()];
            args.extend(godot_launch_args(
                "sargon-passport-manager",
                &["libkupru_gdext.so"],
            ));
            args
        },
    },
    LaunchTarget {
        id: "gilgamesh",
        name: "Gilgamesh Master Key",
        hue: "#d6614f",
        playbook: "playbook_285_launch_sargon_or_gilgamesh_key_tool.yml",
        extra_args: || {
            let mut args = vec!["-e".to_string(), "tool=gilgamesh".to_string()];
            args.extend(godot_launch_args(
                "gilgamesh-master-key",
                &["libkupru_gdext.so"],
            ));
            args
        },
    },
];

/// One thing the dashboard checks liveness of, via a raw TCP connect
/// attempt -- no SSH, no agent, no extra crate. Exactly the same signal
/// PB-212's own `wait_for` tasks already treat as "is this port up."
struct MonitorTarget {
    label: &'static str,
    host_fn: fn() -> String,
    port: u16,
}

fn monitor_targets() -> Vec<MonitorTarget> {
    vec![
        MonitorTarget {
            label: "EnkiDDB Write (uruk-node-write:7101)",
            host_fn: write_host,
            port: 7101,
        },
        MonitorTarget {
            label: "EnkiMDB Write (uruk-node-write:7201)",
            host_fn: write_host,
            port: 7201,
        },
        MonitorTarget {
            label: "EnkiDDB Read (uruk-node-read:7102)",
            host_fn: read_host,
            port: 7102,
        },
        MonitorTarget {
            label: "EnkiMDB Read (uruk-node-read:7202)",
            host_fn: read_host,
            port: 7202,
        },
        MonitorTarget {
            label: "Vault SSH (uruk-node-vault:22)",
            host_fn: vault_host,
            port: 22,
        },
    ]
}

fn tcp_alive(host: &str, port: u16) -> bool {
    let addr = format!("{host}:{port}");
    match addr.parse() {
        Ok(sockaddr) => {
            TcpStream::connect_timeout(&sockaddr, Duration::from_millis(TCP_CHECK_TIMEOUT_MS))
                .is_ok()
        }
        Err(_) => false,
    }
}

/// Local bare-metal metrics read straight from /proc -- no subprocess,
/// no crate, Linux-only (this whole ecosystem is Fedora-only already).
fn local_uptime_seconds() -> Option<f64> {
    let raw = std::fs::read_to_string("/proc/uptime").ok()?;
    raw.split_whitespace().next()?.parse().ok()
}

fn local_loadavg() -> Option<String> {
    let raw = std::fs::read_to_string("/proc/loadavg").ok()?;
    raw.split_whitespace()
        .take(3)
        .collect::<Vec<_>>()
        .join(" ")
        .into()
}

fn main() {
    eprintln!("𒌷𒈬𒍑 Rimush -- BahyWay.Ecosystem Entrance GUI + Monitoring Dashboard");
    let addr = bind_addr();
    let listener = TcpListener::bind(&addr).unwrap_or_else(|e| {
        eprintln!("FATAL: bind {addr}: {e}");
        std::process::exit(1);
    });
    eprintln!("  repo_root  = {}", repo_root().display());
    eprintln!("  write_host = {}", write_host());
    eprintln!("  read_host  = {}", read_host());
    eprintln!("  vault_host = {}", vault_host());
    eprintln!("𒁾 Listening on {addr} (loopback only)");

    std::thread::scope(|scope| {
        for stream in listener.incoming() {
            match stream {
                Ok(s) => {
                    scope.spawn(move || {
                        if let Err(e) = handle(s) {
                            eprintln!("[conn] {e}");
                        }
                    });
                }
                Err(e) => eprintln!("[accept] {e}"),
            }
        }
    });
}

fn handle(mut stream: TcpStream) -> io::Result<()> {
    stream.set_read_timeout(Some(Duration::from_secs(READ_TIMEOUT)))?;
    stream.set_write_timeout(Some(Duration::from_secs(WRITE_TIMEOUT)))?;

    let mut reader = BufReader::new(stream.try_clone()?);
    let mut request_line = String::new();
    reader.read_line(&mut request_line)?;
    let Some((method, req_path)) = parse_request_line(request_line.trim_end()) else {
        return respond(
            &mut stream,
            400,
            "Bad Request",
            "text/plain",
            b"malformed request line",
        );
    };
    let method = method.to_string();
    let req_path = req_path.to_string();

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
        return respond(
            &mut stream,
            405,
            "Method Not Allowed",
            "text/plain",
            b"only GET is served",
        );
    }

    match req_path.as_str() {
        "/" => respond(
            &mut stream,
            200,
            "OK",
            "text/html; charset=utf-8",
            index_html().as_bytes(),
        ),
        "/dashboard" => respond(
            &mut stream,
            200,
            "OK",
            "text/html; charset=utf-8",
            dashboard_html().as_bytes(),
        ),
        "/api/status" => respond(
            &mut stream,
            200,
            "OK",
            "application/json",
            status_json().as_bytes(),
        ),
        p if p.starts_with("/launch/") => {
            let id = &p["/launch/".len()..];
            match LAUNCH_TARGETS.iter().find(|t| t.id == id) {
                Some(target) => {
                    launch(target);
                    respond(
                        &mut stream,
                        200,
                        "OK",
                        "text/html; charset=utf-8",
                        launched_html(target.name).as_bytes(),
                    )
                }
                None => respond(
                    &mut stream,
                    404,
                    "Not Found",
                    "text/plain",
                    b"unknown launch target",
                ),
            }
        }
        _ => respond(&mut stream, 404, "Not Found", "text/plain", b"not found"),
    }
}

// PB-542's own vars: `study_root: "{{ ansible_env.HOME }}/bahyway/study/vulkan"`.
// Girsu has no dedicated launch playbook (see LAUNCH_TARGETS' own
// comment on the "girsu" entry) -- this is the real workspace its
// existing, never-committed desktop shortcut opens.
fn girsu_study_root() -> String {
    format!("{}/bahyway/study/vulkan", home_dir())
}

/// Spawns the launch command for `target`, detached (never waited on --
/// a click must return immediately, not block the HTTP thread until
/// Godot/VSCodium exits), stdout+stderr appended to
/// /tmp/rimush_<id>_launch.log -- same "leave a real log behind, never
/// fail invisibly" discipline PB-288's own desktop launchers already use.
///
/// `target.playbook` empty means "not an ansible-playbook target" --
/// currently only Girsu, which spawns `codium` directly since no launch
/// playbook exists for it.
fn launch(target: &LaunchTarget) {
    let log_path = format!("/tmp/rimush_{}_launch.log", target.id);
    let log_file = OpenOptions::new().create(true).append(true).open(&log_path);
    let (out, err) = match log_file {
        Ok(f) => match f.try_clone() {
            Ok(f2) => (Stdio::from(f), Stdio::from(f2)),
            Err(_) => (Stdio::null(), Stdio::null()),
        },
        Err(_) => (Stdio::null(), Stdio::null()),
    };

    let mut cmd = if target.playbook.is_empty() {
        let mut c = Command::new("codium");
        c.arg(girsu_study_root());
        c
    } else {
        let mut c = Command::new("ansible-playbook");
        c.arg(target.playbook)
            .args((target.extra_args)())
            .current_dir(repo_root().join("playbooks"));
        c
    };
    cmd.stdin(Stdio::null()).stdout(out).stderr(err);

    match cmd.spawn() {
        Ok(_) => eprintln!("[launch] {} (log: {log_path})", target.name),
        Err(e) => eprintln!("[launch] FAILED to spawn {}: {e}", target.name),
    }
}

fn status_json() -> String {
    let mut checks = Vec::new();
    for t in monitor_targets() {
        let host = (t.host_fn)();
        let alive = tcp_alive(&host, t.port);
        checks.push(format!(
            r#"{{"label":"{}","host":"{}","port":{},"alive":{}}}"#,
            json_escape(t.label),
            json_escape(&host),
            t.port,
            alive
        ));
    }
    let uptime = local_uptime_seconds()
        .map(|s| format!("{:.0}", s))
        .unwrap_or_else(|| "null".into());
    let load = local_loadavg()
        .map(|s| format!(r#""{}""#, json_escape(&s)))
        .unwrap_or_else(|| "null".into());
    format!(
        r#"{{"generated_at_unix_secs":{},"local_uptime_secs":{},"local_loadavg":{},"checks":[{}]}}"#,
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0),
        uptime,
        load,
        checks.join(",")
    )
}

fn json_escape(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"")
}

fn parse_request_line(line: &str) -> Option<(&str, &str)> {
    let mut parts = line.split_whitespace();
    let method = parts.next()?;
    let path = parts.next()?;
    parts.next()?;
    Some((method, path))
}

fn respond(
    stream: &mut TcpStream,
    code: u16,
    reason: &str,
    content_type: &str,
    body: &[u8],
) -> io::Result<()> {
    let header = format!(
        "HTTP/1.1 {code} {reason}\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        body.len()
    );
    stream.write_all(header.as_bytes())?;
    stream.write_all(body)?;
    stream.flush()
}

const PAGE_STYLE: &str = r#"
  :root { color-scheme: dark; }
  body { background:#141414; color:#e6e6e6; font-family: -apple-system, "Segoe UI", sans-serif; margin:0; padding:2rem; }
  h1 { font-weight:600; letter-spacing:0.02em; }
  h1 .glyph { color:#d99a2b; }
  nav a { color:#9ab8ff; text-decoration:none; margin-right:1.2rem; font-size:0.95rem; }
  nav a:hover { text-decoration:underline; }
  .subtitle { color:#9a9a9a; margin-top:-0.5rem; margin-bottom:2rem; }
"#;

fn index_html() -> String {
    let tiles: String = LAUNCH_TARGETS
        .iter()
        .map(|t| {
            format!(
                r#"<a class="tile" href="/launch/{id}" style="--accent:{hue}">
                     <div class="tile-glyph">𒌷</div>
                     <div class="tile-name">{name}</div>
                   </a>"#,
                id = t.id,
                hue = t.hue,
                name = t.name
            )
        })
        .collect();

    format!(
        r#"<!doctype html><html><head><meta charset="utf-8"><title>Rimush</title><style>{style}
  .grid {{ display:grid; grid-template-columns: repeat(auto-fill, minmax(180px,1fr)); gap:1rem; max-width:900px; }}
  .tile {{ background:#1e1e1e; border:1px solid #2c2c2c; border-radius:10px; padding:1.4rem 1rem;
           display:flex; flex-direction:column; align-items:center; gap:0.6rem; text-decoration:none;
           color:#e6e6e6; border-top:3px solid var(--accent); transition:transform 0.1s, background 0.15s; }}
  .tile:hover {{ background:#262626; transform:translateY(-2px); }}
  .tile-glyph {{ font-size:2rem; color:var(--accent); }}
  .tile-name {{ font-size:0.95rem; text-align:center; }}
  .hint {{ color:#7a7a7a; font-size:0.85rem; margin-top:2rem; max-width:640px; }}
</style></head><body>
  <nav><a href="/">Entrance</a><a href="/dashboard">Dashboard</a></nav>
  <h1><span class="glyph">𒌷𒈬𒍑</span> Rimush</h1>
  <p class="subtitle">BahyWay.Ecosystem v4.0 -- Entrance. Click a tile to launch it on this host.</p>
  <div class="grid">{tiles}</div>
  <p class="hint">Each tile runs the same playbook its own desktop launcher uses (PB-226/230/285/542) --
     never a second launch mechanism. Output is appended to /tmp/rimush_&lt;id&gt;_launch.log.
     Nothing here handles secrets: Sargon/Gilgamesh still show their own vault-unlock prompt.</p>
</body></html>"#,
        style = PAGE_STYLE
    )
}

fn launched_html(name: &str) -> String {
    format!(
        r#"<!doctype html><html><head><meta charset="utf-8"><meta http-equiv="refresh" content="2;url=/">
        <title>Rimush -- launching</title><style>{style}</style></head><body>
  <h1>𒌷𒈬𒍑 Rimush</h1>
  <p>Launching <strong>{name}</strong>… check its own window in a moment.</p>
  <p class="hint">Returning to the entrance in 2 seconds.</p>
</body></html>"#,
        style = PAGE_STYLE
    )
}

fn dashboard_html() -> String {
    format!(
        r#"<!doctype html><html><head><meta charset="utf-8"><title>Rimush -- Dashboard</title><style>{style}
  table {{ border-collapse: collapse; max-width:720px; width:100%; }}
  td, th {{ padding:0.5rem 0.8rem; text-align:left; border-bottom:1px solid #2c2c2c; }}
  th {{ color:#9a9a9a; font-weight:500; font-size:0.85rem; text-transform:uppercase; letter-spacing:0.04em; }}
  .dot {{ display:inline-block; width:0.6rem; height:0.6rem; border-radius:50%; margin-right:0.5rem; }}
  .up {{ background:#3ba55c; }}
  .down {{ background:#d6614f; }}
  .meta {{ color:#9a9a9a; font-size:0.85rem; margin-top:1.5rem; }}
</style></head><body>
  <nav><a href="/">Entrance</a><a href="/dashboard">Dashboard</a></nav>
  <h1><span class="glyph">𒌷𒈬𒍑</span> Rimush -- Dashboard</h1>
  <p class="subtitle">Live TCP liveness for the CQRS VM nodes + this host's own load. No Grafana, no agent, no external service.</p>
  <table id="checks"><thead><tr><th>Target</th><th>Host:Port</th><th>Status</th></tr></thead><tbody></tbody></table>
  <p class="meta" id="local"></p>
  <script>
    async function refresh() {{
      const r = await fetch('/api/status');
      const data = await r.json();
      const tbody = document.querySelector('#checks tbody');
      tbody.innerHTML = data.checks.map(c =>
        `<tr><td>${{c.label}}</td><td>${{c.host}}:${{c.port}}</td>` +
        `<td><span class="dot ${{c.alive ? 'up' : 'down'}}"></span>${{c.alive ? 'up' : 'down'}}</td></tr>`
      ).join('');
      const upSecs = data.local_uptime_secs;
      const upStr = upSecs ? (upSecs/3600).toFixed(1) + 'h' : 'unknown';
      document.querySelector('#local').textContent =
        `uruk (this host): uptime ${{upStr}}, loadavg ${{data.local_loadavg ?? 'unknown'}} -- refreshed ${{new Date().toLocaleTimeString()}}`;
    }}
    refresh();
    setInterval(refresh, 5000);
  </script>
</body></html>"#,
        style = PAGE_STYLE
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_request_line_extracts_method_and_path() {
        assert_eq!(
            parse_request_line("GET /dashboard HTTP/1.1"),
            Some(("GET", "/dashboard"))
        );
        assert_eq!(
            parse_request_line("GET /launch/sargon HTTP/1.1"),
            Some(("GET", "/launch/sargon"))
        );
        assert_eq!(parse_request_line(""), None);
    }

    #[test]
    fn json_escape_handles_quotes_and_backslashes() {
        assert_eq!(json_escape(r#"a"b\c"#), r#"a\"b\\c"#);
    }

    #[test]
    fn every_launch_target_has_a_unique_id() {
        let mut ids: Vec<&str> = LAUNCH_TARGETS.iter().map(|t| t.id).collect();
        let before = ids.len();
        ids.sort();
        ids.dedup();
        assert_eq!(ids.len(), before, "duplicate launch target id");
    }

    #[test]
    fn girsu_has_no_launch_playbook_and_everything_else_does() {
        // Regression test for the real incident (2026-08-21): Girsu's
        // tile used to point at PB-542, a pure toolchain-provisioning
        // playbook that never opens VSCodium and needs sudo -- it did
        // nothing when clicked. Girsu now spawns `codium` directly
        // (empty playbook is launch()'s signal for that); every other
        // target must still go through a real playbook.
        for t in LAUNCH_TARGETS {
            if t.id == "girsu" {
                assert!(
                    t.playbook.is_empty(),
                    "girsu must have an empty playbook (direct codium spawn)"
                );
            } else {
                assert!(
                    !t.playbook.is_empty(),
                    "{} must have a real launch playbook",
                    t.id
                );
            }
        }
    }

    #[test]
    fn no_launch_target_ever_hardcodes_skip_build() {
        // Regression test for the real incident (2026-08-21): every
        // non-Girsu target used to hardcode -e skip_build=true, copied
        // from PB-288's desktop launchers -- correct only after PB-288's
        // own documented prior manual first run, which Rimush can't
        // assume. That broke DubSar PDM outright (its bridge had never
        // been built). Each playbook's own build/copy step is already
        // self-healing; Rimush must not override that. Unlike
        // skip_prime (see needs_priming_* tests below), there is no
        // scenario where Rimush should ever pass skip_build itself --
        // the build step already checks whether it's needed.
        for t in LAUNCH_TARGETS {
            let args = (t.extra_args)().join(" ");
            assert!(
                !args.contains("skip_build"),
                "{} must not hardcode skip_build",
                t.id
            );
        }
    }

    fn scratch_project_dir(name: &str) -> PathBuf {
        let dir = env::temp_dir().join(format!("rimush_test_project_{name}"));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join("bin")).unwrap();
        dir
    }

    #[test]
    fn needs_priming_is_true_when_the_project_has_never_been_primed() {
        let dir = scratch_project_dir("never_primed");
        // No .godot/ cache at all yet -- the real state of a project
        // that's never been opened/primed on this host before.
        assert!(needs_priming(&dir, &["libkupru_gdext.so"]));
    }

    #[test]
    fn needs_priming_is_false_once_the_cache_is_newer_than_every_bridge() {
        let dir = scratch_project_dir("already_primed");
        std::fs::write(dir.join("bin/libkupru_gdext.so"), b"old build").unwrap();
        std::thread::sleep(std::time::Duration::from_millis(10));
        std::fs::create_dir(dir.join(".godot")).unwrap(); // priming ran after the build
        assert!(!needs_priming(&dir, &["libkupru_gdext.so"]));
    }

    #[test]
    fn needs_priming_is_true_again_after_the_bridge_is_rebuilt_newer_than_the_cache() {
        let dir = scratch_project_dir("rebuilt_after_prime");
        std::fs::write(dir.join("bin/libkupru_gdext.so"), b"old build").unwrap();
        std::thread::sleep(std::time::Duration::from_millis(10));
        std::fs::create_dir(dir.join(".godot")).unwrap();
        std::thread::sleep(std::time::Duration::from_millis(10));
        std::fs::write(
            dir.join("bin/libkupru_gdext.so"),
            b"rebuilt, newer than the cache now",
        )
        .unwrap();
        assert!(needs_priming(&dir, &["libkupru_gdext.so"]));
    }

    #[test]
    fn status_json_is_well_formed_enough_to_pair_braces_and_brackets() {
        // No serde in this crate on purpose -- a cheap structural sanity
        // check instead of a real parser, just to catch a broken format!.
        let j = status_json();
        assert_eq!(j.matches('{').count(), j.matches('}').count());
        assert_eq!(j.matches('[').count(), j.matches(']').count());
        assert!(j.contains("\"checks\":["));
    }

    #[test]
    fn index_html_contains_a_tile_for_every_launch_target() {
        let html = index_html();
        for t in LAUNCH_TARGETS {
            assert!(
                html.contains(&format!("/launch/{}", t.id)),
                "missing tile for {}",
                t.id
            );
        }
    }
}
