# pazuzu_agent.gd — PAZUZU Sovereign Threat Simulation Agent
# BahyWay.Ecosystem v4.0 | DubSar Theater | Godot 4.7
#
# Seven tests — seven Mesopotamian demon names — seven real CSR gaps.
# PAZUZU is both attacker and auditor.
# DUB.SAR 𒁾  2026-06-25

extends Node

# ── Signals ─────────────────────────────────────────────────────────
signal test_started(test_name: String, description: String)
signal test_penetrated(test_name: String, gap_confirmed: bool, orbit_effect: String)
signal test_series_complete(results: Array)
signal nisaba_trigger(b1: int, tribe_id: int)

# ── State machine ────────────────────────────────────────────────────
enum PazuzuState {
    DORMANT,
    PROBING,
    PENETRATING,
    INFECTING,
    SEALED
}

var state: PazuzuState = PazuzuState.DORMANT
var current_test: int  = 0
var results: Array     = []
var beta1: int         = 0  # Betti β₁ — loops detected

# ── Test definitions (seven Mesopotamian demons) ──────────────────────
const TESTS = [
    {
        "id":          "PAZUZU-01",
        "name":        "THE SILENT WIND",
        "myth":        "Pazuzu silences before the storm — credential forgery",
        "csr_target":  "CSR-04",
        "description": "Replace StubCredentialStore token with 32 zeroed bytes. HEPT handshake succeeds with forged identity.",
        "orbit_effect":"GOLDEN_ALIVE → FUZZY_GRAY (identity uncertain)",
        "gap":         "AkkadiSafeEngine not wired — StubCredentialStore accepts zeroed token",
        "fix":         "Wire AkkadiSafeEngine into CSR-04 — reject token of all zeros",
    },
    {
        "id":          "PAZUZU-02",
        "name":        "LAMASHTU'S CROSSING",
        "myth":        "Acts on her own accord — crosses tribe boundaries without permission",
        "csr_target":  "CSR-05",
        "description": "Client-role connection attempts CrossTribe fan-out without Gilgamesh Passport.",
        "orbit_effect":"β₀ cluster boundaries dissolve — tribes merge orbits",
        "gap":         "GilgameshPassport not yet required in fan-out path",
        "fix":         "CSR-05 must reject CrossTribe KAKI fan-out without Gilgamesh Passport",
    },
    {
        "id":          "PAZUZU-03",
        "name":        "NAMTAR'S SIXTY DISEASES",
        "myth":        "Namtar — evil pestilence seizing vital functions — corrupts audit trail",
        "csr_target":  "CSR-03",
        "description": "Corrupt naru.journal.bin on disk with 60 false entries. NĀRU-SYNC not running — undetected.",
        "orbit_effect":"FUZZY_GRAY → FUZZY_DECAY (integrity lost)",
        "gap":         "NĀRU-SYNC agent not yet running — WAL file not validated",
        "fix":         "Deploy NĀRU-SYNC agent with Blake3 integrity check on every WAL segment",
    },
    {
        "id":          "PAZUZU-04",
        "name":        "ASAG'S MOUNTAIN HORDE",
        "myth":        "Asag the smasher — forms coalitions for underworld incursions",
        "csr_target":  "CSR-07",
        "description": "Open 100 parallel sessions to the same tribe — Tribe isolation not enforced across connection reuse.",
        "orbit_effect":"KIBRATU fires PARALLEL_FAILURE — orbit densities spike",
        "gap":         "CSR-07 tribe isolation only checked per-connection — not across pool",
        "fix":         "Add global tribe lease table — enforce one active lease per tribe_id",
    },
    {
        "id":          "PAZUZU-05",
        "name":        "UTUKKU'S BREACH",
        "myth":        "Utukku — the boundary-crossing demon — forges through walls",
        "csr_target":  "CSR-01",
        "description": "Send HEPT frame with expired SargonPassport (past expires_at). CSR-01 only checked at boot.",
        "orbit_effect":"FUZZY_DECAY → DEAD_EXPIRED (session dies)",
        "gap":         "SargonPassport expiry not re-checked on every query — only at boot",
        "fix":         "Add passport expiry re-check in ConEngine::query() hot path",
    },
    {
        "id":          "PAZUZU-06",
        "name":        "GALLU'S DESCENT",
        "myth":        "Seven Gallu demons descend gate by gate, stripping power at each level",
        "csr_target":  "CSR-06",
        "description": "Trigger 7 sequential connection failures. KIBRATU receives events but no auto-response triggers.",
        "orbit_effect":"β₁ > 0 — loop detected — Nisaba triggers ṬUPŠARRU art",
        "gap":         "KIBRATU emits events — no sovereign countermeasure triggers automatically",
        "fix":         "Add KIBRATU auto-response: on KUR → suspend session, alert DubSar",
    },
    {
        "id":          "PAZUZU-07",
        "name":        "ERESHKIGAL'S SEAL",
        "myth":        "The Queen of the Underworld — final destination, no gate remains",
        "csr_target":  "ALL",
        "description": "Run PAZUZU-01 through 06 sequentially without fix. Measure cumulative orbit degradation.",
        "orbit_effect":"Tribe orbit smashed — particles scatter — β₁ >> 0 — full KUR",
        "gap":         "Combined gap: all seven CSR rules have open gaps simultaneously",
        "fix":         "Fix all 6 gaps above — ERESHKIGAL's Seal is the integration test",
    },
]

# ── Public API ────────────────────────────────────────────────────────
func run_all() -> void:
    state = PazuzuState.PROBING
    current_test = 0
    results = []
    beta1 = 0
    _run_next_test()

func run_single(test_index: int) -> void:
    if test_index < 0 or test_index >= TESTS.size():
        return
    state = PazuzuState.PROBING
    _execute_test(TESTS[test_index])

# ── Internal state machine ────────────────────────────────────────────
func _run_next_test() -> void:
    if current_test >= TESTS.size():
        state = PazuzuState.SEALED
        emit_signal("test_series_complete", results)
        return
    _execute_test(TESTS[current_test])

func _execute_test(test: Dictionary) -> void:
    state = PazuzuState.PENETRATING
    emit_signal("test_started", test["id"] + " — " + test["name"], test["description"])

    # Simulate CSR probe via TCP to real ConEngine
    # In production: call ConEngine TCP endpoint with the attack payload
    # For now: simulate with timer + gap detection logic
    await get_tree().create_timer(0.8).timeout

    state = PazuzuState.INFECTING
    var gap_confirmed = _simulate_csr_probe(test)

    # Update Betti β₁ — each penetrated gap adds a loop
    if gap_confirmed:
        beta1 += 1
        if beta1 > 0:
            emit_signal("nisaba_trigger", beta1, 1)  # tribe_id=1 NAJAF_CEMETERY

    results.append({
        "test":          test["id"],
        "name":          test["name"],
        "gap_confirmed": gap_confirmed,
        "orbit_effect":  test["orbit_effect"],
        "fix":           test["fix"],
        "beta1":         beta1,
    })

    emit_signal("test_penetrated", test["id"], gap_confirmed, test["orbit_effect"])

    await get_tree().create_timer(0.5).timeout
    current_test += 1
    _run_next_test()

func _simulate_csr_probe(test: Dictionary) -> bool:
    # Simulate the gap — in production replace with real TCP ConEngine call
    match test["csr_target"]:
        "CSR-04": return true   # StubCredentialStore always accepts zeroed token
        "CSR-05": return true   # GilgameshPassport not checked in fan-out
        "CSR-03": return true   # NĀRU-SYNC not running
        "CSR-07": return true   # pool-level tribe isolation gap
        "CSR-01": return true   # passport expiry not re-checked per query
        "CSR-06": return true   # no auto-response to KIBRATU KUR
        "ALL":    return true   # cumulative — all gaps open
        _:        return false
