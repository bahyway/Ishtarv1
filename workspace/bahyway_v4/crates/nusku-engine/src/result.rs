use crate::types::KakiPK;
use crate::particle::BodyScan;

/// Final pipeline output — sovereign scan result.
#[derive(Debug, Clone)]
pub struct ScanResult {
    pub scan:            BodyScan,
    pub threat_score:    f32,
    pub medical_score:   f32,
    pub face_match:      Option<FaceMatchResult>,
    pub primary_finding: PrimaryFinding,
    pub confidence:      f32,
    pub processing_ms:   u32,
    pub timestamp_ms:    u64,
}

#[derive(Debug, Clone)]
pub enum PrimaryFinding {
    Clear,
    SecurityThreat  { pattern: String, confidence: f32 },
    MedicalCondition{ condition: String, confidence: f32 },
    AmbiguousReview { reason: String },
}

#[derive(Debug, Clone)]
pub struct FaceMatchResult {
    pub kaki_face:   KakiPK,
    pub matched:     bool,
    pub identity:    Option<String>,  // encrypted authority reference
    pub match_score: f32,
    pub watchlist:   bool,
}
