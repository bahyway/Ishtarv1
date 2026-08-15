//! Physical asset registry: maps the electrical graph onto geography.
//! Identity over appearance — two converters may look identical in the
//! field; their KAKI identity and passport never do (Uniqueness Law,
//! applied to matter).

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum AssetKind {
    Tower,
    OverheadLine,
    UndergroundCable,
    Junction,
    Converter,
}

/// Reference into the fdd-core network.
#[derive(Clone, Copy, Debug)]
pub enum GridRef {
    Node(u32),
    Edge(usize),
}

#[derive(Clone, Debug)]
pub struct Asset {
    /// 16-byte KAKI identity (kaki_type 0x01) — the real disambiguator.
    /// Field crews confirm THIS, not looks.
    pub kaki: [u8; 16],
    pub kind: AssetKind,
    pub grid_ref: GridRef,
    /// WGS84 passthrough for the stage. Geodesy is GeoEngine's truth —
    /// never computed here.
    pub lat: f64,
    pub lon: f64,
    /// Meters below grade for underground cables/junctions; 0 for
    /// surface, tower height ignored here.
    pub depth_m: f64,
    /// Passport EAV (name, value) — install date, manufacturer, feeder,
    /// rating, last inspection...
    pub passport: Vec<(String, String)>,
}

#[derive(Default)]
pub struct AssetRegistry {
    pub assets: Vec<Asset>,
}

impl AssetRegistry {
    pub fn by_grid_ref(&self, r: GridRef) -> Option<&Asset> {
        self.assets.iter().find(|a| match (a.grid_ref, r) {
            (GridRef::Node(x), GridRef::Node(y)) => x == y,
            (GridRef::Edge(x), GridRef::Edge(y)) => x == y,
            _ => false,
        })
    }
}
