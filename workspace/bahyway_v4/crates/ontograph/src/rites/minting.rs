//! Rite III — Minting. Nebuchadnezzar = lattice + hyperedges, minted as a template particle
//! (GL-TPL-001) with Hepta Space coordinates. Writes back Optional EAV only.
//! KISPU commit and NĀRU witness are performed by the caller (PB-324).
use super::closure::Lattice;
use super::reading::FormalContext;
use crate::eav::{assert_writable, Attribute, Layer};
use crate::kaki::Kaki;

/// Optional EAV write intent for one member particle.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OptionalWrite {
    pub kaki: Kaki,
    pub attribute: &'static str,
    pub value: String,
}

#[derive(Clone, Debug)]
pub struct Nebuchadnezzar {
    pub name: &'static str, // "Nebuchadnezzar" — NL-001 §6b Landmark Pattern Clause
    pub lattice: Lattice,
    pub writes: Vec<OptionalWrite>,      // Optional-only, checked
    pub hepta_position: (f64, f64, f64), // OrbitalPosition (r, θ, φ) — assigned by GeoEngine at mint
}

impl Nebuchadnezzar {
    pub fn mint(ctx: &FormalContext, lattice: Lattice) -> Result<Self, &'static str> {
        let mut writes = Vec::new();
        for c in lattice.hyperedges() {
            for &g in &c.extent {
                for (attr, val) in [
                    ("onto.concept_id", c.id.to_string()),
                    ("onto.lattice_rank", lattice.ranks[c.id].to_string()),
                    ("onto.extent_size", c.extent.len().to_string()),
                    ("onto.intent_size", c.intent.len().to_string()),
                ] {
                    assert_writable(&Attribute {
                        layer: Layer::Optional,
                        name: attr,
                    })?;
                    writes.push(OptionalWrite {
                        kaki: ctx.objects[g],
                        attribute: attr,
                        value: val,
                    });
                }
            }
        }
        Ok(Nebuchadnezzar {
            name: "Nebuchadnezzar",
            lattice,
            writes,
            hepta_position: (0.0, 0.0, 0.0),
        })
    }

    /// Universality proposal only — promotion is design-time at Gate G4 (GL-ONT-001 §3.4).
    pub fn propose_universal(&self, ctx: &FormalContext) -> Vec<String> {
        let n = ctx.objects.len();
        self.lattice
            .concepts
            .iter()
            .filter(|c| c.extent.len() == n && n > 0)
            .flat_map(|c| c.intent.iter().map(|&m| ctx.attributes[m].clone()))
            .collect()
    }
}
