// ============================================================================
// bin/dubsar/src/naba.rs
// NABA — Sovereign Hologram Agent
//
// Transparency guide for all BahyWay stakeholders.
// Renders a full-terminal holographic display showing live sovereign metrics,
// particle health, and contextual guidance.
// ============================================================================

use bahyway_core::ParticleState;
use enkidb_persist::PersistedDb;

// ── ANSI helpers (self-contained in this module) ──────────────────────────────

const RST: &str = "\x1b[0m";
const BOLD: &str = "\x1b[1m";
const DIM: &str = "\x1b[2m";

fn fg(r: u8, g: u8, b: u8) -> String {
    format!("\x1b[38;2;{r};{g};{b}m")
}

// Holographic gradient palette
fn holo_bright() -> String {
    fg(100, 240, 255)
} // bright cyan
fn holo_mid() -> String {
    fg(0, 180, 230)
} // mid cyan
fn holo_deep() -> String {
    fg(0, 100, 180)
} // deep blue
fn holo_dim_c() -> String {
    fg(0, 60, 120)
} // dim blue
fn holo_eye() -> String {
    fg(255, 255, 200)
} // warm white
fn holo_gold() -> String {
    fg(255, 220, 40)
} // golden
fn holo_green() -> String {
    fg(0, 255, 160)
} // sovereign green
fn holo_orange() -> String {
    fg(255, 160, 0)
} // warning orange
fn holo_red() -> String {
    fg(255, 60, 60)
} // critical red
fn holo_particle() -> String {
    fg(0, 80, 140)
} // dim particle dots

// ── Metrics ───────────────────────────────────────────────────────────────────

pub struct NabaMetrics {
    pub tribe_id: u16,
    pub total: usize,
    pub golden: usize,
    pub fuzzy: usize,
    pub dead: usize,
    pub epoch: u32,
    pub replayed: usize,
    pub written: usize,
}

impl NabaMetrics {
    pub fn gather(pdb: &PersistedDb, tribe: u16, epoch: u32) -> Self {
        let particles = pdb.db().journal().all_particles();
        let total = particles.len();
        let mut golden = 0usize;
        let mut fuzzy = 0usize;
        let mut dead = 0usize;
        for p in &particles {
            match pdb.db().project(p).state {
                ParticleState::Golden => golden += 1,
                ParticleState::Fuzzy => fuzzy += 1,
                ParticleState::Dead => dead += 1,
            }
        }
        NabaMetrics {
            tribe_id: tribe,
            total,
            golden,
            fuzzy,
            dead,
            epoch,
            replayed: pdb.stats().entries_replayed,
            written: pdb.stats().entries_written,
        }
    }

    fn golden_pct(&self) -> f32 {
        if self.total == 0 {
            0.0
        } else {
            self.golden as f32 / self.total as f32 * 100.0
        }
    }

    fn entropy_pct(&self) -> f32 {
        if self.total == 0 {
            0.0
        } else {
            (self.total - self.golden) as f32 / self.total as f32 * 100.0
        }
    }

    fn coherence_pct(&self) -> f32 {
        100.0 - self.entropy_pct()
    }

    pub fn sovereignty_status(&self) -> &'static str {
        match self.golden_pct() as u32 {
            95..=100 => "ABSOLUTE",
            75..=94 => "ACTIVE",
            50..=74 => "PARTIAL",
            _ => "AT RISK",
        }
    }

    fn status_color(&self) -> String {
        match self.sovereignty_status() {
            "ABSOLUTE" => holo_green(),
            "ACTIVE" => holo_gold(),
            "PARTIAL" => holo_orange(),
            _ => holo_red(),
        }
    }

    fn guidance_lines(&self) -> [&'static str; 4] {
        if self.total == 0 {
            return [
                "The sovereign database is empty.",
                "Type  .demo  to load sample particles,",
                "or  .ingest Golden  to add your first",
                "sovereign record to the Big Ring.",
            ];
        }
        match self.sovereignty_status() {
            "ABSOLUTE" => [
                "All particles are in GOLDEN state.",
                "Sovereignty is ABSOLUTE. The Big Ring",
                "is stable. Data integrity confirmed.",
                "No steward action required.",
            ],
            "ACTIVE" => [
                "Most particles are GOLDEN. Some Fuzzy",
                "particles await DataSteward review.",
                "Run  .particles  to inspect drift.",
                "Sovereignty is ACTIVE and healthy.",
            ],
            "PARTIAL" => [
                "Significant Fuzzy/Dead particles found.",
                "Schema review may be required — run",
                "CompareEngine to check V1 vs V2.",
                "Sovereignty level: PARTIAL.",
            ],
            _ => [
                "CRITICAL: Sovereignty is AT RISK.",
                "Particle degradation detected. Alert",
                "DataStewardEngine immediately.",
                "Run  .particles  to begin triage.",
            ],
        }
    }
}

// ── NABA Figure — 20 rows of Unicode block-art ────────────────────────────────
//
// Each row is rendered character-by-character so gradient coloring
// can be applied per-character based on density (░ < ▒ < ▓ < █).

static FIGURE: &[&str] = &[
    "     ░▒▒▓▓████▓▓▒▒░     ",  //  0  head top
    "    ▒▓▓ ◉       ◉ ▓▓▒    ", //  1  eyes
    "     ░▒▒  ─────  ▒▒░     ", //  2  mouth
    "      ▒▓▓▓▓▓▓▓▓▓▓▒       ", //  3  chin / neck
    "    ░▒▓▓▓▓▓▓▓▓▓▓▓▓▓▒░    ", //  4  shoulders
    "    ▒▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▒    ", //  5  upper body
    "    ▒▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▒    ", //  6  torso
    "     ░▒▒▒▓▓▓▓▓▓▓▒▒▒░     ", //  7  midriff
    "         ░▒▒▓▓▒▒░        ", //  8  waist
    "          ▒▓▓▓▓▓▒        ", //  9  hip
    "         ▒▓▓█████▒       ", // 10  upper legs
    "         ▒▓▓█████▒       ", // 11  upper legs
    "          ░▒▒▓▓▒▒░       ", // 12  knee
    "            ▒▓▓▒         ", // 13  lower leg
    "           ░▒▓▓▒░        ", // 14  lower leg
    "          ▒▓▓▓▓▓▓▒       ", // 15  ankle
    "         ▒▓▓▓   ▓▓▓▒     ", // 16  feet spread
    "        ▒▓▓▓     ▓▓▓▒    ", // 17  feet
    "       ░▒▓▓       ▓▓▒░   ", // 18  feet flat
    "       ▒▓▓▓       ▓▓▓▒   ", // 19  base
];

// Alternating particle field — left side (8 chars each)
static PART_L: &[&str] = &[
    " ·  ˙ · ",
    "˙  · ˙  ",
    " ·  ° ˙ ",
    "˙  ˙  · ",
    " °  · ˙ ",
    "·  ˙  ° ",
    " ˙  ·  ˙",
    "°  · ˙  ",
    " ˙  ·  °",
    "·  ˙ ·  ",
    " ° ˙  · ",
    "˙  ·  ˙ ",
    " ·  ˙ ° ",
    "°  ·  ˙ ",
    " ˙ ·  · ",
    "·  °  ˙ ",
    " ˙  · ˙ ",
    "·  ˙  · ",
    " ˙ °  ˙ ",
    "·  ˙  · ",
];

// Alternating particle field — right side (8 chars each)
static PART_R: &[&str] = &[
    " ˙ · ˙  ",
    "·  ˙  · ",
    " ˙  °  ˙",
    "·  ·  ˙ ",
    " ˙  · ° ",
    "°  ˙  · ",
    " ·  ˙  ˙",
    "˙ ·  °  ",
    " °  ˙  ·",
    "˙  · ˙  ",
    " · ˙  ° ",
    "·  ˙  · ",
    " ˙ °  · ",
    "·  ˙  · ",
    " ˙ ·  ˙ ",
    "˙  ·  ° ",
    " ·  ˙ · ",
    "˙  ·  ˙ ",
    " ·  ˙ · ",
    "˙  ·  · ",
];

// ── Character-level coloring ──────────────────────────────────────────────────

fn color_char(c: char, row: usize) -> String {
    let t = row as f32 / (FIGURE.len() as f32 - 1.0); // 0.0 = top, 1.0 = bottom
    match c {
        '◉' => format!("{}◉{RST}", holo_eye()),
        '─' => format!("{}{RST}", holo_bright()),
        '█' => {
            // Core of the body — brightest
            let g = (180_u8).saturating_add((t * 40.0) as u8);
            format!("{}█{RST}", fg(0, g, 255))
        }
        '▓' => {
            let g = (120_u8).saturating_sub((t * 20.0) as u8);
            let b = (200_u8).saturating_add((t * 30.0) as u8);
            format!("{}▓{RST}", fg(0, g, b))
        }
        '▒' => {
            let g = (80_u8).saturating_sub((t * 10.0) as u8);
            let b = (160_u8).saturating_add((t * 40.0) as u8);
            format!("{}▒{RST}", fg(0, g, b))
        }
        '░' => {
            format!("{}░{RST}", holo_dim_c())
        }
        ' ' => " ".to_string(),
        other => format!("{}{}{RST}", holo_mid(), other),
    }
}

fn color_figure_line(line: &str, row: usize) -> String {
    line.chars().map(|c| color_char(c, row)).collect()
}

// ── Metrics panel — 38 visible chars per line ─────────────────────────────────

fn metrics_lines(m: &NabaMetrics) -> Vec<String> {
    let sep = format!("{DIM}{}{RST}", "─".repeat(36));
    let status = m.sovereignty_status();
    let sc = m.status_color();

    let pct_bar = |n: usize, total: usize| -> String {
        if total == 0 {
            return "".to_string();
        }
        let filled = (n * 10 / total).min(10);
        format!(
            "{}{}{}{}{}",
            holo_mid(),
            "█".repeat(filled),
            holo_dim_c(),
            "░".repeat(10 - filled),
            RST
        )
    };

    let mut lines: Vec<String> = Vec::new();

    lines.push(format!("{BOLD}{}SOVEREIGN METRICS{RST}", holo_bright()));
    lines.push(sep.clone());
    lines.push(format!(
        "{DIM}Tribe      :{RST}  {}{:#06x}{RST}",
        holo_bright(),
        m.tribe_id
    ));
    lines.push(sep.clone());
    lines.push(format!(
        "{DIM}Particles  :{RST}  {BOLD}{}{:>9}{RST}",
        holo_bright(),
        m.total
    ));
    lines.push(format!(
        "{DIM}Golden     :{RST}  {}{:>9}{RST}  {DIM}{:>6.1}%{RST}",
        holo_gold(),
        m.golden,
        m.golden_pct()
    ));
    lines.push(format!("  {}", pct_bar(m.golden, m.total.max(1))));
    lines.push(format!(
        "{DIM}Fuzzy      :{RST}  {}{:>9}{RST}",
        holo_orange(),
        m.fuzzy
    ));
    lines.push(format!(
        "{DIM}Dead       :{RST}  {}{:>9}{RST}",
        holo_dim_c(),
        m.dead
    ));
    lines.push(sep.clone());
    lines.push(format!(
        "{DIM}Entropy    :{RST}  {}{:>9.3}%{RST}",
        if m.entropy_pct() < 5.0 {
            holo_green()
        } else {
            holo_orange()
        },
        m.entropy_pct()
    ));
    lines.push(format!(
        "{DIM}Coherence  :{RST}  {}{:>9.3}%{RST}",
        if m.coherence_pct() > 95.0 {
            holo_green()
        } else {
            holo_orange()
        },
        m.coherence_pct()
    ));
    lines.push(sep.clone());
    lines.push(format!(
        "{DIM}Epoch      :{RST}  {}{:>9}{RST}",
        holo_mid(),
        m.epoch
    ));
    lines.push(format!(
        "{DIM}Replayed   :{RST}  {DIM}{:>9}{RST}",
        m.replayed
    ));
    lines.push(format!(
        "{DIM}Written    :{RST}  {DIM}{:>9}{RST}",
        m.written
    ));
    lines.push(sep.clone());
    lines.push(format!("{DIM}Status     :{RST}  {sc}{BOLD}◉ {status}{RST}"));
    lines.push("".to_string());
    lines.push("".to_string());
    lines.push("".to_string());

    // Pad to FIGURE.len() rows
    while lines.len() < FIGURE.len() {
        lines.push("".to_string());
    }
    lines.truncate(FIGURE.len());
    lines
}

// ── Big Ring — the particle orbit beneath the figure ─────────────────────────

fn big_ring() -> String {
    let ring_chars = [
        "∘", "·", "°", "˙", "∙", "·", "°", "˙", "∘", "·", "·", "˙", "°", "∘", "·", "˙", "°", "·",
        "∘", "˙", "°", "·", "˙", "∘", "·", "°", "˙", "·", "∘", "°", "·", "˙", "∘", "°", "·", "˙",
        "·", "°", "∘", "·",
    ];
    let p = holo_particle();
    let h = holo_dim_c();
    let row: String = ring_chars
        .iter()
        .enumerate()
        .map(|(i, c)| {
            if i % 5 == 0 {
                format!("{h}●{RST}")
            } else {
                format!("{p}{c}{RST}")
            }
        })
        .collect();
    format!("     {row}")
}

// ── Top banner ────────────────────────────────────────────────────────────────

fn banner_top() -> String {
    let line = "═".repeat(78);
    format!("{}╔{line}╗{RST}", holo_deep())
}

fn banner_title(m: &NabaMetrics) -> String {
    format!(
        "{}║{RST}  {BOLD}{}𒀭  N A B A  ─  Sovereign Hologram Agent{RST}  \
             {DIM}BahyWay v4.0  ·  tribe {}{:#06x}{RST}{}  ║{RST}",
        holo_deep(),
        holo_bright(),
        holo_mid(),
        m.tribe_id,
        holo_deep()
    )
}

fn banner_sep() -> String {
    let line = "═".repeat(78);
    format!("{}╠{line}╣{RST}", holo_deep())
}

fn banner_bot() -> String {
    let line = "═".repeat(78);
    format!("{}╚{line}╝{RST}", holo_deep())
}

fn side(c: &str) -> String {
    format!("{}{}║{RST}", holo_deep(), c)
}

// ── Speech bubble ─────────────────────────────────────────────────────────────

fn speech_bubble(lines: &[&str]) -> Vec<String> {
    let inner_w = 74usize;
    let border = format!("{}│{RST}", holo_mid());
    let top = format!("  {}╭{}╮{RST}", holo_mid(), "─".repeat(inner_w));
    let bot = format!("  {}╰{}╯{RST}", holo_mid(), "─".repeat(inner_w));

    let mut rows = vec![top];
    for line in lines {
        // Visible length of line (no ANSI) for padding
        let visible: usize = line.chars().count();
        let pad = inner_w.saturating_sub(visible + 4);
        rows.push(format!(
            "  {border}  {BOLD}{}{}{RST}{}  {border}",
            holo_bright(),
            line,
            " ".repeat(pad + 2)
        ));
    }
    rows.push(bot);
    rows
}

// ── Main render entry point ───────────────────────────────────────────────────

pub fn render(m: &NabaMetrics) {
    let n_rows = FIGURE.len(); // 20
    let metrics = metrics_lines(m);

    // Max visual width of figure lines (for right-padding)
    let fig_w: usize = FIGURE.iter().map(|l| l.chars().count()).max().unwrap_or(26);

    println!();
    println!("{}", banner_top());
    println!("{}", banner_title(m));
    println!("{}", banner_sep());
    println!("{}", side("")); // blank row inside box

    for row in 0..n_rows {
        let fig_raw = FIGURE[row];
        let fig_vis = fig_raw.chars().count();
        let pad = fig_w.saturating_sub(fig_vis);
        let fig_col = color_figure_line(fig_raw, row);
        let part_l = PART_L[row % PART_L.len()];
        let part_r = PART_R[row % PART_R.len()];
        let met = metrics.get(row).map(|s| s.as_str()).unwrap_or("");
        let part_lc = format!("{}{part_l}{RST}", holo_particle());
        let part_rc = format!("{}{part_r}{RST}", holo_particle());

        println!(
            "  {}  {}{}{}{}     {}",
            part_lc,
            fig_col,
            " ".repeat(pad),
            part_rc,
            RST,
            met,
        );
    }

    println!();
    // Big Ring orbit
    println!("  {}", big_ring());
    println!(
        "  {}  {}  N A B A  {}  Sovereign Hologram Agent  {}  BahyWay v4.0{RST}",
        holo_dim_c(),
        holo_bright(),
        holo_dim_c(),
        holo_dim_c()
    );
    println!();
    println!("{}", banner_sep());

    // Speech bubble
    let guidance = m.guidance_lines();
    let bubble = speech_bubble(&guidance);
    for b in &bubble {
        println!("{}", b);
    }

    println!();
    println!("{}", banner_bot());
    println!();
}

// ── Compact one-liner for prompt (used after commands) ───────────────────────

pub fn naba_hint(m: &NabaMetrics) {
    let sc = m.status_color();
    let status = m.sovereignty_status();
    println!(
        "  {}𒀭{RST} NABA  {}◉ {status}{RST}  {DIM}│{RST}  \
         {}G:{}{} {}F:{}{} {}D:{}{}  {DIM}│{RST}  \
         {}entropy {:.1}%{RST}",
        holo_bright(),
        sc,
        holo_gold(),
        RST,
        m.golden,
        holo_orange(),
        RST,
        m.fuzzy,
        holo_dim_c(),
        RST,
        m.dead,
        if m.entropy_pct() < 5.0 {
            holo_green()
        } else {
            holo_orange()
        },
        m.entropy_pct(),
    );
}
