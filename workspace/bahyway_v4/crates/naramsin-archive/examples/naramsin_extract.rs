//! naramsin_extract — CLI tool for Phase 1 testing.
//! Usage: cargo run -p naramsin-archive --example naramsin_extract -- <file>

fn main() {
    let path = std::env::args()
        .nth(1)
        .expect("usage: naramsin_extract <archive_file>");

    let data = match std::fs::read(&path) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("cannot read '{path}': {e}");
            std::process::exit(1);
        }
    };

    match naramsin_archive::decompress(&data, 0) {
        Ok(files) => {
            println!("OK: {} file(s) extracted from '{path}'", files.len());
            for f in &files {
                println!("  -> {} ({} bytes)", f.name, f.data.len());
            }
        }
        Err(e) => {
            println!("ERR: {e}");
        }
    }
}
