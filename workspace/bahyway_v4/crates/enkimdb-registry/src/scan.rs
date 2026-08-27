//! Real-filesystem scanner: reads this workspace's actual `Cargo.toml`
//! files under `crates/` and `bin/` and turns each into a `ReleaseProfile`.
//! Deliberately hand-rolled, minimal TOML field extraction (name, version,
//! description, workspace-version fallback) rather than a full TOML
//! parser dependency -- `[package]` is simple enough that a small, honest,
//! line-oriented reader covers every real crate in this workspace, and
//! adding nothing keeps this scan step dependency-free.

use crate::registry::ReleaseProfile;
use std::fs;
use std::path::Path;

/// Extract `version = "X.Y.Z"` from the `[workspace.package]` section of
/// the workspace root `Cargo.toml`, for crates that declare
/// `version.workspace = true` instead of a literal version.
fn parse_workspace_version(root_toml: &str) -> Option<String> {
    let mut in_section = false;
    for line in root_toml.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('[') {
            in_section = trimmed == "[workspace.package]";
            continue;
        }
        if in_section {
            if let Some(v) = extract_quoted_value(trimmed, "version") {
                return Some(v);
            }
        }
    }
    None
}

/// Extract `key = "value"` (quoted string) from one line, if `line` starts
/// with `key` (ignoring surrounding whitespace).
fn extract_quoted_value(line: &str, key: &str) -> Option<String> {
    let line = line.trim();
    let rest = line.strip_prefix(key)?.trim_start();
    let rest = rest.strip_prefix('=')?.trim_start();
    let rest = rest.strip_prefix('"')?;
    let end = rest.find('"')?;
    Some(rest[..end].to_string())
}

fn is_workspace_version_ref(line: &str, key: &str) -> bool {
    let line = line.trim();
    // e.g. "version.workspace = true" or "version = { workspace = true }"
    line.starts_with(&format!("{key}.workspace"))
        || (line.starts_with(key) && line.contains("workspace = true"))
}

#[derive(Default)]
struct PackageFields {
    name: Option<String>,
    version: Option<String>,
    description: Option<String>,
}

fn parse_package_section(
    cargo_toml: &str,
    workspace_version: Option<&str>,
) -> Option<PackageFields> {
    let mut fields = PackageFields::default();
    let mut in_package = false;
    for line in cargo_toml.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('[') {
            in_package = trimmed == "[package]";
            continue;
        }
        if !in_package {
            continue;
        }
        if fields.name.is_none() {
            if let Some(v) = extract_quoted_value(trimmed, "name") {
                fields.name = Some(v);
                continue;
            }
        }
        if fields.version.is_none() {
            if is_workspace_version_ref(trimmed, "version") {
                fields.version = workspace_version.map(|v| v.to_string());
                continue;
            }
            if let Some(v) = extract_quoted_value(trimmed, "version") {
                fields.version = Some(v);
                continue;
            }
        }
        if fields.description.is_none() {
            if let Some(v) = extract_quoted_value(trimmed, "description") {
                fields.description = Some(v);
                continue;
            }
        }
    }
    if fields.name.is_some() {
        Some(fields)
    } else {
        None
    }
}

/// Scan every immediate subdirectory of `crates/` and `bin/` under
/// `workspace_root` for a `Cargo.toml`, and build a `ReleaseProfile` for
/// each real package found. Returns an empty vec (never an error) for a
/// directory that doesn't exist -- an honest "nothing here", not a crash.
pub fn scan_workspace(workspace_root: &Path) -> Vec<ReleaseProfile> {
    let root_toml = fs::read_to_string(workspace_root.join("Cargo.toml")).unwrap_or_default();
    let workspace_version = parse_workspace_version(&root_toml);

    let mut out = Vec::new();
    for subdir in ["crates", "bin"] {
        let dir = workspace_root.join(subdir);
        let Ok(entries) = fs::read_dir(&dir) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }
            let cargo_toml_path = path.join("Cargo.toml");
            let Ok(contents) = fs::read_to_string(&cargo_toml_path) else {
                continue;
            };
            let Some(fields) = parse_package_section(&contents, workspace_version.as_deref())
            else {
                continue;
            };
            let Some(name) = fields.name else { continue };
            out.push(ReleaseProfile {
                crate_name: name,
                version: fields.version.unwrap_or_else(|| "unknown".to_string()),
                description: fields.description.unwrap_or_default(),
                source_subdir: subdir.to_string(),
            });
        }
    }
    out.sort_by(|a, b| a.crate_name.cmp(&b.crate_name));
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_workspace_version_from_root_toml() {
        let root = r#"
[workspace]
members = ["crates/foo"]

[workspace.package]
version = "4.0.2"
edition = "2021"
"#;
        assert_eq!(parse_workspace_version(root), Some("4.0.2".to_string()));
    }

    #[test]
    fn parses_a_literal_version_package() {
        let toml = r#"
[package]
name = "wpd-engine"
version = "4.0.0"
description = "leak localization"
"#;
        let fields = parse_package_section(toml, None).unwrap();
        assert_eq!(fields.name, Some("wpd-engine".to_string()));
        assert_eq!(fields.version, Some("4.0.0".to_string()));
        assert_eq!(fields.description, Some("leak localization".to_string()));
    }

    #[test]
    fn resolves_version_workspace_true_against_the_workspace_version() {
        let toml = r#"
[package]
name              = "dagan-engine"
version.workspace = true
edition.workspace = true
description       = "analysis"
"#;
        let fields = parse_package_section(toml, Some("4.0.2")).unwrap();
        assert_eq!(fields.version, Some("4.0.2".to_string()));
    }

    #[test]
    fn scans_this_real_workspace_and_finds_real_crates_built_this_session() {
        // Grounded against the actual repo, not a fixture -- this is the
        // whole point of this crate.
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let profiles = scan_workspace(&root);
        assert!(
            !profiles.is_empty(),
            "expected to find real crates under crates/"
        );
        let names: Vec<&str> = profiles.iter().map(|p| p.crate_name.as_str()).collect();
        assert!(names.contains(&"dagan-engine"));
        assert!(names.contains(&"girra-engine"));
        assert!(names.contains(&"elu-tribe-hotindex"));
        assert!(names.contains(&"acoustic-leak-engine"));
        assert!(names.contains(&"tupsimati"));
        assert!(names.contains(&"enkimdb-registry")); // finds itself
                                                      // Every found profile has a real, non-"unknown" version (all real
                                                      // crates here declare one, literal or workspace-inherited).
        for p in &profiles {
            assert_ne!(
                p.version, "unknown",
                "{} has no resolvable version",
                p.crate_name
            );
        }
    }

    #[test]
    fn nonexistent_workspace_root_returns_empty_not_a_crash() {
        let profiles = scan_workspace(Path::new("/no/such/path/at/all"));
        assert!(profiles.is_empty());
    }
}
