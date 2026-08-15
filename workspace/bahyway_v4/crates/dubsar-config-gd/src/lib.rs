//! dubsar-config-gd — GDExtension bridge for a future DubSar IDE
//! configuration panel over Tupsimati (the sealed config tablet).
//!
//! HONEST SCOPE (first pass): exposes the EnkiDB-seven `DbConfig` tab
//! model (build, validate, preview, seal, emit-playbook) end to end. The
//! Tertu FIS (Mamdani rule authoring) tab is deliberately deferred — its
//! Godot-side marshalling is a separate, larger piece of work, and a
//! tablet with an empty `FisConfig` validates and seals correctly today
//! (empty sets/rules/weights is a valid, if minimal, FIS). Matches the
//! same "start light" pattern already used for pdm-gdext (deferred
//! vector-calculus ops) and navi-translate-gdext (deferred free-text MT).
//!
//! DELIBERATELY STATELESS, same FFI-boundary law kupru-gdext/pdm-gdext/
//! navi-translate-gdext already establish: the caller (GDScript) owns the
//! accumulated tablet fields as plain data (an Array of Dictionaries) and
//! passes the whole thing into every call. No signing key, no Tablet, no
//! long-lived Rust object crosses or lives behind the FFI boundary.

use godot::prelude::*;
use kupru::SealKeyPair;
use tupsimati::{fis::FisConfig, playbook::emit_playbook, DbConfig, Tablet};

type VDict = Dictionary<GString, Variant>;

struct DubsarConfigExtension;

#[gdextension]
unsafe impl ExtensionLibrary for DubsarConfigExtension {}

#[derive(GodotClass)]
#[class(base=RefCounted, init)]
struct ConfigPanelBridge {
    base: Base<RefCounted>,
}

/// One DbConfig row from a Godot Dictionary. Returns None (row skipped) on
/// any missing/malformed field rather than guessing a default for
/// something the caller should have supplied explicitly.
fn db_config_from_variant(v: &Variant) -> Option<DbConfig> {
    let d: Dictionary<GString, Variant> = v.try_to().ok()?;
    let name: GString = d.get("name")?.try_to().ok()?;
    let port: i64 = d.get("port")?.try_to().ok()?;
    let seg_mb: i64 = d.get("seg_mb")?.try_to().ok()?;
    let partition: GString = d.get("partition")?.try_to().ok()?;
    let pin_s: i64 = d.get("pin_s")?.try_to().ok()?;
    let retention_d: i64 = d.get("retention_d")?.try_to().ok()?;
    let sla_read_ms: i64 = d.get("sla_read_ms")?.try_to().ok()?;
    let sla_avail: f64 = d.get("sla_avail")?.try_to().ok()?;
    let hot_orbits: Vec<String> = d
        .get("hot_orbits")
        .and_then(|v| v.try_to::<PackedStringArray>().ok())
        .map(|a| a.as_slice().iter().map(|s| s.to_string()).collect())
        .unwrap_or_default();
    Some(DbConfig {
        name: name.to_string(),
        port: port as u16,
        journal_segment_mb: seg_mb as u32,
        partition_rule: partition.to_string(),
        anu_hot_orbits: hot_orbits,
        page_pin_window_s: pin_s as u32,
        retention_days: retention_d as u16,
        sla_read_ms: sla_read_ms as u32,
        sla_availability: sla_avail,
    })
}

fn build_tablet(version: i64, created_millis: i64, dbs: &Array<Variant>) -> Tablet {
    let dbs: Vec<DbConfig> = dbs.iter_shared().filter_map(|v| db_config_from_variant(&v)).collect();
    Tablet { version: version as u32, created_millis: created_millis as u64, dbs, fis: FisConfig::default(), seal: None }
}

#[godot_api]
impl ConfigPanelBridge {
    /// Tab names for the Theater UI, pipeline order.
    #[func]
    fn database_tabs(&self) -> PackedStringArray {
        tupsimati::ENKIDB_SEVEN.iter().map(|(n, _)| GString::from(*n)).collect()
    }

    /// Validate the whole tablet built from the caller-supplied db rows.
    /// Real EavStore-style validation (unknown database name, zero-valued
    /// destiny, rule/set consistency), not a placeholder.
    #[func]
    fn validate(&self, version: i64, dbs: Array<Variant>) -> VDict {
        let mut out = VDict::new();
        let tablet = build_tablet(version, 0, &dbs);
        match tablet.validate() {
            Ok(()) => {
                out.set("ok", true);
                out.set("valid", true);
            }
            Err(e) => {
                out.set("ok", true);
                out.set("valid", false);
                out.set("error", e);
            }
        }
        out
    }

    /// Sovereign line-format preview for the Theater's tablet view.
    #[func]
    fn preview_lines(&self, version: i64, created_millis: i64, dbs: Array<Variant>) -> GString {
        GString::from(build_tablet(version, created_millis, &dbs).to_lines().as_str())
    }

    /// Seal the tablet with a real Ed25519 signing key (32 raw bytes --
    /// the Architect's own key material, decrypted GDScript-side via the
    /// existing KupruBridge vault flow; never generated here) and, on
    /// success, immediately emit the enacting playbook text. Refuses
    /// (returns ok:false) on validation failure or a malformed key --
    /// never partially seals.
    #[func]
    fn seal_and_emit_playbook(
        &self,
        version: i64,
        created_millis: i64,
        dbs: Array<Variant>,
        signing_key_32: PackedByteArray,
        pb_number: i64,
    ) -> VDict {
        let mut out = VDict::new();
        let mut tablet = build_tablet(version, created_millis, &dbs);

        let key = match SealKeyPair::from_bytes(signing_key_32.as_slice()) {
            Ok(k) => k,
            Err(e) => {
                out.set("ok", false);
                out.set("error", format!("invalid signing key: {e:?}"));
                return out;
            }
        };

        if let Err(e) = tablet.seal_with(&key) {
            out.set("ok", false);
            out.set("error", e);
            return out;
        }

        match emit_playbook(&tablet, pb_number as u32) {
            Ok(text) => {
                out.set("ok", true);
                out.set("seal", &PackedByteArray::from(tablet.seal.as_deref().unwrap_or(&[])));
                out.set("playbook", text);
            }
            Err(e) => {
                out.set("ok", false);
                out.set("error", format!("sealed but playbook emission refused: {e}"));
            }
        }
        out
    }
}
