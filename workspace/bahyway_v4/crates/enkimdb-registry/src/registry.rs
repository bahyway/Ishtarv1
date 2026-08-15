//! The registry data model: what an artifact is, and the in-memory
//! collection query surface over a scanned set of them.

/// What kind of sovereign artifact a release profile describes. Inferred
/// from where it was found (`crates/` vs `bin/`) -- coarse but honest;
/// finer classification (Engine vs Template vs App) needs a real
/// convention decision, not a guess, so it's left for later.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArtifactKind {
    Crate,
    Binary,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReleaseProfile {
    pub crate_name: String,
    pub version: String,
    pub description: String,
    /// "crates" or "bin" -- which workspace subdirectory this was found
    /// under.
    pub source_subdir: String,
}

impl ReleaseProfile {
    pub fn kind(&self) -> ArtifactKind {
        if self.source_subdir == "bin" {
            ArtifactKind::Binary
        } else {
            ArtifactKind::Crate
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct EnkiMdbRegistry {
    profiles: Vec<ReleaseProfile>,
}

impl EnkiMdbRegistry {
    pub fn from_profiles(profiles: Vec<ReleaseProfile>) -> Self {
        EnkiMdbRegistry { profiles }
    }

    pub fn len(&self) -> usize {
        self.profiles.len()
    }

    pub fn is_empty(&self) -> bool {
        self.profiles.is_empty()
    }

    pub fn all(&self) -> &[ReleaseProfile] {
        &self.profiles
    }

    pub fn find(&self, crate_name: &str) -> Option<&ReleaseProfile> {
        self.profiles.iter().find(|p| p.crate_name == crate_name)
    }

    /// Case-insensitive substring search over name + description -- the
    /// real query shape the DubSar IDE panel needs (a search box).
    pub fn search(&self, query: &str) -> Vec<&ReleaseProfile> {
        let q = query.to_ascii_lowercase();
        if q.is_empty() {
            return self.profiles.iter().collect();
        }
        self.profiles
            .iter()
            .filter(|p| p.crate_name.to_ascii_lowercase().contains(&q) || p.description.to_ascii_lowercase().contains(&q))
            .collect()
    }

    pub fn by_kind(&self, kind: ArtifactKind) -> Vec<&ReleaseProfile> {
        self.profiles.iter().filter(|p| p.kind() == kind).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> EnkiMdbRegistry {
        EnkiMdbRegistry::from_profiles(vec![
            ReleaseProfile {
                crate_name: "dagan-engine".into(),
                version: "4.0.2".into(),
                description: "data analysis station".into(),
                source_subdir: "crates".into(),
            },
            ReleaseProfile {
                crate_name: "dubsar".into(),
                version: "4.0.2".into(),
                description: "the DubSar CLI".into(),
                source_subdir: "bin".into(),
            },
        ])
    }

    #[test]
    fn find_by_exact_name() {
        let r = sample();
        assert!(r.find("dagan-engine").is_some());
        assert!(r.find("nonexistent").is_none());
    }

    #[test]
    fn search_matches_name_or_description_case_insensitively() {
        let r = sample();
        assert_eq!(r.search("DAGAN").len(), 1);
        assert_eq!(r.search("analysis").len(), 1);
        assert_eq!(r.search("cli").len(), 1);
        assert_eq!(r.search("").len(), 2); // empty query returns all
        assert_eq!(r.search("no-such-thing").len(), 0);
    }

    #[test]
    fn kind_inferred_from_source_subdir() {
        let r = sample();
        assert_eq!(r.by_kind(ArtifactKind::Crate).len(), 1);
        assert_eq!(r.by_kind(ArtifactKind::Binary).len(), 1);
    }
}
