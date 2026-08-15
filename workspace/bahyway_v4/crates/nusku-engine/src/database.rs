use crate::types::KakiPK;

// ── DatabaseConnector (from databaseway-wayv354) ──────────────────
// Async dep removed — sovereign checkpoint lookup is synchronous (≤150ms budget).
// No separate crate; lives here as a module inside nusku-engine.

#[derive(Debug, Clone)]
pub enum DatabaseStatus { Online, Offline, Unauthorized }

#[derive(Debug, Clone)]
pub struct AuthorityLookupResult {
    pub queried_kaki: KakiPK,
    pub matched:      bool,
    pub watchlist:    bool,
    /// Encrypted authority reference — None in simulation or unauthorized state
    pub identity_ref: Option<String>,
    pub match_score:  f32,
    pub db_status:    DatabaseStatus,
}

pub struct DatabaseConnector {
    authorized: bool,
}

impl DatabaseConnector {
    /// Requires SargonPassport tier-4 token to activate.
    pub fn new(sargon_passport_tier: u8) -> Self {
        Self { authorized: sargon_passport_tier >= 4 }
    }

    /// Synchronous KAKI lookup against the sovereign Iraqi authority database.
    ///
    /// Returns a simulated no-match in development; production wires the
    /// encrypted EnkiDB KAKI-indexed watchlist query here.
    pub fn lookup(&self, face_kaki: &KakiPK) -> AuthorityLookupResult {
        if !self.authorized {
            return AuthorityLookupResult {
                queried_kaki: *face_kaki,
                matched:      false,
                watchlist:    false,
                identity_ref: None,
                match_score:  0.0,
                db_status:    DatabaseStatus::Unauthorized,
            };
        }
        // Development stub — production: encrypted KAKI lookup against sovereign EnkiDB
        AuthorityLookupResult {
            queried_kaki: *face_kaki,
            matched:      false,
            watchlist:    false,
            identity_ref: None,
            match_score:  0.0,
            db_status:    DatabaseStatus::Online,
        }
    }
}

// ── Tests ─────────────────────────────────────────────────────────
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tier_below_4_is_unauthorized() {
        let db = DatabaseConnector::new(3);
        let result = db.lookup(&[0u8; 16]);
        assert!(!result.matched);
        assert!(matches!(result.db_status, DatabaseStatus::Unauthorized));
    }

    #[test]
    fn tier_4_is_authorized_online() {
        let db = DatabaseConnector::new(4);
        let result = db.lookup(&[0u8; 16]);
        assert!(matches!(result.db_status, DatabaseStatus::Online));
    }

    #[test]
    fn queried_kaki_echoed_in_result() {
        let kaki = [42u8; 16];
        let db   = DatabaseConnector::new(4);
        let result = db.lookup(&kaki);
        assert_eq!(result.queried_kaki, kaki);
    }

    #[test]
    fn no_match_in_simulation() {
        let db = DatabaseConnector::new(4);
        let result = db.lookup(&[1u8; 16]);
        assert!(!result.matched);
        assert!(!result.watchlist);
        assert!(result.identity_ref.is_none());
    }
}
