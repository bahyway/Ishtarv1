//! naming-registry-gd — GDExtension bridge for DubSar Theater's
//! Glossary tab: lets a new team member or an authorized client look
//! up every Akkadian/Sumerian name used across this ecosystem, its
//! category, domain role, and (where one exists) a real source-code
//! snippet.
//!
//! DELIBERATELY STATELESS, same FFI-boundary law as every other
//! bridge in this workspace (`enkimdb-registry-gd`, `marduk-gdext`,
//! ...): each call re-seeds `naming_registry::seed()`, which is an
//! in-memory `Vec` construction cheap enough (microseconds) that
//! caching would be premature complexity for a panel opened
//! occasionally, not a hot loop.
//!
//! Phase 3 (this build): every listing/lookup method takes an
//! `internal: bool` the caller must supply (Godot side: derived from
//! `SessionIdentity.privilege_level`, see `glossary_tab.gd`), and the
//! REDACTION HAPPENS HERE, not just in the Godot UI layer -- a client
//! passport can never make this bridge hand back `crate_path`, and
//! `snippet()` refuses to touch the filesystem at all for `internal:
//! false`, rather than trusting the UI not to render a field it was
//! sent anyway. `source_doc` (a governing-law tablet citation, not a
//! source-file path) is not redacted -- it's the same kind of public
//! pointer the rest of this ecosystem's sealed docs already are.
//!
//! HONEST SCOPE: kupru's passport model (`crates/kupru/src/
//! passport.rs`) only defines two concrete privilege tiers today --
//! `gardener` (privilege_level 1, "minimal read-only... entry level")
//! and `architect` (privilege_level 7, "full sovereign access"). This
//! redaction draws the internal/external line at `privilege_level >=
//! 7` because that's the only tier above gardener that exists in this
//! codebase; a real intermediate "team member, not full Architect"
//! tier would need new work in kupru itself, not something invented
//! here to fit this one panel.

use godot::prelude::*;
use naming_registry::{seed, NamingEntry, Registry};
use std::path::Path;

/// Below this `privilege_level`, `crate_path` is redacted and
/// `snippet()` refuses to read any file. See the module doc comment's
/// HONEST SCOPE note for why 7 (kupru's "architect" tier) and not
/// some intermediate value.
const INTERNAL_PRIVILEGE_THRESHOLD: i64 = 7;

type VDict = Dictionary<GString, Variant>;

struct NamingRegistryExtension;

#[gdextension]
unsafe impl ExtensionLibrary for NamingRegistryExtension {}

#[derive(GodotClass)]
#[class(base=RefCounted, init)]
struct NamingRegistryBridge {
    base: Base<RefCounted>,
}

fn entry_to_dict(e: &NamingEntry, internal: bool) -> VDict {
    let mut d = VDict::new();
    d.set("name", e.name);
    d.set("mytho_category", e.mytho_category.as_str());
    d.set("system_role", e.system_role.as_str());
    let mut tags = Array::<Variant>::new();
    for t in e.domain_tags {
        tags.push(&t.to_variant());
    }
    d.set("domain_tags", &tags);
    d.set("status", e.status.as_str());
    d.set("source_doc", e.source_doc.unwrap_or(""));
    // Redacted for non-internal viewers -- not merely omitted from the
    // Godot render, actually never sent across the FFI boundary.
    d.set(
        "crate_path",
        if internal {
            e.crate_path.unwrap_or("")
        } else {
            ""
        },
    );
    d.set("blurb", e.blurb);
    d
}

fn entries_result(entries: Vec<&NamingEntry>, internal: bool) -> VDict {
    let mut out = VDict::new();
    let mut arr = Array::<Variant>::new();
    for e in &entries {
        arr.push(&entry_to_dict(e, internal).to_variant());
    }
    out.set("ok", true);
    out.set("count", entries.len() as i64);
    out.set("entries", &arr);
    out
}

/// Pure-Rust core of the threshold check, kept independent of any
/// Godot type so it's testable without a live engine -- same split
/// `marduk-gdext::to_seven_slice` uses and for the same reason.
fn is_internal_privilege(privilege_level: i64) -> bool {
    privilege_level >= INTERNAL_PRIVILEGE_THRESHOLD
}

#[godot_api]
impl NamingRegistryBridge {
    /// Single source of truth for the internal/external line -- callers
    /// pass the session's `privilege_level` (see `SessionIdentity` in
    /// `glossary_tab.gd`) and get back whether that session may see
    /// `crate_path`/`snippet()` output. Keeps the threshold defined
    /// once, here, instead of duplicated as a magic number in GDScript.
    #[func]
    fn is_internal(&self, privilege_level: i64) -> bool {
        is_internal_privilege(privilege_level)
    }

    /// Every seeded entry, unfiltered.
    #[func]
    fn all(&self, internal: bool) -> VDict {
        let reg: Registry = seed();
        entries_result(reg.0.iter().collect(), internal)
    }

    /// Case-insensitive substring search over name/blurb/domain tags
    /// -- the search box behind the Glossary tab. Empty query returns
    /// everything, same as `all()`.
    #[func]
    fn search(&self, query: GString, internal: bool) -> VDict {
        let reg = seed();
        entries_result(reg.search(&query.to_string()), internal)
    }

    /// Filter by mytho_category string: "Deity" | "City" | "KingOrEra"
    /// | "GenericTerm" (case-insensitive). Unknown values return an
    /// honest empty result, never a fabricated match.
    #[func]
    fn by_mytho_category(&self, category: GString, internal: bool) -> VDict {
        use naming_registry::MythoCategory::*;
        let reg = seed();
        let cat_str = category.to_string().to_lowercase();
        let matched: Option<naming_registry::MythoCategory> = match cat_str.as_str() {
            "deity" => Some(Deity),
            "city" => Some(City),
            "kingorera" => Some(KingOrEra),
            "genericterm" => Some(GenericTerm),
            _ => None,
        };
        match matched {
            Some(cat) => entries_result(reg.by_mytho_category(cat), internal),
            None => {
                godot_error!("NamingRegistryBridge.by_mytho_category: unknown category {category}");
                entries_result(vec![], internal)
            }
        }
    }

    /// Filter by system_role string, e.g. "Engine", "Calculus",
    /// "AlertClass", "Pattern", "Agent", "Suite", "Tool", "Protocol",
    /// "IndexStack", "Language" (case-insensitive).
    #[func]
    fn by_system_role(&self, role: GString, internal: bool) -> VDict {
        use naming_registry::SystemRole::*;
        let reg = seed();
        let role_str = role.to_string().to_lowercase();
        let matched: Option<naming_registry::SystemRole> = match role_str.as_str() {
            "engine" => Some(Engine),
            "calculus" => Some(Calculus),
            "alertclass" => Some(AlertClass),
            "pattern" => Some(Pattern),
            "agent" => Some(Agent),
            "suite" => Some(Suite),
            "tool" => Some(Tool),
            "protocol" => Some(Protocol),
            "indexstack" => Some(IndexStack),
            "language" => Some(Language),
            _ => None,
        };
        match matched {
            Some(r) => entries_result(reg.by_system_role(r), internal),
            None => {
                godot_error!("NamingRegistryBridge.by_system_role: unknown role {role}");
                entries_result(vec![], internal)
            }
        }
    }

    /// One entry by exact (case-insensitive) name, or {ok:false} if
    /// no such name is seeded.
    #[func]
    fn find(&self, name: GString, internal: bool) -> VDict {
        let reg = seed();
        let mut out = VDict::new();
        match reg.find(&name.to_string()) {
            Some(e) => {
                out.set("ok", true);
                out.set("entry", &entry_to_dict(e, internal).to_variant());
            }
            None => {
                out.set("ok", false);
                out.set("error", format!("no entry named {name}"));
            }
        }
        out
    }

    /// A real excerpt (first `max_lines` lines) of the source file
    /// backing `name`, if one exists AND `internal` is true.
    /// `{ok:false, error:...}` for Reserved/conceptual entries, a read
    /// failure, or a non-internal caller -- the filesystem is never
    /// touched at all when `internal` is false, so this is real
    /// enforcement, not a field the UI merely chooses not to render.
    #[func]
    fn snippet(
        &self,
        name: GString,
        workspace_root: GString,
        max_lines: i64,
        internal: bool,
    ) -> VDict {
        let reg = seed();
        let mut out = VDict::new();
        let Some(entry) = reg.find(&name.to_string()) else {
            out.set("ok", false);
            out.set("error", format!("no entry named {name}"));
            return out;
        };
        if !internal {
            out.set("ok", false);
            out.set(
                "error",
                "redacted: viewing source snippets requires an internal (architect-tier) passport",
            );
            return out;
        }
        let lines = if max_lines > 0 {
            max_lines as usize
        } else {
            60
        };
        match naming_registry::read_snippet(Path::new(&workspace_root.to_string()), entry, lines) {
            Ok(text) => {
                out.set("ok", true);
                out.set("snippet", text.as_str());
                out.set("crate_path", entry.crate_path.unwrap_or(""));
            }
            Err(e) => {
                out.set("ok", false);
                out.set("error", e.as_str());
            }
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gardener_tier_is_not_internal() {
        assert!(!is_internal_privilege(1)); // kupru's gardener() factory
    }

    #[test]
    fn architect_tier_is_internal() {
        assert!(is_internal_privilege(7)); // kupru's architect() factory
    }

    #[test]
    fn unauthenticated_session_is_not_internal() {
        assert!(!is_internal_privilege(0)); // SessionIdentity's default before login
    }

    #[test]
    fn threshold_is_a_hard_boundary_not_a_fuzzy_one() {
        assert!(!is_internal_privilege(INTERNAL_PRIVILEGE_THRESHOLD - 1));
        assert!(is_internal_privilege(INTERNAL_PRIVILEGE_THRESHOLD));
    }
}
