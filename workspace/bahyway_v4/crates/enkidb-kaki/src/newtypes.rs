//! Newtype wrappers: IdentityKaki, EventKaki, CrossTribeKaki.
//!
//! Per §13.2: lifting the physical type to the type level prevents
//! passing an IdentityKaki where an EventKaki is expected.

use bahyway_core::{BahywayError, Result};
use crate::{Kaki, KakiRole, KakiType};

macro_rules! kaki_newtype {
    ($Wrapper:ident, $variant:expr, $doc:literal) => {
        #[doc = $doc]
        #[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
        #[repr(transparent)]
        pub struct $Wrapper(pub Kaki);

        impl $Wrapper {
            /// Wrap a `Kaki` if it has the correct physical type.
            pub fn try_from_kaki(k: Kaki) -> Result<Self> {
                if k.kaki_type() == $variant {
                    Ok(Self(k))
                } else {
                    Err(BahywayError::InvalidKakiType(k.bytes()[6]))
                }
            }

            /// Borrow the inner `Kaki`.
            #[inline]
            pub fn kaki(&self) -> &Kaki { &self.0 }

            /// Logical role encoded in κ[7].
            #[inline]
            pub fn role(&self) -> KakiRole { self.0.kaki_role() }
        }

        impl core::ops::Deref for $Wrapper {
            type Target = Kaki;
            fn deref(&self) -> &Kaki { &self.0 }
        }

        impl core::fmt::Display for $Wrapper {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                core::fmt::Display::fmt(&self.0, f)
            }
        }
    };
}

kaki_newtype!(
    IdentityKaki,
    KakiType::Identity,
    "Identity-Kaki (kaki_type=0x01) — the birth certificate of a particle."
);

kaki_newtype!(
    EventKaki,
    KakiType::Event,
    "Event-Kaki (kaki_type=0x02) — immutable record of a state-transition event."
);

kaki_newtype!(
    CrossTribeKaki,
    KakiType::CrossTribe,
    "CrossTribe-Kaki (kaki_type=0x03) — persistent link between anchor particles."
);

kaki_newtype!(
    PatternKaki,
    KakiType::Pattern,
    "Pattern-Kaki (kaki_type=0x04) — deterministic KAKI for an emergent GA-cluster pattern (NISABA)."
);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mint::KakiMinter;
    use bahyway_core::TribeId;

    fn minter() -> KakiMinter {
        KakiMinter::new(TribeId::from_u16(0x0042))
    }

    #[test]
    fn identity_wraps_identity_kaki() {
        let m = minter();
        let k = m.identity(KakiRole::Zikru);
        let ik = IdentityKaki::try_from_kaki(k).unwrap();
        assert_eq!(ik.kaki_type(), KakiType::Identity);
    }

    #[test]
    fn event_rejects_identity_kaki() {
        let m = minter();
        let k = m.identity(KakiRole::Zikru);
        assert!(EventKaki::try_from_kaki(k).is_err());
    }

    #[test]
    fn crosstribe_wraps_crosstribe_kaki() {
        let m = minter();
        let k = m.crosstribe(KakiRole::Zikru);
        let ck = CrossTribeKaki::try_from_kaki(k).unwrap();
        assert_eq!(ck.kaki_type(), KakiType::CrossTribe);
    }
}
