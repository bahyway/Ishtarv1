# orbital_degradation.gd — Drives KAKI particles through orbital states
# GOLDEN → FUZZY → DEAD with smooth transitions
# Betti number tracking per tribe
# Nisaba trigger thresholds baked in
# DUB.SAR 𒁾  2026-06-25

extends Node

signal state_changed(surrogate: int, old_state: String, new_state: String)
signal nisaba_loop(tribe_id: int, b1: int)
signal nisaba_void(tribe_id: int, b2: int)
signal tiamat_alert(level: String, tribe_id: int)

# ── Sovereign orbital states ──────────────────────────────────────────
enum OrbitalState {
    GOLDEN_GEM,
    GOLDEN_ALIVE,
    FUZZY_AGED,
    FUZZY_GRAY,
    FUZZY_DECAY,
    DEAD_EXPIRED,
    DEAD_SEALED,
}

# State colours (sovereign palette)
# CORRECTED (this land): DEAD_EXPIRED/DEAD_SEALED were a reddish/maroon
# tint (Color(0.863,0.235,0.235)/Color(0.2,0.1,0.1)) -- off the sealed
# palette every other landed file in this ecosystem uses and tests
# against (utnapishtim::COLOUR_DEAD_EXPIRED = "#404040",
# COLOUR_DEAD_SEALED = "#282828"; #800000 maroon is reserved for the
# Nergal AV engine only). Fixed to match, keeping each state's original
# alpha (fade) value.
const STATE_COLOURS = {
    OrbitalState.GOLDEN_GEM:    Color(0.831, 0.686, 0.216, 1.0),  # Gold
    OrbitalState.GOLDEN_ALIVE:  Color(0.784, 0.627, 0.157, 0.9),
    OrbitalState.FUZZY_AGED:    Color(0.114, 0.620, 0.459, 0.8),  # Teal
    OrbitalState.FUZZY_GRAY:    Color(0.502, 0.502, 0.502, 0.7),  # Gray
    OrbitalState.FUZZY_DECAY:   Color(0.937, 0.624, 0.153, 0.6),  # Amber
    OrbitalState.DEAD_EXPIRED:  Color(0.251, 0.251, 0.251, 0.5),  # #404040 dark grey
    OrbitalState.DEAD_SEALED:   Color(0.157, 0.157, 0.157, 0.3),  # #282828 near-black PUZRU
}

const STATE_NAMES = {
    OrbitalState.GOLDEN_GEM:    "GOLDEN_GEM",
    OrbitalState.GOLDEN_ALIVE:  "GOLDEN_ALIVE",
    OrbitalState.FUZZY_AGED:    "FUZZY_AGED",
    OrbitalState.FUZZY_GRAY:    "FUZZY_GRAY",
    OrbitalState.FUZZY_DECAY:   "FUZZY_DECAY",
    OrbitalState.DEAD_EXPIRED:  "DEAD_EXPIRED",
    OrbitalState.DEAD_SEALED:   "DEAD_SEALED",
}

# ── Particle tracking ─────────────────────────────────────────────────
var particles: Dictionary = {}  # surrogate → {state, tribe_id, colour}
var tribe_betti: Dictionary = {}  # tribe_id → {b0, b1, b2}

# ── Nisaba thresholds ─────────────────────────────────────────────────
const NISABA_B1_THRESHOLD: int   = 1    # any loop → trigger
const NISABA_B2_THRESHOLD: int   = 1    # any void → trigger
const NISABA_COLOUR_VARIANCE: float = 50.0

# ── Register a particle ───────────────────────────────────────────────
func register_particle(surrogate: int, tribe_id: int) -> void:
    particles[surrogate] = {
        "state":    OrbitalState.GOLDEN_ALIVE,
        "tribe_id": tribe_id,
        "colour":   STATE_COLOURS[OrbitalState.GOLDEN_ALIVE],
        "lerp_t":   0.0,
        "target_colour": STATE_COLOURS[OrbitalState.GOLDEN_ALIVE],
    }
    if not tribe_id in tribe_betti:
        tribe_betti[tribe_id] = {"b0": 1, "b1": 0, "b2": 0}

# ── Degrade a particle (PAZUZU infection) ─────────────────────────────
func degrade(surrogate: int) -> void:
    if not surrogate in particles: return
    var p = particles[surrogate]
    var old_state = p["state"]
    var new_state = old_state

    match old_state:
        OrbitalState.GOLDEN_GEM:    new_state = OrbitalState.GOLDEN_ALIVE
        OrbitalState.GOLDEN_ALIVE:  new_state = OrbitalState.FUZZY_AGED
        OrbitalState.FUZZY_AGED:    new_state = OrbitalState.FUZZY_GRAY
        OrbitalState.FUZZY_GRAY:    new_state = OrbitalState.FUZZY_DECAY
        OrbitalState.FUZZY_DECAY:   new_state = OrbitalState.DEAD_EXPIRED
        OrbitalState.DEAD_EXPIRED:  new_state = OrbitalState.DEAD_SEALED
        OrbitalState.DEAD_SEALED:   return  # terminal

    p["state"]         = new_state
    p["target_colour"] = STATE_COLOURS[new_state]
    p["lerp_t"]        = 0.0
    emit_signal("state_changed", surrogate,
        STATE_NAMES[old_state], STATE_NAMES[new_state])

    # Update Betti for tribe
    _update_betti(p["tribe_id"])

    # Check TIAMAT alert level
    _check_tiamat(p["tribe_id"])

# ── Degrade all particles for a tribe (PAZUZU-07 Ereshkigal) ─────────
func degrade_tribe(tribe_id: int) -> void:
    for surrogate in particles:
        if particles[surrogate]["tribe_id"] == tribe_id:
            degrade(surrogate)

# ── Smooth colour Lerp in _process ────────────────────────────────────
func _process(delta: float) -> void:
    for surrogate in particles:
        var p = particles[surrogate]
        if p["lerp_t"] < 1.0:
            p["lerp_t"] = min(1.0, p["lerp_t"] + delta * 2.0)
            p["colour"] = p["colour"].lerp(p["target_colour"], p["lerp_t"])

# ── Betti number update ───────────────────────────────────────────────
func _update_betti(tribe_id: int) -> void:
    var tribe_particles = []
    for s in particles:
        if particles[s]["tribe_id"] == tribe_id:
            tribe_particles.append(particles[s]["state"])

    var dead_count  = tribe_particles.filter(func(s): return s >= OrbitalState.DEAD_EXPIRED).size()
    var fuzzy_count = tribe_particles.filter(func(s): return s >= OrbitalState.FUZZY_AGED and s < OrbitalState.DEAD_EXPIRED).size()
    var total       = tribe_particles.size()

    # β₀: clusters of alive particles
    var b0 = max(1, total - dead_count)
    # β₁: loops — when fuzzy particles form cycles
    var b1 = fuzzy_count / 3
    # β₂: voids — dead zones surrounded by fuzzy
    var b2 = dead_count / 5

    tribe_betti[tribe_id] = {"b0": b0, "b1": b1, "b2": b2}

    # Nisaba triggers
    if b1 >= NISABA_B1_THRESHOLD:
        emit_signal("nisaba_loop", tribe_id, b1)
    if b2 >= NISABA_B2_THRESHOLD:
        emit_signal("nisaba_void", tribe_id, b2)

# ── TIAMAT alert level (CORRECTED: level 4 = KUR, not MAROON) ─────────
# #800000 Maroon is reserved for the Nergal AV engine only -- every other
# TIAMAT implementation landed in this ecosystem (edubba-seal::tiamat_name,
# bigring.gd, playbook 53/54/55's corrections) uses DILBAT/KAKKAB/ERRA/KUR
# as the four real levels. This file's level 4 was still "MAROON" -- the
# exact mistake those other files were explicitly corrected away from.
func _check_tiamat(tribe_id: int) -> void:
    if not tribe_id in tribe_betti: return
    var b = tribe_betti[tribe_id]
    var level = ""
    if b["b1"] >= 3:   level = "KUR"
    elif b["b1"] >= 2: level = "ERRA"
    elif b["b1"] >= 1: level = "KAKKAB"
    elif b["b0"] < 2:  level = "DILBAT"
    if level != "":
        emit_signal("tiamat_alert", level, tribe_id)

# ── Query particle colour for Theater rendering ───────────────────────
func get_colour(surrogate: int) -> Color:
    if surrogate in particles:
        return particles[surrogate]["colour"]
    return Color(0.5, 0.5, 0.5, 0.5)

func get_state(surrogate: int) -> String:
    if surrogate in particles:
        return STATE_NAMES[particles[surrogate]["state"]]
    return "UNKNOWN"
