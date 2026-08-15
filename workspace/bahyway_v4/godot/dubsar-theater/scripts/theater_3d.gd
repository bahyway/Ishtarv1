# theater_3d.gd — DubSar IDE v4.0 — Sovereign 3D Theater
# ═════════════════════════════════════════════════════════
#
# Layout (top view):
#
#         LEFT_PANEL (-35°)    CENTER     RIGHT_PANEL (+35°)
#              /                  |                \
#         HeptaScript          BIGRING           ORBIT 3D
#         Query Editor         Galaxy            Results
#         + GRID table         Particles         + Stats
#
# All UI panels are SubViewports projected as textures
# onto physical 3D QuadMesh planes in world space.
#
# PB-217 (2026-07-20): rewired from `enkidb-query-server:7001` (a real,
# buildable crate that was never actually deployed to any real VM in
# this session's infrastructure -- confirmed via `pgrep`-gated
# playbook_111's own header and the absence of any port-7001 task
# anywhere in playbook_212 onward) to the two databases that ARE really
# deployed: EnkiDDB (Tigris, 192.168.122.107:7102) and EnkiMDB
# (Euphrates, 192.168.122.107:7202). The old target's demo data
# ("NAJAF_CEMETERY", person.name_arabic/death.date/grave.section, a
# B11 GEM/ORBIT/FUZZY quality-tier scheme) belonged to a separate,
# unrelated particle store -- none of it exists in EnkiDDB/EnkiMDB's
# real schema, so every data-shaped piece of this file changes, not
# just the host/port. See docs/ARCHITECT_DESIGN_ENKIDDB_ENKIMDB_PODMAN.md
# for the real EAV schemas this now reflects.
#
# Also fixed: the wire protocol itself differs. enkidb-query-server
# streams multiple JSON-array frames terminated by a 0-length sentinel
# frame; EnkiDDB/EnkiMDB's read-servers (confirmed directly against
# bin/enkiddb-read-server/src/main.rs::handle()) answer exactly ONE
# request with exactly ONE response frame, then close the connection --
# no sentinel is ever sent. The old read loop would have worked by
# accident (falling through to its own READ_TIMEOUT, up to 120s, every
# single query) rather than returning the instant the real answer
# arrived. Also: the request itself must be prefixed `QUERY:` --
# enkidb-query-server took raw W5H2 text with no prefix; EnkiDDB/EnkiMDB
# use a small prefixed command set (QUERY:/SEARCH:/FLUSH) and reject
# anything else with `ERR:unrecognized request`.
#
# 2026-07-21: EnkiDDB/EnkiMDB's own read-servers moved off JSON to the
# same binary wire EnkiDB's already used, which made this file's old
# per-target "wire" flag (defaulting anything unmarked to a JSON decode)
# a live regression -- confirmed neither TARGETS entry below carried
# "wire":"binary" explicitly. Removed the flag and the JSON decode path
# entirely: every real Read Node is binary-only now, no exceptions.
# Decoding itself was also de-duplicated into SumuUkinClient.
# decode_binary_response() (scripts/sumuukin_client.gd, built first here
# via EnkiDbTCP_decode_binary and now shared) so there is one decoder,
# not two drifting copies.
#
# Reached via Ctrl+G from the main theater (see theater_controller.gd) --
# this is a full scene switch, not an overlay, since this scene owns its
# own Camera3D/Environment and nesting it inside main_theater.tscn's
# already-active camera would conflict. Escape returns to main_theater.
#
# DUB.SAR 𒁾  2026-06-21 / PB-217 2026-07-20

extends Node3D

const SumuUkinClient = preload("res://scripts/sumuukin_client.gd")

# ── Sovereign colours ──────────────────────────────────────────────
const C_GOLD    = Color(0.831, 0.686, 0.216)
const C_TEAL    = Color(0.114, 0.620, 0.459)
const C_AMBER   = Color(0.937, 0.624, 0.153)
const C_CRIMSON = Color(0.863, 0.235, 0.235)
const C_BG      = Color(0.04,  0.05,  0.08)
const C_PANEL   = Color(0.08,  0.09,  0.14)
const C_DIM     = Color(0.35,  0.38,  0.52)
const C_TEXT    = Color(0.82,  0.84,  0.92)

# ── Real deployed databases (PB-217) ────────────────────────────────
# Both Read Nodes live on enkidb-node-read -- one host, two ports.
const TARGETS = {
    "EnkiDB": {
        # PB-22x: the core operational particle store finally has a real
        # deployed CQRS pair (enkidb-read-server/enkidb-write-server) --
        # previously only the old, never-deployed enkidb-query-server
        # existed for it. 7001 is the port every client already expected
        # (this file's own TARGETS neighbors follow the same read-node
        # convention; see enkidb-read-server's own bind_addr() comment).
        #
        # Every default/preset query here is HOW_MUCH LIMIT-bounded on
        # purpose: at 1M-1B particles, a query that matches a large
        # fraction of the corpus is bound by how many *rows* it returns,
        # not by index lookup cost -- no visualization can render or
        # transfer literally a billion individual rows in under a second,
        # any more than a real database could. A bounded page + a fast
        # match is the honest, correct way to "show it in <1s" at any N;
        # see readnode.rs::safe_early_limit for the fix that makes this
        # bounded page stay fast regardless of the true match count.
        "host": "192.168.122.107", "port": 7001,
        "group_attr": "station",
        "default_query": "WHO T.E\nWHAT E[station]\nWHERE E[station] = \"data-cleansing-station\"\nHOW_MUCH LIMIT 1000",
        "presets": [
            ["Data-cleansing station (LIMIT 1000)", "WHO T.E\nWHAT E[station]\nWHERE E[station] = \"data-cleansing-station\"\nHOW_MUCH LIMIT 1000"],
            ["Data-steward station (LIMIT 1000)",   "WHO T.E\nWHAT E[station]\nWHERE E[station] = \"data-steward-station\"\nHOW_MUCH LIMIT 1000"],
            ["Needle lookup (single particle)",     "WHO T.E\nWHAT E[station, needle_id]\nWHERE E[needle_id] = \"needle-particle-42\""],
        ],
    },
    "EnkiDDB": {
        "host": "192.168.122.107", "port": 7102,
        "group_attr": "meta.collection",
        "default_query": "WHO T.E\nWHAT E[meta.title, meta.collection]\nWHERE E[meta.collection] = \"general\"\nHOW_MUCH LIMIT 100",
        "presets": [
            ["General docs",      "WHO T.E\nWHAT E[meta.title, meta.collection]\nWHERE E[meta.collection] = \"general\"\nHOW_MUCH LIMIT 200"],
            ["Components docs",   "WHO T.E\nWHAT E[meta.title, meta.collection]\nWHERE E[meta.collection] = \"components\"\nHOW_MUCH LIMIT 200"],
            ["Glossary docs",     "WHO T.E\nWHAT E[meta.title, meta.collection]\nWHERE E[meta.collection] = \"glossary\"\nHOW_MUCH LIMIT 200"],
            ["Playbook records",  "WHO T.E\nWHAT E[meta.title, meta.collection]\nWHERE E[meta.collection] = \"playbook-record\"\nHOW_MUCH LIMIT 200"],
        ],
    },
    "EnkiMDB": {
        "host": "192.168.122.107", "port": 7202,
        "group_attr": "artifact.kind",
        "default_query": "WHO T.E\nWHAT E[artifact.name, artifact.kind, artifact.path]\nWHERE E[artifact.kind] = \"Crate\"\nHOW_MUCH LIMIT 200",
        "presets": [
            ["All Crates",    "WHO T.E\nWHAT E[artifact.name, artifact.kind, artifact.path]\nWHERE E[artifact.kind] = \"Crate\"\nHOW_MUCH LIMIT 200"],
            ["All Playbooks", "WHO T.E\nWHAT E[artifact.name, artifact.kind, artifact.path]\nWHERE E[artifact.kind] = \"Playbook\"\nHOW_MUCH LIMIT 200"],
        ],
    },
}
var current_target := "EnkiDB"

# Real attribute -> display label map, EnkiDDB (DocOrbit) + EnkiMDB (flat
# artifact.*) combined -- see docs/ARCHITECT_DESIGN_ENKIDDB_ENKIMDB_PODMAN.md
# section 3 for the schema these come from. Unknown attributes fall back
# to their own real name (not a hash, not a truncated fragment).
const ATTR_NAMES = {
    "meta.title":       "title",
    "meta.collection":  "collection",
    "meta.source_path": "source",
    "body.summary":     "summary",
    "body.text":        "text",
    "artifact.name":    "name",
    "artifact.kind":    "kind",
    "artifact.path":    "path",
    "artifact.version": "version",
}

# ── State ─────────────────────────────────────────────────────────
var connected    := false
var running      := false
var query_rows   := []
var orbit_angle  := 0.0
var theater_tick := 0.0

var thread:        Thread = null
var thread_done   := false
var thread_result := []
var thread_error  := ""
var thread_status := ""

var connect_thread: Thread = null
var connect_done   := false
var connect_ok     := false

# Ring/colour assignment, computed fresh per query from whatever real
# category values (meta.collection / artifact.kind) actually came back --
# not a fixed hardcoded scheme, since EnkiDDB/EnkiMDB have no built-in
# quality tiering the way the old EnkiDB-core B11 scheme did.
const GROUP_PALETTE = [C_GOLD, C_TEAL, C_AMBER, Color(0.62, 0.42, 0.86), Color(0.86, 0.42, 0.62), C_CRIMSON]
var group_rings := {}  # group value (String) -> {"radius": float, "color": Color}

# FIXED (2026-07-27): left_mesh/right_mesh are tilted 3D quads textured
# from SubViewports -- clicking them for real requires a 3D raycast, not
# a raw screen-coordinate guess (see _forward_mouse_to_panel below). Both
# quads use this exact size (see _build_left_panel/_build_right_panel).
const PANEL_SIZE = Vector2(4.2, 3.2)
const PANEL_COLLISION_LAYER = 0x8

# ── Scene nodes ───────────────────────────────────────────────────
var camera:         Camera3D
var center_stage:   GPUParticles3D
var nisaba:         Node3D
var left_mesh:      MeshInstance3D
var right_mesh:     MeshInstance3D
var left_vp:        SubViewport
var right_vp:       SubViewport

# Left panel UI nodes
var lbl_status:     Label
var lbl_entries:    Label
var lbl_target:     Label
var btn_target_edb: Button
var btn_target_ddb: Button
var btn_target_mdb: Button
var presets_box:    VBoxContainer
var btn_connect:    Button
var btn_run:        Button
var txt_editor:     TextEdit
var lbl_running:    Label

# Right panel UI nodes
var orbit_vp:       SubViewport
var orbit_root:     Node3D
var grid_scroll:    ScrollContainer
var grid_box:       VBoxContainer
var btn_orbit:      Button
var btn_grid:       Button
var view_mode       := "ORBIT"

func _ready():
    get_tree().root.title = "DubSar IDE v4.0 — EnkiDB Theater"
    _build_scene()

func _build_scene():
    # ── Camera ────────────────────────────────────────────────────
    camera = Camera3D.new()
    camera.transform.origin = Vector3(0, 1.8, 7.0)
    camera.rotation_degrees = Vector3(-8, 0, 0)
    camera.fov = 75.0
    add_child(camera)

    # ── Environment ───────────────────────────────────────────────
    var env     = Environment.new()
    env.background_mode = Environment.BG_COLOR
    env.background_color = C_BG
    env.ambient_light_source = Environment.AMBIENT_SOURCE_COLOR
    env.ambient_light_color  = Color(0.05, 0.06, 0.10)
    env.ambient_light_energy = 0.4
    camera.environment = env

    # ── CENTER STAGE — BIGRING galaxy ─────────────────────────────
    center_stage = GPUParticles3D.new()
    center_stage.transform.origin = Vector3(0, 1.2, -1.5)
    center_stage.amount   = 50000
    center_stage.lifetime = 5.0
    center_stage.speed_scale = 0.8

    var pmat = ParticleProcessMaterial.new()
    pmat.emission_shape          = ParticleProcessMaterial.EMISSION_SHAPE_SPHERE
    pmat.emission_sphere_radius  = 2.5
    pmat.direction               = Vector3(0, 0, 0)
    pmat.gravity                 = Vector3(0, 0, 0)
    pmat.initial_velocity_min    = 0.2
    pmat.initial_velocity_max    = 1.5
    pmat.angular_velocity_min    = -60.0
    pmat.angular_velocity_max    = 60.0
    pmat.scale_min               = 0.015
    pmat.scale_max               = 0.04
    pmat.color                   = C_TEAL
    # Colour gradient: teal core → gold rim
    var grad = Gradient.new()
    grad.add_point(0.0, C_TEAL)
    grad.add_point(0.4, C_GOLD)
    grad.add_point(0.7, C_AMBER)
    grad.add_point(1.0, Color(0.2, 0.05, 0.05, 0.0))
    var gtex = GradientTexture1D.new()
    gtex.gradient = grad
    pmat.color_ramp = gtex
    center_stage.process_material = pmat

    var pmesh = SphereMesh.new()
    pmesh.radius = 0.025
    pmesh.height = 0.05
    center_stage.draw_pass_1 = pmesh
    add_child(center_stage)

    # Nucleus gold sphere
    var nucleus = MeshInstance3D.new()
    var ns = SphereMesh.new()
    ns.radius = 0.18
    nucleus.mesh = ns
    var nmat = StandardMaterial3D.new()
    nmat.albedo_color          = C_GOLD
    nmat.emission_enabled      = true
    nmat.emission              = C_GOLD
    nmat.emission_energy_multiplier = 3.0
    nucleus.set_surface_override_material(0, nmat)
    nucleus.transform.origin = Vector3(0, 1.2, -1.5)
    add_child(nucleus)

    # ── NISABA Stage 1 — gold spotlight crown ─────────────────────
    nisaba = Node3D.new()
    nisaba.transform.origin = Vector3(0, 4.5, -1.5)
    add_child(nisaba)

    var spot = SpotLight3D.new()
    spot.light_color              = C_GOLD
    spot.light_energy             = 3.0
    spot.spot_range               = 10.0
    spot.spot_angle               = 20.0
    spot.shadow_enabled           = false
    spot.rotation_degrees         = Vector3(-90, 0, 0)
    nisaba.add_child(spot)

    # ── Ambient fill light ────────────────────────────────────────
    var fill = OmniLight3D.new()
    fill.light_color  = C_TEAL
    fill.light_energy = 0.5
    fill.omni_range   = 15.0
    fill.transform.origin = Vector3(0, 3, 2)
    add_child(fill)

    # ── LEFT PANEL — HeptaScript editor ───────────────────────────
    _build_left_panel()

    # ── RIGHT PANEL — ORBIT + GRID ────────────────────────────────
    _build_right_panel()

    # ── Floor glow ────────────────────────────────────────────────
    var floor_mesh = MeshInstance3D.new()
    var quad = QuadMesh.new()
    quad.size = Vector2(18, 12)
    floor_mesh.mesh = quad
    floor_mesh.rotation_degrees = Vector3(-90, 0, 0)
    floor_mesh.transform.origin = Vector3(0, -0.5, 0)
    var fmat = StandardMaterial3D.new()
    fmat.albedo_color     = Color(0.04, 0.06, 0.10)
    fmat.emission_enabled = true
    fmat.emission         = C_TEAL
    fmat.emission_energy_multiplier = 0.08
    floor_mesh.set_surface_override_material(0, fmat)
    add_child(floor_mesh)

# A thin static collider matching a panel quad exactly, so a camera
# raycast under the mouse can tell which panel (if any) was actually
# clicked, and where on it -- see _forward_mouse_to_panel().
func _add_panel_collider(mesh: MeshInstance3D) -> void:
    var body = StaticBody3D.new()
    body.collision_layer = PANEL_COLLISION_LAYER
    body.collision_mask  = 0
    var shape = CollisionShape3D.new()
    var box = BoxShape3D.new()
    box.size = Vector3(PANEL_SIZE.x, PANEL_SIZE.y, 0.05)
    shape.shape = box
    body.add_child(shape)
    mesh.add_child(body)

# ── Build LEFT panel (HeptaScript editor + connector) ─────────────
func _build_left_panel():
    # 3D mesh at -35° Y
    left_mesh = MeshInstance3D.new()
    var quad = QuadMesh.new()
    quad.size = Vector2(4.2, 3.2)
    left_mesh.mesh = quad
    left_mesh.transform.origin = Vector3(-3.8, 1.8, 0.5)
    left_mesh.rotation_degrees = Vector3(0, 35, 0)
    add_child(left_mesh)

    _add_panel_collider(left_mesh)

    # SubViewport
    left_vp = SubViewport.new()
    left_vp.size = Vector2i(900, 680)
    left_vp.render_target_update_mode = SubViewport.UPDATE_ALWAYS
    left_vp.transparent_bg = false
    left_mesh.add_child(left_vp)

    # Apply SubViewport as texture to mesh
    var vp_mat = StandardMaterial3D.new()
    vp_mat.albedo_texture       = left_vp.get_texture()
    vp_mat.emission_enabled     = true
    vp_mat.emission_texture     = left_vp.get_texture()
    vp_mat.emission_energy_multiplier = 0.15
    vp_mat.flags_unshaded       = false
    left_mesh.set_surface_override_material(0, vp_mat)

    # Panel border glow
    var border = MeshInstance3D.new()
    var bq = QuadMesh.new()
    bq.size = Vector2(4.25, 3.25)
    border.mesh = bq
    border.transform.origin = Vector3(0, 0, -0.01)
    var bmat = StandardMaterial3D.new()
    bmat.albedo_color           = Color(0, 0, 0, 0)
    bmat.emission_enabled       = true
    bmat.emission               = C_TEAL
    bmat.emission_energy_multiplier = 0.4
    bmat.flags_unshaded         = true
    border.set_surface_override_material(0, bmat)
    left_mesh.add_child(border)

    # Build 2D UI inside left viewport
    _build_left_ui()

func _build_left_ui():
    var root = Control.new()
    root.set_anchors_preset(Control.PRESET_FULL_RECT)
    left_vp.add_child(root)

    # Dark background
    var bg = ColorRect.new()
    bg.color = C_PANEL
    bg.set_anchors_preset(Control.PRESET_FULL_RECT)
    root.add_child(bg)

    var vbox = VBoxContainer.new()
    vbox.set_anchors_preset(Control.PRESET_FULL_RECT)
    vbox.add_theme_constant_override("separation", 6)
    vbox.add_theme_constant_override("margin_left", 14)
    vbox.add_theme_constant_override("margin_right", 14)
    vbox.add_theme_constant_override("margin_top", 10)
    root.add_child(vbox)

    # Header
    vbox.add_child(_lbl("𒁾  DubSar IDE v4.0", 18, C_GOLD, true))
    vbox.add_child(_sep())
    vbox.add_child(_lbl("Escape to return to the main theater", 9, C_DIM))
    vbox.add_child(_lbl("Database", 11, C_DIM))

    var target_bar = HBoxContainer.new()
    btn_target_edb = Button.new()
    btn_target_edb.text = "EnkiDB (7001)"
    btn_target_edb.toggle_mode = true
    btn_target_edb.button_pressed = true
    btn_target_edb.add_theme_font_size_override("font_size", 11)
    btn_target_edb.pressed.connect(func(): _set_target("EnkiDB"))
    target_bar.add_child(btn_target_edb)

    btn_target_ddb = Button.new()
    btn_target_ddb.text = "EnkiDDB (7102)"
    btn_target_ddb.toggle_mode = true
    btn_target_ddb.add_theme_font_size_override("font_size", 11)
    btn_target_ddb.pressed.connect(func(): _set_target("EnkiDDB"))
    target_bar.add_child(btn_target_ddb)

    btn_target_mdb = Button.new()
    btn_target_mdb.text = "EnkiMDB (7202)"
    btn_target_mdb.toggle_mode = true
    btn_target_mdb.add_theme_font_size_override("font_size", 11)
    btn_target_mdb.pressed.connect(func(): _set_target("EnkiMDB"))
    target_bar.add_child(btn_target_mdb)
    vbox.add_child(target_bar)

    lbl_target = _lbl("Read node:  192.168.122.107:7001", 11, C_TEAL)
    vbox.add_child(lbl_target)
    vbox.add_child(_lbl("Engine:     HeptaScript v1.0  W5H2", 10, C_DIM))

    lbl_entries = _lbl("Entries: — (run a query)", 11, C_TEXT)
    vbox.add_child(lbl_entries)

    btn_connect = Button.new()
    btn_connect.text = "▶  Connect"
    btn_connect.add_theme_font_size_override("font_size", 13)
    btn_connect.pressed.connect(_on_connect)
    vbox.add_child(btn_connect)

    lbl_status = _lbl("Not connected", 10, C_DIM)
    lbl_status.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
    vbox.add_child(lbl_status)

    vbox.add_child(_sep())
    vbox.add_child(_lbl("HeptaScript W5H2 Query Editor", 12, C_GOLD, true))

    txt_editor = TextEdit.new()
    txt_editor.text = TARGETS[current_target]["default_query"]
    txt_editor.custom_minimum_size.y = 140
    txt_editor.add_theme_font_size_override("font_size", 13)
    txt_editor.add_theme_color_override("background_color",
        Color(0.05, 0.06, 0.10))
    txt_editor.add_theme_color_override("font_color", C_TEXT)
    vbox.add_child(txt_editor)

    # Run + view buttons
    var bar = HBoxContainer.new()
    btn_run = Button.new()
    btn_run.text = "▶  Run W5H2 Query"
    btn_run.disabled = true
    btn_run.add_theme_font_size_override("font_size", 13)
    btn_run.pressed.connect(_on_run_query)
    bar.add_child(btn_run)
    vbox.add_child(bar)

    lbl_running = _lbl("", 10, C_AMBER)
    vbox.add_child(lbl_running)

    vbox.add_child(_sep())
    vbox.add_child(_lbl("Quick W5H2 Queries", 11, C_GOLD))

    presets_box = VBoxContainer.new()
    vbox.add_child(presets_box)
    _rebuild_presets()

func _rebuild_presets():
    for c in presets_box.get_children():
        c.queue_free()
    for preset in TARGETS[current_target]["presets"]:
        var b = Button.new()
        b.text = preset[0]
        b.alignment = HORIZONTAL_ALIGNMENT_LEFT
        b.add_theme_font_size_override("font_size", 11)
        var q = preset[1]
        b.pressed.connect(func(): txt_editor.text = q)
        presets_box.add_child(b)

func _set_target(target: String):
    if target == current_target:
        return
    current_target = target
    btn_target_edb.button_pressed = target == "EnkiDB"
    btn_target_ddb.button_pressed = target == "EnkiDDB"
    btn_target_mdb.button_pressed = target == "EnkiMDB"
    var t = TARGETS[target]
    lbl_target.text = "Read node:  %s:%d" % [t["host"], t["port"]]
    txt_editor.text = t["default_query"]
    _rebuild_presets()
    connected = false
    btn_connect.text = "▶  Connect"
    btn_run.disabled = true
    _update_status("Not connected", C_DIM)
    query_rows = []
    _clear_orbit()
    _clear_grid()
    lbl_entries.text = "Entries: — (run a query)"

# ── Build RIGHT panel (ORBIT 3D + GRID table) ─────────────────────
func _build_right_panel():
    # 3D mesh at +35° Y
    right_mesh = MeshInstance3D.new()
    var quad = QuadMesh.new()
    quad.size = Vector2(4.2, 3.2)
    right_mesh.mesh = quad
    right_mesh.transform.origin = Vector3(3.8, 1.8, 0.5)
    right_mesh.rotation_degrees = Vector3(0, -35, 0)
    add_child(right_mesh)

    _add_panel_collider(right_mesh)

    # SubViewport
    right_vp = SubViewport.new()
    right_vp.size = Vector2i(900, 680)
    right_vp.render_target_update_mode = SubViewport.UPDATE_ALWAYS
    right_vp.transparent_bg = false
    right_mesh.add_child(right_vp)

    var vp_mat = StandardMaterial3D.new()
    vp_mat.albedo_texture           = right_vp.get_texture()
    vp_mat.emission_enabled         = true
    vp_mat.emission_texture         = right_vp.get_texture()
    vp_mat.emission_energy_multiplier = 0.15
    right_mesh.set_surface_override_material(0, vp_mat)

    # Border glow
    var border = MeshInstance3D.new()
    var bq = QuadMesh.new()
    bq.size = Vector2(4.25, 3.25)
    border.mesh = bq
    border.transform.origin = Vector3(0, 0, -0.01)
    var bmat = StandardMaterial3D.new()
    bmat.albedo_color               = Color(0, 0, 0, 0)
    bmat.emission_enabled           = true
    bmat.emission                   = C_GOLD
    bmat.emission_energy_multiplier = 0.4
    bmat.flags_unshaded             = true
    border.set_surface_override_material(0, bmat)
    right_mesh.add_child(border)

    _build_right_ui()

func _build_right_ui():
    var root = Control.new()
    root.set_anchors_preset(Control.PRESET_FULL_RECT)
    right_vp.add_child(root)

    var bg = ColorRect.new()
    bg.color = C_PANEL
    bg.set_anchors_preset(Control.PRESET_FULL_RECT)
    root.add_child(bg)

    var vbox = VBoxContainer.new()
    vbox.set_anchors_preset(Control.PRESET_FULL_RECT)
    vbox.add_theme_constant_override("separation", 4)
    root.add_child(vbox)

    # Header + view toggle
    var hdr = HBoxContainer.new()
    hdr.add_child(_lbl("ORBIT — real particles, grouped by category", 13, C_GOLD, true))
    var sp = Control.new()
    sp.size_flags_horizontal = Control.SIZE_EXPAND_FILL
    hdr.add_child(sp)

    btn_orbit = Button.new()
    btn_orbit.text = "ORBIT"
    btn_orbit.toggle_mode = true
    btn_orbit.button_pressed = true
    btn_orbit.add_theme_font_size_override("font_size", 11)
    btn_orbit.pressed.connect(func(): _set_view("ORBIT"))
    hdr.add_child(btn_orbit)

    btn_grid = Button.new()
    btn_grid.text = "GRID"
    btn_grid.toggle_mode = true
    btn_grid.add_theme_font_size_override("font_size", 11)
    btn_grid.pressed.connect(func(): _set_view("GRID"))
    hdr.add_child(btn_grid)
    vbox.add_child(hdr)

    vbox.add_child(_sep())

    # ORBIT SubViewport inside right panel
    var orb_container = SubViewportContainer.new()
    orb_container.size_flags_vertical = Control.SIZE_EXPAND_FILL
    orb_container.stretch = true
    vbox.add_child(orb_container)

    orbit_vp = SubViewport.new()
    orbit_vp.render_target_update_mode = SubViewport.UPDATE_ALWAYS
    orb_container.add_child(orbit_vp)

    orbit_root = Node3D.new()
    orbit_vp.add_child(orbit_root)

    var orb_cam = Camera3D.new()
    orb_cam.transform.origin = Vector3(0, 2.5, 5.5)
    orb_cam.rotation_degrees = Vector3(-20, 0, 0)
    orb_cam.fov = 60.0
    orbit_root.add_child(orb_cam)

    var orb_light = OmniLight3D.new()
    orb_light.light_color  = C_GOLD
    orb_light.light_energy = 1.2
    orb_light.omni_range   = 20.0
    orbit_root.add_child(orb_light)

    # GRID scroll (hidden by default)
    grid_scroll = ScrollContainer.new()
    grid_scroll.size_flags_vertical = Control.SIZE_EXPAND_FILL
    grid_scroll.visible = false
    vbox.add_child(grid_scroll)

    grid_box = VBoxContainer.new()
    grid_box.size_flags_horizontal = Control.SIZE_EXPAND_FILL
    grid_scroll.add_child(grid_box)

# ── _process: animation + thread polling ──────────────────────────
func _process(delta):
    theater_tick += delta
    orbit_angle  += delta * 0.25

    # Slowly rotate BIGRING
    center_stage.rotation.y += delta * 0.04

    # NISABA pulse
    if nisaba:
        var pulse = 1.0 + 0.3 * sin(theater_tick * 2.0)
        for c in nisaba.get_children():
            if c is SpotLight3D:
                c.light_energy = 3.0 * pulse

    # Poll connect thread
    if connect_done and connect_thread != null:
        connect_thread.wait_to_finish()
        connect_thread = null
        connect_done   = false
        var t = TARGETS[current_target]
        if connect_ok:
            connected = true
            btn_connect.text = "✓ Connected — Reconnect"
            btn_run.disabled = false
            _update_status("✓ Connected  %s:%d" % [t["host"], t["port"]], C_TEAL)
        else:
            connected = false
            btn_connect.text = "▶  Connect"
            _update_status("Cannot reach %s:%d" % [t["host"], t["port"]], C_CRIMSON)

    # Poll query thread
    if thread_done and thread != null:
        thread.wait_to_finish()
        thread      = null
        thread_done = false
        running     = false
        btn_run.disabled = false
        query_rows  = thread_result
        lbl_running.text = ""
        if thread_error != "":
            _update_status("✗ " + thread_error, C_CRIMSON)
            lbl_entries.text = "Entries: 0 (query error)"
        else:
            var n = query_rows.size()
            _update_status("✓ W5H2 complete — %d rows" % n, C_TEAL)
            lbl_entries.text = "Entries: %d" % n
        _populate_orbit()
        _populate_grid()

    elif running:
        lbl_running.text = thread_status

    # Animate orbit particles
    _animate_orbit(delta)

# ── Connect ───────────────────────────────────────────────────────
func _on_connect():
    var t = TARGETS[current_target]
    btn_connect.text = "Testing..."
    btn_connect.disabled = true
    _update_status("Connecting to %s:%d..." % [t["host"], t["port"]], C_AMBER)
    connect_done   = false
    connect_thread = Thread.new()
    connect_thread.start(_thread_connect.bind(t["host"], t["port"]))

func _thread_connect(host: String, port: int):
    connect_ok   = EnkiDbTCP_test_connection(host, port)
    connect_done = true
    call_deferred("_on_connect_thread_done")

func _on_connect_thread_done():
    btn_connect.disabled = false

# ── Run query ─────────────────────────────────────────────────────
func _on_run_query():
    if not connected or running:
        return
    running      = true
    btn_run.disabled = true
    lbl_running.text = "Sending QUERY:..."
    query_rows   = []
    thread_error = ""
    _clear_orbit()
    _clear_grid()
    thread       = Thread.new()
    thread_done  = false
    thread_result = []
    var t = TARGETS[current_target]
    thread.start(_thread_query.bind(t["host"], t["port"], txt_editor.text))

func _thread_query(host: String, port: int, src: String):
    var result = EnkiDbTCP_execute_query(host, port, src)
    thread_result = result["rows"]
    thread_error  = result["error"]
    thread_done   = true

# ── ORBIT 3D ──────────────────────────────────────────────────────
func _clear_orbit():
    for c in orbit_root.get_children():
        if c is Camera3D or c is OmniLight3D:
            continue
        c.queue_free()

func _populate_orbit():
    _clear_orbit()
    group_rings.clear()
    if query_rows.is_empty() or view_mode != "ORBIT":
        return
    var n = query_rows.size()
    var group_attr = TARGETS[current_target]["group_attr"]

    # Gold nucleus
    var nucleus = MeshInstance3D.new()
    var ns = SphereMesh.new(); ns.radius = 0.12
    nucleus.mesh = ns
    var nm = StandardMaterial3D.new()
    nm.albedo_color = C_GOLD
    nm.emission_enabled = true
    nm.emission = C_GOLD
    nm.emission_energy_multiplier = 3.0
    nucleus.set_surface_override_material(0, nm)
    nucleus.name = "Nucleus"
    orbit_root.add_child(nucleus)

    # Particles -- ring/colour assigned dynamically per distinct real
    # category value (meta.collection or artifact.kind), not a fixed tier.
    for i in range(min(n, 1000)):
        var row = query_rows[i]
        var group = _group_from_attrs(row.get("attrs", []), group_attr)
        var ring = _ring_for_group(group)
        var ring_r: float = ring["radius"]
        var col:    Color = ring["color"]

        var angle  = (i as float / n) * TAU
        var wobble = sin(i * 0.43) * 0.25

        var dot = MeshInstance3D.new()
        var sm  = SphereMesh.new()
        sm.radius = 0.03
        dot.mesh  = sm
        var mat   = StandardMaterial3D.new()
        mat.albedo_color              = col
        mat.emission_enabled          = true
        mat.emission                  = col
        mat.emission_energy_multiplier = 0.8
        dot.set_surface_override_material(0, mat)
        dot.position = Vector3(
            ring_r * cos(angle),
            wobble,
            ring_r * sin(angle)
        )
        dot.name = "P_%d" % i
        dot.set_meta("base_angle", angle)
        dot.set_meta("ring_r",     ring_r)
        dot.set_meta("wobble",     wobble)
        dot.set_meta("ring_idx",   group_rings.keys().find(group))
        orbit_root.add_child(dot)

    # Ring labels, one per distinct group actually present
    for group in group_rings.keys():
        var ring = group_rings[group]
        var lbl = Label3D.new()
        lbl.text = group
        lbl.font_size = 14
        lbl.modulate = ring["color"]
        lbl.position = Vector3(ring["radius"] + 0.15, 0.05, 0)
        orbit_root.add_child(lbl)

    # Stats label -- real counts per real group, not GEM/ORBIT/FUZZY
    var stats_lines = ["%d particles" % n]
    for group in group_rings.keys():
        var count = query_rows.filter(func(r): return _group_from_attrs(r.get("attrs", []), group_attr) == group).size()
        stats_lines.append("%s: %d" % [group, count])
    var stats = Label3D.new()
    stats.text      = "\n".join(stats_lines)
    stats.font_size = 16
    stats.modulate  = C_GOLD
    stats.position  = Vector3(0, 3.0, 0)
    orbit_root.add_child(stats)

func _ring_for_group(group: String) -> Dictionary:
    if not group_rings.has(group):
        var idx = group_rings.size()
        group_rings[group] = {
            "radius": 1.0 + idx * 0.7,
            "color":  GROUP_PALETTE[idx % GROUP_PALETTE.size()],
        }
    return group_rings[group]

func _animate_orbit(delta):
    if not is_instance_valid(orbit_root): return
    for child in orbit_root.get_children():
        if not child.name.begins_with("P_"): continue
        var base     = child.get_meta("base_angle", 0.0)
        var ring_r   = child.get_meta("ring_r",     2.0)
        var wobble   = child.get_meta("wobble",     0.0)
        var ring_idx = child.get_meta("ring_idx",   0)
        var speed    = max(0.3, 0.9 - ring_idx * 0.15)
        var angle    = base + orbit_angle * speed
        child.position = Vector3(
            ring_r * cos(angle),
            wobble,
            ring_r * sin(angle)
        )

# ── GRID table ────────────────────────────────────────────────────
func _clear_grid():
    for c in grid_box.get_children():
        c.queue_free()

func _populate_grid():
    _clear_grid()
    if query_rows.is_empty() or view_mode != "GRID":
        return
    var group_attr = TARGETS[current_target]["group_attr"]

    # Header
    var hdr = HBoxContainer.new()
    hdr.add_child(_gcell("KAKI",       180, C_GOLD, true))
    hdr.add_child(_gcell("Group",      120, C_GOLD, true))
    hdr.add_child(_gcell("Attributes", 550, C_GOLD, true))
    grid_box.add_child(hdr)
    grid_box.add_child(_sep())

    for i in range(min(query_rows.size(), 1000)):
        var row  = query_rows[i]
        var hex  = row.get("kaki", "")
        var attrs = row.get("attrs", [])
        var group = _group_from_attrs(attrs, group_attr)
        var ring  = _ring_for_group(group)
        var col: Color  = ring["color"]

        var attr_str = ""
        for pair in attrs.slice(0, 4):
            if pair is Array and pair.size() >= 2:
                var k = ATTR_NAMES.get(str(pair[0]), str(pair[0]))
                var v = str(pair[1]).left(40)
                attr_str += "%s=%s  " % [k, v]

        var r = HBoxContainer.new()
        if i % 2 == 0:
            var bg_style = StyleBoxFlat.new()
            bg_style.bg_color = Color(0.06, 0.07, 0.11)
            r.add_theme_stylebox_override("panel", bg_style)
        r.add_child(_gcell(hex.left(16) + "..", 180, C_TEAL))
        r.add_child(_gcell(group, 120, col, true))
        r.add_child(_gcell(attr_str, 550, C_TEXT))
        grid_box.add_child(r)

    grid_box.add_child(_sep())
    var t = TARGETS[current_target]
    grid_box.add_child(_lbl(
        "%d rows  |  TCP: %s:%d" % [query_rows.size(), t["host"], t["port"]],
        10, C_DIM
    ))

# ── View toggle ───────────────────────────────────────────────────
func _set_view(mode: String):
    view_mode = mode
    var orb = mode == "ORBIT"
    orbit_vp.get_parent().visible = orb
    grid_scroll.visible           = not orb
    btn_orbit.button_pressed      = orb
    btn_grid.button_pressed       = not orb
    if orb:   _populate_orbit()
    else:     _populate_grid()

# ── Helpers ───────────────────────────────────────────────────────
# Real category value from EAV, read from whichever attribute the
# current target groups by (meta.collection for EnkiDDB, artifact.kind
# for EnkiMDB) -- both are always projected by this file's own preset/
# default queries. "other" only appears if a custom query drops that
# attribute from its own WHAT clause.
func _group_from_attrs(attrs: Array, group_attr: String) -> String:
    for pair in attrs:
        if pair is Array and pair.size() >= 2 and str(pair[0]) == group_attr:
            return str(pair[1])
    return "other"

func _update_status(msg: String, col: Color = C_DIM):
    if lbl_status:
        lbl_status.text = msg
        lbl_status.add_theme_color_override("font_color", col)

func _lbl(text: String, size: int = 11,
          col: Color = C_TEXT, bold: bool = false) -> Label:
    var l = Label.new()
    l.text = text
    l.add_theme_font_size_override("font_size", size)
    l.add_theme_color_override("font_color", col)
    return l

func _sep() -> HSeparator: return HSeparator.new()

func _gcell(text: String, w: int,
            col: Color = C_TEXT, bold: bool = false) -> Label:
    var l = Label.new()
    l.text = text
    l.custom_minimum_size.x = w
    l.add_theme_font_size_override("font_size", 11)
    l.add_theme_color_override("font_color", col)
    l.clip_text = true
    return l

# ══ Inlined TCP client (no autoload dependency) ════════════════════
# PB-217: EnkiDDB/EnkiMDB's real wire protocol (confirmed against
# bin/enkiddb-read-server/src/main.rs::handle()) is one request, one
# response, connection closes -- not the multi-frame-plus-0-sentinel
# stream enkidb-query-server used. Request must be prefixed "QUERY:".
const CONNECT_TIMEOUT = 5.0
const READ_TIMEOUT    = 30.0

func EnkiDbTCP_test_connection(host: String, port: int) -> bool:
    var peer = StreamPeerTCP.new()
    if peer.connect_to_host(host, port) != OK: return false
    var t = Time.get_ticks_msec()
    while peer.get_status() == StreamPeerTCP.STATUS_CONNECTING:
        peer.poll()
        if Time.get_ticks_msec() - t > CONNECT_TIMEOUT * 1000:
            peer.disconnect_from_host(); return false
    var ok = peer.get_status() == StreamPeerTCP.STATUS_CONNECTED
    peer.disconnect_from_host(); return ok

# Returns {"rows": Array, "error": String}. `error` is non-empty on any
# transport failure or a real `ERR:...` from the server; `rows` is
# always the parsed JSON array on success (possibly empty -- a real,
# valid "no matches" result, not a failure).
func EnkiDbTCP_execute_query(host: String, port: int, w5h2: String) -> Dictionary:
    var peer = StreamPeerTCP.new()
    if peer.connect_to_host(host, port) != OK:
        return {"rows": [], "error": "could not open connection to %s:%d" % [host, port]}
    var t = Time.get_ticks_msec()
    while peer.get_status() == StreamPeerTCP.STATUS_CONNECTING:
        peer.poll()
        if Time.get_ticks_msec() - t > CONNECT_TIMEOUT * 1000:
            return {"rows": [], "error": "connect timed out"}
    if peer.get_status() != StreamPeerTCP.STATUS_CONNECTED:
        return {"rows": [], "error": "connection refused or reset"}

    var payload = "QUERY:" + w5h2
    var qb  = payload.to_utf8_buffer()
    var lb  = PackedByteArray(); lb.resize(4)
    var ql  = qb.size()
    lb[0]=ql&0xFF; lb[1]=(ql>>8)&0xFF; lb[2]=(ql>>16)&0xFF; lb[3]=(ql>>24)&0xFF
    peer.put_data(lb); peer.put_data(qb)

    # Read exactly one length-prefixed response frame.
    var st = Time.get_ticks_msec()
    while peer.get_available_bytes() < 4:
        peer.poll()
        if Time.get_ticks_msec() - st > READ_TIMEOUT * 1000:
            peer.disconnect_from_host()
            return {"rows": [], "error": "timed out waiting for response header"}
        OS.delay_msec(10)
    var lr = peer.get_data(4)
    if lr[0] != OK:
        peer.disconnect_from_host()
        return {"rows": [], "error": "failed reading response header"}
    var lb2: PackedByteArray = lr[1]
    var fl = lb2[0]|(lb2[1]<<8)|(lb2[2]<<16)|(lb2[3]<<24)

    var wt = Time.get_ticks_msec()
    while peer.get_available_bytes() < fl:
        peer.poll()
        if Time.get_ticks_msec() - wt > READ_TIMEOUT * 1000:
            peer.disconnect_from_host()
            return {"rows": [], "error": "timed out waiting for response body"}
        OS.delay_msec(10)
    var fr = peer.get_data(fl)
    peer.disconnect_from_host()
    if fr[0] != OK:
        return {"rows": [], "error": "failed reading response body"}

    var raw: PackedByteArray = fr[1]

    # 2026-07-21: all 7 real Read Nodes speak the same binary wire, no
    # exceptions and no fallback anymore (EnkiDDB/EnkiMDB's own
    # read-servers moved off JSON this session too, which is exactly what
    # made the old per-target "wire" flag here a live bug -- it still
    # defaulted those two to a JSON decode against a server that no
    # longer speaks it). Decoding itself now lives in one place,
    # SumuUkinClient.decode_binary_response() (scripts/sumuukin_client.gd),
    # not duplicated here.
    var decoded = SumuUkinClient.decode_binary_response(raw)
    return {"rows": decoded.rows, "error": decoded.error}

# Called by Godot when mouse is clicked in 3D scene
func _input(event):
    if event is InputEventKey and event.pressed and not event.echo and event.keycode == KEY_ESCAPE:
        get_tree().change_scene_to_file("res://scenes/main_theater.tscn")
        return
    if event is InputEventMouseButton or event is InputEventMouseMotion:
        _forward_mouse_to_panel(event)

# FIXED (2026-07-27): the previous version forwarded the event's RAW
# screen-space position straight into push_input() based only on which
# third of the screen the cursor was in. Both panels are tilted 3D quads
# viewed through a perspective camera, so screen pixels never line up
# with the target SubViewport's own local pixel space -- every click
# landed on the wrong control (or nothing), which is what made the
# panels look completely frozen. This raycasts the real click against
# the panel's collider (_add_panel_collider), finds exactly where on
# the quad it landed, and converts that into the SubViewport's own
# pixel coordinates before forwarding.
func _forward_mouse_to_panel(event: InputEvent) -> void:
    if camera == null:
        return
    var space_state = get_world_3d().direct_space_state
    var origin = camera.project_ray_origin(event.position)
    var dir    = camera.project_ray_normal(event.position)
    var query  = PhysicsRayQueryParameters3D.create(origin, origin + dir * 100.0)
    query.collision_mask = PANEL_COLLISION_LAYER
    var hit = space_state.intersect_ray(query)
    if hit.is_empty():
        return

    var mesh: MeshInstance3D = null
    var vp: SubViewport = null
    var collider_parent = hit.collider.get_parent()
    if collider_parent == left_mesh:
        mesh = left_mesh; vp = left_vp
    elif collider_parent == right_mesh:
        mesh = right_mesh; vp = right_vp
    else:
        return

    var local_hit = mesh.global_transform.affine_inverse() * (hit.position as Vector3)
    var u = clamp((local_hit.x + PANEL_SIZE.x * 0.5) / PANEL_SIZE.x, 0.0, 1.0)
    var v = clamp((PANEL_SIZE.y * 0.5 - local_hit.y) / PANEL_SIZE.y, 0.0, 1.0)
    var vp_pos = Vector2(u * vp.size.x, v * vp.size.y)

    var forwarded = event.duplicate()
    forwarded.position = vp_pos
    if "global_position" in forwarded:
        forwarded.global_position = vp_pos
    vp.push_input(forwarded, true)
