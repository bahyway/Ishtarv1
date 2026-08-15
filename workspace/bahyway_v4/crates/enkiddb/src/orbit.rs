//! DocOrbit — the grid classification for a document's particles.
//!
//! There is no separate physical "orbit file" here (that storage-engine
//! rearchitecture is ADR-012 scope, not this crate's). DocOrbit instead
//! namespaces the `attribute` string on each `enkidb_particles::Particle`
//! (e.g. "meta.title", "head.h2", "body.paragraph"), the same convention
//! already used elsewhere in the codebase ("quality.score", "geo.coord").

/// Grid classification for a document particle.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum DocOrbit {
    /// Document-level metadata (title, language, format).
    Meta,
    /// Structural headers (h1-h6).
    Head,
    /// Content body (paragraphs, lists, blockquotes).
    Body,
    /// Code blocks and snippets.
    Code,
    /// Version history (birth/update events).
    Hist,
    /// Cross-document references (CrossTribe-Kaki links).
    Link,
}

impl DocOrbit {
    /// Dot-namespace prefix used on every attribute string in this orbit.
    pub fn prefix(&self) -> &'static str {
        match self {
            DocOrbit::Meta => "meta",
            DocOrbit::Head => "head",
            DocOrbit::Body => "body",
            DocOrbit::Code => "code",
            DocOrbit::Hist => "hist",
            DocOrbit::Link => "link",
        }
    }

    /// Build a namespaced attribute string, e.g. `DocOrbit::Meta.attr("title")` -> "meta.title".
    pub fn attr(&self, name: &str) -> String {
        format!("{}.{}", self.prefix(), name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn attr_namespaces_correctly() {
        assert_eq!(DocOrbit::Meta.attr("title"), "meta.title");
        assert_eq!(DocOrbit::Head.attr("h2"), "head.h2");
        assert_eq!(DocOrbit::Body.attr("paragraph"), "body.paragraph");
    }
}
