extends PanelContainer
class_name DashboardTheater
# =============================================================================
# DashboardTheater — the New DubSar Theater's tabbed 2D dashboard shell.
# 2026-07-21.
#
# Lives side by side with the existing 3D Theater (main_theater.tscn /
# theater_3d.tscn) as an alternative mode, per the Architect's explicit
# instruction -- neither replaces the other; a header button switches
# between them. Tabs across the top: all 7 EnkiDB Types, driven by
# EnkiEngines.TABLE (the same single source of truth theater_3d.gd's
# wizard and target list already use -- no second engine list invented
# here).
#
# Each tab is a Monitor Dashboard for that engine, populated from a REAL
# query against that engine's real, currently-deployed Read Node (via
# SumuUkinClient, the same binary-wire client every other real panel in
# this project now uses). Honesty boundary, stated plainly: HeptaScript
# has no COUNT/aggregate verb (the anti-SQL law -- see
# docs/08_pipeline_alaktu/HEPTASCRIPT_QUERY_LANGUAGE.md), so there is no real way to show a true
# global entity count without asking every one of possibly a billion
# particles. What this dashboard shows instead is real and honestly
# labeled: whether the engine is reachable right now, how many rows a
# bounded LIMIT probe actually returned (never presented as a total),
# and the real measured round-trip latency of that one query. No number
# on this screen is fabricated or hardcoded to look like live telemetry.
#
# The probe query itself is deliberately schema-agnostic (`WHO T.E
# HOW_MUCH LIMIT N`, no WHERE clause at all -- confirmed valid via
# heptascript::parser's own parse_minimal_who() test, since every WHERE/
# WHAT/HOW_MUCH clause is optional) because EnkiSDB/EnkiODB/EnkiQDB/
# EnkiDW have no fixed EAV schema (their attributes come from whatever
# CSV/TSV columns a real landing-zone file happened to carry) -- a fixed
# WHERE clause tuned to EnkiDB/EnkiDDB/EnkiMDB's own schemas would just
# silently return zero rows against the other four for the wrong reason
# (schema mismatch, not "no data yet"). CachedReadNode's real full-scan
# fallback path (crates/enkidb-readnode/src/cached.rs) is exactly what
# answers a WHERE-less query -- this was already proven this session
# (cached_serves_queries_readnode_would_reject test).
# =============================================================================

const SumuUkinClient = preload("res://scripts/sumuukin_client.gd")
const DubSarTheme = preload("res://scripts/dubsar_theme.gd")
const EnkiEngines = preload("res://scripts/enki_engines.gd")

const PROBE_LIMIT := 50
const PROBE_QUERY := "WHO T.E\nHOW_MUCH LIMIT %d" % PROBE_LIMIT

var _current_engine: String = "EnkiDB"
var _tab_buttons: Dictionary = {}   # engine name -> Button
var _content_root: VBoxContainer
var _busy := false

# Per-engine cached last result, so switching tabs back and forth doesn't
# refire a query until the Architect actually asks (Refresh button).
var _last_result: Dictionary = {}   # engine name -> {"ok","rows","elapsed_ms","error","checked_at"}

func _ready() -> void:
	_build()
	_choose_engine("EnkiDB")

func _build() -> void:
	var sb = StyleBoxFlat.new()
	sb.bg_color = DubSarTheme.BG
	add_theme_stylebox_override("panel", sb)

	var root = VBoxContainer.new()
	root.add_theme_constant_override("separation", 0)
	add_child(root)

	root.add_child(_build_header())
	root.add_child(_build_tab_bar())

	var scroll = ScrollContainer.new()
	scroll.size_flags_vertical = Control.SIZE_EXPAND_FILL
	scroll.horizontal_scroll_mode = ScrollContainer.SCROLL_MODE_DISABLED
	root.add_child(scroll)

	_content_root = VBoxContainer.new()
	_content_root.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	_content_root.add_theme_constant_override("separation", 16)
	var margin = MarginContainer.new()
	margin.add_theme_constant_override("margin_left", 24)
	margin.add_theme_constant_override("margin_top", 20)
	margin.add_theme_constant_override("margin_right", 24)
	margin.add_theme_constant_override("margin_bottom", 24)
	margin.add_child(_content_root)
	scroll.add_child(margin)

func _build_header() -> PanelContainer:
	var header = PanelContainer.new()
	header.add_theme_stylebox_override("panel", DubSarTheme.header_stylebox())

	var hbox = HBoxContainer.new()
	hbox.add_theme_constant_override("separation", 12)
	header.add_child(hbox)

	var title = Label.new()
	title.text = "𒁾 DubSar Theater — Dashboards"
	title.add_theme_font_size_override("font_size", DubSarTheme.SIZE_TITLE)
	title.add_theme_color_override("font_color", DubSarTheme.TEXT_PRIMARY)
	hbox.add_child(title)

	var spacer = Control.new()
	spacer.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	hbox.add_child(spacer)

	var btn_3d = Button.new()
	btn_3d.text = "⇄ Switch to 3D Theater (Grid & Orbit)"
	btn_3d.pressed.connect(func(): get_tree().change_scene_to_file("res://scenes/main_theater.tscn"))
	hbox.add_child(btn_3d)

	return header

func _unhandled_input(event: InputEvent) -> void:
	# Same Escape-returns-to-shell convention theater_3d.gd already uses.
	if event is InputEventKey and event.pressed and not event.echo and event.keycode == KEY_ESCAPE:
		get_tree().change_scene_to_file("res://scenes/main_theater.tscn")

func _build_tab_bar() -> PanelContainer:
	var bar = PanelContainer.new()
	var sb = StyleBoxFlat.new()
	sb.bg_color = DubSarTheme.BG_ELEVATED
	sb.border_width_bottom = 1
	sb.border_color = DubSarTheme.BORDER
	sb.content_margin_left = 16
	sb.content_margin_top = 6
	sb.content_margin_right = 16
	sb.content_margin_bottom = 6
	bar.add_theme_stylebox_override("panel", sb)

	var hbox = HBoxContainer.new()
	hbox.add_theme_constant_override("separation", 4)
	bar.add_child(hbox)

	for engine_name in EnkiEngines.TABLE.keys():
		var cfg: Dictionary = EnkiEngines.TABLE[engine_name]
		var btn = Button.new()
		btn.text = "%s  %s" % [cfg.get("icon", ""), engine_name]
		btn.flat = true
		btn.pressed.connect(_choose_engine.bind(engine_name))
		hbox.add_child(btn)
		_tab_buttons[engine_name] = btn

	return bar

func _choose_engine(engine_name: String) -> void:
	_current_engine = engine_name
	for name in _tab_buttons.keys():
		var btn: Button = _tab_buttons[name]
		var active = name == engine_name
		btn.add_theme_stylebox_override("normal", DubSarTheme.tab_stylebox(active, EnkiEngines.TABLE[name].get("color", DubSarTheme.ACCENT_CYAN)))
		var col = DubSarTheme.TEXT_PRIMARY if active else DubSarTheme.TEXT_DIM
		btn.add_theme_color_override("font_color", col)
	_render_engine(engine_name)
	if not _last_result.has(engine_name):
		_refresh_current()

func _render_engine(engine_name: String) -> void:
	for child in _content_root.get_children():
		child.queue_free()

	var cfg: Dictionary = EnkiEngines.TABLE[engine_name]
	var accent: Color = cfg.get("color", DubSarTheme.ACCENT_CYAN)

	# ── Title row ──
	var title_row = HBoxContainer.new()
	title_row.add_theme_constant_override("separation", 10)
	_content_root.add_child(title_row)

	var name_lbl = Label.new()
	name_lbl.text = "%s  %s" % [cfg.get("icon", ""), engine_name]
	name_lbl.add_theme_font_size_override("font_size", DubSarTheme.SIZE_TITLE + 4)
	name_lbl.add_theme_color_override("font_color", accent)
	title_row.add_child(name_lbl)

	title_row.add_child(DubSarTheme.pill(cfg.get("desc", ""), DubSarTheme.TEXT_DIM))
	if cfg.get("read_only", false):
		title_row.add_child(DubSarTheme.pill("READ NODE", accent))

	var spacer = Control.new()
	spacer.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	title_row.add_child(spacer)

	var btn_refresh = Button.new()
	btn_refresh.text = "⟳ Refresh" if not _busy else "…"
	btn_refresh.disabled = _busy
	btn_refresh.pressed.connect(_refresh_current)
	title_row.add_child(btn_refresh)

	var host_lbl = Label.new()
	host_lbl.text = "%s:%s" % [cfg.get("host", "192.168.122.112"), cfg.get("port", "?")]
	host_lbl.add_theme_font_size_override("font_size", DubSarTheme.SIZE_LABEL)
	host_lbl.add_theme_color_override("font_color", DubSarTheme.TEXT_FAINT)
	_content_root.add_child(host_lbl)

	# ── Stat tiles ──
	var result = _last_result.get(engine_name, null)
	var stats_row = HBoxContainer.new()
	stats_row.add_theme_constant_override("separation", 14)
	_content_root.add_child(stats_row)

	if result == null:
		stats_row.add_child(DubSarTheme.stat_tile("Status", "—", "not queried yet", DubSarTheme.TEXT_DIM))
	else:
		var reachable_text = "Reachable" if result.ok else "Unreachable"
		stats_row.add_child(DubSarTheme.stat_tile("Status", reachable_text, "checked %s" % result.checked_at, accent if result.ok else DubSarTheme.ACCENT_RED))
		if result.ok:
			var real_total = result.get("real_total", null)
			var sample_trend = "of %s total (real, via TRIBES:)" % real_total if real_total != null else "of up to %d requested" % PROBE_LIMIT
			stats_row.add_child(DubSarTheme.stat_tile("Sample rows", str(result.rows.size()), sample_trend, accent))
			stats_row.add_child(DubSarTheme.stat_tile("Query latency", "%d ms" % result.elapsed_ms, "this probe only", accent))
		else:
			stats_row.add_child(DubSarTheme.stat_tile("Error", "", result.error, DubSarTheme.ACCENT_RED))

	# ── Active query panel ──
	var query_card = PanelContainer.new()
	query_card.add_theme_stylebox_override("panel", DubSarTheme.card_stylebox())
	_content_root.add_child(query_card)

	var query_vbox = VBoxContainer.new()
	query_vbox.add_theme_constant_override("separation", 6)
	query_card.add_child(query_vbox)

	var query_label = Label.new()
	query_label.text = "ACTIVE PROBE QUERY  (schema-agnostic -- see this script's own header for why)"
	query_label.add_theme_font_size_override("font_size", DubSarTheme.SIZE_LABEL)
	query_label.add_theme_color_override("font_color", DubSarTheme.TEXT_DIM)
	query_vbox.add_child(query_label)

	var query_text = Label.new()
	query_text.text = PROBE_QUERY
	query_text.add_theme_font_size_override("font_size", DubSarTheme.SIZE_CODE)
	query_text.add_theme_color_override("font_color", accent)
	query_vbox.add_child(query_text)

	# ── Sample rows ──
	if result != null and result.ok and result.rows.size() > 0:
		var rows_label = Label.new()
		rows_label.text = "SAMPLE ROWS"
		rows_label.add_theme_font_size_override("font_size", DubSarTheme.SIZE_LABEL)
		rows_label.add_theme_color_override("font_color", DubSarTheme.TEXT_DIM)
		_content_root.add_child(rows_label)

		var rows_card = PanelContainer.new()
		rows_card.add_theme_stylebox_override("panel", DubSarTheme.card_stylebox())
		_content_root.add_child(rows_card)

		var rows_vbox = VBoxContainer.new()
		rows_vbox.add_theme_constant_override("separation", 4)
		rows_card.add_child(rows_vbox)

		var shown = min(result.rows.size(), 12)
		for i in range(shown):
			rows_vbox.add_child(_row_label(result.rows[i]))
		if result.rows.size() > shown:
			var more = Label.new()
			more.text = "… %d more row(s) not shown" % (result.rows.size() - shown)
			more.add_theme_font_size_override("font_size", DubSarTheme.SIZE_LABEL)
			more.add_theme_color_override("font_color", DubSarTheme.TEXT_FAINT)
			rows_vbox.add_child(more)
	elif result != null and result.ok:
		var empty_lbl = Label.new()
		empty_lbl.text = "No rows returned. Real, honest \"empty\" -- either this type has no data staged yet, or nothing matched within the probe LIMIT."
		empty_lbl.add_theme_color_override("font_color", DubSarTheme.TEXT_DIM)
		empty_lbl.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
		_content_root.add_child(empty_lbl)

func _row_label(row: Dictionary) -> Label:
	var kaki: String = str(row.get("kaki", "?"))
	var attrs_text := ""
	for pair in row.get("attrs", []):
		if pair is Array and pair.size() >= 2:
			if not attrs_text.is_empty():
				attrs_text += "  "
			attrs_text += "%s=%s" % [pair[0], pair[1]]
	var lbl = Label.new()
	lbl.text = "%s…  %s" % [kaki.left(12), attrs_text]
	lbl.add_theme_font_size_override("font_size", DubSarTheme.SIZE_CODE)
	lbl.add_theme_color_override("font_color", DubSarTheme.TEXT_PRIMARY)
	lbl.clip_text = true
	return lbl


# 2026-07-21: the real per-tribe particle-count corpus every *-read-server
# now also loads (crates/enkidb-readnode/src/materialize.rs, the
# Architect's "PU" idea) -- TRIBES: is a real, separate, cheap query
# (typically a handful of rows, one per distinct tribe) that lets this
# dashboard show a REAL total instead of the vague "of up to N requested"
# it showed before. Summed across every tribe.particle_count row returned
# -- most engines here have exactly one tribe today, but summing is
# correct either way and costs nothing extra.
const TRIBES_TOTAL_QUERY := "TRIBES:WHO T.E\nWHAT E[tribe.particle_count]\nHOW_MUCH LIMIT 50"

func _refresh_current() -> void:
	if _busy:
		return
	_busy = true
	var engine_name = _current_engine
	var cfg: Dictionary = EnkiEngines.TABLE[engine_name]
	var target = {"engine": engine_name, "host": cfg.get("host", "192.168.122.112"), "port": int(cfg.get("port", "0"))}

	_render_engine(engine_name)  # show the disabled "…" refresh button immediately

	var thread = Thread.new()
	thread.start(func():
		var result = SumuUkinClient.route([target], PROBE_QUERY, SumuUkinClient.FanOutPolicy.FIRST_ONLY)
		var tribes_result = SumuUkinClient.route([target], TRIBES_TOTAL_QUERY, SumuUkinClient.FanOutPolicy.FIRST_ONLY)
		call_deferred("_on_refresh_complete", engine_name, result, tribes_result, thread)
	)

func _on_refresh_complete(engine_name: String, result: Dictionary, tribes_result: Dictionary, thread: Thread) -> void:
	thread.wait_to_finish()
	_busy = false

	var pt = result.per_target[0] if not result.per_target.is_empty() else {"ok": false, "rows": [], "error": "no target contacted", "elapsed_ms": 0}

	# real_total stays null (never a fabricated number) unless the TRIBES:
	# corpus is genuinely reachable and returns at least one real row.
	var real_total = null
	var tp = tribes_result.per_target[0] if not tribes_result.per_target.is_empty() else {"ok": false, "rows": []}
	if tp.ok and not tp.rows.is_empty():
		var sum := 0
		for row in tp.rows:
			for pair in row.get("attrs", []):
				if pair is Array and pair.size() >= 2 and pair[0] == "tribe.particle_count":
					sum += int(str(pair[1]).to_int())
		real_total = sum

	_last_result[engine_name] = {
		"ok": pt.ok,
		"rows": pt.rows,
		"error": pt.get("error", ""),
		"elapsed_ms": pt.get("elapsed_ms", 0),
		"checked_at": Time.get_time_string_from_system(),
		"real_total": real_total,
	}
	if _current_engine == engine_name:
		_render_engine(engine_name)
