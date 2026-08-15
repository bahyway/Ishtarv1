extends Control
class_name DubSarPdmWorkspace
# =============================================================================
# DubSarPdmWorkspace — 2026-07-24, the real landing scene after login.
#
# Deliberately minimal for this first pass: proves the full chain works
# end to end (Godot scene -> PdmBridge GDExtension -> bahyway-algebra /
# najaf-engine's real, tested Rust math -> back to GDScript, displayed) --
# not yet the full Houdini-style procedural node editor task #116 scoped.
# That's real, separate future work; this is the "start light" first
# rung, matching the Architect's own plan for how DubSar's plugin
# architecture should grow.
#
# The demo below runs three real PdmBridge calls against a genuine
# 2-simplex (triangle) in 7D and displays their actual results -- not
# canned/fake output. If pdm-gdext isn't loaded, every call below fails
# honestly (Godot logs a missing-class error) rather than silently
# showing fabricated numbers.
# =============================================================================

const DubSarTheme = preload("res://scripts/dubsar_theme.gd")
const SessionIdentity = preload("res://scripts/session_identity.gd")

var _bridge: PdmBridge
var _log: RichTextLabel

func _ready() -> void:
	get_tree().root.title = "BahyWay.Ecosystem v4.0 — DubSar PDM"
	DubSarTheme.apply_page_background(self)
	_bridge = PdmBridge.new()
	_build_ui()

func _build_ui() -> void:
	var root_vb = VBoxContainer.new()
	root_vb.set_anchors_preset(Control.PRESET_FULL_RECT)
	root_vb.add_theme_constant_override("separation", 0)
	add_child(root_vb)

	# ── Header ───────────────────────────────────────────────────────
	var header = PanelContainer.new()
	header.add_theme_stylebox_override("panel", DubSarTheme.header_stylebox())
	root_vb.add_child(header)

	var header_hb = HBoxContainer.new()
	header.add_child(header_hb)

	var title = Label.new()
	title.text = "𒁾 DubSar PDM — Particle Data Modeling"
	title.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	title.add_theme_font_size_override("font_size", DubSarTheme.SIZE_TITLE)
	title.add_theme_color_override("font_color", DubSarTheme.TEXT_PRIMARY)
	header_hb.add_child(title)

	var identity_lbl = Label.new()
	identity_lbl.text = "%s (%s)" % [SessionIdentity.identity, SessionIdentity.passport_type]
	identity_lbl.add_theme_font_size_override("font_size", DubSarTheme.SIZE_LABEL)
	identity_lbl.add_theme_color_override("font_color", DubSarTheme.TEXT_DIM)
	header_hb.add_child(identity_lbl)

	# ── Body ─────────────────────────────────────────────────────────
	var body_margin = MarginContainer.new()
	body_margin.size_flags_vertical = Control.SIZE_EXPAND_FILL
	body_margin.add_theme_constant_override("margin_left", 20)
	body_margin.add_theme_constant_override("margin_top", 16)
	body_margin.add_theme_constant_override("margin_right", 20)
	body_margin.add_theme_constant_override("margin_bottom", 16)
	root_vb.add_child(body_margin)

	var body_vb = VBoxContainer.new()
	body_vb.add_theme_constant_override("separation", 12)
	body_margin.add_child(body_vb)

	var intro = Label.new()
	intro.text = "Real math, wrapped, not reimplemented: bahyway-algebra::topology (20 canonical operators, 115 passing tests) + najaf-engine::topology (simplicial-complex machinery), via pdm-gdext."
	intro.autowrap_mode = TextServer.AUTOWRAP_WORD
	intro.add_theme_font_size_override("font_size", DubSarTheme.SIZE_BODY)
	intro.add_theme_color_override("font_color", DubSarTheme.TEXT_DIM)
	body_vb.add_child(intro)

	var btn_run = Button.new()
	btn_run.text = "▶ Run Simplicial Complex Demo"
	btn_run.add_theme_color_override("font_color", DubSarTheme.ACCENT_GREEN)
	btn_run.pressed.connect(_on_run_demo_pressed)
	body_vb.add_child(btn_run)

	var log_panel = PanelContainer.new()
	log_panel.add_theme_stylebox_override("panel", DubSarTheme.card_stylebox())
	log_panel.size_flags_vertical = Control.SIZE_EXPAND_FILL
	body_vb.add_child(log_panel)

	_log = RichTextLabel.new()
	_log.bbcode_enabled = true
	_log.scroll_following = true
	_log.size_flags_vertical = Control.SIZE_EXPAND_FILL
	_log.add_theme_font_size_override("normal_font_size", DubSarTheme.SIZE_CODE)
	_log.add_theme_color_override("default_color", DubSarTheme.TEXT_PRIMARY)
	log_panel.add_child(_log)

	_log_line("[color=#%s]Ready. Click \"Run Simplicial Complex Demo\" to call real pdm-gdext functions.[/color]" %
		DubSarTheme.TEXT_DIM.to_html(false))

# ── The real demo ────────────────────────────────────────────────────

func _on_run_demo_pressed() -> void:
	_log.clear()

	# A 2-simplex (triangle) in the plane, embedded in 7D with the
	# remaining axes at zero -- three real vertices, not fabricated.
	var v0 := PackedFloat64Array([0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0])
	var v1 := PackedFloat64Array([4.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0])
	var v2 := PackedFloat64Array([0.0, 4.0, 0.0, 0.0, 0.0, 0.0, 0.0])
	var vertices: Array[PackedFloat64Array] = [v0, v1, v2]

	_log_line("[b]Triangle vertices:[/b] (0,0) (4,0) (0,4) — embedded in 7D")
	_log_line("")

	# 1. particle_in_complex — centroid should be inside, a far point outside.
	var centroid := PackedFloat64Array([1.333, 1.333, 0.0, 0.0, 0.0, 0.0, 0.0])
	var far_point := PackedFloat64Array([10.0, 10.0, 0.0, 0.0, 0.0, 0.0, 0.0])

	var inside_result := _bridge.particle_in_complex(centroid, vertices)
	var outside_result := _bridge.particle_in_complex(far_point, vertices)
	_log_result("particle_in_complex(centroid)", inside_result)
	_log_result("particle_in_complex(far point)", outside_result)
	_log_line("")

	# 2. boundary_n — the triangle's boundary faces (three edges).
	var faces_result := _bridge.boundary_n(vertices)
	if faces_result.get("ok", false):
		var faces: Array = faces_result["faces"]
		_log_line("[color=#%s]boundary_n -> %d face(s)[/color]" %
			[DubSarTheme.ACCENT_GREEN.to_html(false), faces.size()])
	else:
		_log_line("[color=#%s]boundary_n FAILED: %s[/color]" %
			[DubSarTheme.ACCENT_RED.to_html(false), faces_result.get("error", "?")])
	_log_line("")

	# 3. reconstruct_ghost — estimate a missing 4th point from the three
	# known vertices (their centroid).
	var ghost_result := _bridge.reconstruct_ghost(vertices)
	if ghost_result.get("ok", false):
		var ghost: PackedFloat64Array = ghost_result["ghost"]
		_log_line("[color=#%s]reconstruct_ghost -> (%.3f, %.3f, ...)[/color]" %
			[DubSarTheme.ACCENT_CYAN.to_html(false), ghost[0], ghost[1]])
	else:
		_log_line("[color=#%s]reconstruct_ghost FAILED: %s[/color]" %
			[DubSarTheme.ACCENT_RED.to_html(false), ghost_result.get("error", "?")])

func _log_result(label: String, result: Dictionary) -> void:
	if not result.get("ok", false):
		_log_line("[color=#%s]%s FAILED: %s[/color]" %
			[DubSarTheme.ACCENT_RED.to_html(false), label, result.get("error", "?")])
		return
	var value: bool = result.get("value", false)
	var color := DubSarTheme.ACCENT_GREEN if value else DubSarTheme.ACCENT_AMBER
	_log_line("[color=#%s]%s -> %s[/color]" % [color.to_html(false), label, value])

func _log_line(text: String) -> void:
	_log.append_text(text + "\n")
