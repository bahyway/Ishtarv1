//! commands.rs — TamuzAI quick commands (!adr, !crate, !next, !law, !kaki).
//!
//! Quick commands bypass the LLM for instant sovereign knowledge lookups.
//! They use the built-in BahyWay knowledge base — no model inference needed.
#![forbid(unsafe_code)]

/// A parsed quick command.
#[derive(Debug, Clone, PartialEq)]
pub enum QuickCommand {
    Adr(u32),        // !adr 3 → explain ADR-003
    Crate(String),   // !crate enkidb-kaki → explain crate
    Next,            // !next → what to build next
    Law(u32),        // !law 1 → explain sovereign law 1
    Kaki,            // !kaki → full KAKI byte layout
    Enkiddb(String), // !enkiddb <phrase> → real SEARCH: against EnkiDDB's Read Node
    Help,            // !help → all commands
    Status,          // !status → model status
    History,         // !history → session history
    Clear,           // !clear → clear conversation
    Unknown(String), // unknown command
}

/// Parse a user message for quick commands (starts with !).
pub fn parse_quick_command(msg: &str) -> Option<QuickCommand> {
    let msg = msg.trim();
    if !msg.starts_with('!') {
        return None;
    }

    let parts: Vec<&str> = msg[1..].splitn(2, ' ').collect();
    let cmd = parts[0].to_lowercase();
    let arg = parts.get(1).map(|s| s.trim());

    Some(match cmd.as_str() {
        "adr" => {
            let n = arg.and_then(|s| s.parse::<u32>().ok()).unwrap_or(0);
            QuickCommand::Adr(n)
        }
        "crate" => QuickCommand::Crate(arg.unwrap_or("").to_string()),
        "next" => QuickCommand::Next,
        "law" => {
            let n = arg.and_then(|s| s.parse::<u32>().ok()).unwrap_or(0);
            QuickCommand::Law(n)
        }
        "kaki" => QuickCommand::Kaki,
        "enkiddb" => QuickCommand::Enkiddb(arg.unwrap_or("").to_string()),
        "help" => QuickCommand::Help,
        "status" => QuickCommand::Status,
        "history" => QuickCommand::History,
        "clear" => QuickCommand::Clear,
        other => QuickCommand::Unknown(other.to_string()),
    })
}

/// Handle a quick command and return the response text.
pub fn handle_quick_command(cmd: &QuickCommand) -> String {
    match cmd {
        QuickCommand::Kaki => KAKI_RESPONSE.to_string(),
        // Real network I/O (crate::enkiddb_bridge::search) doesn't belong
        // in this pure, no-I/O function -- ChatEngine::chat() intercepts
        // QuickCommand::Enkiddb before it ever reaches here, the same way
        // it already special-cases Clear. This arm only exists so the
        // match stays exhaustive if handle_quick_command is ever called
        // directly (e.g. from a test) without going through chat().
        QuickCommand::Enkiddb(_) => "!enkiddb must be run through ChatEngine::chat(), which performs the real EnkiDDB network call.".to_string(),
        QuickCommand::Help => HELP_RESPONSE.to_string(),
        QuickCommand::Next => NEXT_RESPONSE.to_string(),
        QuickCommand::Adr(n) => adr_response(*n),
        QuickCommand::Law(n) => law_response(*n),
        QuickCommand::Crate(name) => crate_response(name),
        QuickCommand::Status   => "Use the model status indicator in the panel header.".to_string(),
        QuickCommand::History  => "Conversation history is shown in the panel above.".to_string(),
        QuickCommand::Clear    => "🧹 Conversation cleared.".to_string(),
        QuickCommand::Unknown(s) => format!("Unknown command: !{s}\nType !help for available commands."),
    }
}

const KAKI_RESPONSE: &str = "\
𒁾 KAKI v4.0 — The 16-Byte Sovereign Identity
═══════════════════════════════════════════

κ[0..3]   uuid_hash    32-bit  FNV-1a of content at birth
κ[4..5]   tribe_id     16-bit  Tribe membership (partition key)
κ[6]      kaki_type     8-bit  0x01=Identity 0x02=Event 0x03=CrossTribe
κ[7]      kaki_role     8-bit  0x01=KISHIB   0x02=ZIKRU  0x03=PARZU
κ[8..11]  seq_counter  32-bit  Per-tribe-per-epoch sequence (ADR-003)
κ[12..13] timestamp    16-bit  Birth epoch (immutable angular position)
κ[14..15] checksum     16-bit  CRC-16/CCITT tamper seal

⚠  ColorID (RGB) = MANDATORY EAV attribute — NOT in KAKI bytes
   RED=Domain · GREEN=Quality · BLUE=Freshness

Sovereign laws:
  §2.4 — no assessments in KAKI nucleus
  §8.3 — CrossTribe state computed on PROBE, never stored";

const HELP_RESPONSE: &str = "\
𒀭 TamuzAI Quick Commands
═════════════════════════
!kaki          — KAKI 16-byte layout (full spec)
!adr  <N>      — Explain ADR-N (e.g. !adr 3)
!law  <N>      — Explain sovereign law N (e.g. !law 1)
!crate <name>  — Explain a crate (e.g. !crate enkidb-kaki)
!enkiddb <phrase> — Real semantic search against a live EnkiDDB Read Node
!next          — What to build next (roadmap)
!status        — Model loading status
!history       — Show conversation summary
!clear         — Clear conversation history
!help          — Show this help

For general questions, just type naturally:
  'What does enkidb-kaki do?'
  'Show me the H(P) equation'
  'Write a KakiMinter usage example'";

const NEXT_RESPONSE: &str = "\
𒁾 Next Build Steps — BahyWay v4.0 Roadmap
══════════════════════════════════════════

Phase 1 — MetaEngine (NOW):
  ✅ enkidullm-model   — GGUF runner (v4.1.0)
  ✅ enkidullm-memory  — Conversation journal (v4.1.0)
  ✅ enkidullm-chat    — TamuzAI panel (current)
  🔴 enkidullm-codex  — BahyWay knowledge base
  🔴 Download model   — DeepSeek-Coder-6.7B-Q4_K_M.gguf

Phase 2 — Domain Data:
  🔴 NajafEngine real data import
  🔴 WPDEngine Baghdad pipe data
  🔴 EnkiStream replication live

Phase 3 — Partner Demo:
  🔴 1M particle 3D Vulkan visualization
  🔴 DubSar IDE TamuzAI panel live demo

Open ADRs blocking progress:
  ADR-011 → EnkiduLLM model (DeepSeek-Coder-6.7B ✓ chosen)
  ADR-012 → WGSL compute shader GPU target
  ADR-013 → EnkiStream replication protocol";

fn adr_response(n: u32) -> String {
    match n {
        1  => "ADR-001 — No External Dependencies\nNo PostgreSQL, Redis, SQLite, Kafka, or any external DB. Pure Rust only. QUALITY_DIVISOR=240 (Plimpton 322, not 255).".to_string(),
        2  => "ADR-002 — Cost Model\nProof-aware optimizer: total = io×1 + cpu×0.5 + proof×3 + memory×1.5. ProofCtxTime carries proof=6.0 so proof term dominates.".to_string(),
        3  => "ADR-003 — KAKI seq_counter\nκ[8..11] = seq_counter (u32, per-tribe-per-epoch). Solves birthday paradox: deterministic uniqueness, free creation ordering, gap detection. Resets to 0x00000001 each epoch.".to_string(),
        4  => "ADR-004 — GEM Rate Target\nGEM_RATE_TARGET = 35.4% — eternal sovereign constant. At least 35.4% of all particles must be in GEM state (B11 ≥ 200) for a healthy tribe.".to_string(),
        6  => "ADR-006 — No DELETE\nAll data is append-only forever. No record is ever modified or deleted. UPDATE is replaced by INSERT of a superseding Event-Kaki.".to_string(),
        8  => "ADR-008 — OOO Foundation / Forbidden Operations\n17 forbidden operations at engine level. CrossTribe effective state computed at query time, never stored (§8.3). No assessments in KAKI nucleus (§2.4).".to_string(),
        9  => "ADR-009 — Algebra Additions\nBirthday paradox analysis prompted seq_counter decision. Confirms PA-12 through PA-16 theorems. Particles Algebra v1.0 declared as new branch of mathematics.".to_string(),
        11 => "ADR-011 — EnkiduLLM Model Selection\nStatus: RESOLVED (v4.1.0)\nChosen: DeepSeek-Coder-6.7B-Instruct-Q4_K_M.gguf\nRationale: fits GTX 1650 Max-Q 4GB VRAM, excellent Rust understanding, sovereign offline.".to_string(),
        _  => format!("ADR-{n:03} is not in the quick-reference database.\nCheck docs/14_decisions_adr/ in the repository for the full text."),
    }
}

fn law_response(n: u32) -> String {
    match n {
        1  => "§1 — No external database\nNo PostgreSQL, Redis, SQLite, RocksDB, Kafka, or any external storage substrate. EnkiDB is the only storage layer.".to_string(),
        2  => "§2 — No UPDATE or DELETE\nAll data is append-only forever. Axiom 8 of Particles Algebra. State evolution = new Event-Kaki inserted. Old state never deleted.".to_string(),
        3  => "§3 — No assessments in KAKI nucleus (§2.4)\nThe 16 KAKI bytes carry only structural facts: identity, tribe, type, role, sequence, epoch, checksum. Never quality scores, lanes, or color.".to_string(),
        4  => "§4 — CrossTribe state on PROBE only (§8.3)\nCrosstribe-Kaki effective state (Gold/Orange/Gray) computed at query time by IDU Probing Rule. Never stored in any field.".to_string(),
        5  => "§5 — B11 = round(H(P) × 240)\nThe only valid B11 derivation. QUALITY_DIVISOR=240 (Plimpton 322). Never manually assigned. B11 comes from the H(P) equation only.".to_string(),
        6  => "§6 — Every autonomous decision = KAKI-stamped .akk file\nTransparency by architecture. Every automated system decision is expressed as a human-readable, VaultWay-persisted, KAKI-stamped .akk file.".to_string(),
        7  => "§7 — No unwrap() in production code\nAll errors handled via Result<T, E>. Panics are forbidden in production paths. Use ? operator or explicit error handling.".to_string(),
        8  => "§8 — No logic in main.rs\nmain.rs only wires together crates. All business logic lives in library crates. main.rs is a sovereign entry point, not a logic container.".to_string(),
        9  => "§9 — ColorID (RGB) lives in EAV\nRED=Domain, GREEN=Quality, BLUE=Freshness. These are MANDATORY EAV attributes in the Orbit. They are NOT part of the immutable KAKI bytes.".to_string(),
        10 => "§10 — The -Way suffix is reserved\nDeprecated from all component names in v4.0. Reserved exclusively for WAYv2.0 security policy language .way files (Sargon Seal, Gilgamesh Seal, ABAC policies).".to_string(),
        _  => format!("Sovereign Law §{n} is not in the quick-reference database.\nThere are 10 sovereign laws (§1-§10)."),
    }
}

fn crate_response(name: &str) -> String {
    let name = name.trim().to_lowercase();
    match name.as_str() {
        "enkidb-kaki" => "enkidb-kaki — Layer 1 Identity Foundation\nThe KakiMinter — sole authority for minting 16-byte sovereign KAKI identities.\nKey types: Kaki, KakiMinter, KakiType, KakiRole, IdentityKaki, EventKaki, CrossTribeKaki\nADR-003: seq_counter at κ[8..11] for deterministic uniqueness.".to_string(),
        "bahyway-core" => "bahyway-core — Layer 0 Mathematical Foundation\nSovereign constants, TribeId, BahywayError, Result type.\nContains: QUALITY_DIVISOR=240, GEM_B11=200, TAU_R7=11, RESONANCE_RADIUS=0.15\nAll other crates depend on this.".to_string(),
        "bahyway-algebra" => "bahyway-algebra — Layer 0 Mathematics\nImplements TOP Algebra (Ṭupšarrūtu), Enlil Algebra (JNF), Particles Algebra.\nThe mathematical substrate for the entire ecosystem.\nTrait hierarchy: Algebra, Morphism, Homomorphism.".to_string(),
        "hepta-score" => "hepta-score — H(P) Quality Equation\nH(P) = 1/(1+√Σᵢwᵢ(Pᵢ−Tᵢ)²)\nB11 = round(H(P) × 240)\nThe canonical quality scoring engine. Weights: D1=0.30, D2=0.20, D3=0.15, D4=0.15, D5=0.10, D6=0.05, D7=0.05".to_string(),
        "story-engine" => "story-engine — CQRS Event Projection\nProjects current particle state from the append-only event journal.\nStoryEngine::project(identity) → ProjectedState\nSnapshot-accelerated replay for large event histories.".to_string(),
        "enkidb-journal" => "enkidb-journal — Append-Only WAL\nThe sovereign write-ahead log. Every particle event is appended here.\nSupports: append, replay, read_particle_history, snapshot acceleration.\nNo DELETE, no UPDATE — ever.".to_string(),
        "vgca-engine" => "vgca-engine — Vector Geometric Cleansing Analysis\nThree algorithms: VGCA-Σ (field geometry), VGCA-Δ (block trajectory), VGCA-Λ (columnar).\nPure mathematical geometry — no ML, no training data.\nBLAKE3(FSV)[0..7] → KAKI bytes B0-B6.".to_string(),
        "heptascript" => "heptascript — W5H2 Physics Query Language\nW5H2 clauses: WHO, WHAT, WHERE, WHEN, WHY, HOW, HOW_MUCH\nNOT SQL — based on Triple-O (Orbits-Oriented Ontology)\nExample: WHO Citizens.E\\nWHERE E[state] = \"Golden\"\\nWHY LANE = gold".to_string(),
        "aaol" | "akkadianool" => "aaol — AkkadianAOL Sovereign Language\nNOT a DSL — full sovereign orchestration language.\nCompiles .akk → Rust, Python, JSON, PowerShell, XML simultaneously.\nAKKA actor model + cuneiform keywords + code generator.".to_string(),
        "enkidullm-model" => "enkidullm-model — TamuzAI GGUF Runner (v4.1.0)\nSovereign pure-Rust GGUF model runner — no PyTorch, no candle, no llama-cpp-sys.\nSupports: Q4_K_M, Q4_0, Q8_0, F16, F32 quantization.\nTarget: DeepSeek-Coder-6.7B-Instruct-Q4_K_M.gguf".to_string(),
        "enkidullm-memory" => "enkidullm-memory — TamuzAI Conversation Journal (v4.1.0)\nEvery conversation turn is a KAKI-stamped particle.\nSessions, turns, memory store, TF-IDF search.\nAll conversations persist as sovereign particles.".to_string(),
        _ => format!("Crate '{name}' is not in the quick-reference database.\nTry: enkidb-kaki, bahyway-core, hepta-score, story-engine, vgca-engine, heptascript, aaol, enkidullm-model, enkidullm-memory"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_kaki_command() {
        assert_eq!(parse_quick_command("!kaki"), Some(QuickCommand::Kaki));
    }

    #[test]
    fn parse_adr_command() {
        assert_eq!(parse_quick_command("!adr 3"), Some(QuickCommand::Adr(3)));
    }

    #[test]
    fn parse_law_command() {
        assert_eq!(parse_quick_command("!law 9"), Some(QuickCommand::Law(9)));
    }

    #[test]
    fn parse_crate_command() {
        assert_eq!(
            parse_quick_command("!crate enkidb-kaki"),
            Some(QuickCommand::Crate("enkidb-kaki".to_string()))
        );
    }

    #[test]
    fn parse_enkiddb_command() {
        assert_eq!(
            parse_quick_command("!enkiddb HeptaScript ingestion"),
            Some(QuickCommand::Enkiddb("HeptaScript ingestion".to_string()))
        );
    }

    #[test]
    fn parse_non_command_returns_none() {
        assert_eq!(parse_quick_command("What is KAKI?"), None);
    }

    #[test]
    fn handle_kaki_command_contains_bytes() {
        let resp = handle_quick_command(&QuickCommand::Kaki);
        assert!(resp.contains("uuid_hash"));
        assert!(resp.contains("seq_counter"));
        assert!(resp.contains("checksum"));
    }

    #[test]
    fn handle_adr3_contains_seq_counter() {
        let resp = handle_quick_command(&QuickCommand::Adr(3));
        assert!(resp.contains("seq_counter"));
        assert!(resp.contains("birthday paradox"));
    }

    #[test]
    fn handle_law9_contains_colorid() {
        let resp = handle_quick_command(&QuickCommand::Law(9));
        assert!(resp.contains("ColorID") || resp.contains("EAV"));
    }

    #[test]
    fn handle_crate_enkidb_kaki() {
        let resp = handle_quick_command(&QuickCommand::Crate("enkidb-kaki".to_string()));
        assert!(resp.contains("KakiMinter"));
    }

    #[test]
    fn handle_unknown_command() {
        let resp = handle_quick_command(&QuickCommand::Unknown("xyz".to_string()));
        assert!(resp.contains("Unknown command"));
    }

    #[test]
    fn help_contains_all_commands() {
        let resp = handle_quick_command(&QuickCommand::Help);
        assert!(resp.contains("!kaki"));
        assert!(resp.contains("!adr"));
        assert!(resp.contains("!law"));
        assert!(resp.contains("!crate"));
        assert!(resp.contains("!next"));
    }
}
