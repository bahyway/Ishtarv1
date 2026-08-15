//! WayFile — BahyWay Sovereign Security Declaration (.way format).
//!
//! A `.way` file is a line-based key-value declaration that describes
//! the sovereignty, template, and routing metadata for a data stream
//! arriving in the landing zone.  It is compiled to AAOL (.akk) by
//! `way_compiler`.
//!
//! Syntax (each directive is `<keyword> <value>`, one per line):
//!   # comment
//!   tribe       0x0001          — sovereign tribe ID (hex)
//!   class       civil.registry  — BahyWay class name
//!   sovereignty PA-15           — legal sovereignty tag
//!   clearance   3               — clearance level (1-9)
//!   template    civil.registry  — template name for VGCA validation
//!   emit        0x0001          — destination tribe ID (hex); omit for same tribe
//!   role        Zikru           — KAKI role for generated particles
//!   sign        CRC-16          — signing method (informational)

/// A parsed BahyWay Sovereign Security Declaration.
#[derive(Debug, Clone)]
pub struct WayFile {
    pub tribe_id:     u16,
    pub class:        String,
    pub sovereignty:  String,
    pub clearance:    u8,
    pub template:     String,
    pub emit_tribe:   Option<u16>,
    pub role:         String,
    pub sign:         String,
}

impl Default for WayFile {
    fn default() -> Self {
        WayFile {
            tribe_id:    0x0001,
            class:       "civil.registry".to_string(),
            sovereignty: "PA-15".to_string(),
            clearance:   3,
            template:    "civil.registry".to_string(),
            emit_tribe:  None,
            role:        "Zikru".to_string(),
            sign:        "CRC-16".to_string(),
        }
    }
}

impl WayFile {
    /// Parse a `.way` source string into a `WayFile`.
    /// Lines starting with `#` or empty are ignored.
    pub fn parse(src: &str) -> Result<Self, String> {
        let mut w = WayFile::default();
        for (lineno, raw) in src.lines().enumerate() {
            let line = raw.trim();
            if line.is_empty() || line.starts_with('#') { continue; }

            let (key, val) = split_kv(line)
                .ok_or_else(|| format!("line {}: expected 'key value', got: {line:?}", lineno + 1))?;

            match key {
                "tribe" => {
                    w.tribe_id = parse_hex_u16(val)
                        .ok_or_else(|| format!("line {}: bad tribe hex '{val}'", lineno + 1))?;
                }
                "class"       => { w.class       = val.to_string(); }
                "sovereignty" => { w.sovereignty = val.to_string(); }
                "clearance"   => {
                    w.clearance = val.parse::<u8>()
                        .map_err(|_| format!("line {}: bad clearance '{val}'", lineno + 1))?;
                }
                "template"    => { w.template    = val.to_string(); }
                "emit"        => {
                    w.emit_tribe = Some(
                        parse_hex_u16(val)
                            .ok_or_else(|| format!("line {}: bad emit hex '{val}'", lineno + 1))?
                    );
                }
                "role"        => { w.role = val.to_string(); }
                "sign"        => { w.sign = val.to_string(); }
                other         => {
                    return Err(format!("line {}: unknown directive '{other}'", lineno + 1));
                }
            }
        }
        Ok(w)
    }

    /// The effective destination tribe (emit_tribe or own tribe).
    pub fn destination_tribe(&self) -> u16 {
        self.emit_tribe.unwrap_or(self.tribe_id)
    }

    /// Derive a CamelCase tribe name from the class string.
    /// "civil.registry" → "CivilRegistry"
    pub fn tribe_name(&self) -> String {
        self.class
            .split('.')
            .map(|part| {
                let mut c = part.chars();
                match c.next() {
                    None    => String::new(),
                    Some(f) => f.to_ascii_uppercase().to_string() + c.as_str(),
                }
            })
            .collect::<Vec<_>>()
            .join("")
    }
}

fn split_kv(line: &str) -> Option<(&str, &str)> {
    let mut it = line.splitn(2, char::is_whitespace);
    let key = it.next()?.trim();
    let val = it.next()?.trim();
    Some((key, val))
}

fn parse_hex_u16(s: &str) -> Option<u16> {
    let hex = s.trim_start_matches("0x").trim_start_matches("0X");
    u16::from_str_radix(hex, 16).ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = r#"
# Najaf Wadi Al-Salaam
tribe       0x0001
class       civil.registry
sovereignty PA-15
clearance   3
template    civil.registry
emit        0x0001
role        Zikru
sign        CRC-16
"#;

    #[test]
    fn parse_full_way_file() {
        let w = WayFile::parse(SAMPLE).unwrap();
        assert_eq!(w.tribe_id, 0x0001);
        assert_eq!(w.class, "civil.registry");
        assert_eq!(w.sovereignty, "PA-15");
        assert_eq!(w.clearance, 3);
        assert_eq!(w.role, "Zikru");
        assert_eq!(w.emit_tribe, Some(0x0001));
    }

    #[test]
    fn tribe_name_camel_case() {
        let w = WayFile::parse(SAMPLE).unwrap();
        assert_eq!(w.tribe_name(), "CivilRegistry");
    }

    #[test]
    fn defaults_for_missing_fields() {
        let w = WayFile::parse("tribe 0x0002").unwrap();
        assert_eq!(w.tribe_id, 0x0002);
        assert_eq!(w.clearance, 3);   // default
        assert_eq!(w.role, "Zikru"); // default
    }

    #[test]
    fn unknown_directive_errors() {
        let res = WayFile::parse("unknown value");
        assert!(res.is_err());
    }
}
