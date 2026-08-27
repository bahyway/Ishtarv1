//! Same minimal argv parser as `bin/bahyway-enkidb/src/args.rs` and
//! `bin/bahyway-lamassu/src/args.rs` -- kept as a third small copy rather
//! than a shared crate, same reasoning as those two: each CLI's own
//! subcommand set never needs to share behavior beyond this parsing.

use std::collections::HashMap;

#[derive(Debug, Default)]
pub struct Flags {
    values: HashMap<String, String>,
    bools: std::collections::HashSet<String>,
    pub positionals: Vec<String>,
}

impl Flags {
    pub fn get(&self, name: &str) -> Option<&str> {
        self.values.get(name).map(|s| s.as_str())
    }

    pub fn get_or(&self, name: &str, default: &str) -> String {
        self.get(name).unwrap_or(default).to_string()
    }

    pub fn has(&self, name: &str) -> bool {
        self.bools.contains(name) || self.values.contains_key(name)
    }

    pub fn port(&self) -> u16 {
        self.get("port").and_then(|s| s.parse().ok()).unwrap_or(0)
    }
}

/// A token is a flag value only when it doesn't itself look like another
/// `--flag`. A bare `-3..3`-style negative range (PB-663's `--shells
/// -3..3`) is a single dash, so it's still accepted as a value.
pub fn parse(args: &[String]) -> Flags {
    let mut f = Flags::default();
    let mut i = 0;
    while i < args.len() {
        let a = &args[i];
        if let Some(name) = a.strip_prefix("--") {
            let has_value = i + 1 < args.len() && !args[i + 1].starts_with("--");
            if has_value {
                f.values.insert(name.to_string(), args[i + 1].clone());
                i += 2;
            } else {
                f.bools.insert(name.to_string());
                i += 1;
            }
        } else {
            f.positionals.push(a.clone());
            i += 1;
        }
    }
    f
}
