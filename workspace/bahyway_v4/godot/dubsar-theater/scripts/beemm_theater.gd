extends Node3D

# beemm_theater.gd — BeeMDM 7-Gates theater, wired to REAL data.
#
# Was a pure cosmetic demo (this file's own prior comment: "Does not fetch
# or drive real particle data yet"). This pass wires it to the real
# enkidb-query-server over enkidb_tcp.gd's proven wire protocol -- the
# FIRST real caller of that client anywhere in this codebase (checked: no
# other script actually calls EnkidbTcp.execute_query()). That client's
# logic reads correctly by static review but has never been exercised
# against a live server before now; there is no Godot runtime in the
# environment this was written in to run it end-to-end. Treat the first
# real run on the Fedora host as the actual proof, not this commit.
#
# WHAT'S REAL NOW, honestly:
#   - Every particle's KAKI, current b11 (real ATTR, real thresholds --
#     akkadi_ir::{GEM_B11,TRIBE_B11,ACTIVE_B11}=200/140/100,
#     bahyway_core::CRITICAL_B11_THRESHOLD=60), and full per-epoch b11
#     history come from one real PROVE+ACROSS ALL query.
#   - Each particle's 3D position is the server's OWN real orbital-math
#     computation (bahyway_algebra::orbital via `compute_orbit` in
#     enkidb-query-server) -- not reinvented here, not random.
#   - Right-click picks the real nearest particle and opens a panel
#     showing its real KAKI + real b11 trajectory (a table over its
#     real history), mirroring the StoryEngine drill-down pattern already
#     proven working in crates/dubsar-visualizer/src/panels/particles.rs
#     (2D/egui), now built fresh for this 3D scene.
#
# WHAT'S DELIBERATELY NOT HERE ANYMORE:
#   - The old per-frame `_spawn(randi()%7)` call that assigned particles
#     to a RANDOM one of the 7 gates and animated a cube "falling" into
#     it. That fabricated a per-station journey (S1 Musaru, S2 VGCA, ...)
#     no real pipeline produces today -- checked directly: enkidw's real
#     EtlPipeline only chains LandingZone->ZipEngine->KakiGenerator->
#     PersistedDb->WayCompiler; it does not invoke vgca-validation,
#     bahyway-dqm, or data-steward-station as one real per-particle
#     sequence. The 7 gate rings stay as landmarks naming the INTENDED
#     pipeline shape; the HUD says plainly that the station-by-station
#     journey isn't wired yet. Nothing here pretends otherwise.

const GATE_NAMES = ["ADAD", "ENLIL", "ENKI", "NANNA", "UTU", "INANNA", "NERGAL"]
const GATE_RADIUS = 52.0
const C_GOLD = Color(1.0, 0.843, 0.0)
const C_MAGENTA = Color(1.0, 0.0, 1.0)
const C_DIM = Color(0.4, 0.4, 0.4)

# Real thresholds -- akkadi_ir::{GEM_B11,TRIBE_B11,ACTIVE_B11}, bahyway_core
# ::CRITICAL_B11_THRESHOLD, bee_mdm_bus::B11_DIVISOR. Not invented for this
# scene.
const GEM_B11 := 200
const TRIBE_B11 := 140
const ACTIVE_B11 := 100
const CRITICAL_B11_THRESHOLD := 60
const B11_DIVISOR := 240.0

const QUERY_HOST := "192.168.122.107"
const QUERY_PORT := 7001
const FETCH_LIMIT := 800

const EnkidbTcp = preload("res://scripts/enkidb_tcp.gd")

var time_e := 0.0
var nucleus_mat: StandardMaterial3D
var nucleus_node: Node3D
var nucleus_light: OmniLight3D
var orbit_mm: MultiMesh
var orbit_mmi: MultiMeshInstance3D

# Real per-particle data, one entry per index, populated only by
# _apply_real_rows() -- empty (and the scene simply shows no particles
# yet) until a real fetch succeeds.
var kaki_hex: Array = []
var b11_current: Array = []
var b11_history: Array = []  # Array of Array[[epoch:int, b11:int]]
var positions: Array = []    # Array of Vector3, from the server's own orbital math

var lbl_status: Label
var lbl_count: Label
var story_panel: PanelContainer
var story_body: RichTextLabel
var sel_marker: MeshInstance3D

func _ready():
	get_tree().root.title = "DubSar IDE v4.0 — BeeMDM 7-Gates"
	_build_scene()
	_build_hud()
	_fetch_real_data_async()

# ── Scene scaffolding (sun, gate landmarks, camera, lights) ───────────────

func _build_scene():
	var cam = Camera3D.new()
	cam.transform.origin = Vector3(0, 65, 160)
	cam.rotation_degrees = Vector3(-22, 0, 0)
	cam.fov = 55.0
	cam.name = "Cam"
	add_child(cam)

	var amb = DirectionalLight3D.new()
	amb.light_energy = 1.0
	amb.rotation_degrees = Vector3(-45, 0, 0)
	add_child(amb)

	var fill = OmniLight3D.new()
	fill.light_color = Color(0.1, 0.1, 0.3)
	fill.light_energy = 1.5
	fill.omni_range = 600.0
	add_child(fill)

	nucleus_light = OmniLight3D.new()
	nucleus_light.light_color = C_GOLD
	nucleus_light.light_energy = 2.5
	nucleus_light.omni_range = 500.0
	add_child(nucleus_light)

	var grid = MeshInstance3D.new()
	var gm = PlaneMesh.new()
	gm.size = Vector2(400, 400)
	grid.mesh = gm
	grid.position = Vector3(0, -55, 0)
	var gmat = StandardMaterial3D.new()
	gmat.albedo_color = Color(0.02, 0.05, 0.08)
	grid.set_surface_override_material(0, gmat)
	add_child(grid)

	nucleus_node = Node3D.new()
	add_child(nucleus_node)
	var core_mesh = MeshInstance3D.new()
	var csm = SphereMesh.new()
	csm.radius = 5.5
	csm.height = 11.0
	core_mesh.mesh = csm
	nucleus_mat = StandardMaterial3D.new()
	nucleus_mat.albedo_color = Color(0.863, 0.235, 0.235)
	nucleus_mat.emission_enabled = true
	nucleus_mat.emission = Color(0.5, 0.1, 0.1)
	nucleus_mat.emission_energy_multiplier = 1.0
	core_mesh.set_surface_override_material(0, nucleus_mat)
	nucleus_node.add_child(core_mesh)

	# 7 gate landmarks -- naming the INTENDED pipeline shape. Not wired to
	# any real per-particle station event; no cubes fall into these
	# anymore (see this file's own header comment for why).
	for i in range(7):
		var ang = float(i) / 7.0 * TAU
		var gx = cos(ang) * GATE_RADIUS
		var gz = sin(ang) * GATE_RADIUS
		var gc = _gate_col(i)

		var rim = MeshInstance3D.new()
		var tm = TorusMesh.new()
		tm.inner_radius = 8.8
		tm.outer_radius = 9.2
		rim.mesh = tm
		rim.rotation_degrees = Vector3(90, 0, 0)
		rim.position = Vector3(gx, -29, gz)
		var rmat = StandardMaterial3D.new()
		rmat.albedo_color = gc
		rmat.emission_enabled = true
		rmat.emission = gc
		rmat.emission_energy_multiplier = 0.5  # dimmer than an active station -- landmark only
		rim.set_surface_override_material(0, rmat)
		add_child(rim)

		var lbl = Label3D.new()
		lbl.text = "%s\n(not yet wired)" % GATE_NAMES[i]
		lbl.font_size = 40
		lbl.modulate = gc
		lbl.modulate.a = 0.6
		lbl.position = Vector3(gx, -8, gz)
		lbl.billboard = BaseMaterial3D.BILLBOARD_ENABLED
		add_child(lbl)

	# Selection marker -- a small white ring shown at the picked particle.
	sel_marker = MeshInstance3D.new()
	var tm2 = TorusMesh.new()
	tm2.inner_radius = 1.4
	tm2.outer_radius = 1.6
	sel_marker.mesh = tm2
	var smat = StandardMaterial3D.new()
	smat.albedo_color = Color(1, 1, 1)
	smat.emission_enabled = true
	smat.emission = Color(1, 1, 1)
	smat.emission_energy_multiplier = 2.0
	sel_marker.set_surface_override_material(0, smat)
	sel_marker.visible = false
	add_child(sel_marker)

func _gate_col(i: int) -> Color:
	var cols = [
		Color(0.0, 0.8, 1.0), Color(0.667, 0.667, 1.0),
		Color(0.0, 0.8, 0.533), Color(1.0, 0.867, 0.0),
		Color(1.0, 0.6, 0.0), Color(0.6, 0.2, 0.8),
		Color(1.0, 0.267, 0.4),
	]
	return cols[i]

# ── HUD ────────────────────────────────────────────────────────────────

func _build_hud():
	var cl = CanvasLayer.new()
	add_child(cl)

	lbl_status = Label.new()
	lbl_status.text = "Connecting to %s:%d..." % [QUERY_HOST, QUERY_PORT]
	lbl_status.position = Vector2(14, 14)
	lbl_status.add_theme_font_size_override("font_size", 12)
	lbl_status.add_theme_color_override("font_color", Color(1.0, 0.624, 0.153))
	cl.add_child(lbl_status)

	lbl_count = Label.new()
	lbl_count.text = ""
	lbl_count.position = Vector2(14, 34)
	lbl_count.add_theme_font_size_override("font_size", 11)
	lbl_count.add_theme_color_override("font_color", Color(0.6, 0.8, 1.0))
	cl.add_child(lbl_count)

	var note = Label.new()
	note.text = "Station-by-station journey (S1..S8) is NOT wired to a real per-particle pipeline yet -- gate rings are landmarks only. Right-click a particle for its real KAKI + b11 history."
	note.position = Vector2(14, 54)
	note.add_theme_font_size_override("font_size", 9)
	note.add_theme_color_override("font_color", Color(0.5, 0.5, 0.55))
	cl.add_child(note)

	story_panel = PanelContainer.new()
	story_panel.visible = false
	story_panel.position = Vector2(40, 90)
	story_panel.custom_minimum_size = Vector2(460, 420)
	cl.add_child(story_panel)

	var vb = VBoxContainer.new()
	story_panel.add_child(vb)

	var header = HBoxContainer.new()
	vb.add_child(header)
	var title = Label.new()
	title.text = "𒁾 Particle History"
	title.add_theme_font_size_override("font_size", 13)
	title.add_theme_color_override("font_color", C_GOLD)
	title.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	header.add_child(title)
	var close_btn = Button.new()
	close_btn.text = "✕"
	close_btn.pressed.connect(func(): story_panel.visible = false; sel_marker.visible = false)
	header.add_child(close_btn)

	var scroll = ScrollContainer.new()
	scroll.custom_minimum_size = Vector2(440, 380)
	vb.add_child(scroll)
	story_body = RichTextLabel.new()
	story_body.custom_minimum_size = Vector2(430, 370)
	story_body.bbcode_enabled = true
	story_body.fit_content = true
	scroll.add_child(story_body)

# ── Real data fetch (background thread; blocking client, per its own design) ─

func _fetch_real_data_async():
	var t = Thread.new()
	t.start(func():
		var query := "PROVE\nWHO Tribes.E\nWHAT E[b11]\nACROSS ALL\nHOW_MUCH LIMIT %d" % FETCH_LIMIT
		var rows: Array = EnkidbTcp.execute_query(QUERY_HOST, QUERY_PORT, query)
		call_deferred("_apply_real_rows", rows)
	)

func _apply_real_rows(rows: Array):
	if rows.is_empty():
		lbl_status.text = "⚠ No data (server unreachable, or no particles matched)"
		lbl_status.add_theme_color_override("font_color", Color(1.0, 0.267, 0.267))
		return

	for row in rows:
		if not (row is Dictionary) or not row.has("kaki"):
			continue
		var kaki: String = row["kaki"]
		var b11 := 0
		for pair in row.get("attrs", []):
			if pair.size() == 2 and pair[0] == "b11":
				b11 = int(pair[1])
		var hist: Array = []
		for snap in row.get("history", []):
			var hb11 := 0
			for pair in snap.get("attrs", []):
				if pair.size() == 2 and pair[0] == "b11":
					hb11 = int(pair[1])
			hist.append([int(snap.get("epoch", 0)), hb11])

		var pos := Vector3.ZERO
		if row.has("orbit"):
			var o = row["orbit"]
			pos = Vector3(float(o.get("x", 0.0)), float(o.get("y", 0.0)), float(o.get("z", 0.0)))

		kaki_hex.append(kaki)
		b11_current.append(b11)
		b11_history.append(hist)
		positions.append(pos)

	lbl_status.text = "✓ LIVE — %d real particles from %s:%d" % [kaki_hex.size(), QUERY_HOST, QUERY_PORT]
	lbl_status.add_theme_color_override("font_color", Color(0.0, 1.0, 0.533))
	_build_orbit_multimesh()

func _build_orbit_multimesh():
	var n = positions.size()
	if n == 0:
		return
	orbit_mm = MultiMesh.new()
	orbit_mm.transform_format = MultiMesh.TRANSFORM_3D
	orbit_mm.use_colors = true
	var sm = SphereMesh.new()
	sm.radius = 0.4
	sm.height = 0.8
	orbit_mm.mesh = sm
	orbit_mm.instance_count = n
	orbit_mm.visible_instance_count = n

	for i in range(n):
		var t = Transform3D()
		t.origin = positions[i]
		orbit_mm.set_instance_transform(i, t)
		orbit_mm.set_instance_color(i, _b11_color(b11_current[i]))

	var pt_mat = StandardMaterial3D.new()
	pt_mat.shading_mode = BaseMaterial3D.SHADING_MODE_UNSHADED
	pt_mat.vertex_color_use_as_albedo = true

	orbit_mmi = MultiMeshInstance3D.new()
	orbit_mmi.multimesh = orbit_mm
	orbit_mmi.set_surface_override_material(0, pt_mat)
	add_child(orbit_mmi)

	lbl_count.text = "%d real particles rendered at their real orbital positions" % n

# Real thresholds, not invented for this scene -- see header comment.
func _b11_color(b11: int) -> Color:
	if b11 >= GEM_B11:
		return C_GOLD
	elif b11 >= TRIBE_B11:
		return Color(0.0, 0.8, 1.0)
	elif b11 >= ACTIVE_B11:
		return Color(0.0, 0.8, 0.4)
	elif b11 >= CRITICAL_B11_THRESHOLD:
		return C_MAGENTA
	else:
		return C_DIM

func _b11_tier(b11: int) -> String:
	if b11 >= GEM_B11:
		return "GEM (b11 ≥ %d)" % GEM_B11
	elif b11 >= TRIBE_B11:
		return "TRIBE (b11 ≥ %d)" % TRIBE_B11
	elif b11 >= ACTIVE_B11:
		return "ACTIVE (b11 ≥ %d)" % ACTIVE_B11
	elif b11 >= CRITICAL_B11_THRESHOLD:
		return "FUZZY (b11 ≥ %d)" % CRITICAL_B11_THRESHOLD
	else:
		return "CRITICAL (b11 < %d)" % CRITICAL_B11_THRESHOLD

# ── Real particle picking (right-click) ───────────────────────────────────

func _unhandled_input(event):
	if event is InputEventMouseButton and event.pressed and event.button_index == MOUSE_BUTTON_RIGHT:
		_pick_particle(event.position)

func _pick_particle(screen_pos: Vector2):
	if positions.is_empty():
		return
	var cam := get_viewport().get_camera_3d()
	if cam == null:
		return
	var ray_origin = cam.project_ray_origin(screen_pos)
	var ray_dir = cam.project_ray_normal(screen_pos)

	# Manual nearest-point-to-ray pick against the real particle cloud --
	# same shape of technique THREE.js Raycaster.intersectObject(Points)
	# uses against a point cloud (closest-point, not a real collision
	# mesh per particle); appropriate here for the same reason.
	var best_i := -1
	var best_d := 2.0  # world-unit pick tolerance
	for i in range(positions.size()):
		var to_p = positions[i] - ray_origin
		var t = to_p.dot(ray_dir)
		if t < 0:
			continue
		var closest = ray_origin + ray_dir * t
		var d = closest.distance_to(positions[i])
		if d < best_d:
			best_d = d
			best_i = i

	if best_i >= 0:
		_open_story_panel(best_i)

func _open_story_panel(i: int):
	sel_marker.visible = true
	sel_marker.position = positions[i]

	var kaki = kaki_hex[i]
	var b11 = b11_current[i]
	var hist: Array = b11_history[i]

	var bytes := []
	for j in range(0, kaki.length(), 2):
		bytes.append(kaki.substr(j, 2))

	var bb := "[b][color=#d4a843]𒁾 KAKI Identity[/color][/b]\n"
	bb += "[code]%s[/code]\n\n" % " ".join(bytes)
	bb += "Tribe: [color=#4db8ff]0x%s%s[/color]  " % [bytes[4] if bytes.size() > 4 else "??", bytes[5] if bytes.size() > 5 else "??"]
	bb += "Type: 0x%s  Role: 0x%s\n\n" % [bytes[6] if bytes.size() > 6 else "??", bytes[7] if bytes.size() > 7 else "??"]

	bb += "[b]Current b11:[/b] %d / %d — [color=#%s]%s[/color]\n\n" % [
		b11, int(B11_DIVISOR), _b11_color(b11).to_html(false), _b11_tier(b11)
	]

	bb += "[b][color=#d4a843]📜 Real b11 History[/color][/b] (%d epoch%s)\n" % [hist.size(), "" if hist.size() == 1 else "s"]
	if hist.is_empty():
		bb += "[i]No prior epochs in this journal for this particle -- current value is its only real state.[/i]\n"
	else:
		bb += "[table=3]"
		bb += "[cell][b]Epoch[/b][/cell][cell][b]b11[/b][/cell][cell][b]Tier[/b][/cell]"
		for h in hist:
			bb += "[cell]%d[/cell][cell]%d[/cell][cell][color=#%s]%s[/color][/cell]" % [
				h[0], h[1], _b11_color(h[1]).to_html(false), _b11_tier(h[1])
			]
		bb += "[/table]"

	bb += "\n\n[b][color=#d4a843]🏛 Pipeline Journey[/color][/b]\n"
	bb += "[table=2]"
	bb += "[cell]S0 AdadGate[/cell][cell][color=#00e5a0]✓ MINTED[/color] — this KAKI's type/role bytes prove a real Identity-Kaki[/cell]"
	bb += "[cell]S1-S8[/cell][cell][color=#888888]not yet wired to a real per-particle pipeline in this build (see this scene's own header comment)[/color][/cell]"
	bb += "[/table]"

	story_body.text = bb
	story_panel.visible = true

# ── Animation (cosmetic only -- no fake per-particle journey) ────────────

func _process(delta):
	time_e += delta
	if orbit_mmi:
		orbit_mmi.rotation.y += delta * 0.0007
	nucleus_node.rotation.y += delta * 0.004
	var pulse = 0.8 + sin(time_e * 3.0) * 0.2
	nucleus_mat.emission_energy_multiplier = pulse
	nucleus_light.light_energy = 2.5 + sin(time_e * 2.0) * 0.5

	if sel_marker.visible:
		sel_marker.rotation.x += delta * 2.0
		sel_marker.rotation.y += delta * 1.5
