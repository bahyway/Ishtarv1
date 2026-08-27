//! HARVEST MODE — the accelerant (PB-603).
//! GL-NAM-002 made Ḫubullu glosses mandatory at sealing, so the sealed
//! corpus is a pre-existing witness archive. Harvest replays it through
//! the wound registry in seconds: archaeological first, sentinel after.
//!
//! Speed the witnesses, never the verdict: trigger thresholds (§1) are
//! untouched. Harvest accelerates ARRIVAL of testimony only.
//!
//! Machine-readable witness line convention (one per gloss, in tablets):
//!   ḪUBULLU:: engine=<name> particle=<p> orbit=<o> tribe=<t> gloss="..."
//! ASCII form `HUBULLU::` is accepted equally.

use crate::{PatternWitness, Signature};
use std::fs;
use std::io;
use std::path::Path;

/// Scan every `*.md` tablet in a directory for witness lines.
pub fn harvest_dir(dir: &Path) -> io::Result<Vec<PatternWitness>> {
    let mut out = Vec::new();
    if !dir.exists() {
        return Ok(out);
    }
    let mut paths: Vec<_> = fs::read_dir(dir)?
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.extension().and_then(|e| e.to_str()) == Some("md"))
        .collect();
    paths.sort(); // deterministic harvest order
    for path in paths {
        let text = fs::read_to_string(&path)?;
        out.extend(harvest_text(&text));
    }
    Ok(out)
}

/// Extract witness lines from one tablet's text.
pub fn harvest_text(text: &str) -> Vec<PatternWitness> {
    text.lines().filter_map(parse_line).collect()
}

fn parse_line(line: &str) -> Option<PatternWitness> {
    let l = line.trim();
    let rest = l
        .strip_prefix("ḪUBULLU::")
        .or_else(|| l.strip_prefix("HUBULLU::"))?;
    let gstart = rest.find("gloss=\"")?;
    let after = &rest[gstart + 7..];
    let gend = after.find('"')?;
    let gloss = after[..gend].to_string();
    let head = &rest[..gstart];

    enum Field {
        Engine,
        Particle,
        Orbit,
        Tribe,
    }

    let (mut engine, mut p, mut o, mut t) = (String::new(), String::new(), String::new(), String::new());
    let (mut have_engine, mut have_p, mut have_o, mut have_t) = (false, false, false, false);
    let mut current: Option<Field> = None;

    // Field values may themselves contain whitespace (e.g. `particle=Leak Onset`);
    // accumulate tokens into the current field until the next recognized key.
    for tok in head.split_whitespace() {
        if let Some(v) = tok.strip_prefix("engine=") {
            engine = v.to_string();
            have_engine = true;
            current = Some(Field::Engine);
        } else if let Some(v) = tok.strip_prefix("particle=") {
            p = v.to_string();
            have_p = true;
            current = Some(Field::Particle);
        } else if let Some(v) = tok.strip_prefix("orbit=") {
            o = v.to_string();
            have_o = true;
            current = Some(Field::Orbit);
        } else if let Some(v) = tok.strip_prefix("tribe=") {
            t = v.to_string();
            have_t = true;
            current = Some(Field::Tribe);
        } else if let Some(field) = &current {
            let target = match field {
                Field::Engine => &mut engine,
                Field::Particle => &mut p,
                Field::Orbit => &mut o,
                Field::Tribe => &mut t,
            };
            target.push(' ');
            target.push_str(tok);
        }
    }

    if !(have_engine && have_p && have_o && have_t) {
        return None;
    }

    Some(PatternWitness {
        engine,
        signature: Signature::new(&p, &o, &t),
        hubullu_gloss: gloss,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const TABLET: &str = r#"
# GL-TEST-001 — Some Sealed Tablet
Ordinary prose the harvester must ignore.
ḪUBULLU:: engine=wpdengine particle=leak-onset orbit=night-flow tribe=pipe gloss="the first breath of a hidden loss"
More prose.
HUBULLU:: engine=nanshe particle=Leak Onset orbit=Night Flow tribe=Pipe gloss="earliest visible onset"
HUBULLU:: engine=broken particle=x orbit=y gloss="missing tribe field, must be skipped"
"#;

    #[test]
    fn harvests_witness_lines_and_skips_noise() {
        let ws = harvest_text(TABLET);
        assert_eq!(ws.len(), 2, "two well-formed lines; malformed line skipped");
        assert_eq!(ws[0].engine, "wpdengine");
        // normalization makes the two testimonies collide on one signature:
        assert_eq!(ws[0].signature.key(), ws[1].signature.key());
    }

    #[test]
    fn harvest_dir_is_deterministic_and_tolerant_of_absence(){
        let dir = std::env::temp_dir().join("gula_harvest_test");
        let _ = std::fs::remove_dir_all(&dir);
        assert!(harvest_dir(&dir).unwrap().is_empty(), "absent dir harvests empty, no error");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("b_tablet.md"), TABLET).unwrap();
        std::fs::write(dir.join("a_tablet.md"), TABLET).unwrap();
        let ws = harvest_dir(&dir).unwrap();
        assert_eq!(ws.len(), 4);
    }
}
