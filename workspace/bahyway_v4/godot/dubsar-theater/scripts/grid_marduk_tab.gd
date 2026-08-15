extends MarginContainer
# =============================================================================
# Grid & Marduk tab -- 2026-07-25. DubSar Theater's window onto EGDEngine
# (Gibil Calculus) and MardukEngine's first slice.
#
# Real computation, demo data: dubsar-gridnav-gd and marduk-gdext are both
# real, tested GDExtension bridges wrapping already-tested Rust crates
# (state_estimate, exclusion, dispatch, horizon in egd-engine; position,
# horizon, topology in marduk-engine). What's synthetic here is the SOURCE
# data -- a small 3-bus radial feeder and a figure-eight relation graph,
# not real telemetry -- because the live EnkiDB feed into these engines is
# a separate, still-deferred item (see docs/07_file_formats/GL-EGD-001.md). Every number
# shown is a real function call through the FFI boundary, not a mock.
# =============================================================================

const DubSarTheme = preload("res://scripts/dubsar_theme.gd")

var _grid: GridNavLens
var _marduk: MardukBridge
var _results_vbox: VBoxContainer

func _ready() -> void:
	_grid = GridNavLens.new()
	_marduk = MardukBridge.new()
	_build_ui()
	_run_demo()

func _build_ui() -> void:
	var root_vb = VBoxContainer.new()
	root_vb.add_theme_constant_override("separation", 10)
	add_child(root_vb)

	var title = Label.new()
	title.text = "Grid & Marduk -- EGDEngine (Gibil) + MardukEngine live bridge"
	title.add_theme_font_size_override("font_size", DubSarTheme.SIZE_TITLE)
	title.add_theme_color_override("font_color", DubSarTheme.TEXT_PRIMARY)
	root_vb.add_child(title)

	var subtitle = Label.new()
	subtitle.text = "Real state estimation, KCL residuals, N-1 contingency, dispatch priority, and Betti topology, computed through the FFI boundary on a demo feeder, not mocked. Live EnkiDB feed remains a deferred item (docs/07_file_formats/GL-EGD-001.md)."
	subtitle.autowrap_mode = TextServer.AUTOWRAP_WORD
	subtitle.add_theme_font_size_override("font_size", DubSarTheme.SIZE_LABEL)
	subtitle.add_theme_color_override("font_color", DubSarTheme.TEXT_DIM)
	root_vb.add_child(subtitle)

	var btn_refresh = Button.new()
	btn_refresh.text = "Recompute"
	btn_refresh.pressed.connect(_run_demo)
	root_vb.add_child(btn_refresh)

	var scroll = ScrollContainer.new()
	scroll.size_flags_vertical = Control.SIZE_EXPAND_FILL
	root_vb.add_child(scroll)

	_results_vbox = VBoxContainer.new()
	_results_vbox.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	_results_vbox.add_theme_constant_override("separation", 4)
	scroll.add_child(_results_vbox)

func _section(text: String) -> void:
	var lbl = Label.new()
	lbl.text = text
	lbl.add_theme_font_size_override("font_size", DubSarTheme.SIZE_BODY)
	lbl.add_theme_color_override("font_color", DubSarTheme.ACCENT_CYAN)
	_results_vbox.add_child(lbl)

func _row(text: String, accent: Color = DubSarTheme.TEXT_PRIMARY) -> void:
	var panel = PanelContainer.new()
	panel.add_theme_stylebox_override("panel", DubSarTheme.card_stylebox(6, DubSarTheme.BORDER))
	var lbl = Label.new()
	lbl.text = text
	lbl.add_theme_font_size_override("font_size", DubSarTheme.SIZE_LABEL)
	lbl.add_theme_color_override("font_color", accent)
	panel.add_child(lbl)
	_results_vbox.add_child(panel)

func _run_demo() -> void:
	for child in _results_vbox.get_children():
		child.queue_free()

	_run_egdengine_demo()
	_run_mardukengine_demo()

func _complex_mul(a_re: float, a_im: float, b_re: float, b_im: float) -> Vector2:
	return Vector2(a_re * b_re - a_im * b_im, a_re * b_im + a_im * b_re)

# EGDEngine: 3-bus radial feeder, feeder-head measured, node 1/2 estimated.
func _run_egdengine_demo() -> void:
	_section("EGDEngine -- Gibil Calculus (3-bus radial feeder demo)")

	# Ground truth used only to derive consistent injections (same
	# construction egd-engine's own round-trip test uses) -- the
	# estimator below is NOT given these voltages except at node 0, the
	# "measured" feeder head; nodes 1 and 2 are solved from topology.
	var y_re := 8.0
	var y_im := -3.0
	var v_true := [Vector2(1.0, 0.0), Vector2(0.97, -0.01), Vector2(0.94, -0.02)]
	var i01 := _complex_mul(y_re, y_im, v_true[0].x - v_true[1].x, v_true[0].y - v_true[1].y)
	var i12 := _complex_mul(y_re, y_im, v_true[1].x - v_true[2].x, v_true[1].y - v_true[2].y)

	# GridNavLens fixes a node's injection at add_node time (matching
	# fdd_core::Node -- injection is a required field, not settable
	# later), so the correct injections are computed above BEFORE
	# construction, not patched in afterward.
	var n0 := _grid.add_node(-i01.x, -i01.y)
	var n1 := _grid.add_node(i01.x - i12.x, i01.y - i12.y)
	var n2 := _grid.add_node(i12.x, i12.y)
	_grid.add_edge(n0, n1, y_re, y_im, true)
	_grid.add_edge(n1, n2, y_re, y_im, true)

	_grid.set_known_voltage(n0, v_true[0].x, v_true[0].y)
	var ok := _grid.estimate_voltages()
	_row("estimate_voltages() from feeder-head measurement only: %s" % ("OK" if ok else "FAILED (unexpected -- check topology)"), DubSarTheme.ACCENT_GREEN if ok else DubSarTheme.ACCENT_RED)
	if ok:
		for i in range(3):
			var v: Vector2 = _grid.estimated_voltage(i)
			var kappa := _grid.node_kappa(i)
			# FIXED (2026-07-27): Godot's String `%` operator has no `%e`
			# (scientific notation) format character -- reproduced live
			# as "String formatting error: unsupported format character."
			# String.num_scientific() is the Godot-native equivalent.
			_row("  node %d: estimated V=(%.4f, %.4f)  KCL residual kappa=%s" % [i, v.x, v.y, String.num_scientific(kappa)], DubSarTheme.TEXT_DIM)

	# FIXED (2026-07-27): both KAKI hex literals were 31 characters, one
	# hex digit short of the required 32 (16 bytes) -- GridNavLens.add_asset
	# correctly rejected them as malformed, reproduced live. Each was
	# missing the last digit of its final byte pair (...ee"f" -> ...ee"ff",
	# ...88"9" -> ...88"99").
	var kaki_a := "00112233445566778899aabbccddeeff"
	var kaki_b := "aabbccddeeff00112233445566778899"
	_grid.add_asset(kaki_a, 1, true, 0, 33.3, 44.4, 0.0)
	_grid.add_asset(kaki_b, 1, true, 1, 33.31, 44.41, 0.0)
	_grid.push_kappa(0, -50.0, 0.2)
	_grid.push_kappa(0, 0.0, 0.85)
	_grid.push_kappa(1, -50.0, 0.1)
	_grid.push_kappa(1, 0.0, 0.15)

	for i in range(2):
		var tier := _grid.asset_tier(i)
		var tier_name: String = ["SOUND", "HORIZON", "BIRQU"][tier]
		var days := _grid.horizon_days(i)
		var n1_unserved := _grid.n1_unserved_count(i, 1e-6)
		var priority := _grid.dispatch_priority_for_asset(i, 1e-6)
		var accent: Color = [DubSarTheme.ACCENT_GREEN, DubSarTheme.ACCENT_AMBER, DubSarTheme.ACCENT_RED][tier]
		_row("Asset %d: tier=%s horizon_days=%.1f  N-1(this edge) unserved=%s  dispatch_priority=%.2f" % [i, tier_name, days, str(n1_unserved), priority], accent)

# MardukEngine: Position + Horizon + Topology, all real calls.
func _run_mardukengine_demo() -> void:
	_section("MardukEngine -- Position / Horizon / Topology")

	var p := PackedFloat64Array([10, 20, 30, 40, 50, 60, 70])
	var t := PackedFloat64Array([12, 18, 33, 38, 55, 58, 66])
	var w := PackedFloat64Array([1, 1, 1, 1, 1, 1, 1])
	var r := _marduk.radial_coordinate(p, t, w)
	var hp := _marduk.template_score(p, t, w)
	_row("Position: r=%.3f  H(P)=%.4f  (particle vs. template, 7D Hepta Space)" % [r, hp])

	var declining_b11 := PackedFloat64Array()
	for i in range(30):
		declining_b11.append(-(30 - i))
		declining_b11.append(220.0 + (30 - i) * 1.0)
	var horizon_days := _marduk.golden_horizon_days(declining_b11)
	var horizon_text := ("%.1f days to GEM floor" % horizon_days) if horizon_days >= 0.0 else "stable/improving"
	var horizon_accent = DubSarTheme.ACCENT_AMBER if (horizon_days >= 0.0 and horizon_days <= 90.0) else DubSarTheme.TEXT_PRIMARY
	_row("Horizon: declining B11 series -> %s" % horizon_text, horizon_accent)

	# Figure-eight relation graph: the cheap analog of a mule-ring /
	# illegal-connection-ring signature (two overlapping cycles sharing
	# one node).
	var edges := PackedInt64Array([0, 1, 1, 2, 2, 0, 2, 3, 3, 4, 4, 2])
	var betti: Vector2i = _marduk.betti_numbers(5, edges)
	var ring_text := "%d independent loop(s) -- mule-ring/illegal-connection-ring signature if >0" % betti.y
	_row("Topology: beta0=%d beta1=%d (%s)" % [betti.x, betti.y, ring_text], DubSarTheme.ACCENT_RED if betti.y > 0 else DubSarTheme.ACCENT_GREEN)
