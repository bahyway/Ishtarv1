//! UṢURTU — the Suspicion Map (PB-608; candidate name, pending CSR-08).
//! Uṣurtu: drawing, plan, ordinance. The court's printable exhibit.
//!
//! Agû streams area tiles in; the analyzer drains every stream; the map
//! prints where suspicion lives. Two laws:
//!
//!   SUSPICION, NEVER CONFIRMATION — a suspect cell is a summons for a
//!   field team, ranked worst-first (Civil Protection: never averaged),
//!   not a verdict. Confirmation happens with a spade and a Kanīku
//!   receipt.
//!
//!   NO FINAL MAP WHILE STREAMS REMAIN — printing before the last tile
//!   is analyzed burns a PROVISIONAL watermark across the sheet. A map
//!   that silently omits unanalyzed ground is the certified-island lie
//!   on paper. No silent close, in ink.

use crate::noise::{judge_noise, NoiseVerdict, K_MAX_DEFAULT, R_MAX_DEFAULT};
use std::collections::{BTreeMap, VecDeque};

#[derive(Debug, Clone, PartialEq)]
pub enum CellState {
    Unassessed,
    InStream,
    /// Insufficient witness (e.g. surface-only) — needs another pass.
    Summoned { cause: String },
    /// Structured residue or bound not met — field team, worst first.
    Suspect { cause: String, priority: f64 },
    CertifiedBounded { rho: f64 },
}

#[derive(Debug, Clone)]
pub struct Tile {
    pub cell_id: usize,
    pub residual_series: Vec<f64>,
    pub has_subsurface: bool,
    pub rho: f64,
    /// Urban proximity weight ≥ 1.0 — homes nearby raise priority.
    pub urban_weight: f64,
}

#[derive(Debug, Clone)]
pub struct MapCell {
    pub id: usize,
    /// Axial-ish layout coordinates for the printed sheet.
    pub x: f64,
    pub y: f64,
    pub state: CellState,
}

#[derive(Debug, Default)]
pub struct StreamAnalyzer {
    pending: VecDeque<Tile>,
    pub cells: BTreeMap<usize, MapCell>,
    pub tiles_seen: usize,
    pub tiles_done: usize,
    eps_theta: f64,
}

impl StreamAnalyzer {
    pub fn new(layout: &[(usize, f64, f64)], eps_theta: f64) -> Self {
        let mut s = Self { eps_theta, ..Default::default() };
        for &(id, x, y) in layout {
            s.cells.insert(id, MapCell { id, x, y, state: CellState::Unassessed });
        }
        s
    }

    /// Agû pushes more area: enqueue a tile stream.
    pub fn push_tile(&mut self, t: Tile) {
        if let Some(c) = self.cells.get_mut(&t.cell_id) {
            c.state = CellState::InStream;
        }
        self.tiles_seen += 1;
        self.pending.push_back(t);
    }

    /// Analyze one tile. Returns (cell_id, verdict text) or None if idle.
    pub fn step(&mut self) -> Option<(usize, String)> {
        let t = self.pending.pop_front()?;
        self.tiles_done += 1;
        let state = if !t.has_subsurface {
            CellState::Summoned {
                cause: "no subsurface witness — LiDAR summons, never certifies".into(),
            }
        } else {
            match judge_noise(&t.residual_series, R_MAX_DEFAULT, K_MAX_DEFAULT) {
                NoiseVerdict::Structured { cause, .. } => CellState::Suspect {
                    priority: t.rho.max(1e-6) * t.urban_weight,
                    cause: format!("huburu: {}", cause),
                },
                NoiseVerdict::White => {
                    if t.rho <= self.eps_theta {
                        CellState::CertifiedBounded { rho: t.rho }
                    } else {
                        CellState::Suspect {
                            priority: t.rho * t.urban_weight,
                            cause: format!("residual {:.2e} above ε(Θ)", t.rho),
                        }
                    }
                }
            }
        };
        let verdict = format!("cell {} → {:?}", t.cell_id, discriminant_name(&state));
        if let Some(c) = self.cells.get_mut(&t.cell_id) {
            c.state = state;
        }
        Some((t.cell_id, verdict))
    }

    pub fn drain(&mut self) {
        while self.step().is_some() {}
    }

    pub fn finished(&self) -> bool {
        self.pending.is_empty()
            && self.tiles_done == self.tiles_seen
            && !self.cells.values().any(|c| matches!(c.state, CellState::InStream))
    }

    /// Field-team summons list, worst first — never averaged, never mixed.
    pub fn suspects_ranked(&self) -> Vec<(usize, f64, String)> {
        let mut v: Vec<_> = self
            .cells
            .values()
            .filter_map(|c| match &c.state {
                CellState::Suspect { priority, cause } => Some((c.id, *priority, cause.clone())),
                _ => None,
            })
            .collect();
        v.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        v
    }

    /// Print the Uṣurtu as standalone SVG. If streams remain, the sheet
    /// is watermarked PROVISIONAL — the map may not claim finality.
    pub fn render_svg(&self) -> String {
        let final_map = self.finished();
        let mut cells_svg = String::new();
        for c in self.cells.values() {
            let (fill, tag) = match &c.state {
                CellState::Unassessed => ("#3a3128", "UNASSESSED"),
                CellState::InStream => ("#3d6b9e", "IN-STREAM"),
                CellState::Summoned { .. } => ("#c99a2e", "SUMMONED"),
                CellState::Suspect { .. } => ("#b8442c", "SUSPECT"),
                CellState::CertifiedBounded { .. } => ("#e8b23a", "CERTIFIED"),
            };
            let cx = 90.0 + c.x * 62.0;
            let cy = 110.0 + c.y * 62.0;
            let mut pts = String::new();
            for k in 0..7 {
                let a = k as f64 / 7.0 * std::f64::consts::TAU - std::f64::consts::FRAC_PI_2;
                pts.push_str(&format!("{:.1},{:.1} ", cx + 26.0 * a.cos(), cy + 26.0 * a.sin()));
            }
            cells_svg.push_str(&format!(
                "<polygon points=\"{}\" fill=\"{}\" stroke=\"#7a5c3a\" stroke-width=\"1.5\"/>\
                 <text x=\"{:.1}\" y=\"{:.1}\" font-size=\"9\" fill=\"#17110c\" \
                 text-anchor=\"middle\" font-family=\"monospace\">{} {}</text>\n",
                pts.trim(),
                fill,
                cx,
                cy + 3.0,
                c.id,
                tag
            ));
        }
        let watermark = if final_map {
            String::new()
        } else {
            format!(
                "<text x=\"250\" y=\"150\" font-size=\"34\" fill=\"#b8442c\" opacity=\"0.45\" \
                 text-anchor=\"middle\" transform=\"rotate(-18 250 150)\" \
                 font-family=\"monospace\">PROVISIONAL · {} / {} TILES</text>",
                self.tiles_done, self.tiles_seen
            )
        };
        format!(
            "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"500\" height=\"300\" \
             viewBox=\"0 0 500 300\">\
             <rect width=\"500\" height=\"300\" fill=\"#241a12\"/>\
             <text x=\"12\" y=\"22\" font-size=\"13\" fill=\"#e8b23a\" \
             font-family=\"monospace\">UṢURTU · SUSPICION MAP · MF-URBAN-12 · \
             suspicion, never confirmation</text>\n{}{}</svg>",
            cells_svg, watermark
        )
    }
}

fn discriminant_name(s: &CellState) -> &'static str {
    match s {
        CellState::Unassessed => "UNASSESSED",
        CellState::InStream => "IN-STREAM",
        CellState::Summoned { .. } => "SUMMONED",
        CellState::Suspect { .. } => "SUSPECT",
        CellState::CertifiedBounded { .. } => "CERTIFIED-BOUNDED",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn honest(n: usize) -> Vec<f64> {
        let mut state: u64 = 3600;
        let mut next = move || {
            state = state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            ((state >> 33) as f64) / (u32::MAX as f64 + 1.0)
        };
        (0..n).map(|_| next() + next() + next() + next() - 2.0).collect()
    }

    fn layout3() -> Vec<(usize, f64, f64)> {
        vec![(0, 0.0, 0.0), (1, 1.0, 0.0), (2, 2.0, 0.0)]
    }

    #[test]
    fn stream_drains_and_no_tile_is_dropped() {
        let mut a = StreamAnalyzer::new(&layout3(), 1e-3);
        for id in 0..3 {
            a.push_tile(Tile {
                cell_id: id,
                residual_series: honest(600),
                has_subsurface: true,
                rho: 1e-4,
                urban_weight: 1.0,
            });
        }
        assert!(!a.finished(), "streams remain");
        a.drain();
        assert!(a.finished());
        assert_eq!(a.tiles_done, a.tiles_seen);
    }

    #[test]
    fn provisional_watermark_until_the_last_tile() {
        let mut a = StreamAnalyzer::new(&layout3(), 1e-3);
        a.push_tile(Tile {
            cell_id: 0,
            residual_series: honest(600),
            has_subsurface: true,
            rho: 1e-4,
            urban_weight: 1.0,
        });
        assert!(a.render_svg().contains("PROVISIONAL"), "early print must confess");
        a.drain();
        // cells 1,2 never streamed → Unassessed is honest; but streams are
        // drained, so the sheet may print final with those cells shown grey.
        assert!(!a.render_svg().contains("PROVISIONAL"));
        assert!(a.render_svg().contains("UNASSESSED"), "omission is shown, never hidden");
    }

    #[test]
    fn suspects_are_ranked_worst_first_urban_weighted() {
        let mut a = StreamAnalyzer::new(&layout3(), 1e-3);
        let mut spiky = honest(600);
        for i in (0..spiky.len()).step_by(60) {
            spiky[i] += 14.0;
        }
        a.push_tile(Tile { cell_id: 0, residual_series: spiky.clone(), has_subsurface: true, rho: 2e-3, urban_weight: 1.0 });
        a.push_tile(Tile { cell_id: 1, residual_series: spiky, has_subsurface: true, rho: 2e-3, urban_weight: 5.0 });
        a.push_tile(Tile { cell_id: 2, residual_series: honest(600), has_subsurface: false, rho: 1e-4, urban_weight: 1.0 });
        a.drain();
        let ranked = a.suspects_ranked();
        assert_eq!(ranked.len(), 2, "summoned (surface-only) is not a suspect");
        assert_eq!(ranked[0].0, 1, "homes nearby rank first — worst first, never averaged");
        assert!(matches!(a.cells[&2].state, CellState::Summoned { .. }));
    }

    #[test]
    fn svg_carries_every_state_and_the_honesty_line() {
        let mut a = StreamAnalyzer::new(&layout3(), 1e-3);
        a.push_tile(Tile { cell_id: 0, residual_series: honest(600), has_subsurface: true, rho: 1e-4, urban_weight: 1.0 });
        a.drain();
        let svg = a.render_svg();
        assert!(svg.contains("suspicion, never confirmation"));
        assert!(svg.contains("CERTIFIED") && svg.contains("UNASSESSED"));
        assert!(svg.starts_with("<svg") && svg.ends_with("</svg>"));
    }
}
