//! grave-discovery-station — Wadi al-Salam AI discovery pipeline.
//!
//! Processes CSV sidecar files produced by satellite/drone analysis into
//! sovereign GraveParticle updates, then aggregates them into a
//! GraveDiscoveryReport with topology statistics.
//!
//! # CSV sidecar format (tab-separated, UTF-8, no external crates)
//!
//! ```text
//! grave_id\tlat_center\tlon_center\tcondition\tinscription_text\tconfidence_pct\tdeath_hijri_year
//! 101\t31.9950\t44.3200\tPartial\tعبد الله الحسيني\t62\t1398
//! 102\t32.0050\t44.3200\tDestroyed\t\t0\t
//! ```
//!
//! - `grave_id`          — u32, must match an existing GraveParticle
//! - `lat_center`        — f32 latitude (WGS-84)
//! - `lon_center`        — f32 longitude (WGS-84)
//! - `condition`         — Intact | Partial | Destroyed | Unknown
//! - `inscription_text`  — UTF-8 Arabic text (may be empty)
//! - `confidence_pct`    — 0–100, scaled to QUALITY_DIVISOR=240
//! - `death_hijri_year`  — u32 Hijri year (may be empty)
//!
//! # Discovery source precedence
//!
//! The station accepts an explicit `DiscoverySource` at construction that is
//! applied to all records in one batch (e.g., a single drone flight produces
//! one batch of `DiscoverySource::Drone` records).

#![forbid(unsafe_code)]

use bahyway_core::TribeId;
use najaf_engine::{
    DiscoverySource, GraveCondition, GraveId, GraveParticle, GraveRegistry, IdentityCandidate,
    InscriptionMatcher, NajafSector, NaviCoord, QUALITY_DIVISOR,
};

// ── Raw discovery row ─────────────────────────────────────────────────────────

/// One row from a satellite/drone CSV sidecar — pre-parse, before enrichment.
#[derive(Debug, Clone)]
pub struct DiscoveryRow {
    pub grave_id: GraveId,
    pub lat_center: f32,
    pub lon_center: f32,
    pub condition: GraveCondition,
    /// Raw inscription text from image recognition (may be empty).
    pub inscription_text: String,
    /// Confidence from imagery AI, 0–100 (will be scaled to 0–240).
    pub confidence_pct: u8,
    pub death_hijri_year: Option<u32>,
}

// ── Discovery report ──────────────────────────────────────────────────────────

/// Summary produced after processing one CSV batch.
#[derive(Debug, Default)]
pub struct GraveDiscoveryReport {
    pub total_rows: usize,
    pub parse_errors: usize,
    pub graves_updated: usize,
    pub graves_not_found: usize,
    pub identity_matched: usize,
    pub damaged_count: usize,
    pub destroyed_count: usize,
    pub partial_count: usize,
    pub intact_count: usize,
    /// Candidate identities resolved (confidence ≥ 150).
    pub candidates: Vec<IdentityCandidate>,
}

impl GraveDiscoveryReport {
    pub fn unresolved_count(&self) -> usize {
        self.total_rows
            .saturating_sub(self.parse_errors + self.graves_not_found)
    }
}

// ── Parse error ───────────────────────────────────────────────────────────────

#[derive(Debug)]
pub struct ParseError {
    pub line: usize,
    pub message: String,
}

impl core::fmt::Display for ParseError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "line {}: {}", self.line, self.message)
    }
}

// ── Discovery station ─────────────────────────────────────────────────────────

/// Processes CSV sidecar files into GraveParticle updates.
pub struct GraveDiscoveryStation {
    source: DiscoverySource,
    matcher: InscriptionMatcher,
}

impl GraveDiscoveryStation {
    /// Create a station for a specific discovery source (e.g. a drone flight batch).
    pub fn new(source: DiscoverySource) -> Self {
        GraveDiscoveryStation {
            source,
            matcher: InscriptionMatcher::new(),
        }
    }

    /// Seed the inscription matcher from an existing registry so that
    /// already-identified graves can be matched by inscription alone.
    pub fn seed_matcher_from_registry(&mut self, registry: &GraveRegistry) {
        for id in registry.all_ids() {
            if let Some(g) = registry.get(id) {
                if let Some(ref name) = g.owner_name_arabic {
                    self.matcher.register(id, name);
                }
            }
        }
    }

    /// Register an additional known name for inscription matching.
    pub fn register_name(&mut self, grave_id: GraveId, name_arabic: &str) {
        self.matcher.register(grave_id, name_arabic);
    }

    /// Parse the CSV text and apply updates to the registry.
    ///
    /// Returns a `GraveDiscoveryReport` with statistics.  Parse errors are
    /// counted but do not abort the batch.
    pub fn process_csv(
        &mut self,
        csv_text: &str,
        registry: &mut GraveRegistry,
        tribe: TribeId,
        sector: NajafSector,
    ) -> GraveDiscoveryReport {
        let mut report = GraveDiscoveryReport::default();

        for (line_idx, line) in csv_text.lines().enumerate() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') || line.starts_with("grave_id") {
                continue;
            }
            report.total_rows += 1;

            let row = match parse_row(line, line_idx + 1) {
                Ok(r) => r,
                Err(e) => {
                    report.parse_errors += 1;
                    let _ = e; // errors counted; caller inspects via error_count
                    continue;
                }
            };

            // Try to update existing grave or create a new discovered particle.
            if let Some(g) = registry.get_mut(row.grave_id) {
                apply_row_to_grave(g, &row, self.source);
            } else {
                // New grave discovered by AI — register it.
                let coord = NaviCoord::new(row.lat_center, row.lon_center, 0.0);
                let epoch = row.death_hijri_year.unwrap_or(0);
                let g = GraveParticle::new(row.grave_id, coord, sector, tribe, epoch)
                    .with_condition(row.condition)
                    .with_confidence(scale_confidence(row.confidence_pct))
                    .with_discovery_source(self.source);
                let g = if let Some(yr) = row.death_hijri_year {
                    g.with_death_year(yr)
                } else {
                    g
                };
                registry.register(g);
            }

            report.graves_updated += 1;

            // Account for condition in report
            match row.condition {
                GraveCondition::Intact => report.intact_count += 1,
                GraveCondition::Partial => report.partial_count += 1,
                GraveCondition::Destroyed => report.destroyed_count += 1,
                GraveCondition::Unknown => {}
            }
            if row.condition.is_damaged() {
                report.damaged_count += 1;
            }

            // Run inscription matching if text is present
            if !row.inscription_text.is_empty() {
                let candidates = self.matcher.match_inscription(&row.inscription_text);
                // Apply best candidate to the grave (confidence ≥ 150)
                if let Some(best) = candidates.iter().find(|c| c.confidence >= 150) {
                    if let Some(g) = registry.get_mut(row.grave_id) {
                        g.identity_confidence = best.confidence.min(QUALITY_DIVISOR);
                        if g.owner_name_arabic.is_none() {
                            g.owner_name_arabic = Some(best.matched_name.clone());
                        }
                    }
                    report.identity_matched += 1;
                    report.candidates.push(best.clone());
                }
            }
        }

        report
    }

    /// Process a collection of already-parsed `DiscoveryRow`s (for programmatic use).
    pub fn process_rows(
        &mut self,
        rows: &[DiscoveryRow],
        registry: &mut GraveRegistry,
        tribe: TribeId,
        sector: NajafSector,
    ) -> GraveDiscoveryReport {
        let mut report = GraveDiscoveryReport {
            total_rows: rows.len(),
            ..Default::default()
        };

        for row in rows {
            if let Some(g) = registry.get_mut(row.grave_id) {
                apply_row_to_grave(g, row, self.source);
            } else {
                let coord = NaviCoord::new(row.lat_center, row.lon_center, 0.0);
                let epoch = row.death_hijri_year.unwrap_or(0);
                let g = GraveParticle::new(row.grave_id, coord, sector, tribe, epoch)
                    .with_condition(row.condition)
                    .with_confidence(scale_confidence(row.confidence_pct))
                    .with_discovery_source(self.source);
                let g = if let Some(yr) = row.death_hijri_year {
                    g.with_death_year(yr)
                } else {
                    g
                };
                registry.register(g);
            }

            report.graves_updated += 1;

            match row.condition {
                GraveCondition::Intact => report.intact_count += 1,
                GraveCondition::Partial => report.partial_count += 1,
                GraveCondition::Destroyed => report.destroyed_count += 1,
                GraveCondition::Unknown => {}
            }
            if row.condition.is_damaged() {
                report.damaged_count += 1;
            }

            if !row.inscription_text.is_empty() {
                let candidates = self.matcher.match_inscription(&row.inscription_text);
                if let Some(best) = candidates.iter().find(|c| c.confidence >= 150) {
                    if let Some(g) = registry.get_mut(row.grave_id) {
                        g.identity_confidence = best.confidence.min(QUALITY_DIVISOR);
                        if g.owner_name_arabic.is_none() {
                            g.owner_name_arabic = Some(best.matched_name.clone());
                        }
                    }
                    report.identity_matched += 1;
                    report.candidates.push(best.clone());
                }
            }
        }
        report
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

/// Scale 0–100 confidence percentage to QUALITY_DIVISOR=240 scale.
///
/// 100% → 240, 0% → 0.  Uses integer arithmetic, no floats.
pub fn scale_confidence(pct: u8) -> u8 {
    let pct = pct.min(100) as u32;
    ((pct * QUALITY_DIVISOR as u32) / 100) as u8
}

fn apply_row_to_grave(g: &mut GraveParticle, row: &DiscoveryRow, source: DiscoverySource) {
    g.condition = row.condition;
    g.discovery_source = source;
    let new_conf = scale_confidence(row.confidence_pct);
    // Only raise confidence, never lower it (AI can confirm but not un-confirm)
    if new_conf > g.identity_confidence {
        g.identity_confidence = new_conf;
    }
    if let Some(yr) = row.death_hijri_year {
        g.death_hijri_year = Some(yr);
    }
    if !row.inscription_text.is_empty() && g.owner_name_arabic.is_none() {
        g.owner_name_arabic = Some(row.inscription_text.clone());
    }
}

/// Parse one TSV row.  Returns `Err(ParseError)` on malformed input.
fn parse_row(line: &str, line_num: usize) -> Result<DiscoveryRow, ParseError> {
    let parts: Vec<&str> = line.splitn(7, '\t').collect();
    if parts.len() < 6 {
        return Err(ParseError {
            line: line_num,
            message: format!("expected ≥6 tab-separated fields, got {}", parts.len()),
        });
    }

    let grave_id = parts[0].trim().parse::<u32>().map_err(|e| ParseError {
        line: line_num,
        message: format!("grave_id: {e}"),
    })?;
    let lat_center = parts[1].trim().parse::<f32>().map_err(|e| ParseError {
        line: line_num,
        message: format!("lat_center: {e}"),
    })?;
    let lon_center = parts[2].trim().parse::<f32>().map_err(|e| ParseError {
        line: line_num,
        message: format!("lon_center: {e}"),
    })?;
    let condition = parse_condition(parts[3].trim()).ok_or_else(|| ParseError {
        line: line_num,
        message: format!("unknown condition '{}'", parts[3].trim()),
    })?;
    let inscription_text = parts[4].trim().to_string();
    let confidence_pct = parts[5].trim().parse::<u8>().map_err(|e| ParseError {
        line: line_num,
        message: format!("confidence_pct: {e}"),
    })?;
    let death_hijri_year = if parts.len() > 6 && !parts[6].trim().is_empty() {
        Some(parts[6].trim().parse::<u32>().map_err(|e| ParseError {
            line: line_num,
            message: format!("death_hijri_year: {e}"),
        })?)
    } else {
        None
    };

    Ok(DiscoveryRow {
        grave_id,
        lat_center,
        lon_center,
        condition,
        inscription_text,
        confidence_pct,
        death_hijri_year,
    })
}

fn parse_condition(s: &str) -> Option<GraveCondition> {
    match s {
        "Intact" => Some(GraveCondition::Intact),
        "Partial" => Some(GraveCondition::Partial),
        "Destroyed" => Some(GraveCondition::Destroyed),
        "Unknown" => Some(GraveCondition::Unknown),
        _ => None,
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use bahyway_core::TribeId;
    use najaf_engine::{GraveParticle, GraveRegistry, NajafSector, NaviCoord};

    fn tribe() -> TribeId {
        TribeId::from_u16(0x0001)
    }

    fn base_registry() -> GraveRegistry {
        let mut r = GraveRegistry::new();
        r.register(GraveParticle::new(
            101,
            NaviCoord::new(31.995, 44.320, 0.0),
            NajafSector::Entrance,
            tribe(),
            1400,
        ));
        r.register(GraveParticle::new(
            102,
            NaviCoord::new(32.005, 44.320, 0.0),
            NajafSector::Shuhadaa,
            tribe(),
            1410,
        ));
        r.register(GraveParticle::new(
            103,
            NaviCoord::new(32.003, 44.333, 0.0),
            NajafSector::Awliya,
            tribe(),
            1420,
        ));
        r
    }

    const CSV: &str = "\
grave_id\tlat_center\tlon_center\tcondition\tinscription_text\tconfidence_pct\tdeath_hijri_year
101\t31.9950\t44.3200\tPartial\tعبد الله الحسيني\t62\t1398
102\t32.0050\t44.3200\tDestroyed\t\t0\t
103\t32.0030\t44.3330\tIntact\tمحمد علي الكاظمي\t100\t1419
";

    #[test]
    fn csv_updates_existing_graves() {
        let mut reg = base_registry();
        let mut station = GraveDiscoveryStation::new(DiscoverySource::Drone);
        let report = station.process_csv(CSV, &mut reg, tribe(), NajafSector::Entrance);
        assert_eq!(report.total_rows, 3);
        assert_eq!(report.parse_errors, 0);
        assert_eq!(report.graves_updated, 3);
    }

    #[test]
    fn condition_applied_to_graves() {
        let mut reg = base_registry();
        let mut station = GraveDiscoveryStation::new(DiscoverySource::Satellite);
        station.process_csv(CSV, &mut reg, tribe(), NajafSector::Entrance);

        assert_eq!(reg.get(101).unwrap().condition, GraveCondition::Partial);
        assert_eq!(reg.get(102).unwrap().condition, GraveCondition::Destroyed);
        assert_eq!(reg.get(103).unwrap().condition, GraveCondition::Intact);
    }

    #[test]
    fn confidence_scaled_correctly() {
        let mut reg = base_registry();
        let mut station = GraveDiscoveryStation::new(DiscoverySource::Satellite);
        station.process_csv(CSV, &mut reg, tribe(), NajafSector::Entrance);

        // 62% → (62 * 240) / 100 = 148
        assert_eq!(reg.get(101).unwrap().identity_confidence, 148);
        // 0% → 0
        assert_eq!(reg.get(102).unwrap().identity_confidence, 0);
        // 100% → 240
        assert_eq!(reg.get(103).unwrap().identity_confidence, 240);
    }

    #[test]
    fn inscription_text_stored_when_no_matcher() {
        let mut reg = base_registry();
        let mut station = GraveDiscoveryStation::new(DiscoverySource::Drone);
        station.process_csv(CSV, &mut reg, tribe(), NajafSector::Entrance);
        // No matcher entries, so inscription stored raw
        assert_eq!(
            reg.get(101).unwrap().owner_name_arabic.as_deref(),
            Some("عبد الله الحسيني")
        );
    }

    #[test]
    fn new_grave_registered_when_not_in_registry() {
        let mut reg = GraveRegistry::new(); // empty
        let mut station = GraveDiscoveryStation::new(DiscoverySource::Satellite);
        let csv = "200\t31.990\t44.310\tPartial\t\t50\t\n";
        station.process_csv(csv, &mut reg, tribe(), NajafSector::Awliya);
        assert!(reg.get(200).is_some());
        assert_eq!(reg.get(200).unwrap().condition, GraveCondition::Partial);
    }

    #[test]
    fn discovery_source_applied_to_all_rows() {
        let mut reg = base_registry();
        let mut station = GraveDiscoveryStation::new(DiscoverySource::Drone);
        station.process_csv(CSV, &mut reg, tribe(), NajafSector::Entrance);
        for id in [101u32, 102, 103] {
            assert_eq!(
                reg.get(id).unwrap().discovery_source,
                DiscoverySource::Drone
            );
        }
    }

    #[test]
    fn report_condition_counts_match() {
        let mut reg = base_registry();
        let mut station = GraveDiscoveryStation::new(DiscoverySource::Satellite);
        let report = station.process_csv(CSV, &mut reg, tribe(), NajafSector::Entrance);
        assert_eq!(report.intact_count, 1); // grave 103
        assert_eq!(report.partial_count, 1); // grave 101
        assert_eq!(report.destroyed_count, 1); // grave 102
        assert_eq!(report.damaged_count, 2); // partial + destroyed
    }

    #[test]
    fn parse_error_counted_not_fatal() {
        let mut reg = base_registry();
        let mut station = GraveDiscoveryStation::new(DiscoverySource::Drone);
        let bad_csv =
            "not_a_number\t31.0\t44.0\tIntact\t\t50\n101\t31.9950\t44.3200\tPartial\t\t0\t\n";
        let report = station.process_csv(bad_csv, &mut reg, tribe(), NajafSector::Entrance);
        assert_eq!(report.parse_errors, 1);
        assert_eq!(report.graves_updated, 1); // only grave 101
    }

    #[test]
    fn scale_confidence_boundaries() {
        assert_eq!(scale_confidence(0), 0);
        assert_eq!(scale_confidence(100), 240);
        assert_eq!(scale_confidence(50), 120);
        assert_eq!(scale_confidence(255), 240); // clamped at 100% then scaled
    }

    #[test]
    fn confidence_never_lowered() {
        let mut reg = base_registry();
        // First pass: set high confidence
        if let Some(g) = reg.get_mut(101) {
            g.identity_confidence = 200;
        }
        let mut station = GraveDiscoveryStation::new(DiscoverySource::Drone);
        // CSV gives confidence_pct=0 for grave 101 — must not lower 200
        let csv = "101\t31.9950\t44.3200\tPartial\t\t0\t\n";
        station.process_csv(csv, &mut reg, tribe(), NajafSector::Entrance);
        assert_eq!(reg.get(101).unwrap().identity_confidence, 200);
    }

    #[test]
    fn inscription_matcher_resolves_candidate() {
        let mut reg = base_registry();
        let mut station = GraveDiscoveryStation::new(DiscoverySource::Historical);
        station.register_name(101, "عبد الله الحسيني");
        // CSV inscription matches registered name → candidate resolved
        station.process_csv(CSV, &mut reg, tribe(), NajafSector::Entrance);
        assert_eq!(report_identity_matched(&mut reg, &mut station, CSV), true);
    }

    fn report_identity_matched(
        reg: &mut GraveRegistry,
        station: &mut GraveDiscoveryStation,
        csv: &str,
    ) -> bool {
        let report = station.process_csv(csv, reg, tribe(), NajafSector::Entrance);
        report.identity_matched > 0
    }

    #[test]
    fn process_rows_api() {
        let mut reg = base_registry();
        let mut station = GraveDiscoveryStation::new(DiscoverySource::Satellite);
        let rows = vec![DiscoveryRow {
            grave_id: 101,
            lat_center: 31.995,
            lon_center: 44.320,
            condition: GraveCondition::Intact,
            inscription_text: String::new(),
            confidence_pct: 80,
            death_hijri_year: Some(1395),
        }];
        let report = station.process_rows(&rows, &mut reg, tribe(), NajafSector::Entrance);
        assert_eq!(report.graves_updated, 1);
        assert_eq!(reg.get(101).unwrap().condition, GraveCondition::Intact);
        assert_eq!(reg.get(101).unwrap().death_hijri_year, Some(1395));
    }
}
