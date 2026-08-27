#![forbid(unsafe_code)]
//! enkidb-session-registry — Sovereign sync session registry.
//!
//! Parses `enkidb-sessions.toml` line-by-line with no external crates.

// ── Types ──────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NodeType {
    EnkiDB,
    EnkiDW,
    EnkiSDB,
    EnkiODB,
    EnkiQDB,
    EnkiMDB,
    EnkiDDB,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SessionRole {
    Write,
    Read,
}

#[derive(Debug, Clone)]
pub struct SessionEntry {
    pub id: String,
    pub host: String,
    pub port: u16,
    pub role: SessionRole,
    pub node_type: NodeType,
    pub tribe_ids: Vec<u32>,
    pub label: String,
    pub enabled: bool,
}

// ── Error ──────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub enum RegistryError {
    ParseError(String),
    InvalidRole(String),
    InvalidNodeType(String),
    InvalidPort(String),
}

impl core::fmt::Display for RegistryError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::ParseError(s) => write!(f, "session registry parse error: {s}"),
            Self::InvalidRole(s) => write!(f, "invalid session role: {s}"),
            Self::InvalidNodeType(s) => write!(f, "invalid node type: {s}"),
            Self::InvalidPort(s) => write!(f, "invalid port: {s}"),
        }
    }
}

// ── Registry ───────────────────────────────────────────────────────────────────

#[derive(Debug)]
pub struct SessionRegistry {
    sessions: Vec<SessionEntry>,
}

impl SessionRegistry {
    /// Parse a TOML string in the enkidb-sessions.toml format.
    /// Uses a line-by-line native parser — no serde, no external crates.
    pub fn from_toml(src: &str) -> Result<Self, RegistryError> {
        let mut sessions: Vec<SessionEntry> = Vec::new();

        // Working builder fields
        let mut id = String::new();
        let mut host = String::new();
        let mut port: u16 = 0;
        let mut role = SessionRole::Read;
        let mut node_type = NodeType::EnkiDB;
        let mut tribe_ids: Vec<u32> = Vec::new();
        let mut label = String::new();
        let mut enabled = true;
        let mut in_session = false;

        for (lineno, raw_line) in src.lines().enumerate() {
            let line = raw_line.trim();

            // Skip blanks and comments
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            // Section header
            if line == "[[session]]" {
                if in_session {
                    sessions.push(SessionEntry {
                        id: id.clone(),
                        host: host.clone(),
                        port,
                        role: role.clone(),
                        node_type: node_type.clone(),
                        tribe_ids: tribe_ids.clone(),
                        label: label.clone(),
                        enabled,
                    });
                }
                // Reset builder
                id = String::new();
                host = String::new();
                port = 0;
                role = SessionRole::Read;
                node_type = NodeType::EnkiDB;
                tribe_ids = Vec::new();
                label = String::new();
                enabled = true;
                in_session = true;
                continue;
            }

            if !in_session {
                // Top-level keys before any [[session]] — ignore
                continue;
            }

            // Parse key = value
            let Some(eq_pos) = line.find('=') else {
                return Err(RegistryError::ParseError(format!(
                    "line {}: no '=' found: {line}",
                    lineno + 1
                )));
            };
            let key = line[..eq_pos].trim();
            let val_raw = line[eq_pos + 1..].trim();

            match key {
                "id" => id = parse_string_val(val_raw),
                "host" => host = parse_string_val(val_raw),
                "label" => label = parse_string_val(val_raw),
                "enabled" => enabled = val_raw == "true",
                "port" => {
                    port = val_raw
                        .parse::<u16>()
                        .map_err(|_| RegistryError::InvalidPort(val_raw.to_string()))?;
                }
                "role" => {
                    let s = parse_string_val(val_raw);
                    role = match s.to_uppercase().as_str() {
                        "WRITE" => SessionRole::Write,
                        "READ" => SessionRole::Read,
                        _ => return Err(RegistryError::InvalidRole(s)),
                    };
                }
                "node_type" => {
                    let s = parse_string_val(val_raw);
                    node_type = match s.as_str() {
                        "EnkiDB" => NodeType::EnkiDB,
                        "EnkiDW" => NodeType::EnkiDW,
                        "EnkiSDB" => NodeType::EnkiSDB,
                        "EnkiODB" => NodeType::EnkiODB,
                        "EnkiQDB" => NodeType::EnkiQDB,
                        "EnkiMDB" => NodeType::EnkiMDB,
                        "EnkiDDB" => NodeType::EnkiDDB,
                        _ => return Err(RegistryError::InvalidNodeType(s)),
                    };
                }
                "tribe_ids" => {
                    tribe_ids = parse_u32_array(val_raw)
                        .map_err(|e| RegistryError::ParseError(format!("tribe_ids: {e}")))?;
                }
                _ => { /* Unknown key — ignore for forward compat */ }
            }
        }

        // Flush last session
        if in_session {
            sessions.push(SessionEntry {
                id,
                host,
                port,
                role,
                node_type,
                tribe_ids,
                label,
                enabled,
            });
        }

        Ok(SessionRegistry { sessions })
    }

    pub fn sessions(&self) -> &[SessionEntry] {
        &self.sessions
    }

    pub fn enabled(&self) -> impl Iterator<Item = &SessionEntry> {
        self.sessions.iter().filter(|s| s.enabled)
    }

    pub fn by_role(&self, role: SessionRole) -> Vec<&SessionEntry> {
        self.sessions.iter().filter(|s| s.role == role).collect()
    }

    /// Returns sessions that serve the given tribe_id.
    /// A session with empty tribe_ids serves all tribes.
    pub fn by_tribe(&self, tribe_id: u32) -> Vec<&SessionEntry> {
        self.sessions
            .iter()
            .filter(|s| s.tribe_ids.is_empty() || s.tribe_ids.contains(&tribe_id))
            .collect()
    }

    pub fn by_id(&self, id: &str) -> Option<&SessionEntry> {
        self.sessions.iter().find(|s| s.id == id)
    }
}

// ── Parsing helpers ────────────────────────────────────────────────────────────

/// Remove surrounding double-quotes from a TOML string value.
fn parse_string_val(s: &str) -> String {
    let s = s.trim();
    if s.starts_with('"') && s.ends_with('"') && s.len() >= 2 {
        s[1..s.len() - 1].to_string()
    } else {
        s.to_string()
    }
}

/// Parse a TOML inline array of u32: `[]` or `[1, 2, 3]`.
fn parse_u32_array(s: &str) -> Result<Vec<u32>, String> {
    let s = s.trim();
    if !s.starts_with('[') || !s.ends_with(']') {
        return Err(format!("expected array, got: {s}"));
    }
    let inner = s[1..s.len() - 1].trim();
    if inner.is_empty() {
        return Ok(Vec::new());
    }
    inner
        .split(',')
        .map(|tok| {
            tok.trim()
                .parse::<u32>()
                .map_err(|_| format!("not a u32: '{}'", tok.trim()))
        })
        .collect()
}

// ── Tests ──────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_TOML: &str = r#"
[[session]]
id = "enkidb-write"
host = "192.168.122.101"
port = 7001
role = "WRITE"
node_type = "EnkiDB"
tribe_ids = []
label = "EnkiDB Primary Write"
enabled = true

[[session]]
id = "enkidb-read"
host = "192.168.122.102"
port = 7002
role = "READ"
node_type = "EnkiDB"
tribe_ids = [1, 2, 3]
label = "EnkiDB Read Replica"
enabled = true

[[session]]
id = "enkidw-write"
host = "192.168.122.103"
port = 7003
role = "WRITE"
node_type = "EnkiDW"
tribe_ids = [10, 20]
label = "EnkiDW Write"
enabled = false
"#;

    #[test]
    fn parse_valid_toml_three_sessions() {
        let reg = SessionRegistry::from_toml(SAMPLE_TOML).expect("parse failed");
        assert_eq!(reg.sessions().len(), 3);
        assert_eq!(reg.sessions()[0].id, "enkidb-write");
        assert_eq!(reg.sessions()[0].port, 7001);
        assert_eq!(reg.sessions()[0].role, SessionRole::Write);
        assert_eq!(reg.sessions()[0].node_type, NodeType::EnkiDB);
        assert!(reg.sessions()[0].tribe_ids.is_empty());
        assert_eq!(reg.sessions()[0].label, "EnkiDB Primary Write");
        assert!(reg.sessions()[0].enabled);
    }

    #[test]
    fn tribe_filtering_empty_means_all_tribes() {
        let reg = SessionRegistry::from_toml(SAMPLE_TOML).expect("parse failed");
        // Session 0 has empty tribe_ids → serves all tribes
        // Session 1 has [1,2,3]
        // Session 2 has [10,20]
        let tribe1 = reg.by_tribe(1);
        // Should include session 0 (empty = all) and session 1 (contains 1)
        assert_eq!(tribe1.len(), 2);

        let tribe10 = reg.by_tribe(10);
        // session 0 (all) + session 2 (contains 10)
        assert_eq!(tribe10.len(), 2);

        let tribe99 = reg.by_tribe(99);
        // only session 0 (all tribes)
        assert_eq!(tribe99.len(), 1);
        assert_eq!(tribe99[0].id, "enkidb-write");
    }

    #[test]
    fn role_filtering() {
        let reg = SessionRegistry::from_toml(SAMPLE_TOML).expect("parse failed");
        let writes = reg.by_role(SessionRole::Write);
        assert_eq!(writes.len(), 2);
        let reads = reg.by_role(SessionRole::Read);
        assert_eq!(reads.len(), 1);
        assert_eq!(reads[0].id, "enkidb-read");
    }

    #[test]
    fn enabled_filter() {
        let reg = SessionRegistry::from_toml(SAMPLE_TOML).expect("parse failed");
        let enabled: Vec<_> = reg.enabled().collect();
        assert_eq!(enabled.len(), 2);
    }

    #[test]
    fn invalid_role_returns_error() {
        let bad = r#"
[[session]]
id = "bad"
host = "127.0.0.1"
port = 1234
role = "SUPERUSER"
node_type = "EnkiDB"
tribe_ids = []
label = "bad"
enabled = true
"#;
        let err = SessionRegistry::from_toml(bad).expect_err("should fail");
        assert!(matches!(err, RegistryError::InvalidRole(_)));
    }

    #[test]
    fn by_id_lookup() {
        let reg = SessionRegistry::from_toml(SAMPLE_TOML).expect("parse failed");
        let entry = reg.by_id("enkidw-write").expect("should find");
        assert_eq!(entry.node_type, NodeType::EnkiDW);
        assert!(!entry.enabled);
        assert!(reg.by_id("nonexistent").is_none());
    }
}
