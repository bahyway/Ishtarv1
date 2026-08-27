//! Nabu Notebook (`.akknb`) — typed cell format for BahyWay v4.0.
//!
//! A `.akknb` file is markdown with typed code fences.  Each fence opener
//! carries a cell-kind tag that tells the Nabu runtime how to execute the
//! cell (compile `.akk`, run a query, render a canvas, etc.).
//!
//! The 10 canonical cell kinds are sealed here.  Adding a new kind is a
//! minor-version change; removing or renaming a kind is breaking.

/// The 10 canonical cell kinds for Nabu Notebook (`.akknb`) files.
///
/// In a notebook source file the fence tag is the lowercase `as_tag()` string:
/// ` ```akk `, ` ```hepta `, ` ```vgca `, etc.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum CellKind {
    /// AkkadianAOL source code — compiled and executed by the Nabu engine.
    Akk,
    /// HeptaScript query — evaluated against the live EnkiDB.
    Hepta,
    /// VGCA / geometric validation cell — runs the geometry pipeline.
    Vgca,
    /// Way security policy fragment — compiled to Lamassu runtime config.
    Way,
    /// Markdown prose — rendered as HTML, never executed.
    Prose,
    /// HTML Canvas 2D visualization cell — static render, no animation.
    Canvas,
    /// Shell command — executed in the sovereign shell sandbox.
    Shell,
    /// JSON data cell — inlined data, no execution.
    Json,
    /// SQL query cell — sent to the EnkiDB SQL gateway.
    Sql,
    /// Output cell — read-only result block written by the runtime.
    Output,
}

impl CellKind {
    /// The fence tag as it appears in the `.akknb` source (lowercase).
    ///
    /// ```text
    /// ```akk
    /// let x = 1;
    /// ```
    /// ```
    pub fn as_tag(self) -> &'static str {
        match self {
            CellKind::Akk => "akk",
            CellKind::Hepta => "hepta",
            CellKind::Vgca => "vgca",
            CellKind::Way => "way",
            CellKind::Prose => "prose",
            CellKind::Canvas => "canvas",
            CellKind::Shell => "shell",
            CellKind::Json => "json",
            CellKind::Sql => "sql",
            CellKind::Output => "output",
        }
    }

    /// Human-readable description of what this cell kind does at runtime.
    pub fn description(self) -> &'static str {
        match self {
            CellKind::Akk => "AkkadianAOL source — compiled by Nabu engine",
            CellKind::Hepta => "HeptaScript query — evaluated against EnkiDB",
            CellKind::Vgca => "VGCA geometric validation pipeline",
            CellKind::Way => "Way security policy — compiled to Lamassu config",
            CellKind::Prose => "Markdown prose — rendered as HTML",
            CellKind::Canvas => "HTML Canvas 2D visualization (no animation)",
            CellKind::Shell => "Shell command — sovereign sandbox",
            CellKind::Json => "Inline JSON data — not executed",
            CellKind::Sql => "SQL query — sent to EnkiDB SQL gateway",
            CellKind::Output => "Runtime output — read-only result block",
        }
    }

    /// Returns `true` when the cell is executable (produces output at runtime).
    pub fn is_executable(self) -> bool {
        matches!(
            self,
            CellKind::Akk
                | CellKind::Hepta
                | CellKind::Vgca
                | CellKind::Way
                | CellKind::Shell
                | CellKind::Sql
        )
    }

    /// Returns `true` when the cell is read-only (never executed or modified
    /// by the runtime — only by the user).
    pub fn is_readonly(self) -> bool {
        matches!(self, CellKind::Output | CellKind::Json)
    }

    /// Parse a fence tag string into a `CellKind`, if known.
    pub fn from_tag(tag: &str) -> Option<Self> {
        match tag {
            "akk" => Some(CellKind::Akk),
            "hepta" => Some(CellKind::Hepta),
            "vgca" => Some(CellKind::Vgca),
            "way" => Some(CellKind::Way),
            "prose" => Some(CellKind::Prose),
            "canvas" => Some(CellKind::Canvas),
            "shell" => Some(CellKind::Shell),
            "json" => Some(CellKind::Json),
            "sql" => Some(CellKind::Sql),
            "output" => Some(CellKind::Output),
            _ => None,
        }
    }

    /// All 10 canonical cell kinds, in declaration order.
    pub fn all() -> &'static [CellKind] {
        &[
            CellKind::Akk,
            CellKind::Hepta,
            CellKind::Vgca,
            CellKind::Way,
            CellKind::Prose,
            CellKind::Canvas,
            CellKind::Shell,
            CellKind::Json,
            CellKind::Sql,
            CellKind::Output,
        ]
    }
}

impl core::fmt::Display for CellKind {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(self.as_tag())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_returns_ten_kinds() {
        assert_eq!(CellKind::all().len(), 10);
    }

    #[test]
    fn from_tag_round_trip() {
        for kind in CellKind::all() {
            let parsed = CellKind::from_tag(kind.as_tag());
            assert_eq!(
                parsed,
                Some(*kind),
                "round-trip failed for {}",
                kind.as_tag()
            );
        }
    }

    #[test]
    fn unknown_tag_returns_none() {
        assert_eq!(CellKind::from_tag("rust"), None);
        assert_eq!(CellKind::from_tag(""), None);
        assert_eq!(CellKind::from_tag("HEPTA"), None); // case-sensitive
        assert_eq!(CellKind::from_tag("markdown"), None);
    }

    #[test]
    fn executable_kinds() {
        assert!(CellKind::Akk.is_executable());
        assert!(CellKind::Hepta.is_executable());
        assert!(CellKind::Vgca.is_executable());
        assert!(CellKind::Way.is_executable());
        assert!(CellKind::Shell.is_executable());
        assert!(CellKind::Sql.is_executable());
        assert!(!CellKind::Prose.is_executable());
        assert!(!CellKind::Canvas.is_executable());
        assert!(!CellKind::Json.is_executable());
        assert!(!CellKind::Output.is_executable());
    }

    #[test]
    fn readonly_kinds() {
        assert!(CellKind::Output.is_readonly());
        assert!(CellKind::Json.is_readonly());
        assert!(!CellKind::Akk.is_readonly());
        assert!(!CellKind::Hepta.is_readonly());
    }

    #[test]
    fn display_matches_tag() {
        for kind in CellKind::all() {
            assert_eq!(kind.to_string(), kind.as_tag());
        }
    }

    #[test]
    fn descriptions_non_empty() {
        for kind in CellKind::all() {
            assert!(!kind.description().is_empty());
        }
    }

    #[test]
    fn no_duplicate_tags() {
        let mut seen = std::collections::HashSet::new();
        for kind in CellKind::all() {
            assert!(
                seen.insert(kind.as_tag()),
                "duplicate tag: {}",
                kind.as_tag()
            );
        }
    }
}
