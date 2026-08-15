use crate::{HEPTA_DIMS, PLANET_NAMES, PLANET_GLYPHS, DISTRICTS, output::format::OutputFormat};

struct TribeInfo {
    dim:      &'static str,
    planet:   &'static str,
    glyph:    &'static str,
    district: &'static str,
    orbit_r:  f32,
    particles: u32,
}

fn all_tribes() -> [TribeInfo; 7] {
    let mut tribes = std::array::from_fn(|i| TribeInfo {
        dim:      HEPTA_DIMS[i],
        planet:   PLANET_NAMES[i],
        glyph:    PLANET_GLYPHS[i],
        district: DISTRICTS[i],
        orbit_r:  45.0 + i as f32 * 9.0,
        particles: 9362,
    });
    // small variation per orbit
    for (i, t) in tribes.iter_mut().enumerate() {
        t.particles = 9000 + (i as u32 * 362 + 180) % 1000;
    }
    tribes
}

pub fn list_tribes(format: OutputFormat) {
    let tribes = all_tribes();
    match format {
        OutputFormat::Json => {
            let arr: Vec<serde_json::Value> = tribes.iter().map(|t| {
                serde_json::json!({
                    "dim":      t.dim,
                    "planet":   t.planet,
                    "glyph":    t.glyph,
                    "district": t.district,
                    "orbit_r":  t.orbit_r,
                    "particles":t.particles,
                })
            }).collect();
            println!("{}", serde_json::to_string_pretty(&arr).unwrap());
        }
        OutputFormat::Cuneiform => {
            println!("𒀭𒆳𒁺 Ibn Wahshiyya — 7 Planetary Tribes");
            for t in &tribes {
                println!("  {} {:<4} {:<10} — {} — orbit {:.0}u — {} particles",
                    t.glyph, t.dim, t.planet, t.district, t.orbit_r, t.particles);
            }
        }
        OutputFormat::Table => {
            println!("{:<2} {:<4} {:<10} {:<14} {:>8} {:>10}",
                "G", "DIM", "PLANET", "DISTRICT", "ORBIT R", "PARTICLES");
            println!("{}", "─".repeat(54));
            for t in &tribes {
                println!("{:<2} {:<4} {:<10} {:<14} {:>7.0}u {:>10}",
                    t.glyph, t.dim, t.planet, t.district, t.orbit_r, t.particles);
            }
        }
    }
}

pub fn show_tribe(dim: &str, format: OutputFormat) {
    let tribes = all_tribes();
    let t = match tribes.iter().find(|t| t.dim.eq_ignore_ascii_case(dim)) {
        Some(t) => t,
        None    => { eprintln!("unknown tribe '{dim}'; valid: ME GU SAG IZI A UD URU"); return; }
    };

    match format {
        OutputFormat::Json => {
            println!("{}", serde_json::json!({
                "dim":      t.dim,
                "planet":   t.planet,
                "glyph":    t.glyph,
                "district": t.district,
                "orbit_r":  t.orbit_r,
                "particles":t.particles,
            }));
        }
        OutputFormat::Cuneiform => {
            println!("𒀭𒆳𒁺 Tribe {}", t.dim);
            println!("  {} {}", t.glyph, t.planet);
            println!("  District : {}", t.district);
            println!("  Orbit R  : {:.0} sovereign units", t.orbit_r);
            println!("  Particles: {}", t.particles);
        }
        OutputFormat::Table => {
            println!("{:<12} {}", "DIM",      t.dim);
            println!("{:<12} {} {}", "PLANET",  t.glyph, t.planet);
            println!("{:<12} {}", "DISTRICT",  t.district);
            println!("{:<12} {:.0}u", "ORBIT R", t.orbit_r);
            println!("{:<12} {}", "PARTICLES", t.particles);
        }
    }
}
