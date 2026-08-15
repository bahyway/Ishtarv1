//! Rite II — Closure. Ganter's NextClosure over the formal context.
//! Every formal concept is a node; every extent is a hyperedge. No pairwise edges.
use super::reading::FormalContext;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Concept {
    pub id: usize,
    pub extent: Vec<usize>,   // object indices — this IS the hyperedge
    pub intent: Vec<usize>,   // attribute indices
}

#[derive(Clone, Debug, Default)]
pub struct Lattice {
    pub concepts: Vec<Concept>,
    /// rank = |intent| (depth from top); used for onto.lattice_rank
    pub ranks: Vec<usize>,
}

fn extent_of(ctx: &FormalContext, intent: &[bool]) -> Vec<usize> {
    (0..ctx.objects.len()).filter(|&g| (0..intent.len()).all(|m| !intent[m] || ctx.has(g, m))).collect()
}

fn intent_of(ctx: &FormalContext, extent: &[usize]) -> Vec<bool> {
    (0..ctx.attributes.len()).map(|m| extent.iter().all(|&g| ctx.has(g, m))).collect()
}

fn closure(ctx: &FormalContext, a: &[bool]) -> Vec<bool> { intent_of(ctx, &extent_of(ctx, a)) }

fn next_closure(ctx: &FormalContext, a: &[bool]) -> Option<Vec<bool>> {
    let n = a.len();
    for i in (0..n).rev() {
        if a[i] { continue; }
        let mut b: Vec<bool> = a.iter().enumerate().map(|(j, &v)| v && j < i).collect();
        b[i] = true;
        let c = closure(ctx, &b);
        if (0..i).all(|j| !(c[j] && !a[j])) { return Some(c); }
    }
    None
}

impl Lattice {
    /// Compute all formal concepts in lectic order.
    pub fn compute(ctx: &FormalContext) -> Lattice {
        let n = ctx.attributes.len();
        let mut lat = Lattice::default();
        let mut cur = closure(ctx, &vec![false; n]);
        loop {
            let intent: Vec<usize> = (0..n).filter(|&m| cur[m]).collect();
            let extent = extent_of(ctx, &cur);
            let id = lat.concepts.len();
            lat.ranks.push(intent.len());
            lat.concepts.push(Concept { id, extent, intent });
            match next_closure(ctx, &cur) { Some(nx) => cur = nx, None => break }
        }
        lat
    }

    /// Structural check (validation ladder step 1): every extent is closed.
    pub fn is_closed(&self, ctx: &FormalContext) -> bool {
        self.concepts.iter().all(|c| {
            let ib = intent_of(ctx, &c.extent);
            extent_of(ctx, &ib) == c.extent
        })
    }

    /// Hyperedges of the OntoGraph: one per non-empty extent.
    pub fn hyperedges(&self) -> Vec<&Concept> { self.concepts.iter().filter(|c| !c.extent.is_empty()).collect() }
}
