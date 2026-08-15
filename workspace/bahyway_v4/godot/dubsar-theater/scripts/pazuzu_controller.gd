# pazuzu_controller.gd — Wires PazuzuAgent + OrbitalDegradation + Theater UI
# DUB.SAR 𒁾  2026-06-25
extends CanvasLayer

@onready var agent       = $PazuzuAgent
@onready var degradation = $OrbitalDegradation
@onready var run_btn     = $Panel/VBox/BtnRow/RunAllBtn
@onready var close_btn   = $Panel/VBox/BtnRow/CloseBtn
@onready var current_lbl = $Panel/VBox/ContentRow/StatusPanel/CurrentTest
@onready var betti_lbl   = $Panel/VBox/ContentRow/StatusPanel/BettiDisplay
@onready var tiamat_lbl  = $Panel/VBox/ContentRow/StatusPanel/TiamtAlert
@onready var nisaba_lbl  = $Panel/VBox/ContentRow/StatusPanel/NisabaStatus
@onready var results_box = $Panel/VBox/ResultsScroll/ResultsContainer
@onready var test_list   = $Panel/VBox/ContentRow/TestList

const C_GOLD    = Color(0.784, 0.627, 0.157, 1)
const C_RED     = Color(0.863, 0.235, 0.235, 1)
const C_GREEN   = Color(0.200, 0.700, 0.300, 1)
const C_AMBER   = Color(0.937, 0.624, 0.153, 1)
const C_PURPLE  = Color(0.612, 0.502, 0.910, 1)
const C_DIM     = Color(0.392, 0.431, 0.510, 1)

# Register 10 demo particles per tribe for degradation
var DEMO_TRIBE_ID = 1

func _ready() -> void:
    # Connect signals
    agent.test_started.connect(_on_test_started)
    agent.test_penetrated.connect(_on_test_penetrated)
    agent.test_series_complete.connect(_on_series_complete)
    agent.nisaba_trigger.connect(_on_nisaba_trigger)
    degradation.tiamat_alert.connect(_on_tiamat_alert)
    degradation.nisaba_loop.connect(_on_nisaba_loop)

    run_btn.pressed.connect(_on_run_all)
    close_btn.pressed.connect(func(): visible = false)

    # Register demo particles
    for i in range(20):
        degradation.register_particle(i, DEMO_TRIBE_ID)

    # Build test list UI
    _build_test_list()

func _build_test_list() -> void:
    for test in agent.TESTS:
        var row = HBoxContainer.new()
        var status = Label.new()
        status.text = "○"
        status.add_theme_color_override("font_color", C_DIM)
        status.custom_minimum_size = Vector2(20, 0)
        status.name = "Status_" + test["id"].replace("-","_")
        row.add_child(status)
        var lbl = Label.new()
        lbl.text = test["id"] + "  " + test["name"]
        lbl.add_theme_font_size_override("font_size", 10)
        lbl.add_theme_color_override("font_color", C_DIM)
        row.add_child(lbl)
        test_list.add_child(row)

func _on_run_all() -> void:
    run_btn.disabled = true
    for c in results_box.get_children(): c.queue_free()
    agent.run_all()

func _on_test_started(test_name: String, description: String) -> void:
    current_lbl.text = "▶ " + test_name
    current_lbl.add_theme_color_override("font_color", C_AMBER)
    # Degrade one particle per test
    degradation.degrade(agent.current_test)

func _on_test_penetrated(
    test_name: String, gap_confirmed: bool, orbit_effect: String
) -> void:
    var row = VBoxContainer.new()
    row.add_theme_constant_override("separation", 2)
    var name_lbl = Label.new()
    name_lbl.text = ("✗ " if gap_confirmed else "✓ ") + test_name
    name_lbl.add_theme_font_size_override("font_size", 11)
    name_lbl.add_theme_color_override("font_color", C_RED if gap_confirmed else C_GREEN)
    var effect_lbl = Label.new()
    effect_lbl.text = "  → " + orbit_effect
    effect_lbl.add_theme_font_size_override("font_size", 10)
    effect_lbl.add_theme_color_override("font_color", C_DIM)
    row.add_child(name_lbl)
    row.add_child(effect_lbl)
    results_box.add_child(row)

    # Update Betti display
    var b = degradation.tribe_betti.get(DEMO_TRIBE_ID, {"b0":0,"b1":0,"b2":0})
    betti_lbl.text = "β₀: %d  |  β₁: %d  |  β₂: %d" % [b["b0"], b["b1"], b["b2"]]

func _on_series_complete(results: Array) -> void:
    run_btn.disabled = false
    current_lbl.text = "PAZUZU series complete — %d gaps confirmed" % results.size()
    current_lbl.add_theme_color_override("font_color", C_RED)

func _on_nisaba_trigger(b1: int, tribe_id: int) -> void:
    nisaba_lbl.text = "𒀭𒉀 Nisaba triggered — β₁=%d — generating ṬUPŠARRU art..." % b1
    nisaba_lbl.add_theme_color_override("font_color", C_PURPLE)

func _on_tiamat_alert(level: String, tribe_id: int) -> void:
    # CORRECTED (this land): matched "MAROON" here, but
    # orbital_degradation.gd's _check_tiamat() emits "KUR" for its most
    # severe level (fixed in the previous land for the same reason --
    # #800000 maroon is reserved for the Nergal AV engine, never TIAMAT).
    # As uploaded, this case could never match its own emitter and the
    # most severe alert silently fell through to the "healthy" green
    # default below.
    var col = Color(0.200, 0.700, 0.300, 1)
    match level:
        "DILBAT": col = Color(0.937, 0.624, 0.153, 1)
        "KAKKAB": col = Color(0.937, 0.500, 0.100, 1)
        "ERRA":   col = Color(0.863, 0.235, 0.235, 1)
        "KUR":    col = Color(0.550, 0.000, 0.130, 1)
    tiamat_lbl.text = "TIAMAT: " + level + " — tribe %d" % tribe_id
    tiamat_lbl.add_theme_color_override("font_color", col)

func _on_nisaba_loop(tribe_id: int, b1: int) -> void:
    nisaba_lbl.text = "𒀭𒉀 β₁=%d LOOP — ṬUPŠARRU memorial plate generating..." % b1
    nisaba_lbl.add_theme_color_override("font_color", C_PURPLE)
