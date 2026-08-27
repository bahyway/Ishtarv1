//! BookKaki — a book as a sovereign KAKI particle.
//!
//! Tribe IDs occupy the EnkiduLLM namespace 0x1001–0x10FF, distinct from
//! particle/data tribes (0x0001–0x00FF) to prevent namespace collision.

use crate::orbit::BookOrbit;
use bahyway_core::TribeId;
use enkidb_kaki::{Kaki, KakiMinter, KakiRole};

/// FNV-1a 32-bit hash — sovereign implementation, no external deps.
pub fn book_uuid_hash(isbn_or_path: &[u8]) -> u32 {
    const OFFSET: u32 = 2_166_136_261;
    const PRIME: u32 = 16_777_619;
    let mut h = OFFSET;
    for &b in isbn_or_path {
        h ^= b as u32;
        h = h.wrapping_mul(PRIME);
    }
    h
}

/// Book domain tribe IDs (EnkiduLLM namespace 0x1001–0x1007).
pub struct BookDomainTribe;

impl BookDomainTribe {
    pub const COMPUTER_SCIENCE: TribeId = TribeId::from_u16(0x1001);
    pub const MATHEMATICS: TribeId = TribeId::from_u16(0x1002);
    pub const PHYSICAL_SCIENCES: TribeId = TribeId::from_u16(0x1003);
    pub const LIFE_SCIENCES: TribeId = TribeId::from_u16(0x1004);
    pub const SOCIAL_SCIENCES: TribeId = TribeId::from_u16(0x1005);
    pub const HUMANITIES: TribeId = TribeId::from_u16(0x1006);
    pub const CROSS_DOMAIN: TribeId = TribeId::from_u16(0x1007);
    /// Reserved for linguistic token KAKIs only.
    pub const LINGUISTIC: TribeId = TribeId::from_u16(0x10FF);

    /// Derive tribe from a domain string (case-insensitive prefix match).
    pub fn from_domain(domain: &str) -> TribeId {
        let d = domain.to_ascii_lowercase();
        if d.contains("computer") || d.contains("software") || d.contains("systems") {
            Self::COMPUTER_SCIENCE
        } else if d.contains("math") || d.contains("logic") || d.contains("statistic") {
            Self::MATHEMATICS
        } else if d.contains("physics") || d.contains("chemistry") || d.contains("astro") {
            Self::PHYSICAL_SCIENCES
        } else if d.contains("biology") || d.contains("medicine") || d.contains("life") {
            Self::LIFE_SCIENCES
        } else if d.contains("social") || d.contains("economics") || d.contains("psych") {
            Self::SOCIAL_SCIENCES
        } else if d.contains("history") || d.contains("literature") || d.contains("philos") {
            Self::HUMANITIES
        } else {
            Self::CROSS_DOMAIN
        }
    }
}

/// A book as a sovereign triple-O KAKI particle.
///
/// Nucleus = structural identity (immutable, 16 bytes).
/// Orbit   = four assessment shells (mutable via versioned EAV updates).
#[derive(Debug, Clone)]
pub struct BookKaki {
    pub nucleus: Kaki,
    pub orbit: BookOrbit,
}

impl BookKaki {
    /// Mint a new BookKaki from metadata.
    /// `isbn_or_path` produces a stable, deterministic uuid_hash.
    pub fn new(isbn_or_path: &[u8], tribe_id: TribeId, role: KakiRole, orbit: BookOrbit) -> Self {
        let minter = KakiMinter::new(tribe_id);
        let nucleus = minter.mint_identity(book_uuid_hash(isbn_or_path), role);
        Self { nucleus, orbit }
    }

    /// Mint an ingestion Event-Kaki (kaki_type=0x02, kaki_role=Zikru) for the journal.
    pub fn ingestion_event(&self) -> Kaki {
        KakiMinter::new(self.nucleus.tribe_id()).event(KakiRole::Zikru)
    }

    pub fn uuid_hash(&self) -> u32 {
        self.nucleus.uuid_hash()
    }
    pub fn tribe_id(&self) -> TribeId {
        self.nucleus.tribe_id()
    }
    pub fn role(&self) -> KakiRole {
        self.nucleus.kaki_role()
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::orbit::{BookOrbit, CoreShell};

    fn dummy_orbit() -> BookOrbit {
        BookOrbit::from_core(CoreShell {
            title: "Test Book".into(),
            author: "Test Author".into(),
            publisher: None,
            isbn: Some("978-0000000000".into()),
            source_path: "/books/test.pdf".into(),
            source_seal: 0xDEADBEEF,
        })
    }

    #[test]
    fn book_uuid_hash_stable() {
        let h1 = book_uuid_hash(b"978-0596516963");
        let h2 = book_uuid_hash(b"978-0596516963");
        assert_eq!(h1, h2);
    }

    #[test]
    fn book_uuid_hash_distinct() {
        let h1 = book_uuid_hash(b"978-0596516963");
        let h2 = book_uuid_hash(b"978-1491950357");
        assert_ne!(h1, h2);
    }

    #[test]
    fn book_kaki_nucleus_valid() {
        let bk = BookKaki::new(
            b"978-0596516963",
            BookDomainTribe::COMPUTER_SCIENCE,
            KakiRole::Kishib,
            dummy_orbit(),
        );
        assert!(bk.nucleus.verify_checksum());
        assert_eq!(bk.nucleus.tribe_id(), BookDomainTribe::COMPUTER_SCIENCE);
        assert_eq!(bk.nucleus.kaki_role(), KakiRole::Kishib);
        assert_eq!(bk.uuid_hash(), bk.uuid_hash(), "uuid_hash must be stable");
    }

    #[test]
    fn ingestion_event_is_event_kaki() {
        use enkidb_kaki::KakiType;
        let bk = BookKaki::new(
            b"isbn-test",
            BookDomainTribe::MATHEMATICS,
            KakiRole::Zikru,
            dummy_orbit(),
        );
        let ev = bk.ingestion_event();
        assert_eq!(ev.kaki_type(), KakiType::Event);
        assert_eq!(ev.tribe_id(), BookDomainTribe::MATHEMATICS);
        assert!(ev.verify_checksum());
    }

    #[test]
    fn domain_tribe_computer_science() {
        assert_eq!(
            BookDomainTribe::from_domain("distributed_systems"),
            BookDomainTribe::COMPUTER_SCIENCE
        );
        assert_eq!(
            BookDomainTribe::from_domain("software engineering"),
            BookDomainTribe::COMPUTER_SCIENCE
        );
    }

    #[test]
    fn domain_tribe_cross_domain_fallback() {
        assert_eq!(
            BookDomainTribe::from_domain("unknown_field"),
            BookDomainTribe::CROSS_DOMAIN
        );
    }

    #[test]
    fn linguistic_tribe_reserved() {
        assert_eq!(BookDomainTribe::LINGUISTIC.as_u16(), 0x10FF);
    }
}
