//! The 7 mandatory EAV attributes every KAKI-identified particle in the
//! 7 Types EnkiDB carries — the Architect's law, extended this session
//! from an original 5-attribute design (`State`, `ColorID(RGB)`,
//! `UserName`, `Fresh`, `Velocity`) to 7, adding `SystemUser`/`UserGroup`
//! so AnuGovernor's OTAP provenance ("who ran this, in which
//! environment") is a real, queryable EAV attribute rather than
//! unrecorded.
//!
//! Three of these seven (`state`/`color_rgb`/`freshness`) already had
//! hash constants, duplicated identically in `template-library::
//! defaults` and `story-engine::projection`. This module is the first
//! place all seven live together as one canonical, complete set — it
//! reuses the SAME hash values for those three (same `crc16(name)`
//! convention, same result) rather than introducing a third divergent
//! copy. `username`/`velocity`/`systemuser`/`usergroup` had no real hash
//! constant anywhere in code before this module.
//!
//! KAKI itself stays immutable and untouched by this — these are EAV
//! attributes attached to a particle's Orbit, never encoded into the
//! KAKI bytes themselves.

// NOTE on the first three: `template-library::defaults` and
// `story-engine::projection` both label these `// crc16("state")` etc.,
// but verified directly against the real `bahyway_crc::crc16` function,
// that label is wrong -- the real crc16("state") is 0x3E19, not 0x1A4B
// (same story for color_rgb/freshness). A pre-existing inconsistency in
// those two crates, found while building this module, not introduced by
// it. Kept here at the SAME (mislabeled) values for interoperability
// with whatever already reads/writes EAV rows using those two crates'
// constants -- changing the literal value would break that
// interoperability, which matters more here than the comment being
// accurate. Reported, not silently perpetuated: see this module's own
// tests, which check equality against the sibling crates' literal
// values, not against a real crc16 computation, for these three only.
pub const ATTR_STATE: u32 = 0x1A4B;
pub const ATTR_COLOR_RGB: u32 = 0x3D7F;
pub const ATTR_FRESHNESS: u32 = 0x4E89;

// These four are new (no prior constant anywhere in the codebase) and
// ARE real, verified crc16(name) values.
pub const ATTR_USERNAME: u32 = 0xC6D5; // crc16("username")
pub const ATTR_VELOCITY: u32 = 0x0F4D; // crc16("velocity")
pub const ATTR_SYSTEMUSER: u32 = 0x2A73; // crc16("systemuser")
pub const ATTR_USERGROUP: u32 = 0x5709; // crc16("usergroup")

/// One of the 7 mandatory attributes every particle carries.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MandatoryAttr {
    State,
    ColorRgb,
    Username,
    Freshness,
    Velocity,
    SystemUser,
    UserGroup,
}

impl MandatoryAttr {
    pub const ALL: [MandatoryAttr; 7] = [
        MandatoryAttr::State,
        MandatoryAttr::ColorRgb,
        MandatoryAttr::Username,
        MandatoryAttr::Freshness,
        MandatoryAttr::Velocity,
        MandatoryAttr::SystemUser,
        MandatoryAttr::UserGroup,
    ];

    /// The canonical EAV attribute name string this variant hashes from.
    pub fn name(&self) -> &'static str {
        match self {
            MandatoryAttr::State => "state",
            MandatoryAttr::ColorRgb => "color_rgb",
            MandatoryAttr::Username => "username",
            MandatoryAttr::Freshness => "freshness",
            MandatoryAttr::Velocity => "velocity",
            MandatoryAttr::SystemUser => "systemuser",
            MandatoryAttr::UserGroup => "usergroup",
        }
    }

    pub fn attr_hash(&self) -> u32 {
        match self {
            MandatoryAttr::State => ATTR_STATE,
            MandatoryAttr::ColorRgb => ATTR_COLOR_RGB,
            MandatoryAttr::Username => ATTR_USERNAME,
            MandatoryAttr::Freshness => ATTR_FRESHNESS,
            MandatoryAttr::Velocity => ATTR_VELOCITY,
            MandatoryAttr::SystemUser => ATTR_SYSTEMUSER,
            MandatoryAttr::UserGroup => ATTR_USERGROUP,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bahyway_crc::crc16;

    #[test]
    fn the_four_new_attributes_match_a_real_crc16_of_their_own_name() {
        // State/ColorRgb/Freshness are deliberately excluded here -- see
        // this module's own doc comment on why their literal values
        // don't equal a real crc16 of their names (a pre-existing
        // inconsistency in the crates they're kept interoperable with).
        for attr in [
            MandatoryAttr::Username,
            MandatoryAttr::Velocity,
            MandatoryAttr::SystemUser,
            MandatoryAttr::UserGroup,
        ] {
            let computed = crc16(attr.name().as_bytes()) as u32;
            assert_eq!(
                attr.attr_hash(),
                computed,
                "{:?}'s hash constant must equal crc16({:?})",
                attr,
                attr.name()
            );
        }
    }

    #[test]
    fn exactly_seven_mandatory_attributes() {
        assert_eq!(MandatoryAttr::ALL.len(), 7);
    }

    #[test]
    fn no_two_mandatory_attributes_share_a_hash() {
        let mut hashes: Vec<u32> = MandatoryAttr::ALL.iter().map(|a| a.attr_hash()).collect();
        hashes.sort_unstable();
        hashes.dedup();
        assert_eq!(
            hashes.len(),
            7,
            "every mandatory attribute must have a distinct hash"
        );
    }

    #[test]
    fn state_color_rgb_and_freshness_match_the_pre_existing_duplicated_constants() {
        // Real, existing values from template-library::defaults and
        // story-engine::projection -- this module must not silently
        // diverge from what's already live elsewhere.
        assert_eq!(MandatoryAttr::State.attr_hash(), 0x1A4B);
        assert_eq!(MandatoryAttr::ColorRgb.attr_hash(), 0x3D7F);
        assert_eq!(MandatoryAttr::Freshness.attr_hash(), 0x4E89);
    }
}
