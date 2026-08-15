//! Supporting types for AkkValue — geographic, temporal, identity, policy.

// ── Geographic ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub struct AkkCoordinate {
    pub lat: f64,
    pub lon: f64,
    pub alt: Option<f64>,
}

impl AkkCoordinate {
    pub fn new(lat: f64, lon: f64) -> Self { Self { lat, lon, alt: None } }
    pub fn with_alt(lat: f64, lon: f64, alt: f64) -> Self { Self { lat, lon, alt: Some(alt) } }
}

// ── Temporal ─────────────────────────────────────────────────────────────────

/// Gregorian date (ISO 8601).
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AkkDate {
    pub year:  i32,
    pub month: u8,   // 1–12
    pub day:   u8,   // 1–31
}

impl AkkDate {
    pub fn new(year: i32, month: u8, day: u8) -> Self { Self { year, month, day } }
}

impl core::fmt::Display for AkkDate {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:04}-{:02}-{:02}", self.year, self.month, self.day)
    }
}

/// Hijri (Islamic) date.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AkkHijriDate {
    pub year:  u16,
    pub month: u8,   // 1–12
    pub day:   u8,   // 1–30
}

impl AkkHijriDate {
    pub fn new(year: u16, month: u8, day: u8) -> Self { Self { year, month, day } }
}

impl core::fmt::Display for AkkHijriDate {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:04}H-{:02}-{:02}", self.year, self.month, self.day)
    }
}

// ── Identity ──────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AkkNationalId {
    pub country_code: String,   // ISO 3166-1 alpha-2
    pub number:       String,
}

impl AkkNationalId {
    pub fn new(country_code: impl Into<String>, number: impl Into<String>) -> Self {
        Self { country_code: country_code.into(), number: number.into() }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AkkPhone {
    pub country_code: u16,
    pub number:       String,
}

impl AkkPhone {
    pub fn new(country_code: u16, number: impl Into<String>) -> Self {
        Self { country_code, number: number.into() }
    }
}

// ── Linguistic ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AkkNameVector {
    pub parts: Vec<(AkkNamePartLabel, String)>,
}

impl AkkNameVector {
    pub fn new() -> Self { Self { parts: Vec::new() } }
    pub fn push(&mut self, label: AkkNamePartLabel, part: impl Into<String>) {
        self.parts.push((label, part.into()));
    }
}

impl Default for AkkNameVector { fn default() -> Self { Self::new() } }

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AkkNamePartLabel {
    Given,
    Family,
    Father,
    Grandfather,
    Tribe,
    Honorific,
    Kunyah,
    Laqab,
}

// ── Policy ────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PolicyVerdict {
    Allow,
    Deny,
    Escalate,
    Redact,
    Audit,
}

impl core::fmt::Display for PolicyVerdict {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Allow    => write!(f, "ALLOW"),
            Self::Deny     => write!(f, "DENY"),
            Self::Escalate => write!(f, "ESCALATE"),
            Self::Redact   => write!(f, "REDACT"),
            Self::Audit    => write!(f, "AUDIT"),
        }
    }
}

// ── Crypto / Pipeline ────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CipherAlgorithm {
    ChaCha20Poly1305,
    Aes256Gcm,
    Ed25519,
    Dilithium3,
}

impl core::fmt::Display for CipherAlgorithm {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::ChaCha20Poly1305 => write!(f, "ChaCha20-Poly1305"),
            Self::Aes256Gcm        => write!(f, "AES-256-GCM"),
            Self::Ed25519          => write!(f, "Ed25519"),
            Self::Dilithium3       => write!(f, "Dilithium-3"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PipelineStatus {
    Pending,
    Running,
    Completed,
    Failed(String),
    Cancelled,
}

impl core::fmt::Display for PipelineStatus {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Pending          => write!(f, "PENDING"),
            Self::Running          => write!(f, "RUNNING"),
            Self::Completed        => write!(f, "COMPLETED"),
            Self::Failed(reason)   => write!(f, "FAILED: {reason}"),
            Self::Cancelled        => write!(f, "CANCELLED"),
        }
    }
}
