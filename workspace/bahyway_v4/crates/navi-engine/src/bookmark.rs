//! BookmarkStore — sovereign named locations with KAKI identity.
//!
//! Every bookmark is a sovereign particle: it has a KAKI identity minted at
//! creation time, a category, Arabic name, and geographic coordinate.
//! Bookmarks persist across routing sessions and are shared across
//! NajafEngine (pilgrimage waypoints) and WpdEngine (infrastructure landmarks).

#![forbid(unsafe_code)]

use std::collections::HashMap;
use bahyway_core::TribeId;
use enkidb_kaki::{Kaki, KakiMinter, KakiRole};

use crate::particle::NaviCoord;
use crate::route::haversine_m;

// ── BookmarkCategory ──────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BookmarkCategory {
    /// Cemetery / burial site — NajafEngine pilgrimage target.
    Grave,
    /// Water/power/telecoms infrastructure inspection point.
    Infrastructure,
    /// Generic routing waypoint (transit hub, checkpoint).
    Waypoint,
    /// Known hazard zone (flood risk, restricted area).
    Hazard,
    /// Sacred or historically significant site.
    Sacred,
    /// Personal point of interest.
    Personal,
}

impl BookmarkCategory {
    pub fn label(self) -> &'static str {
        match self {
            BookmarkCategory::Grave          => "Grave",
            BookmarkCategory::Infrastructure => "Infrastructure",
            BookmarkCategory::Waypoint       => "Waypoint",
            BookmarkCategory::Hazard         => "Hazard",
            BookmarkCategory::Sacred         => "Sacred",
            BookmarkCategory::Personal       => "Personal",
        }
    }
}

// ── Bookmark ──────────────────────────────────────────────────────────────────

pub struct Bookmark {
    pub id:           u32,
    /// Immutable sovereign identity.
    pub kaki:         Kaki,
    /// Canonical Arabic name — sovereign label.
    pub name_arabic:  String,
    /// Optional Latin transliteration.
    pub name_latin:   Option<String>,
    pub coord:        NaviCoord,
    pub category:     BookmarkCategory,
    pub notes:        Option<String>,
    /// Creation epoch (Hijri year or Unix seconds — caller's choice).
    pub epoch:        u32,
}

impl Bookmark {
    pub fn kaki_hash(&self) -> u32 { self.kaki.uuid_hash() }
}

// ── BookmarkStore ─────────────────────────────────────────────────────────────

pub struct BookmarkStore {
    entries: HashMap<u32, Bookmark>,
    next_id: u32,
}

impl BookmarkStore {
    pub fn new() -> Self { BookmarkStore { entries: HashMap::new(), next_id: 1 } }

    /// Create and store a new bookmark.  Returns the assigned ID.
    pub fn add(
        &mut self,
        name_arabic: impl Into<String>,
        coord:       NaviCoord,
        category:    BookmarkCategory,
        tribe:       TribeId,
        epoch:       u32,
    ) -> u32 {
        let id     = self.next_id;
        let minter = KakiMinter::new(tribe);
        let kaki   = minter.mint_identity(id, KakiRole::Zikru);
        self.entries.insert(id, Bookmark {
            id,
            kaki,
            name_arabic: name_arabic.into(),
            name_latin:  None,
            coord,
            category,
            notes: None,
            epoch,
        });
        self.next_id += 1;
        id
    }

    /// Add Latin transliteration to an existing bookmark.
    pub fn set_latin(&mut self, id: u32, name_latin: impl Into<String>) -> bool {
        if let Some(b) = self.entries.get_mut(&id) {
            b.name_latin = Some(name_latin.into());
            true
        } else { false }
    }

    /// Add notes to an existing bookmark.
    pub fn set_notes(&mut self, id: u32, notes: impl Into<String>) -> bool {
        if let Some(b) = self.entries.get_mut(&id) {
            b.notes = Some(notes.into());
            true
        } else { false }
    }

    pub fn get(&self, id: u32) -> Option<&Bookmark> { self.entries.get(&id) }

    pub fn remove(&mut self, id: u32) -> bool { self.entries.remove(&id).is_some() }

    /// All bookmarks in a given category.
    pub fn by_category(&self, category: BookmarkCategory) -> Vec<&Bookmark> {
        self.entries.values().filter(|b| b.category == category).collect()
    }

    /// The nearest bookmark to a coordinate.
    pub fn nearest(&self, coord: NaviCoord) -> Option<&Bookmark> {
        self.entries.values().min_by(|a, b| {
            let da = haversine_m(a.coord.lat, a.coord.lon, coord.lat, coord.lon);
            let db = haversine_m(b.coord.lat, b.coord.lon, coord.lat, coord.lon);
            da.partial_cmp(&db).unwrap_or(std::cmp::Ordering::Equal)
        })
    }

    /// Nearest bookmark in a specific category.
    pub fn nearest_in_category(&self, coord: NaviCoord, category: BookmarkCategory) -> Option<&Bookmark> {
        self.entries.values()
            .filter(|b| b.category == category)
            .min_by(|a, b| {
                let da = haversine_m(a.coord.lat, a.coord.lon, coord.lat, coord.lon);
                let db = haversine_m(b.coord.lat, b.coord.lon, coord.lat, coord.lon);
                da.partial_cmp(&db).unwrap_or(std::cmp::Ordering::Equal)
            })
    }

    /// Bookmarks within `radius_m` of a coordinate.
    pub fn within_radius(&self, coord: NaviCoord, radius_m: f32) -> Vec<&Bookmark> {
        self.entries.values()
            .filter(|b| haversine_m(b.coord.lat, b.coord.lon, coord.lat, coord.lon) <= radius_m)
            .collect()
    }

    pub fn count(&self) -> usize { self.entries.len() }

    pub fn all(&self) -> impl Iterator<Item = &Bookmark> { self.entries.values() }

    pub fn is_empty(&self) -> bool { self.entries.is_empty() }
}

impl Default for BookmarkStore {
    fn default() -> Self { Self::new() }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use bahyway_core::TribeId;
    use crate::particle::NaviCoord;

    fn tribe() -> TribeId { TribeId::from_u16(0x0001) }
    fn najaf() -> NaviCoord { NaviCoord::new(31.995, 44.320, 0.0) }

    fn store_with_entries() -> BookmarkStore {
        let mut s = BookmarkStore::new();
        s.add("مرقد الإمام علي", najaf(), BookmarkCategory::Sacred, tribe(), 1445);
        s.add("مقبرة الشهيد الكاظمي", NaviCoord::new(32.005, 44.320, 0.0),
              BookmarkCategory::Grave, tribe(), 1440);
        s.add("محطة ضخ الماء", NaviCoord::new(31.980, 44.310, 0.0),
              BookmarkCategory::Infrastructure, tribe(), 1445);
        s
    }

    #[test]
    fn add_and_retrieve() {
        let mut s = BookmarkStore::new();
        let id = s.add("اختبار", najaf(), BookmarkCategory::Waypoint, tribe(), 1445);
        assert!(s.get(id).is_some());
        assert_eq!(s.count(), 1);
    }

    #[test]
    fn kaki_minted_and_valid() {
        let mut s = BookmarkStore::new();
        let id = s.add("موضع", najaf(), BookmarkCategory::Sacred, tribe(), 1445);
        let b = s.get(id).unwrap();
        assert!(b.kaki.verify_checksum());
    }

    #[test]
    fn remove_decrements_count() {
        let mut s = store_with_entries();
        assert_eq!(s.count(), 3);
        let id = s.all().next().unwrap().id;
        s.remove(id);
        assert_eq!(s.count(), 2);
    }

    #[test]
    fn by_category_filters_correctly() {
        let s = store_with_entries();
        assert_eq!(s.by_category(BookmarkCategory::Sacred).len(),         1);
        assert_eq!(s.by_category(BookmarkCategory::Grave).len(),          1);
        assert_eq!(s.by_category(BookmarkCategory::Infrastructure).len(), 1);
        assert_eq!(s.by_category(BookmarkCategory::Hazard).len(),         0);
    }

    #[test]
    fn nearest_returns_closest() {
        let s = store_with_entries();
        let near = s.nearest(NaviCoord::new(32.005, 44.320, 0.0)).unwrap();
        // Grave bookmark at 32.005 is the closest
        assert_eq!(near.category, BookmarkCategory::Grave);
    }

    #[test]
    fn nearest_in_category() {
        let s = store_with_entries();
        let infra = s.nearest_in_category(najaf(), BookmarkCategory::Infrastructure).unwrap();
        assert_eq!(infra.category, BookmarkCategory::Infrastructure);
    }

    #[test]
    fn within_radius_finds_nearby() {
        let s = store_with_entries();
        // Only the Sacred entry is near najaf() within 500 m
        let near = s.within_radius(najaf(), 500.0);
        assert!(!near.is_empty());
        assert!(near.iter().any(|b| b.category == BookmarkCategory::Sacred));
    }

    #[test]
    fn set_latin_returns_true_for_existing() {
        let mut s = BookmarkStore::new();
        let id = s.add("مرقد", najaf(), BookmarkCategory::Sacred, tribe(), 1445);
        assert!(s.set_latin(id, "Maqam"));
        assert_eq!(s.get(id).unwrap().name_latin.as_deref(), Some("Maqam"));
    }

    #[test]
    fn set_notes_returns_false_for_missing() {
        let mut s = BookmarkStore::new();
        assert!(!s.set_notes(9999, "note"));
    }

    #[test]
    fn empty_store() {
        let s = BookmarkStore::new();
        assert!(s.is_empty());
        assert!(s.nearest(najaf()).is_none());
    }

    #[test]
    fn sequential_ids_unique() {
        let mut s = BookmarkStore::new();
        let id1 = s.add("أ", najaf(), BookmarkCategory::Waypoint, tribe(), 1445);
        let id2 = s.add("ب", najaf(), BookmarkCategory::Waypoint, tribe(), 1445);
        assert_ne!(id1, id2);
    }

    #[test]
    fn bookmark_category_labels_non_empty() {
        for cat in [
            BookmarkCategory::Grave, BookmarkCategory::Infrastructure,
            BookmarkCategory::Waypoint, BookmarkCategory::Hazard,
            BookmarkCategory::Sacred, BookmarkCategory::Personal,
        ] {
            assert!(!cat.label().is_empty());
        }
    }
}
