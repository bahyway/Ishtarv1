//! navi-translate-gdext — GDExtension bridge for DubSar IDE's AR-EN
//! Navigator glossary plugin.
//!
//! HONEST SCOPE (2026-07-24): this answers the Architect's own question
//! ("can vectorizing Arabic words help in real translation, or only for
//! security purposes?") with real, already-tested building blocks rather
//! than a new algorithm. It is a bilingual NAVIGATION-TERM GLOSSARY —
//! exact + fuzzy lookup of place/POI names by either their English or
//! Arabic label — not free-text machine translation. That distinction
//! matters: adapa-recall's TF-IDF + cosine similarity (crates/adapa-recall,
//! already real and tested) is lexical retrieval over whatever text a
//! caller supplies; it finds documents/names that share distinguishing
//! terms, it does not understand grammar or produce fluent sentences in
//! the other language. For a "which grave/gate/shrine is this Arabic name
//! referring to" lookup during Najaf/Baghdad field navigation, lexical
//! retrieval over a bilingual gazetteer is exactly the right tool — using
//! neural embeddings here would not just be unnecessary, it would trade a
//! transparent, auditable match for an opaque one, which is a real cost
//! for a security-sensitive naming context (see this crate's own EAV
//! convention below, `ATTR_ACCESS_LEVEL`, for how seriously that's taken
//! elsewhere in navi-engine).
//!
//! Two real crates, glued, nothing reimplemented:
//!   - `navi-engine::eav` — the sovereign EAV attribute store, specifically
//!     `ATTR_NAME` / `ATTR_NAME_ARABIC`, the SAME convention already used
//!     for real in `dubsar-theater`'s own PDM tab default query
//!     (`ATTR person.name_arabic` in pdm_tab.gd's `najaf_deaths` model)
//!     and in navi-engine's own road/POI schema. `validate_place` below
//!     uses `EavStore::required_missing()` for real, not a fabricated
//!     check.
//!   - `adapa-recall::RecallIndex` — real TF-IDF + cosine retrieval
//!     (crates/adapa-recall, already wired into TamuzAI's memory search).
//!     Rebuilt fresh per lookup call, matching that crate's own documented
//!     pattern ("rebuild when the corpus changes" — no incremental-index
//!     API exists, so none is invented here).
//!
//! DELIBERATELY STATELESS, same law `kupru-gdext`/`pdm-gdext` already
//! establish: no long-lived Rust object (no in-memory gazetteer) crosses
//! or lives behind the FFI boundary. The caller (GDScript side, see
//! `navi_translate_tab.gd`) owns the actual list of known places as a
//! plain Array of Dictionaries and passes the whole thing into every
//! call — small enough (a navigation glossary, not a live infra dataset)
//! that this costs nothing and keeps every call independently testable.

use adapa_recall::{Document, RecallIndex};
use godot::prelude::*;
use navi_engine::{
    haversine_m, ATTR_NAME, ATTR_NAME_ARABIC, AttrValue, EavAttr, EavStore,
};

type VDict = Dictionary<GString, Variant>;

struct NaviTranslateExtension;

#[gdextension]
unsafe impl ExtensionLibrary for NaviTranslateExtension {}

#[derive(GodotClass)]
#[class(base=RefCounted, init)]
struct NaviTranslateBridge {
    base: Base<RefCounted>,
}

/// One entry of the caller-supplied gazetteer: {"name": String,
/// "name_arabic": String, "lat": float, "lon": float}. Any other keys are
/// ignored. Returns None (and lets the caller skip the row) if "name" or
/// "name_arabic" is missing/empty — never fabricates a placeholder label.
fn place_from_variant(v: &Variant) -> Option<(GString, GString, f64, f64)> {
    let d: Dictionary<GString, Variant> = v.try_to().ok()?;
    let name: GString = d.get("name")?.try_to().ok()?;
    let name_arabic: GString = d.get("name_arabic")?.try_to().ok()?;
    if name.is_empty() || name_arabic.is_empty() {
        return None;
    }
    let lat: f64 = d.get("lat").and_then(|v| v.try_to().ok()).unwrap_or(0.0);
    let lon: f64 = d.get("lon").and_then(|v| v.try_to().ok()).unwrap_or(0.0);
    Some((name, name_arabic, lat, lon))
}

#[godot_api]
impl NaviTranslateBridge {
    /// Real use of navi-engine's own EAV convention: builds an EavStore
    /// with ATTR_NAME/ATTR_NAME_ARABIC both marked required, and reports
    /// EavStore::required_missing() honestly. Given two non-empty strings
    /// this always reports valid=true — its purpose is to demonstrate
    /// (and let the Godot side exercise) the real sovereign attribute
    /// convention, not to invent a validation rule that doesn't exist.
    #[func]
    fn validate_place(&self, name: GString, name_arabic: GString) -> VDict {
        let mut out = VDict::new();
        let mut eav = EavStore::new();
        eav.set(EavAttr::required(ATTR_NAME, AttrValue::Text(name.to_string())));
        eav.set(EavAttr::required(
            ATTR_NAME_ARABIC,
            AttrValue::Text(name_arabic.to_string()),
        ));
        let missing = eav.required_missing();
        out.set("ok", true);
        out.set("valid", missing.is_empty());
        out.set("attr_count", eav.attr_count() as i64);
        out
    }

    /// Bilingual glossary lookup. `query` may be Arabic or English (or a
    /// transliterated fragment of either) — adapa-recall's TF-IDF/cosine
    /// index is built fresh over `name + " " + name_arabic` for every
    /// place in `places`, so a query naming either script's distinguishing
    /// terms ranks the right place first. Returns at most `top_k` matches,
    /// score-descending, each carrying both names back so the caller never
    /// has to guess which script a match came from.
    #[func]
    fn lookup_by_name(&self, query: GString, places: Array<Variant>, top_k: i64) -> VDict {
        let mut out = VDict::new();
        let mut rows: Vec<(GString, GString, f64, f64)> = Vec::new();
        for v in places.iter_shared() {
            if let Some(row) = place_from_variant(&v) {
                rows.push(row);
            }
        }
        if rows.is_empty() {
            out.set("ok", true);
            out.set("matches", &Array::<Variant>::new());
            return out;
        }

        let docs: Vec<Document> = rows
            .iter()
            .enumerate()
            .map(|(i, (name, name_arabic, _, _))| {
                Document::new(i.to_string(), format!("{name} {name_arabic}"))
            })
            .collect();
        let index = RecallIndex::build(docs);
        let k = if top_k > 0 { top_k as usize } else { 5 };
        let results = index.query(&query.to_string(), k);

        let mut matches = Array::<Variant>::new();
        for (doc_id, score) in results {
            let idx: usize = match doc_id.parse() {
                Ok(i) => i,
                Err(_) => continue,
            };
            let (name, name_arabic, lat, lon) = &rows[idx];
            let mut m = VDict::new();
            m.set("name", name);
            m.set("name_arabic", name_arabic);
            m.set("lat", *lat);
            m.set("lon", *lon);
            m.set("score", score as f64);
            matches.push(&m.to_variant());
        }
        out.set("ok", true);
        out.set("matches", &matches);
        out
    }

    /// Real haversine great-circle distance in metres (navi-engine's own
    /// `haversine_m`, already used by RouteEngine's A* heuristic — not
    /// reimplemented here).
    #[func]
    fn distance_m(&self, lat1: f64, lon1: f64, lat2: f64, lon2: f64) -> f64 {
        haversine_m(lat1 as f32, lon1 as f32, lat2 as f32, lon2 as f32) as f64
    }

    /// Nearest place in `places` to (lat, lon), by real haversine distance.
    /// Returns ok:false if `places` is empty — never fabricates a result.
    #[func]
    fn nearest_place(&self, lat: f64, lon: f64, places: Array<Variant>) -> VDict {
        let mut out = VDict::new();
        let mut best: Option<(GString, GString, f64, f64, f64)> = None;
        for v in places.iter_shared() {
            if let Some((name, name_arabic, plat, plon)) = place_from_variant(&v) {
                let d = haversine_m(lat as f32, lon as f32, plat as f32, plon as f32) as f64;
                if best.as_ref().map_or(true, |(_, _, _, _, bd)| d < *bd) {
                    best = Some((name, name_arabic, plat, plon, d));
                }
            }
        }
        match best {
            Some((name, name_arabic, plat, plon, d)) => {
                out.set("ok", true);
                out.set("name", &name);
                out.set("name_arabic", &name_arabic);
                out.set("lat", plat);
                out.set("lon", plon);
                out.set("distance_m", d);
            }
            None => {
                out.set("ok", false);
                out.set("error", "no places supplied");
            }
        }
        out
    }
}
