class_name OnionTower3D
extends Node3D
# =============================================================================
# Onion Layers — 3D Interactive Tower — 2026-07-21
#
# Real data, transcribed directly from docs/09_observatory/BAHYWAY_ONION_LAYERS.html (itself
# transcribed from the real workspace/bahyway_v4/Cargo.toml, 163 members,
# sealed by playbook_197_onion_layers_design_and_artifact.yml, 2026-07-19).
# Same 25 layers + 1 ghost layer, same 5 functional band colors -- this is a
# second, 3D-interactive rendering of that same already-sealed dataset, not
# new/invented data. Crate lists per layer are copied verbatim.
#
# "Clearer than flat rings": the 2D onion artifact nests 25 thin concentric
# circles that get visually cramped near the center. This tower instead
# STACKS the same 25 layers vertically (L0 Foundation at the base, TEST at
# the top -- the artifact's own dependency-order reading direction) as one
# torus ring per layer, so every layer gets full-size screen space instead
# of being squeezed toward a point. Ring radius grows slightly with height
# (a gentle cone), keeping the "each layer wraps around the ones below it"
# onion reading even though the layout is now vertical.
#
# Real interactivity, not decoration: mouse-drag orbit camera (yaw/pitch,
# scroll-wheel zoom, both clamped), and real 3D click-picking on each ring
# via Area3D.input_event (Godot's own documented "Physics Picking" -- no
# hand-rolled raycasting) showing that layer's real crate list.
# =============================================================================

const DubSarTheme = preload("res://scripts/dubsar_theme.gd")

# Band colors are the onion artifact's OWN dark-theme hex values (its
# --gold/--cyan/--ember/--green/--violet/--alarm CSS vars), copied verbatim
# rather than reinterpreted through DubSarTheme's dashboard palette -- this
# keeps the tower's 5 functional colors matching the sealed 2D artifact
# exactly. DubSarTheme is still used for everything else (UI chrome,
# background, lighting tint).
const BAND := {
	"core":     {"color": Color("#ffd700"), "label": "Foundation & Identity"},
	"engine":   {"color": Color("#35f2f2"), "label": "Engines & Intelligence"},
	"security": {"color": Color("#ff9d33"), "label": "Security & Pipeline"},
	"language": {"color": Color("#35ffa0"), "label": "Languages & Protocol"},
	"surface":  {"color": Color("#d072ff"), "label": "Runtime, UI & Theater"},
	"ghost":    {"color": Color("#ff5c5c"), "label": "Sealed name, not yet a crate"},
}

const LAYERS := [
	{"n": "L0",   "name": "Foundation", "band": "core",
		"crates": ["bahyway-core","bahyway-crc","bahyway-algebra","bahyway-field","algebra-arsenal","bahyway-fabric"]},
	{"n": "L1",   "name": "KAKI Identity", "band": "core",
		"crates": ["enkidb-kaki","enkidb-vector-id"]},
	{"n": "L1.5", "name": "Particles (7D EAV)", "band": "core",
		"crates": ["enkidb-particles","enkiddb","enkiddb-ingest","enkimdb","susa-engine"]},
	{"n": "L2",   "name": "Storage Substrate", "band": "core",
		"crates": ["enkidb-block","enkidb-journal","enkidb-storage","enkidb-snapshot","enkidb-recovery","enkidb-quantdb","enkidb-persist","enkidb-datafile","enkidb-readnode","enkidw","enkisdb","enkiqdb","enkiodb","storage-sector","blackbox-station"]},
	{"n": "L3",   "name": "Native Indexes", "band": "core",
		"crates": ["enkidb-indexes","enkidb-dictionary"]},
	{"n": "L4",   "name": "EnkiDB Engine", "band": "engine",
		"crates": ["enkidb-engine","enkidb-query","enkidb-raft","enkidb-ingest"]},
	{"n": "L4.5", "name": "Physics & Intelligence", "band": "engine",
		"crates": ["vgca-engine","tribe-orbit-engine","ammas-engine","shedu-engine","riksu-engine","vault-engine","graph-engine"]},
	{"n": "L5",   "name": "Operational Engines", "band": "engine",
		"crates": ["kinetic-engine","sumer-engine","pollution-engine","story-engine","story-sentinel","fuzzy-engine","score-engine","hepta-score","alert-engine","snapshot-job","navi-engine","najaf-engine","dmw-engine","nusku-engine","azuga-engine","nusku-fuzzy","iris-engine","panam-engine","nusku-score","shulman-engine","wpd-engine","homt-engine"]},
	{"n": "L6",   "name": "Cross-Tribe / IDU", "band": "engine",
		"crates": ["idu-prober","idu-batching"]},
	{"n": "L7",   "name": "Templates & Governance", "band": "engine",
		"crates": ["template-engine","template-library","diagnosis-templates","diagnosis-engine","damadmbok-dictionary"]},
	{"n": "L8",   "name": "HeptaSec", "band": "security",
		"crates": ["hepta-sec-firewall","hepta-sec-policy","hepta-sec-sentinel","hepta-sec-web","enkidb-replication","pii-vault","sla-engine"]},
	{"n": "L8.5", "name": "ENKI-TERRA", "band": "security",
		"crates": ["esarhaddon","ashnan","naramsin-archive","naramsin-format","enkidb-session-registry","enkidb-con-engine","naramsin-integrity","naramsin-audit","tiamat-engine4","urnammu-attestationd","kittu-engine","naramsin-bridge"]},
	{"n": "L8.6", "name": "Pattern Intelligence", "band": "engine",
		"crates": ["steward-lens","agent-council","nisaba","lamassu-engine","enki-genesis","enki-pattern","enkidu-protocol"]},
	{"n": "L8-P", "name": "Pipeline / Stations", "band": "security",
		"crates": ["media-type-detector","akk-loader","adad-gate","musaru-security","compare-tribe-schema","vgca-validation","data-structure-station","data-cleansing-station","data-steward-station","permanent-storage","client-dq-profile","grave-discovery-station"]},
	{"n": "L9",   "name": "Languages", "band": "language",
		"crates": ["hepta","aaol","heptascript","akkadi","akkadi-ir"]},
	{"n": "L9.1", "name": "Sovereign Crypto / Value", "band": "language",
		"crates": ["kupru","akkvalue","istar"]},
	{"n": "L9.2", "name": "Ezida (Query Compiler)", "band": "language",
		"crates": ["ezida-ir"]},
	{"n": "L9.3", "name": "DFG + HDF Bridge", "band": "language",
		"crates": ["dfg-engine","hdf-bridge"]},
	{"n": "L9.5", "name": "EnkiduLLM", "band": "language",
		"crates": ["enkidullm-core","enkidullm-model","enkidullm-memory","adapa-recall","enkidullm-chat","ea-agent-core","ea-agent-algebra","ea-agent-oracle","ea-agent-chat","enkidullm-ingest","zikru-embed","enkidullm-audit"]},
	{"n": "L10",  "name": "Runtime / OS", "band": "surface",
		"crates": ["eridu-runtime","eridu-scheduler","eridu-supervisor"]},
	{"n": "L11",  "name": "UI / IDE", "band": "surface",
		"crates": ["dubsar-ide","dubsar-visualizer","bee-mdm-bus"]},
	{"n": "L12",  "name": "Website", "band": "surface",
		"crates": ["bahyway-web"]},
	{"n": "DT",   "name": "DubSar Theater", "band": "surface",
		"crates": ["utnapishtim","edubba-seal","enuma"]},
	{"n": "BIN",  "name": "Binaries & Agents", "band": "surface",
		"crates": ["bahyway-server","enkidb-query-server","enkiddb-write-server","enkiddb-read-server","enkimdb-write-server","enkimdb-read-server","ninsun-agent","ninsun-steward-bridge","namtar-kaki","ereskigal-kaki","nanshe-ingest","esarhaddon-ingest","ashnan-ingest","bahyway-cli","najaf-ingest","dubsar","enkidw-cli","akkadi-cli","bee-watchdog","bahyway-api"]},
	{"n": "TEST", "name": "Integration Tests", "band": "surface",
		"crates": ["tests/e2e","tests/chaos"]},
]

const GHOST := {
	"n": "—", "name": "Sealed, Unbuilt", "band": "ghost",
	"crates": ["AkkadiSafeEngine (PB-88)", "AkkadiRulesEngine (PB-88)", "AkkadiCipherEngine (PB-88 — likely = kupru)", "Nergal-as-crate (Architect-confirmed, no coded identity yet)"],
}

const BASE_RADIUS := 3.0
const RADIUS_STEP := 0.32
const HEIGHT_STEP := 1.15
const RING_THICKNESS := 0.34

var _cam: Camera3D
var _dragging := false
var _last_mouse := Vector2.ZERO
var _yaw := -0.7
var _pitch := -0.35
var _distance := 26.0
const MIN_DIST := 10.0
const MAX_DIST := 55.0
var _tower_center: Vector3

var _detail_panel: PanelContainer
var _detail_title: Label
var _detail_body: RichTextLabel
var _status_label: Label

func _ready() -> void:
	get_viewport().physics_object_picking = true
	_setup_environment()
	_setup_camera()
	_build_tower()
	_build_ui()
	_update_cam_transform()

func _setup_environment() -> void:
	var env = Environment.new()
	env.background_mode = Environment.BG_COLOR
	env.background_color = DubSarTheme.BG
	env.ambient_light_source = Environment.AMBIENT_SOURCE_COLOR
	env.ambient_light_color = Color(0.10, 0.11, 0.16)
	env.ambient_light_energy = 0.9
	var world_env = WorldEnvironment.new()
	world_env.environment = env
	add_child(world_env)

	var key_light = DirectionalLight3D.new()
	key_light.light_color = DubSarTheme.ACCENT_CYAN
	key_light.light_energy = 0.9
	key_light.rotation_degrees = Vector3(-55, -30, 0)
	add_child(key_light)

	var fill_light = DirectionalLight3D.new()
	fill_light.light_color = DubSarTheme.ACCENT_AMBER
	fill_light.light_energy = 0.35
	fill_light.rotation_degrees = Vector3(-30, 150, 0)
	add_child(fill_light)

func _setup_camera() -> void:
	_tower_center = Vector3(0, float(LAYERS.size()) * HEIGHT_STEP * 0.5, 0)
	_cam = Camera3D.new()
	_cam.fov = 55.0
	add_child(_cam)

func _build_tower() -> void:
	for i in range(LAYERS.size()):
		var layer: Dictionary = LAYERS[i]
		_add_ring(layer, i)
	# Ghost layer: dashed conceptually in the 2D artifact -- rendered here
	# as a translucent ring just above the real stack, same alarm-red band
	# color, clearly separated to avoid implying it's a real built layer.
	_add_ring(GHOST, LAYERS.size(), true)

func _add_ring(layer: Dictionary, index: int, is_ghost: bool = false) -> void:
	var band: Dictionary = BAND[layer["band"]]
	var radius: float = BASE_RADIUS + index * RADIUS_STEP
	var y: float = index * HEIGHT_STEP

	var mesh_inst = MeshInstance3D.new()
	var torus = TorusMesh.new()
	torus.inner_radius = radius - RING_THICKNESS
	torus.outer_radius = radius
	torus.rings = 6
	torus.ring_segments = 48
	mesh_inst.mesh = torus
	mesh_inst.position = Vector3(0, y, 0)

	var mat = StandardMaterial3D.new()
	mat.albedo_color = band["color"]
	mat.emission_enabled = true
	mat.emission = band["color"]
	mat.emission_energy_multiplier = 0.55
	if is_ghost:
		mat.transparency = BaseMaterial3D.TRANSPARENCY_ALPHA
		mat.albedo_color.a = 0.35
		mat.emission_energy_multiplier = 0.25
	mesh_inst.set_surface_override_material(0, mat)
	add_child(mesh_inst)

	# Real 3D click-picking (Godot's documented Area3D input_event pattern,
	# not hand-rolled raycasting). A thin CylinderShape3D shell approximates
	# the torus for picking purposes -- doesn't need to be pixel-exact, just
	# clickable near the visible ring.
	var area = Area3D.new()
	area.input_ray_pickable = true
	area.position = mesh_inst.position
	var col = CollisionShape3D.new()
	var shape = CylinderShape3D.new()
	shape.radius = radius
	shape.height = RING_THICKNESS * 2.0
	col.shape = shape
	area.add_child(col)
	area.input_event.connect(_on_ring_input_event.bind(layer, band))
	add_child(area)

func _on_ring_input_event(_camera: Node, event: InputEvent, _pos: Vector3, _normal: Vector3, _shape_idx: int, layer: Dictionary, band: Dictionary) -> void:
	if event is InputEventMouseButton and event.pressed and event.button_index == MOUSE_BUTTON_LEFT:
		_show_detail(layer, band)

func _build_ui() -> void:
	var canvas = CanvasLayer.new()
	add_child(canvas)

	var header = PanelContainer.new()
	header.add_theme_stylebox_override("panel", DubSarTheme.header_stylebox())
	header.set_anchors_preset(Control.PRESET_TOP_WIDE)
	canvas.add_child(header)

	var header_vbox = VBoxContainer.new()
	header.add_child(header_vbox)
	var title = Label.new()
	title.text = "𒁾  Onion Layers — BahyWay.Ecosystem v4.0 (25 layers, 163 crates, from Cargo.toml)"
	title.add_theme_font_size_override("font_size", DubSarTheme.SIZE_TITLE)
	title.add_theme_color_override("font_color", DubSarTheme.TEXT_PRIMARY)
	header_vbox.add_child(title)

	_status_label = Label.new()
	_status_label.text = "Drag to orbit · Scroll to zoom · Click a ring for detail · Escape to return"
	_status_label.add_theme_font_size_override("font_size", DubSarTheme.SIZE_LABEL)
	_status_label.add_theme_color_override("font_color", DubSarTheme.TEXT_DIM)
	header_vbox.add_child(_status_label)

	_detail_panel = PanelContainer.new()
	_detail_panel.add_theme_stylebox_override("panel", DubSarTheme.card_stylebox())
	_detail_panel.custom_minimum_size = Vector2(340, 0)
	_detail_panel.set_anchors_preset(Control.PRESET_TOP_RIGHT)
	_detail_panel.position = Vector2(-360, 80)
	canvas.add_child(_detail_panel)

	var detail_vbox = VBoxContainer.new()
	detail_vbox.add_theme_constant_override("separation", 6)
	_detail_panel.add_child(detail_vbox)

	_detail_title = Label.new()
	_detail_title.text = "Click a ring to inspect its layer."
	_detail_title.add_theme_font_size_override("font_size", DubSarTheme.SIZE_TITLE)
	_detail_title.add_theme_color_override("font_color", DubSarTheme.TEXT_PRIMARY)
	_detail_title.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	detail_vbox.add_child(_detail_title)

	_detail_body = RichTextLabel.new()
	_detail_body.bbcode_enabled = true
	_detail_body.fit_content = true
	_detail_body.custom_minimum_size = Vector2(0, 120)
	_detail_body.add_theme_color_override("default_color", DubSarTheme.TEXT_DIM)
	detail_vbox.add_child(_detail_body)

	var btn_back = Button.new()
	btn_back.text = "← Back (Esc)"
	btn_back.pressed.connect(func(): get_tree().change_scene_to_file("res://scenes/main_theater.tscn"))
	canvas.add_child(btn_back)
	btn_back.set_anchors_preset(Control.PRESET_TOP_RIGHT)
	btn_back.position = Vector2(-140, 10)

func _show_detail(layer: Dictionary, band: Dictionary) -> void:
	_detail_title.text = "%s — %s" % [layer["n"], layer["name"]]
	_detail_title.add_theme_color_override("font_color", band["color"])
	var crates: Array = layer["crates"]
	var body := "[color=#%s]%s[/color]  ·  %d crate(s)\n\n" % [band["color"].to_html(false), band["label"], crates.size()]
	for c in crates:
		body += "• %s\n" % c
	_detail_body.text = body

# ── Orbit camera: mouse-drag yaw/pitch + scroll-wheel zoom, both clamped.
# Standard Godot orbit pattern -- rebuild the transform from yaw/pitch each
# update rather than compounding rotate_x/rotate_y (which drifts/gimbals).
func _unhandled_input(event: InputEvent) -> void:
	if event is InputEventKey and event.pressed and not event.echo and event.keycode == KEY_ESCAPE:
		get_tree().change_scene_to_file("res://scenes/main_theater.tscn")
		return

	if event is InputEventMouseButton:
		if event.button_index == MOUSE_BUTTON_LEFT:
			_dragging = event.pressed
			_last_mouse = event.position
		elif event.button_index == MOUSE_BUTTON_WHEEL_UP and event.pressed:
			_distance = max(MIN_DIST, _distance - 2.5)
			_update_cam_transform()
		elif event.button_index == MOUSE_BUTTON_WHEEL_DOWN and event.pressed:
			_distance = min(MAX_DIST, _distance + 2.5)
			_update_cam_transform()
	elif event is InputEventMouseMotion and _dragging:
		var delta: Vector2 = event.position - _last_mouse
		_last_mouse = event.position
		_yaw -= delta.x * 0.008
		_pitch = clamp(_pitch - delta.y * 0.008, -1.4, 1.4)
		_update_cam_transform()

func _update_cam_transform() -> void:
	var basis = Basis.from_euler(Vector3(_pitch, _yaw, 0))
	_cam.global_position = _tower_center + basis * Vector3(0, 0, _distance)
	_cam.look_at(_tower_center, Vector3.UP)
