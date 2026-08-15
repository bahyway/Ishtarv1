//! Real filesystem scanners — no bootstrap/demo data. Every
//! `ArtifactProfile` returned here corresponds to an actual file on disk
//! at scan time.

use std::fs;
use std::path::Path;

use crate::artifact::{ArtifactKind, ArtifactProfile};

/// Scan `<root>/crates/*/Cargo.toml` for workspace crates.
pub fn scan_crates(workspace_root: &Path) -> std::io::Result<Vec<ArtifactProfile>> {
    let crates_dir = workspace_root.join("crates");
    let mut out = Vec::new();

    let mut entries: Vec<_> = fs::read_dir(&crates_dir)?.filter_map(|e| e.ok()).collect();
    entries.sort_by_key(|e| e.file_name());

    for entry in entries {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let manifest = path.join("Cargo.toml");
        if !manifest.is_file() {
            continue;
        }
        let text = fs::read_to_string(&manifest)?;
        let (name, version) = parse_package_name_version(&text);
        let name = name.unwrap_or_else(|| {
            path.file_name().unwrap().to_string_lossy().to_string()
        });
        out.push(ArtifactProfile {
            name,
            kind: ArtifactKind::Crate,
            path: manifest.display().to_string(),
            version,
        });
    }

    Ok(out)
}

/// Scan `<root>/playbooks/*.yml` for reconciled/real playbooks.
pub fn scan_playbooks(repo_root: &Path) -> std::io::Result<Vec<ArtifactProfile>> {
    let playbooks_dir = repo_root.join("playbooks");
    let mut out = Vec::new();

    let mut entries: Vec<_> = fs::read_dir(&playbooks_dir)?
        .filter_map(|e| e.ok())
        .collect();
    entries.sort_by_key(|e| e.file_name());

    for entry in entries {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("yml") {
            continue;
        }
        let name = path
            .file_stem()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_default();
        out.push(ArtifactProfile {
            name,
            kind: ArtifactKind::Playbook,
            path: path.display().to_string(),
            version: None,
        });
    }

    Ok(out)
}

/// Minimal `[package]` section parser — extracts `name` and `version`.
/// Pure std, no toml crate. A `version.workspace = true` line resolves to
/// `"workspace"` rather than a literal semver, since the real value lives
/// in the workspace root manifest.
fn parse_package_name_version(cargo_toml: &str) -> (Option<String>, Option<String>) {
    let mut name = None;
    let mut version = None;
    let mut in_package_section = false;

    for line in cargo_toml.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('[') {
            in_package_section = trimmed == "[package]";
            continue;
        }
        if !in_package_section {
            continue;
        }
        if let Some(rest) = trimmed.strip_prefix("name") {
            if let Some(v) = extract_quoted_value(rest) {
                name = Some(v);
            }
        } else if let Some(rest) = trimmed.strip_prefix("version") {
            if rest.trim_start().starts_with(".workspace") {
                version = Some("workspace".to_string());
            } else if let Some(v) = extract_quoted_value(rest) {
                version = Some(v);
            }
        }
    }

    (name, version)
}

/// Given the text after a key (e.g. ` = "enkiddb"`), extract the quoted value.
fn extract_quoted_value(rest: &str) -> Option<String> {
    let eq_pos = rest.find('=')?;
    let after_eq = rest[eq_pos + 1..].trim();
    let stripped = after_eq.strip_prefix('"')?;
    let end = stripped.find('"')?;
    Some(stripped[..end].to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn parses_explicit_name_and_version() {
        let toml = "[package]\nname = \"enkiddb\"\nversion = \"4.0.2\"\nedition = \"2021\"\n";
        let (name, version) = parse_package_name_version(toml);
        assert_eq!(name.as_deref(), Some("enkiddb"));
        assert_eq!(version.as_deref(), Some("4.0.2"));
    }

    #[test]
    fn parses_workspace_inherited_version() {
        let toml = "[package]\nname = \"enkimdb\"\nversion.workspace = true\nedition.workspace = true\n";
        let (name, version) = parse_package_name_version(toml);
        assert_eq!(name.as_deref(), Some("enkimdb"));
        assert_eq!(version.as_deref(), Some("workspace"));
    }

    #[test]
    fn does_not_read_dependency_names_as_package_name() {
        let toml = "[package]\nname = \"real-crate\"\nversion = \"1.0.0\"\n\n[dependencies]\nname = \"not-this-one\"\n";
        let (name, _version) = parse_package_name_version(toml);
        assert_eq!(name.as_deref(), Some("real-crate"));
    }

    #[test]
    fn scan_crates_finds_this_very_repo_crate() {
        // The workspace root is two levels up from this crate's manifest dir.
        let workspace_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .to_path_buf();
        let found = scan_crates(&workspace_root).expect("scan_crates should succeed");
        assert!(
            found.iter().any(|a| a.name == "enkiddb"),
            "expected to find the enkiddb crate on disk"
        );
        assert!(
            found.iter().any(|a| a.name == "enkimdb"),
            "expected to find enkimdb itself on disk"
        );
    }
}
