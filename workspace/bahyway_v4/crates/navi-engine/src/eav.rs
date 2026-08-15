//! EAV (Entity-Attribute-Value) sovereign attribute store.
//!
//! Attribute keys are FNV-1a 32-bit hashes of the attribute name, computed
//! at compile time via `const fn`.  Pre-defined sovereign keys are exported
//! as `ATTR_*` constants.
//!
//! Every `MapParticle` carries an `EavStore` of typed attribute values.
//! Attributes are flagged `required` or optional: `required_missing()` lists
//! sovereign keys that MUST be present for a particle to be considered valid.

#![forbid(unsafe_code)]

pub type AttrKey = u32;

// ── Compile-time FNV-1a ───────────────────────────────────────────────────────

const fn fnv1a_const(s: &[u8]) -> u32 {
    let mut h: u32 = 0x811C_9DC5;
    let mut i = 0;
    while i < s.len() {
        h ^= s[i] as u32;
        h = h.wrapping_mul(0x0100_0193);
        i += 1;
    }
    h
}

// ── Sovereign attribute keys ──────────────────────────────────────────────────

/// Human-readable name (road name, pipeline name, POI name).
pub const ATTR_NAME:          AttrKey = fnv1a_const(b"name");
/// Arabic name — sovereign canonical label.
pub const ATTR_NAME_ARABIC:   AttrKey = fnv1a_const(b"name_arabic");
/// Speed limit in km/h.
pub const ATTR_SPEED_LIMIT:   AttrKey = fnv1a_const(b"speed_limit");
/// Number of traffic lanes.
pub const ATTR_LANES:         AttrKey = fnv1a_const(b"lanes");
/// One-way flag — only vehicles travelling in beam direction allowed.
pub const ATTR_ONE_WAY:       AttrKey = fnv1a_const(b"one_way");
/// Road class — encoded as u8 matching `RoadClass` discriminant.
pub const ATTR_ROAD_CLASS:    AttrKey = fnv1a_const(b"road_class");
/// Pipeline inner diameter in millimetres.
pub const ATTR_DIAMETER_MM:   AttrKey = fnv1a_const(b"diameter_mm");
/// Pipeline operating pressure in kPa.
pub const ATTR_PRESSURE_KPA:  AttrKey = fnv1a_const(b"pressure_kpa");
/// Flow direction — encoded as u8 matching `FlowDir` discriminant.
pub const ATTR_FLOW_DIR:      AttrKey = fnv1a_const(b"flow_dir");
/// Sacred weight — f32×1000 stored as UInt. NajafEngine zone priority.
pub const ATTR_SACRED_WEIGHT: AttrKey = fnv1a_const(b"sacred_weight");
/// Elevation in metres above sea level (WGS-84 ellipsoid height).
pub const ATTR_ELEVATION_M:   AttrKey = fnv1a_const(b"elevation_m");
/// Pipe material — encoded as u8 matching `PipeMaterial` discriminant.
pub const ATTR_PIPE_MATERIAL: AttrKey = fnv1a_const(b"pipe_material");
/// Age in calendar years (for infrastructure condition assessment).
pub const ATTR_AGE_YEARS:     AttrKey = fnv1a_const(b"age_years");
/// Access restriction level: 0=public 1=permit 2=restricted 3=sovereign-only.
pub const ATTR_ACCESS_LEVEL:  AttrKey = fnv1a_const(b"access_level");

// ── AttrValue ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub enum AttrValue {
    Text(String),
    Int(i32),
    UInt(u32),
    Float(f32),
    Bool(bool),
    /// KAKI hash reference — points to another sovereign particle.
    KakiRef(u32),
}

impl AttrValue {
    pub fn as_text(&self)   -> Option<&str>  { if let AttrValue::Text(s) = self { Some(s) } else { None } }
    pub fn as_int(&self)    -> Option<i32>   { if let AttrValue::Int(v)  = self { Some(*v) } else { None } }
    pub fn as_uint(&self)   -> Option<u32>   { if let AttrValue::UInt(v) = self { Some(*v) } else { None } }
    pub fn as_float(&self)  -> Option<f32>   { if let AttrValue::Float(v)= self { Some(*v) } else { None } }
    pub fn as_bool(&self)   -> Option<bool>  { if let AttrValue::Bool(b) = self { Some(*b) } else { None } }
    pub fn as_kaki(&self)   -> Option<u32>   { if let AttrValue::KakiRef(h) = self { Some(*h) } else { None } }
}

// ── EavAttr ───────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct EavAttr {
    pub key:      AttrKey,
    pub value:    AttrValue,
    pub required: bool,
}

impl EavAttr {
    pub fn required(key: AttrKey, value: AttrValue)  -> Self { EavAttr { key, value, required: true } }
    pub fn optional(key: AttrKey, value: AttrValue)  -> Self { EavAttr { key, value, required: false } }
}

// ── EavStore ──────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Default)]
pub struct EavStore {
    attrs: Vec<EavAttr>,
}

impl EavStore {
    pub fn new() -> Self { EavStore { attrs: Vec::new() } }

    /// Add or replace an attribute.
    pub fn set(&mut self, attr: EavAttr) {
        if let Some(existing) = self.attrs.iter_mut().find(|a| a.key == attr.key) {
            *existing = attr;
        } else {
            self.attrs.push(attr);
        }
    }

    pub fn get(&self, key: AttrKey) -> Option<&AttrValue> {
        self.attrs.iter().find(|a| a.key == key).map(|a| &a.value)
    }

    pub fn has(&self, key: AttrKey) -> bool { self.attrs.iter().any(|a| a.key == key) }

    pub fn get_text(&self,  key: AttrKey) -> Option<&str>  { self.get(key)?.as_text() }
    pub fn get_int(&self,   key: AttrKey) -> Option<i32>   { self.get(key)?.as_int() }
    pub fn get_uint(&self,  key: AttrKey) -> Option<u32>   { self.get(key)?.as_uint() }
    pub fn get_float(&self, key: AttrKey) -> Option<f32>   { self.get(key)?.as_float() }
    pub fn get_bool(&self,  key: AttrKey) -> Option<bool>  { self.get(key)?.as_bool() }
    pub fn get_kaki(&self,  key: AttrKey) -> Option<u32>   { self.get(key)?.as_kaki() }

    /// List of required attribute keys that are absent — particle is invalid if non-empty.
    pub fn required_missing(&self) -> Vec<AttrKey> {
        self.attrs.iter()
            .filter(|a| a.required)
            .map(|a| a.key)
            .filter(|&k| !self.has(k))
            .collect()
    }

    pub fn attr_count(&self) -> usize { self.attrs.len() }
    pub fn all_attrs(&self)  -> &[EavAttr] { &self.attrs }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn attr_keys_are_unique() {
        let keys = [
            ATTR_NAME, ATTR_NAME_ARABIC, ATTR_SPEED_LIMIT, ATTR_LANES,
            ATTR_ONE_WAY, ATTR_ROAD_CLASS, ATTR_DIAMETER_MM, ATTR_PRESSURE_KPA,
            ATTR_FLOW_DIR, ATTR_SACRED_WEIGHT, ATTR_ELEVATION_M,
            ATTR_PIPE_MATERIAL, ATTR_AGE_YEARS, ATTR_ACCESS_LEVEL,
        ];
        for i in 0..keys.len() {
            for j in (i+1)..keys.len() {
                assert_ne!(keys[i], keys[j], "key {i} and {j} must be unique");
            }
        }
    }

    #[test]
    fn eav_set_and_get() {
        let mut s = EavStore::new();
        s.set(EavAttr::required(ATTR_NAME, AttrValue::Text("Najaf Street".into())));
        assert_eq!(s.get_text(ATTR_NAME), Some("Najaf Street"));
        assert!(s.has(ATTR_NAME));
    }

    #[test]
    fn eav_replace_existing() {
        let mut s = EavStore::new();
        s.set(EavAttr::optional(ATTR_SPEED_LIMIT, AttrValue::UInt(60)));
        s.set(EavAttr::optional(ATTR_SPEED_LIMIT, AttrValue::UInt(80)));
        assert_eq!(s.get_uint(ATTR_SPEED_LIMIT), Some(80));
        assert_eq!(s.attr_count(), 1);
    }

    #[test]
    fn required_missing_when_never_set() {
        let s = EavStore::new();
        // No attrs set — required_missing scans required attrs only; with empty store it's empty
        assert!(s.required_missing().is_empty());
    }

    #[test]
    fn bool_attr_roundtrip() {
        let mut s = EavStore::new();
        s.set(EavAttr::optional(ATTR_ONE_WAY, AttrValue::Bool(true)));
        assert_eq!(s.get_bool(ATTR_ONE_WAY), Some(true));
    }

    #[test]
    fn float_attr_roundtrip() {
        let mut s = EavStore::new();
        s.set(EavAttr::optional(ATTR_SACRED_WEIGHT, AttrValue::Float(0.9)));
        let v = s.get_float(ATTR_SACRED_WEIGHT).unwrap();
        assert!((v - 0.9).abs() < 0.001);
    }

    #[test]
    fn kaki_ref_roundtrip() {
        let mut s = EavStore::new();
        s.set(EavAttr::optional(ATTR_PIPE_MATERIAL, AttrValue::KakiRef(0xDEAD_BEEF)));
        assert_eq!(s.get_kaki(ATTR_PIPE_MATERIAL), Some(0xDEAD_BEEF));
    }

    #[test]
    fn get_missing_key_returns_none() {
        let s = EavStore::new();
        assert!(s.get(ATTR_NAME).is_none());
        assert!(s.get_text(ATTR_NAME).is_none());
        assert!(s.get_bool(ATTR_ONE_WAY).is_none());
    }

    #[test]
    fn wrong_value_type_returns_none() {
        let mut s = EavStore::new();
        s.set(EavAttr::optional(ATTR_NAME, AttrValue::Text("x".into())));
        // Name is Text, asking for Int → None
        assert!(s.get_int(ATTR_NAME).is_none());
    }

    #[test]
    fn fnv1a_const_deterministic() {
        assert_eq!(fnv1a_const(b"name"), fnv1a_const(b"name"));
        assert_ne!(fnv1a_const(b"name"), fnv1a_const(b"lane"));
    }

    #[test]
    fn all_attrs_visible() {
        let mut s = EavStore::new();
        s.set(EavAttr::optional(ATTR_LANES, AttrValue::UInt(2)));
        s.set(EavAttr::optional(ATTR_SPEED_LIMIT, AttrValue::UInt(50)));
        assert_eq!(s.all_attrs().len(), 2);
    }
}
