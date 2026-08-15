#![forbid(unsafe_code)]
//! Parse and serialize `.dqprofile` files.
//!
//! Format (human-readable key = value, sections with [attr <name>]):
//!
//! ```text
//! # BahyWay DQ Profile
//! batch_name      = najaf_cemetery_batch_001
//! format_layer    = CivilRegistry
//! source_trust    = Official
//! gem_threshold   = 200
//! tribe_threshold = 140
//!
//! [attr name]
//! nullable        = false
//! fill_threshold  = 0.95
//! dim             = Name
//! weight          = 1.0
//!
//! [attr notes]
//! nullable        = true
//! fill_threshold  = 0.00
//! dim             = General
//! weight          = 0.1
//! ```

use crate::profile::{AttrProfile, ClientDqProfile, DimTag, FormatLayerCode, SourceTrustLevel};

/// Parse a .dqprofile text file into a `ClientDqProfile`.
/// Returns `Err(description)` on any parse failure.
pub fn parse_profile(src: &str) -> Result<ClientDqProfile, String> {
    let mut batch_name      = String::new();
    let mut format_layer    = FormatLayerCode::CivilRegistry;
    let mut source_trust    = SourceTrustLevel::Official;
    let mut gem_threshold   = 200u8;
    let mut tribe_threshold = 140u8;
    let mut attrs: Vec<AttrProfile> = Vec::new();

    // Current attr being built
    let mut cur_name:      Option<String> = None;
    let mut cur_nullable   = true;
    let mut cur_threshold  = 0.0f32;
    let mut cur_dim        = DimTag::General;
    let mut cur_weight     = 1.0f32;

    let flush = |cur_name: &mut Option<String>,
                 nullable: bool, threshold: f32, dim: DimTag, weight: f32,
                 attrs: &mut Vec<AttrProfile>| {
        if let Some(name) = cur_name.take() {
            let mut ap = AttrProfile::new(&name, nullable);
            ap.fill_threshold = threshold;
            ap.dim_tag        = dim;
            ap.weight         = weight;
            attrs.push(ap);
        }
    };

    for raw_line in src.lines() {
        let line = raw_line.trim();
        if line.is_empty() || line.starts_with('#') { continue; }

        if line.starts_with("[attr ") && line.ends_with(']') {
            flush(&mut cur_name, cur_nullable, cur_threshold, cur_dim, cur_weight, &mut attrs);
            let name = line[6..line.len() - 1].trim().to_string();
            if name.is_empty() { return Err("empty attr name in [attr ...]".into()); }
            cur_name      = Some(name);
            cur_nullable  = true;
            cur_threshold = 0.0;
            cur_dim       = DimTag::General;
            cur_weight    = 1.0;
            continue;
        }

        let (key, val) = split_kv(line)?;

        if cur_name.is_some() {
            match key {
                "nullable"       => cur_nullable  = parse_bool(val)?,
                "fill_threshold" => cur_threshold = parse_f32(val)?,
                "dim"            => cur_dim       = DimTag::from_str(val),
                "weight"         => cur_weight    = parse_f32(val)?,
                other => return Err(format!("unknown attr key: {other}")),
            }
        } else {
            match key {
                "batch_name"      => batch_name      = val.to_string(),
                "format_layer"    => format_layer    = FormatLayerCode::from_str(val),
                "source_trust"    => source_trust    = SourceTrustLevel::from_str(val),
                "gem_threshold"   => gem_threshold   = parse_u8(val)?,
                "tribe_threshold" => tribe_threshold = parse_u8(val)?,
                other => return Err(format!("unknown header key: {other}")),
            }
        }
    }
    flush(&mut cur_name, cur_nullable, cur_threshold, cur_dim, cur_weight, &mut attrs);

    if batch_name.is_empty() {
        return Err("batch_name is required".into());
    }

    Ok(ClientDqProfile { batch_name, attrs, gem_threshold, tribe_threshold, format_layer, source_trust })
}

/// Serialize a `ClientDqProfile` to .dqprofile text.
pub fn to_profile_text(p: &ClientDqProfile) -> String {
    let mut s = String::new();
    s.push_str("# BahyWay DQ Profile\n");
    s.push_str(&format!("batch_name      = {}\n", p.batch_name));
    s.push_str(&format!("format_layer    = {}\n", p.format_layer.label()));
    s.push_str(&format!("source_trust    = {}\n", p.source_trust.label()));
    s.push_str(&format!("gem_threshold   = {}\n", p.gem_threshold));
    s.push_str(&format!("tribe_threshold = {}\n", p.tribe_threshold));
    for attr in &p.attrs {
        s.push('\n');
        s.push_str(&format!("[attr {}]\n", attr.attr_name));
        s.push_str(&format!("nullable        = {}\n", attr.nullable));
        s.push_str(&format!("fill_threshold  = {:.2}\n", attr.fill_threshold));
        s.push_str(&format!("dim             = {}\n", attr.dim_tag.label()));
        s.push_str(&format!("weight          = {:.1}\n", attr.weight));
    }
    s
}

fn split_kv(line: &str) -> Result<(&str, &str), String> {
    let pos = line.find('=').ok_or_else(|| format!("expected key = value, got: {line}"))?;
    Ok((line[..pos].trim(), line[pos + 1..].trim()))
}

fn parse_bool(s: &str) -> Result<bool, String> {
    match s.trim().to_lowercase().as_str() {
        "true" | "yes" | "1"  => Ok(true),
        "false" | "no" | "0"  => Ok(false),
        other => Err(format!("expected bool, got: {other}")),
    }
}

fn parse_f32(s: &str) -> Result<f32, String> {
    s.trim().parse::<f32>().map_err(|e| format!("expected float: {e}"))
}

fn parse_u8(s: &str) -> Result<u8, String> {
    s.trim().parse::<u8>().map_err(|e| format!("expected u8: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = r#"
# BahyWay DQ Profile
batch_name      = najaf_batch_001
format_layer    = CivilRegistry
source_trust    = Official
gem_threshold   = 200
tribe_threshold = 140

[attr name]
nullable        = false
fill_threshold  = 0.95
dim             = Name
weight          = 1.0

[attr date_of_death]
nullable        = true
fill_threshold  = 0.60
dim             = Date
weight          = 0.8

[attr notes]
nullable        = true
fill_threshold  = 0.00
dim             = General
weight          = 0.1
"#;

    #[test]
    fn round_trip_preserves_fields() {
        let p    = parse_profile(SAMPLE).expect("parse must succeed");
        let text = to_profile_text(&p);
        let p2   = parse_profile(&text).expect("re-parse must succeed");

        assert_eq!(p2.batch_name,      "najaf_batch_001");
        assert_eq!(p2.gem_threshold,   200);
        assert_eq!(p2.tribe_threshold, 140);
        assert_eq!(p2.attrs.len(),     3);
    }

    #[test]
    fn nullable_parsed_correctly() {
        let p = parse_profile(SAMPLE).unwrap();
        let name = p.find_attr({
            let mut h: u32 = 0x811C_9DC5;
            for &b in b"name" { h ^= b as u32; h = h.wrapping_mul(0x0100_0193); }
            h
        }).unwrap();
        assert!(!name.nullable, "name must be non-nullable");

        let notes = p.find_attr({
            let mut h: u32 = 0x811C_9DC5;
            for &b in b"notes" { h ^= b as u32; h = h.wrapping_mul(0x0100_0193); }
            h
        }).unwrap();
        assert!(notes.nullable, "notes must be nullable");
    }

    #[test]
    fn missing_batch_name_errors() {
        let bad = "gem_threshold = 200\n";
        assert!(parse_profile(bad).is_err());
    }

    #[test]
    fn unknown_header_key_errors() {
        let bad = "batch_name = x\nunknown_key = val\n";
        assert!(parse_profile(bad).is_err());
    }

    #[test]
    fn dim_tags_parsed() {
        let p = parse_profile(SAMPLE).unwrap();
        let dod = p.attrs.iter().find(|a| a.attr_name == "date_of_death").unwrap();
        assert!(matches!(dod.dim_tag, DimTag::Date));
    }
}
