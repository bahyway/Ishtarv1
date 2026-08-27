# theater_controller.gd — DubSar Theater orchestrator
# DUB.SAR 𒁾  2026-06-21 / corrected PB-217, 2026-07-20
#
# This is the scene that actually loads on launch (main_theater.tscn).
# It owns:
#   - CENTER_STAGE: an ambient decorative particle backdrop (not
#     data-driven -- LEFT/RIGHT_CURVED_PANEL are static background
#     meshes with no live content of their own, despite older comments
#     here once implying an egui-texture query UI lived on them
#     directly; that was never actually wired into this scene's node
#     tree, confirmed against main_theater.tscn's own [node] list)
#   - NISABA Stage 1: light cone above center stage
#   - EnkiDbWizard (scenes/wizard.tscn): the real connection wizard
#   - HeptaScriptEditor overlay (Ctrl+E): real, live-wired to EnkiDDB/
#     EnkiMDB (192.168.122.112:7102/7202)
#
# The actual "Grid & Orbit" 3D particle-visualization UI lives in the
# separate scenes/theater_3d.tscn, reached via Ctrl+G (open_grid_orbit_
# theater below) -- a full scene switch, not something nested in this
# scene's tree. That file's own header explains why (it owns its own
# Camera3D/Environment) and documents the PB-217 rewire from a never-
# deployed enkidb-query-server:7001 target to real EnkiDDB/EnkiMDB.
# There is no Rust GDExtension in this codebase driving any of this --
# an earlier version of this comment claimed a "BigRingNode3D" handled
# the query/3D-mapping; no such extension exists anywhere in this repo.

extends Node3D

const DubSarTheme = preload("res://scripts/dubsar_theme.gd")
const SessionIdentity = preload("res://scripts/session_identity.gd")

# ŠARRU (king) -- matches kupru's own IshtarLayer::architect() constant
# (crates/kupru/src/passport.rs), the real privilege_level a Gilgamesh-
# minted Architect passport carries. A Sargon-minted gardener passport
# (privilege_level 1) already gets past login.gd's mandatory gate to
# REACH Theater at all -- this is the one further check login.gd's own
# "HONEST LIMIT" comment names as real, separate future work: nothing
# in Theater reads privilege_level to gate an individual feature. The
# EnkiDB Connector Wizard is the first one to.
const ARCHITECT_PRIVILEGE_LEVEL := 7

@onready var _enkidb_wizard = $EnkiDbWizard
@onready var _heptascript_editor = $HeptaScriptEditor
@onready var _graph_explorer = $GraphExplorer
@onready var center_stage  = $CENTER_STAGE
@onready var left_panel    = $LEFT_CURVED_PANEL
@onready var right_panel   = $RIGHT_CURVED_PANEL
@onready var nisaba        = $NISABA_STAGE1

# Sovereign colour palette
const GOLD   = Color(0.831, 0.686, 0.216, 1.0)
const TEAL   = Color(0.114, 0.620, 0.459, 1.0)
const AMBER  = Color(0.937, 0.624, 0.153, 1.0)
const CRIMSON = Color(0.863, 0.235, 0.235, 1.0)

func _ready():
    _scale_for_software_rendering()
    print("𒁾 DubSar Theater initialized")
    print("  CENTER_STAGE: %d particles" % center_stage.amount)
    print("  Ctrl+E: HeptaScript editor (EnkiDDB/EnkiMDB)  |  Ctrl+G: Grid & Orbit  |  Ctrl+K: Graph Explorer  |  Ctrl+W: connection wizard  |  Ctrl+D: Dashboards (all 7 EnkiDB Types)  |  Ctrl+F: Document Explorer (EnkiDDB)  |  Ctrl+O: Onion Layers 3D Tower")
    _setup_panels()
    _setup_nisaba()
    _setup_menu()
    # 2026-07-22: the "𒆳 EnkiDB Connect" desktop icon (PB-91) sets
    # DUBSAR_LAUNCH_MODE=enkidb_connect on the Godot process before launch
    # (via PB-226's dubsar_launch_mode var) so it auto-opens the wizard
    # here -- but only after this scene loads, i.e. only after the
    # mandatory login gate (scenes/login.tscn) has already succeeded. This
    # replaces the old dubsar_proof.gd patch, which set the same env var
    # but on a script that ran *before* any login gate existed.
    if OS.get_environment("DUBSAR_LAUNCH_MODE") == "enkidb_connect":
        open_enkidb_wizard()

# 2026-07-28: eriduous-vdi has no render-capable GPU -- confirmed live
# via /dev/dri (only a display node, no render node) and vulkaninfo
# (Intel's real driver refuses with VK_ERROR_INCOMPATIBLE_DRIVER), so
# Godot falls back to Mesa's llvmpipe software rasterizer for
# everything. Switching PB-226's launch to gl_compatibility/opengl3
# fixed the "Godot" title flash's unrelated Vulkan crash risk but did
# NOT fix a real, separate cost: CENTER_STAGE's 50000-particle
# GPUParticles3D uses compute-shader-driven simulation, which is cheap
# on real hardware and very expensive for llvmpipe to emulate entirely
# in software -- confirmed live pegging 2+ CPU cores and freezing the
# VDI. This only ever reduces the count on detected software
# rendering; a machine with a real GPU still gets the full
# 50000-particle nebula exactly as designed.
func _scale_for_software_rendering() -> void:
    var adapter := RenderingServer.get_video_adapter_name().to_lower()
    if "llvmpipe" in adapter or "software" in adapter:
        var reduced := 2000
        print("  ⚠ Software renderer detected (%s) -- reducing CENTER_STAGE from %d to %d particles" % [RenderingServer.get_video_adapter_name(), center_stage.amount, reduced])
        center_stage.amount = reduced

func _setup_panels():
    # Apply sovereign dark material to side panels
    var mat = StandardMaterial3D.new()
    mat.albedo_color = Color(0.07, 0.08, 0.13, 0.92)
    mat.transparency = BaseMaterial3D.TRANSPARENCY_ALPHA
    mat.emission_enabled = true
    mat.emission = TEAL
    mat.emission_energy_multiplier = 0.3
    left_panel.set_surface_override_material(0, mat)
    right_panel.set_surface_override_material(0, mat.duplicate())

func _setup_nisaba():
    # Stage 1: Gold light cone above center stage
    var spot = SpotLight3D.new()
    spot.light_color = GOLD
    spot.light_energy = 2.0
    spot.spot_range = 8.0
    spot.spot_angle = 25.0
    spot.transform.origin = Vector3(0, 0, 0)
    nisaba.add_child(spot)
    print("  NISABA Stage 1: light cone active")

# 2026-07-28: the panel-open shortcuts below were keyboard-only, printed
# to a console output that a remote/touch viewer never sees -- a
# stakeholder logging in over eriduous-vdi's remote viewer had no way to
# discover the wizard, HeptaScript editor, Grid & Orbit, or any other
# panel existed at all; the theater rendered as two bare side panels
# with nothing clickable. This adds a real, always-visible top bar with
# one button per shortcut, wired to the exact same open_*() methods the
# keyboard shortcuts already call -- Ctrl+E/W/G/K/D/F/O keep working
# unchanged for anyone with a physical keyboard, this just stops
# discoverability from depending on having one.
func _setup_menu():
    var layer = CanvasLayer.new()
    layer.name = "TheaterMenuLayer"
    add_child(layer)

    var bar = PanelContainer.new()
    bar.name = "TheaterMenuBar"
    bar.add_theme_stylebox_override("panel", DubSarTheme.header_stylebox())
    bar.set_anchors_and_offsets_preset(Control.PRESET_TOP_WIDE)
    layer.add_child(bar)

    var hbox = HBoxContainer.new()
    hbox.add_theme_constant_override("separation", 8)
    bar.add_child(hbox)

    var title = Label.new()
    title.text = "𒁾 DubSar Theater"
    title.add_theme_color_override("font_color", DubSarTheme.ACCENT_CYAN)
    title.add_theme_font_size_override("font_size", DubSarTheme.SIZE_TITLE)
    title.custom_minimum_size = Vector2(160, 0)
    hbox.add_child(title)
    hbox.add_child(VSeparator.new())

    var buttons = [
        ["EnkiDB Connector Wizard (Ctrl+W)", open_enkidb_wizard],
        ["HeptaScript Editor (Ctrl+E)", open_heptascript_editor],
        ["Grid & Orbit (Ctrl+G)", open_grid_orbit_theater],
        ["Graph Explorer (Ctrl+K)", open_graph_explorer],
        ["Dashboards (Ctrl+D)", open_dashboard_theater],
        ["Document Explorer (Ctrl+F)", open_document_explorer],
        ["Onion Layers Tower (Ctrl+O)", open_onion_tower],
    ]
    for entry in buttons:
        var btn = Button.new()
        btn.text = entry[0]
        btn.pressed.connect(entry[1])
        hbox.add_child(btn)

func _process(delta):
    # Rotate BIGRING slowly on Y axis
    center_stage.rotation.y += delta * 0.05

# CONSOLIDATION 2026-07-10: wired to scripts/wizard.gd's actual signals
# (wizard_completed / connection_tested), not the retired
# enkidb_wizard.gd's (connection_confirmed / wizard_cancelled). The
# connection_data dict key is "database", not "engine".
func _on_enkidb_wizard_confirmed(connection_data: Dictionary) -> void:
    print("EnkiDB connected: ", connection_data.get("database", "?"), " @ ", connection_data.get("host", "?"))

func _on_enkidb_wizard_tested(success: bool, message: String) -> void:
    print("EnkiDB wizard test: ", "OK" if success else "FAILED", " — ", message)

func open_enkidb_wizard() -> void:
    if SessionIdentity.privilege_level < ARCHITECT_PRIVILEGE_LEVEL:
        _show_architect_required_dialog()
        return
    if _enkidb_wizard:
        _enkidb_wizard.open()

# Shown when a passport below Architect privilege (a Sargon-minted
# gardener, privilege_level 1) tries to open the EnkiDB Connector
# Wizard. login.gd already guarantees SessionIdentity.valid is true by
# the time this scene is even reachable, so the only real question here
# is privilege_level, not whether anyone is logged in at all.
func _show_architect_required_dialog() -> void:
    var template := "The EnkiDB Connector Wizard needs an Architect passport (privilege_level %d). " \
        + "This session is signed in as \"%s\" at privilege_level %d.\n\n" \
        + "Mint or import a Gilgamesh Master Key passport, then log in again to DubSar IDE with it."
    var dialog := AcceptDialog.new()
    dialog.title = "Architect passport required"
    dialog.dialog_text = template % [ARCHITECT_PRIVILEGE_LEVEL, SessionIdentity.identity, SessionIdentity.privilege_level]
    add_child(dialog)
    dialog.popup_centered()
    dialog.confirmed.connect(dialog.queue_free)
    dialog.canceled.connect(dialog.queue_free)

# Added 2026-07-10 alongside scenes/heptascript_editor.tscn: opens the
# HeptaScript editor (CodeEdit + CLI log + NODE-target fan-out + PDM tab).
func open_heptascript_editor() -> void:
    if _heptascript_editor:
        _heptascript_editor.open()

# PB-218/219 (2026-07-20): scenes/graph_explorer.tscn is an overlay, like
# HeptaScriptEditor -- unlike Grid & Orbit it needs no 3D camera of its
# own, so a scene switch would be needless ceremony here.
func open_graph_explorer() -> void:
    if _graph_explorer:
        _graph_explorer.open()

func _unhandled_input(event: InputEvent) -> void:
    # Keyboard shortcuts -- kept alongside the on-screen menu bar
    # (_setup_menu) for anyone with a physical keyboard; the menu bar
    # is what makes these discoverable without one.
    if event is InputEventKey and event.pressed and not event.echo:
        if event.keycode == KEY_E and event.ctrl_pressed:
            open_heptascript_editor()
        elif event.keycode == KEY_W and event.ctrl_pressed:
            open_enkidb_wizard()
        elif event.keycode == KEY_G and event.ctrl_pressed:
            open_grid_orbit_theater()
        elif event.keycode == KEY_K and event.ctrl_pressed:
            open_graph_explorer()
        elif event.keycode == KEY_D and event.ctrl_pressed:
            open_dashboard_theater()
        elif event.keycode == KEY_F and event.ctrl_pressed:
            open_document_explorer()
        elif event.keycode == KEY_O and event.ctrl_pressed:
            open_onion_tower()

# PB-217 (2026-07-20): scenes/theater_3d.tscn is a full scene switch, not
# an overlay -- it owns its own Camera3D/Environment, which would
# conflict with this scene's already-active camera if nested as a child
# instead. Escape inside theater_3d.gd switches back to this scene.
func open_grid_orbit_theater() -> void:
    get_tree().change_scene_to_file("res://scenes/theater_3d.tscn")

# 2026-07-21: the New DubSar Theater's tabbed 2D dashboard shell
# (scenes/dashboard_theater.tscn) -- all 7 EnkiDB Types, each in its own
# tab, per the Architect's explicit instruction that it live side by
# side with this 3D theater as an alternative mode, not replace it.
# Full scene switch, same reasoning as Grid & Orbit above (its own
# top-level Control tree, no 3D camera to conflict with here, but still
# cleaner as its own scene than nested inside this one's node tree).
# Escape inside dashboard_theater.gd switches back to this scene.
func open_dashboard_theater() -> void:
    get_tree().change_scene_to_file("res://scenes/dashboard_theater.tscn")

# 2026-07-21: the Document Explorer (scenes/document_explorer.tscn) --
# tree sidebar + content + real Related Links, built against EnkiDDB
# specifically since it's the one real document-shaped EnkiDB Type. See
# that scene's own script header for the two things it honestly omits
# from the Architect's reference mockup (a per-document "KAKI vector"
# and a 7D orbit mini-widget) rather than faking either with placeholder
# numbers. Full scene switch, same reasoning as the other Ctrl-shortcuts
# above; Escape returns here.
func open_document_explorer() -> void:
    get_tree().change_scene_to_file("res://scenes/document_explorer.tscn")

# 2026-07-21: Onion Layers 3D Tower (scenes/onion_tower_3d.tscn) -- the same
# real 25-layer/163-crate dataset as the sealed 2D artifact
# (docs/09_observatory/BAHYWAY_ONION_LAYERS.html), rendered as a stacked, interactive 3D
# tower instead of nested flat circles. Owns its own Camera3D/Environment,
# so it's a full scene switch like Grid & Orbit, not an overlay. Escape
# inside onion_tower_3d.gd switches back to this scene.
func open_onion_tower() -> void:
    get_tree().change_scene_to_file("res://scenes/onion_tower_3d.tscn")
