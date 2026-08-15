//! Complete catalog of BahyWay compliance requirements.
//!
//! IDs are namespaced by domain:
//!   1000–1999  GDPR / Privacy
//!   2000–2999  Encryption & Key Management
//!   3000–3999  Network & Runtime Security
//!   4000–4999  Operational Security
//!   5000–5999  NajafEngine-specific
//!   6000–6999  WPD Infrastructure
//!   7000–7999  Governance & Assurance

// ── ComplianceDomain ──────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ComplianceDomain {
    /// GDPR Articles 9, 17, 25, 30, 35 — special category PII
    GdprPrivacy,
    /// Encryption at rest — volumes, keys, PII fields
    EncryptionAtRest,
    /// Key management — rotation, HSM, passphrase protection
    KeyManagement,
    /// Network and HTTP boundary security
    NetworkSecurity,
    /// Operational controls — IR plan, monitoring, training
    OperationalSecurity,
    /// NajafEngine-specific — burial records, Arabic names, family lineage
    NajafPrivacy,
    /// WPD infrastructure — water pipeline data, NIS2, sensor integrity
    WpdInfrastructure,
    /// Governance and independent assurance
    GovernanceAssurance,
}

impl ComplianceDomain {
    pub fn as_str(self) -> &'static str {
        match self {
            ComplianceDomain::GdprPrivacy          => "GDPR / Privacy",
            ComplianceDomain::EncryptionAtRest     => "Encryption at Rest",
            ComplianceDomain::KeyManagement        => "Key Management",
            ComplianceDomain::NetworkSecurity      => "Network Security",
            ComplianceDomain::OperationalSecurity  => "Operational Security",
            ComplianceDomain::NajafPrivacy         => "NajafEngine Privacy",
            ComplianceDomain::WpdInfrastructure    => "WPD Infrastructure",
            ComplianceDomain::GovernanceAssurance  => "Governance & Assurance",
        }
    }

    /// Icon code for HTML rendering (emoji-safe single char).
    pub fn icon(self) -> &'static str {
        match self {
            ComplianceDomain::GdprPrivacy          => "&#x1F512;",  // lock
            ComplianceDomain::EncryptionAtRest     => "&#x1F510;",  // locked with key
            ComplianceDomain::KeyManagement        => "&#x1F511;",  // key
            ComplianceDomain::NetworkSecurity      => "&#x1F6E1;",  // shield
            ComplianceDomain::OperationalSecurity  => "&#x26A0;",   // warning
            ComplianceDomain::NajafPrivacy         => "&#x1F54C;",  // mosque (closest available)
            ComplianceDomain::WpdInfrastructure    => "&#x1F6B0;",  // water pipe
            ComplianceDomain::GovernanceAssurance  => "&#x2705;",   // checkmark
        }
    }

    /// Weight of this domain in the overall composite score (0–240 scale).
    /// Mandatory domains (GDPR, Encryption) have higher weight.
    pub fn weight(self) -> u8 {
        match self {
            ComplianceDomain::GdprPrivacy          => 240, // blocking — must pass
            ComplianceDomain::EncryptionAtRest     => 220,
            ComplianceDomain::KeyManagement        => 200,
            ComplianceDomain::NetworkSecurity      => 200,
            ComplianceDomain::OperationalSecurity  => 160,
            ComplianceDomain::NajafPrivacy         => 240, // blocking for NajafEngine
            ComplianceDomain::WpdInfrastructure    => 180,
            ComplianceDomain::GovernanceAssurance  => 140,
        }
    }
}

// ── SlaRequirement ────────────────────────────────────────────────────────────

/// A single measurable compliance requirement.
#[derive(Debug, Clone, Copy)]
pub struct SlaRequirement {
    /// Unique requirement ID within the BahyWay catalog.
    pub id:          u16,
    pub domain:      ComplianceDomain,
    pub name:        &'static str,
    pub description: &'static str,
    /// Weight within the domain (0–240).  Higher = more impact on domain score.
    pub weight:      u8,
    /// If `true`, scoring below `MANDATORY_THRESHOLD` blocks go-live entirely.
    pub mandatory:   bool,
    /// Associated WAYv4.0 `.akk` policy file, if any.
    pub akk_policy:  Option<&'static str>,
    /// Recommended action if this requirement is not implemented.
    pub gap_action:  &'static str,
}

// ── Complete Requirements Catalog ─────────────────────────────────────────────

pub static ALL_REQUIREMENTS: &[SlaRequirement] = &[

    // ── GDPR / Privacy (1000–1999) ────────────────────────────────────────────

    SlaRequirement {
        id: 1001, domain: ComplianceDomain::GdprPrivacy,
        name: "DPIA completed (Article 35)",
        description: "Data Protection Impact Assessment for special-category PII processing. Legally mandatory for large-scale processing of ethnic/religious data.",
        weight: 240, mandatory: true,
        akk_policy: None,
        gap_action: "Engage a qualified DPO; complete DPIA using standard template; file with supervisory authority.",
    },
    SlaRequirement {
        id: 1002, domain: ComplianceDomain::GdprPrivacy,
        name: "Lawful basis documented (Article 6 / 9)",
        description: "Written documentation of the Article 9(2) lawful basis for processing ethnic/religious affiliation data.",
        weight: 220, mandatory: true,
        akk_policy: None,
        gap_action: "Document explicit consent or public-interest basis; include in RoPA under Article 30.",
    },
    SlaRequirement {
        id: 1003, domain: ComplianceDomain::GdprPrivacy,
        name: "Record of Processing Activities (Article 30)",
        description: "RoPA documenting all processing operations, purposes, and retention periods for NajafEngine and WPD data.",
        weight: 200, mandatory: true,
        akk_policy: None,
        gap_action: "Create RoPA document covering all data categories, processors, and cross-border transfers.",
    },
    SlaRequirement {
        id: 1004, domain: ComplianceDomain::GdprPrivacy,
        name: "Right to erasure mechanism (Article 17)",
        description: "Technical mechanism to fulfil erasure requests within 30 days. Implemented via PiiVault encrypted-field approach.",
        weight: 240, mandatory: true,
        akk_policy: None,
        gap_action: "Deploy pii-vault crate; PiiSidecar::erase(kaki_hash) must be callable from the admin API.",
    },
    SlaRequirement {
        id: 1005, domain: ComplianceDomain::GdprPrivacy,
        name: "Data access audit trail",
        description: "Append-only log of every read access to PII fields: (kaki_hash, particle_id, accessor_kaki, epoch).",
        weight: 200, mandatory: true,
        akk_policy: None,
        gap_action: "Add read-access logging to GraveRegistry query methods; write to .enkwal sidecar log.",
    },
    SlaRequirement {
        id: 1006, domain: ComplianceDomain::GdprPrivacy,
        name: "Data subject rights procedure",
        description: "Documented procedure for handling access, portability, rectification, and erasure requests within statutory timeframes.",
        weight: 160, mandatory: false,
        akk_policy: None,
        gap_action: "Create data subject rights form and internal procedure; assign a named DPO or equivalent.",
    },
    SlaRequirement {
        id: 1007, domain: ComplianceDomain::GdprPrivacy,
        name: "Privacy notice published",
        description: "Article 13/14 compliant privacy notice for users of the HeptaScript website.",
        weight: 180, mandatory: false,
        akk_policy: None,
        gap_action: "Draft and publish privacy notice at /privacy on heptascript.nl before go-live.",
    },

    // ── Encryption at Rest (2000–2099) ────────────────────────────────────────

    SlaRequirement {
        id: 2001, domain: ComplianceDomain::EncryptionAtRest,
        name: "PII fields encrypted (pii-vault)",
        description: "Arabic names, death years, and family lineage encrypted via PiiVault (ChaCha20-Poly1305) before storage.",
        weight: 240, mandatory: true,
        akk_policy: None,
        gap_action: "Deploy pii-vault crate; migrate existing GraveParticle PII fields to PiiSidecar.",
    },
    SlaRequirement {
        id: 2002, domain: ComplianceDomain::EncryptionAtRest,
        name: "EnkiDB volume encrypted (LUKS)",
        description: "Full-disk or volume-level encryption for the Podman volumes containing EnkiDB data directories.",
        weight: 220, mandatory: true,
        akk_policy: None,
        gap_action: "Enable LUKS2 encryption on the host volume mount; key managed via TPM or operator passphrase.",
    },
    SlaRequirement {
        id: 2003, domain: ComplianceDomain::EncryptionAtRest,
        name: "SealKeyPair encrypted at rest",
        description: "Write Pod Ed25519 signing key stored encrypted (AkkadianCipher + SargonKdf passphrase), not in plaintext.",
        weight: 220, mandatory: true,
        akk_policy: None,
        gap_action: "Wrap SealKeyPair serialization in AkkadianCipher; require operator passphrase on startup.",
    },
    SlaRequirement {
        id: 2004, domain: ComplianceDomain::EncryptionAtRest,
        name: "PiiVault master key encrypted at rest",
        description: "The PiiVault master key itself must be stored encrypted, not as a plaintext file.",
        weight: 200, mandatory: true,
        akk_policy: None,
        gap_action: "Derive master key via SargonKdf from operator passphrase; never write raw key to disk.",
    },

    // ── Key Management (2100–2199) ─────────────────────────────────────────────

    SlaRequirement {
        id: 2101, domain: ComplianceDomain::KeyManagement,
        name: "Key rotation procedure defined",
        description: "Documented procedure for rotating the Write Pod SealKeyPair without restarting the replication chain.",
        weight: 180, mandatory: false,
        akk_policy: Some("policies/enkidb_replication_protocol.akk"),
        gap_action: "Implement KeyRotationEvent in ReplEventKind; define operator procedure for key rotation.",
    },
    SlaRequirement {
        id: 2102, domain: ComplianceDomain::KeyManagement,
        name: "PiiVault erased-hash persistence",
        description: "The set of erased KAKI hashes must survive vault restarts — persisted to durable storage.",
        weight: 220, mandatory: true,
        akk_policy: None,
        gap_action: "Persist PiiVault::erased_hashes() to an append-only erasure log on each erase() call.",
    },
    SlaRequirement {
        id: 2103, domain: ComplianceDomain::KeyManagement,
        name: "HSM or TPM for production keys",
        description: "Production SealKeyPair and PiiVault master key held in hardware security module or TPM.",
        weight: 160, mandatory: false,
        akk_policy: None,
        gap_action: "P3 roadmap — evaluate PKCS#11 HSM or TPM2 integration for production hardware.",
    },
    SlaRequirement {
        id: 2104, domain: ComplianceDomain::KeyManagement,
        name: "Post-quantum signing (Dilithium-3)",
        description: "algorithm_id=0x02 path in SealKeyPair and SovereignVerifier using Dilithium-3.",
        weight: 120, mandatory: false,
        akk_policy: None,
        gap_action: "P3 roadmap — implement algorithm_id=0x02 when pqcrypto-dilithium reaches stability.",
    },

    // ── Network Security (3000–3099) ──────────────────────────────────────────

    SlaRequirement {
        id: 3001, domain: ComplianceDomain::NetworkSecurity,
        name: "hepta-sec-web adapter deployed",
        description: "WebSentinelGuard integrated into bahyway-server request dispatch. X-Kaki-Hash enforced on all routes.",
        weight: 240, mandatory: true,
        akk_policy: Some("policies/heptasec_web_access.akk"),
        gap_action: "Integrate hepta-sec-web::WebSentinelGuard in bin/bahyway-server outermost handler.",
    },
    SlaRequirement {
        id: 3002, domain: ComplianceDomain::NetworkSecurity,
        name: "Rate limiting active (RateTracker)",
        description: "PolicyCondition::RateExceeds evaluated via evaluate_with_rate() for all web routes.",
        weight: 200, mandatory: true,
        akk_policy: Some("policies/heptasec_web_access.akk"),
        gap_action: "Wire RateTracker into WebSentinelGuard; configure per-route thresholds from SLA profile.",
    },
    SlaRequirement {
        id: 3003, domain: ComplianceDomain::NetworkSecurity,
        name: "HTTPS / TLS 1.3 enforced",
        description: "All traffic to heptascript.nl served over TLS 1.3. HTTP redirected to HTTPS.",
        weight: 220, mandatory: true,
        akk_policy: None,
        gap_action: "Configure nginx TLS; obtain certificate via Let's Encrypt; enforce HSTS header.",
    },
    SlaRequirement {
        id: 3004, domain: ComplianceDomain::NetworkSecurity,
        name: "Broker network isolation (--network none)",
        description: "ReplicationBroker runs in Podman with --network none — no outbound TCP exists.",
        weight: 200, mandatory: true,
        akk_policy: Some("policies/enkidb_replication_protocol.akk"),
        gap_action: "Verify Podman compose file includes network: none for the broker service.",
    },
    SlaRequirement {
        id: 3005, domain: ComplianceDomain::NetworkSecurity,
        name: "Read/Write Pod CQRS isolation",
        description: "Read Pod and Write Pod are separate Podman containers with no TCP between them. Shared only via .enkwal volume.",
        weight: 220, mandatory: true,
        akk_policy: Some("policies/enkidb_replication_protocol.akk"),
        gap_action: "Verify Podman compose volume mounts: write.enkwal :ro for Broker, :rw for Write Pod only.",
    },

    // ── Operational Security (4000–4099) ──────────────────────────────────────

    SlaRequirement {
        id: 4001, domain: ComplianceDomain::OperationalSecurity,
        name: "Incident Response Plan (IRP)",
        description: "Written IRP with escalation paths, contact list, and defined RTO/RPO for a signing-key compromise scenario.",
        weight: 200, mandatory: true,
        akk_policy: None,
        gap_action: "Draft IRP; include tabletop exercise scenario: SealKeyPair compromised — what do you do in 4 hours?",
    },
    SlaRequirement {
        id: 4002, domain: ComplianceDomain::OperationalSecurity,
        name: "Independent penetration test",
        description: "CREST-certified external penetration test of the web gateway, ENKWAL pipeline, and Podman network isolation.",
        weight: 200, mandatory: true,
        akk_policy: None,
        gap_action: "Engage a CREST-certified tester; provide architecture docs from docs/09_security/.",
    },
    SlaRequirement {
        id: 4003, domain: ComplianceDomain::OperationalSecurity,
        name: "SecurityEvent persistent audit log",
        description: "SecurityEvent ring buffer replaced with append-only .enkwal log so evidence is never destroyed under load.",
        weight: 180, mandatory: false,
        akk_policy: None,
        gap_action: "Implement HeptaSecSentinel persistent event log (P1 in security evaluation roadmap).",
    },
    SlaRequirement {
        id: 4004, domain: ComplianceDomain::OperationalSecurity,
        name: "Vulnerability disclosure policy",
        description: "Published security.txt and responsible disclosure policy at heptascript.nl/security.txt.",
        weight: 140, mandatory: false,
        akk_policy: None,
        gap_action: "Create security.txt per RFC 9116; define disclosure timeline and contact.",
    },
    SlaRequirement {
        id: 4005, domain: ComplianceDomain::OperationalSecurity,
        name: "Staff security training",
        description: "All staff with access to EnkiDB or NajafEngine data complete annual security and GDPR awareness training.",
        weight: 160, mandatory: false,
        akk_policy: None,
        gap_action: "Enroll staff in a GDPR awareness course; document completion; include in ISO 27001 roadmap.",
    },

    // ── NajafEngine Privacy (5000–5099) ───────────────────────────────────────

    SlaRequirement {
        id: 5001, domain: ComplianceDomain::NajafPrivacy,
        name: "GraveParticle PII migrated to PiiSidecar",
        description: "owner_name_arabic, death_hijri_year, and tribe fields stored exclusively in PiiSidecar — not in the main particle log.",
        weight: 240, mandatory: true,
        akk_policy: None,
        gap_action: "Migrate GraveParticle PII to PiiSidecar; remove plaintext fields from GraveParticle struct.",
    },
    SlaRequirement {
        id: 5002, domain: ComplianceDomain::NajafPrivacy,
        name: "ATTR_ACCESS_LEVEL enforced at query time",
        description: "GraveRegistry queries filter results based on the requesting KAKI's access level (0–3).",
        weight: 200, mandatory: true,
        akk_policy: None,
        gap_action: "Add access-level check in GraveRegistry::search(); return Err(AccessDenied) for insufficient level.",
    },
    SlaRequirement {
        id: 5003, domain: ComplianceDomain::NajafPrivacy,
        name: "Islamic data sovereignty respected",
        description: "Family lineage and tribe data shared only with living relatives who can demonstrate kinship (verified KAKI).",
        weight: 200, mandatory: false,
        akk_policy: None,
        gap_action: "Define kinship-verification protocol; document in DPIA; add kin-check gate to lineage queries.",
    },
    SlaRequirement {
        id: 5004, domain: ComplianceDomain::NajafPrivacy,
        name: "Data minimisation (Article 5(1)(c))",
        description: "Only PII fields strictly necessary for pilgrimage navigation are stored. No speculative data collection.",
        weight: 160, mandatory: false,
        akk_policy: None,
        gap_action: "Review all GraveParticle fields; remove any fields not required for the stated purpose.",
    },

    // ── WPD Infrastructure (6000–6099) ────────────────────────────────────────

    SlaRequirement {
        id: 6001, domain: ComplianceDomain::WpdInfrastructure,
        name: "NIS2 incident reporting (24h)",
        description: "Procedure for reporting significant water-infrastructure incidents to the competent authority within 24 hours.",
        weight: 200, mandatory: false,
        akk_policy: Some("policies/eridu_packet_ingress.akk"),
        gap_action: "Define incident classification criteria; draft 24-hour notification template; test with tabletop exercise.",
    },
    SlaRequirement {
        id: 6002, domain: ComplianceDomain::WpdInfrastructure,
        name: "WPD sensor data integrity — ENKWAL protected",
        description: "WPD sensor readings replicated via ENKWAL pipeline. Injecting false readings requires breaking all 7 verification layers.",
        weight: 220, mandatory: true,
        akk_policy: Some("policies/enkidb_replication_protocol.akk"),
        gap_action: "Confirm WpdEngine uses ReplicationEmitter for all sensor write operations.",
    },
    SlaRequirement {
        id: 6003, domain: ComplianceDomain::WpdInfrastructure,
        name: "WPD Write Pod key compromise threat model",
        description: "Documented threat model for what happens if the Write Pod signing key is compromised and an adversary injects false WPD sensor events.",
        weight: 180, mandatory: false,
        akk_policy: None,
        gap_action: "Add WPD-specific threat scenario to the STRIDE threat model document.",
    },

    // ── Governance & Assurance (7000–7099) ────────────────────────────────────

    SlaRequirement {
        id: 7001, domain: ComplianceDomain::GovernanceAssurance,
        name: "Formal threat model (STRIDE)",
        description: "STRIDE threat model document covering NajafEngine, WPD, and the HeptaScript website.",
        weight: 180, mandatory: false,
        akk_policy: None,
        gap_action: "Create STRIDE threat model using docs/09_security/ as input; review with external security architect.",
    },
    SlaRequirement {
        id: 7002, domain: ComplianceDomain::GovernanceAssurance,
        name: "ISO 27001 gap analysis",
        description: "Gap analysis against ISO 27001:2022 Annex A controls as foundation for future certification.",
        weight: 160, mandatory: false,
        akk_policy: None,
        gap_action: "P2 roadmap — commission a gap analysis from a qualified auditor within 90 days of go-live.",
    },
    SlaRequirement {
        id: 7003, domain: ComplianceDomain::GovernanceAssurance,
        name: "SOC 2 Type II readiness",
        description: "Evidence collection process in place for Trust Service Criteria (Security, Availability, Confidentiality).",
        weight: 140, mandatory: false,
        akk_policy: None,
        gap_action: "P3 roadmap — engage a SOC 2 auditor after ISO 27001 gap analysis is complete.",
    },
    SlaRequirement {
        id: 7004, domain: ComplianceDomain::GovernanceAssurance,
        name: "Business Continuity Plan",
        description: "BCP covering EnkiDB data loss, Podman host failure, and NajafEngine unavailability scenarios.",
        weight: 160, mandatory: false,
        akk_policy: None,
        gap_action: "Define RPO/RTO for each EnkiDB instance; document backup/restore procedure; test quarterly.",
    },
];

/// Look up a requirement by its numeric ID.
pub fn requirement_by_id(id: u16) -> Option<&'static SlaRequirement> {
    ALL_REQUIREMENTS.iter().find(|r| r.id == id)
}

/// All requirements for a specific domain.
pub fn requirements_for_domain(domain: ComplianceDomain) -> impl Iterator<Item = &'static SlaRequirement> {
    ALL_REQUIREMENTS.iter().filter(move |r| r.domain == domain)
}
