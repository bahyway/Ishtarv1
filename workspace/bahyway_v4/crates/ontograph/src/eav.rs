//! Three-Layer Attribute Law (GL-ONT-001 §3).
//! Mandatory: sealed set, mutable values, guaranteed present — READ ONLY for OntoGraph.
//! Optional: open set — the only layer OntoGraph may WRITE.

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Layer { Mandatory, Optional }

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Attribute { pub layer: Layer, pub name: &'static str }

/// Mandatory facets read by OntoGraph as the invariant spine of the formal context.
pub const MANDATORY_FACETS: &[&str] = &[
    "w5h2.who", "w5h2.what", "w5h2.when", "w5h2.where", "w5h2.why", "w5h2.how", "w5h2.howmuch",
    "sowa.agnt", "sowa.thme", "sowa.ptim", "sowa.loc", "sowa.rson", "sowa.manr", "sowa.meas",
    "state.class",      // GOLDEN | FUZZY | DEAD
    "colour.id",        // via PASHIRU, never from KAKI bytes
    "freshness",
    "domain",
];

/// Optional attributes minted by OntoGraph (harvest).
pub const ONTO_OPTIONAL: &[&str] = &[
    "onto.concept_id", "onto.lattice_rank", "onto.hyperedge_ids", "onto.betti_signature",
    "onto.extent_size", "onto.intent_size", "onto.stability",
];

/// Optional attributes an organization brings (DMBOK semantics).
pub const DMBOK_OPTIONAL: &[&str] = &[
    "dmbok.glossary_term", "dmbok.steward", "dmbok.data_domain", "dmbok.lineage_ref",
    "dmbok.classification", "dmbok.retention", "dmbok.quality_dimension",
];

/// Guard: OntoGraph may only write Optional attributes. Mandatory writes are a law breach.
pub fn assert_writable(attr: &Attribute) -> Result<(), &'static str> {
    match attr.layer {
        Layer::Optional => Ok(()),
        Layer::Mandatory => Err("GL-ONT-001 §3.3: OntoGraph may not write Mandatory EAV"),
    }
}
