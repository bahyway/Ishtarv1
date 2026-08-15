//! enkimdb-registry-gd — GDExtension bridge for DubSar IDE's Engine Map
//! tab: the tool showing how every built engine, crate, and playbook in
//! this workspace fits together, grounded in a real filesystem scan (see
//! enkimdb-registry::scan_workspace) rather than a hand-maintained list
//! that inevitably drifts from reality.
//!
//! DELIBERATELY STATELESS, same FFI-boundary law as every other bridge in
//! this workspace: each call re-scans the workspace root the caller
//! supplies. A filesystem scan over ~150 Cargo.toml files is cheap enough
//! (milliseconds) that caching would be premature complexity for a panel
//! a stakeholder opens occasionally, not a hot loop.

use enkimdb_registry::{scan_workspace, ArtifactKind, EnkiMdbRegistry, ReleaseProfile};
use godot::prelude::*;
use std::path::Path;

type VDict = Dictionary<GString, Variant>;

struct EnkimdbRegistryExtension;

#[gdextension]
unsafe impl ExtensionLibrary for EnkimdbRegistryExtension {}

#[derive(GodotClass)]
#[class(base=RefCounted, init)]
struct EnkiMdbRegistryBridge {
    base: Base<RefCounted>,
}

fn profile_to_dict(p: &ReleaseProfile) -> VDict {
    let mut d = VDict::new();
    d.set("name", p.crate_name.as_str());
    d.set("version", p.version.as_str());
    d.set("description", p.description.as_str());
    d.set("kind", match p.kind() { ArtifactKind::Crate => "crate", ArtifactKind::Binary => "binary" });
    d.set("source_subdir", p.source_subdir.as_str());
    d
}

#[godot_api]
impl EnkiMdbRegistryBridge {
    /// Real filesystem scan of `workspace_root/{crates,bin}/*/Cargo.toml`.
    /// Returns every artifact found, sorted by name -- honest empty array
    /// (never fabricated rows) if the path doesn't exist or has none.
    #[func]
    fn scan(&self, workspace_root: GString) -> VDict {
        let mut out = VDict::new();
        let profiles = scan_workspace(Path::new(&workspace_root.to_string()));
        let mut arr = Array::<Variant>::new();
        for p in &profiles {
            arr.push(&profile_to_dict(p).to_variant());
        }
        out.set("ok", true);
        out.set("count", profiles.len() as i64);
        out.set("engines", &arr);
        out
    }

    /// Same real scan, filtered by a case-insensitive name/description
    /// substring search -- the query box behind the tab's search field.
    #[func]
    fn search(&self, workspace_root: GString, query: GString) -> VDict {
        let mut out = VDict::new();
        let profiles = scan_workspace(Path::new(&workspace_root.to_string()));
        let registry = EnkiMdbRegistry::from_profiles(profiles);
        let matches = registry.search(&query.to_string());
        let mut arr = Array::<Variant>::new();
        for p in matches {
            arr.push(&profile_to_dict(p).to_variant());
        }
        out.set("ok", true);
        out.set("count", arr.len() as i64);
        out.set("engines", &arr);
        out
    }
}
