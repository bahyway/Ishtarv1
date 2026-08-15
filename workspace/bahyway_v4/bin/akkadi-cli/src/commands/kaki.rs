use crate::{format_kaki, parse_kaki, KakiInspection, output::format::OutputFormat};

pub fn inspect_kaki(kaki_str: &str, format: OutputFormat) {
    let raw = match parse_kaki(kaki_str) {
        Ok(b)  => b,
        Err(e) => { eprintln!("error: {e}"); return; }
    };
    let k = KakiInspection::from_bytes(raw);

    match format {
        OutputFormat::Json => {
            let v = serde_json::json!({
                "display":       k.display,
                "b0_b6":         hex_slice(&k.b0_b6),
                "b7_seq":        k.b7,
                "b8_b9_seal":    hex_slice(&k.b8_b9),
                "b10_domain":    format!("0x{:02X}", k.b10),
                "b11_quality":   k.b11,
                "b12_hijri":     k.b12,
                "b13_b15_ghost": hex_slice(&k.b13_b15),
                "particle_kind": k.particle_kind.label(),
                "quality_tier":  k.quality_tier.label(),
                "brightness":    k.brightness,
            });
            println!("{}", serde_json::to_string_pretty(&v).unwrap());
        }
        OutputFormat::Cuneiform => {
            println!("𒀭𒆳𒁺 KAKI Inspection");
            println!("𒌋 Display   : {}", k.display);
            println!("𒌋 B0–B6     : {}", hex_slice(&k.b0_b6));
            println!("𒌋 B7 Seq    : {}", k.b7);
            println!("𒌋 B8–B9     : {} (AkkadiSeal)", hex_slice(&k.b8_b9));
            println!("𒌋 B10 Domain: 0x{:02X} — {}", k.b10, k.particle_kind.label());
            println!("𒌋 B11 Quality: {} → {} (brightness {:.3})",
                k.b11, k.quality_tier.label(), k.brightness);
            println!("𒌋 B12 Hijri : {}", k.b12);
            println!("𒌋 B13–B15   : {} (Ghost)", hex_slice(&k.b13_b15));
        }
        OutputFormat::Table => {
            println!("{:<16} {}", "FIELD", "VALUE");
            println!("{}", "─".repeat(52));
            println!("{:<16} {}", "Display",   k.display);
            println!("{:<16} {}", "B0–B6",     hex_slice(&k.b0_b6));
            println!("{:<16} {}", "B7 Seq",    k.b7);
            println!("{:<16} {} (AkkadiSeal)", "B8–B9",  hex_slice(&k.b8_b9));
            println!("{:<16} 0x{:02X} — {}", "B10 Domain", k.b10, k.particle_kind.label());
            println!("{:<16} {} → {} (brightness {:.3})",
                "B11 Quality", k.b11, k.quality_tier.label(), k.brightness);
            println!("{:<16} {}", "B12 Hijri",  k.b12);
            println!("{:<16} {} (Ghost)", "B13–B15", hex_slice(&k.b13_b15));
        }
    }
}

pub fn generate_kaki(domain_byte: u8, quality: u8, format: OutputFormat) {
    // Deterministic demo generation — sovereign B0–B6 via pseudo-BLAKE3 placeholder
    let mut raw = [0u8; 16];
    // B0–B6: fill with domain+quality derived pattern (real impl uses kaki-core)
    for i in 0..7usize {
        raw[i] = domain_byte.wrapping_add(i as u8).wrapping_mul(quality);
    }
    raw[7]  = 0x00; // seq
    raw[8]  = 0xA4; // AkkadiSeal stub
    raw[9]  = 0x2C;
    raw[10] = domain_byte;
    raw[11] = quality;
    raw[12] = 0x18; // HijriPeriod stub
    raw[13] = 0x42; // Ghost stub
    raw[14] = 0x00;
    raw[15] = 0x00;

    let display = format_kaki(&raw);
    match format {
        OutputFormat::Json => {
            println!("{}", serde_json::json!({"kaki": display, "domain": format!("0x{domain_byte:02X}"), "quality": quality}));
        }
        _ => println!("{display}"),
    }
}

fn hex_slice(b: &[u8]) -> String {
    b.iter().map(|x| format!("{x:02X}")).collect::<Vec<_>>().join(" ")
}
