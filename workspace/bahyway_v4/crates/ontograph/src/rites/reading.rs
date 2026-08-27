//! Rite I — Reading. Assemble the FCA formal context <G, M, I>.
//! G = particles (KAKI), M = Mandatory facets ∪ topological classes ∪ Optional DMBOK facets.
use crate::kaki::Kaki;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum TopoClass {
    Golden,
    Fuzzy,
    Dead,
}

impl TopoClass {
    pub fn attribute_name(self) -> &'static str {
        match self {
            TopoClass::Golden => "topo.golden",
            TopoClass::Fuzzy => "topo.fuzzy",
            TopoClass::Dead => "topo.dead",
        }
    }
}

/// One row per particle; incidence as a bitset over attribute indices.
#[derive(Clone, Debug, Default)]
pub struct FormalContext {
    pub objects: Vec<Kaki>,
    pub attributes: Vec<String>,
    pub rows: Vec<Vec<u64>>, // bitset words per object
}

impl FormalContext {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn attribute_index(&mut self, name: &str) -> usize {
        if let Some(i) = self.attributes.iter().position(|a| a == name) {
            return i;
        }
        self.attributes.push(name.to_string());
        let words = self.attributes.len().div_ceil(64);
        for r in &mut self.rows {
            r.resize(words, 0);
        }
        self.attributes.len() - 1
    }

    /// Add a particle with the attribute names it satisfies (facets present, topo class, DMBOK facets).
    pub fn add_particle(&mut self, kaki: Kaki, satisfied: &[&str]) -> Result<(), &'static str> {
        if !kaki.verify() {
            return Err("KAKI CRC-16 verification failed");
        }
        let idxs: Vec<usize> = satisfied.iter().map(|a| self.attribute_index(a)).collect();
        let words = self.attributes.len().div_ceil(64);
        let mut row = vec![0u64; words];
        for i in idxs {
            row[i / 64] |= 1u64 << (i % 64);
        }
        self.objects.push(kaki);
        self.rows.push(row);
        Ok(())
    }

    pub fn has(&self, obj: usize, attr: usize) -> bool {
        self.rows[obj]
            .get(attr / 64)
            .is_some_and(|w| w & (1u64 << (attr % 64)) != 0)
    }
}
