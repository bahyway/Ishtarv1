extends MarginContainer
# =============================================================================
# AR-EN Navigator tab — 2026-07-24.
#
# Bilingual (Arabic/English) navigation-term glossary for the Najaf/Baghdad
# projects, requested by the Architect as a DubSar IDE plugin. Wraps two
# real crates via navi-translate-gdext (see that crate's own lib.rs header
# for the full honesty note): navi-engine's ATTR_NAME/ATTR_NAME_ARABIC EAV
# convention (the same one already used for real in this project's own PDM
# tab default query, `ATTR person.name_arabic`), and adapa-recall's real
# TF-IDF + cosine retrieval.
#
# HONEST SCOPE: this is a GLOSSARY (exact/fuzzy name lookup for places —
# "which grave/gate/shrine does this Arabic name refer to"), not free-text
# machine translation. That is a deliberate, narrower scope, matching what
# the Architect's own question about vectorizing Arabic names implied and
# what NaviTranslateBridge.lookup_by_name() actually does under the hood
# (see navi-translate-gdext/src/lib.rs).
#
# The seed list below (_seed_gazetteer) is a small set of PLACEHOLDER
# entries -- illustrative Najaf/Baghdad navigation-term shapes, NOT
# verified real coordinates or Architect-confirmed Arabic spellings.
# Replace with the real surveyed Wadi al-Salam / Baghdad POI set (e.g.
# once NajafEngine/WPDEngine field data is digitized) via "+ Add Place"
# below, or by editing _seed_gazetteer directly. Never presented to the
# stakeholder as verified data -- the UI labels it "seed / placeholder".
# =============================================================================

const DubSarTheme = preload("res://scripts/dubsar_theme.gd")

var _bridge: NaviTranslateBridge
var _gazetteer: Array = []   # Array[Dictionary{name, name_arabic, lat, lon}]

var _query_edit: LineEdit
var _results_log: RichTextLabel
var _add_name: LineEdit
var _add_name_arabic: LineEdit
var _add_lat: LineEdit
var _add_lon: LineEdit
var _gazetteer_count_lbl: Label

func _seed_gazetteer() -> Array:
	# Placeholder shapes only -- see this file's header. lat/lon fall
	# inside Wadi al-Salam's real bounding box (navi-engine's own
	# ParticleMap::najaf_map(): 31.960-32.040N, 44.290-44.360E) so
	# distance/nearest calls are at least geographically plausible, but
	# these are NOT surveyed positions.
	return [
		{"name": "Wadi al-Salam Main Gate", "name_arabic": "البوابة الرئيسية لوادي السلام", "lat": 31.997, "lon": 44.325},
		{"name": "Imam Ali Shrine",         "name_arabic": "مرقد الإمام علي",              "lat": 32.001, "lon": 44.323},
		{"name": "Najaf Old City Gate",     "name_arabic": "باب المدينة القديمة",           "lat": 31.999, "lon": 44.331},
		{"name": "Wadi al-Salam North Block", "name_arabic": "الحي الشمالي لوادي السلام",   "lat": 32.010, "lon": 44.320},
		{"name": "Najaf-Baghdad Highway Junction", "name_arabic": "تقاطع طريق النجف بغداد",  "lat": 32.030, "lon": 44.340},
	]

func _ready() -> void:
	_bridge = NaviTranslateBridge.new()
	_gazetteer = _seed_gazetteer()
	_build()

func _build() -> void:
	var root_vb = VBoxContainer.new()
	root_vb.add_theme_constant_override("separation", 10)
	add_child(root_vb)

	var title = Label.new()
	title.text = "𒁾  AR-EN Navigator — bilingual navigation-term glossary (Najaf / Baghdad)"
	title.add_theme_font_size_override("font_size", DubSarTheme.SIZE_TITLE)
	title.add_theme_color_override("font_color", DubSarTheme.TEXT_PRIMARY)
	root_vb.add_child(title)

	var subtitle = Label.new()
	subtitle.text = "Real TF-IDF + cosine lookup (adapa-recall) over ATTR_NAME / ATTR_NAME_ARABIC (navi-engine) — glossary lookup, not machine translation."
	subtitle.autowrap_mode = TextServer.AUTOWRAP_WORD
	subtitle.add_theme_font_size_override("font_size", DubSarTheme.SIZE_LABEL)
	subtitle.add_theme_color_override("font_color", DubSarTheme.TEXT_DIM)
	root_vb.add_child(subtitle)

	var hsplit = HSplitContainer.new()
	hsplit.size_flags_vertical = Control.SIZE_EXPAND_FILL
	root_vb.add_child(hsplit)

	hsplit.add_child(_build_lookup_panel())
	hsplit.add_child(_build_add_panel())

	_render_results_hint()

func _build_lookup_panel() -> PanelContainer:
	var panel = PanelContainer.new()
	panel.add_theme_stylebox_override("panel", DubSarTheme.card_stylebox())
	panel.custom_minimum_size = Vector2(420, 0)
	panel.size_flags_horizontal = Control.SIZE_EXPAND_FILL

	var vb = VBoxContainer.new()
	vb.add_theme_constant_override("separation", 8)
	panel.add_child(vb)

	var lbl = Label.new()
	lbl.text = "LOOKUP (type Arabic or English)"
	lbl.add_theme_font_size_override("font_size", DubSarTheme.SIZE_LABEL)
	lbl.add_theme_color_override("font_color", DubSarTheme.TEXT_DIM)
	vb.add_child(lbl)

	var row = HBoxContainer.new()
	row.add_theme_constant_override("separation", 8)
	vb.add_child(row)

	_query_edit = LineEdit.new()
	_query_edit.placeholder_text = "e.g. \"شرين\" or \"gate\" or \"وادي\""
	_query_edit.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	_query_edit.text_submitted.connect(func(_t): _on_lookup_pressed())
	row.add_child(_query_edit)

	var btn = Button.new()
	btn.text = "🔍 Lookup"
	btn.pressed.connect(_on_lookup_pressed)
	row.add_child(btn)

	_gazetteer_count_lbl = Label.new()
	_gazetteer_count_lbl.add_theme_font_size_override("font_size", DubSarTheme.SIZE_LABEL)
	_gazetteer_count_lbl.add_theme_color_override("font_color", DubSarTheme.TEXT_FAINT)
	vb.add_child(_gazetteer_count_lbl)
	_update_gazetteer_count()

	var results_panel = PanelContainer.new()
	results_panel.add_theme_stylebox_override("panel", DubSarTheme.card_stylebox(8, DubSarTheme.BORDER))
	results_panel.size_flags_vertical = Control.SIZE_EXPAND_FILL
	vb.add_child(results_panel)

	_results_log = RichTextLabel.new()
	_results_log.bbcode_enabled = true
	_results_log.scroll_following = false
	_results_log.size_flags_vertical = Control.SIZE_EXPAND_FILL
	_results_log.add_theme_font_size_override("normal_font_size", DubSarTheme.SIZE_CODE)
	_results_log.add_theme_color_override("default_color", DubSarTheme.TEXT_PRIMARY)
	results_panel.add_child(_results_log)

	return panel

func _build_add_panel() -> PanelContainer:
	var panel = PanelContainer.new()
	panel.add_theme_stylebox_override("panel", DubSarTheme.card_stylebox())
	panel.custom_minimum_size = Vector2(320, 0)

	var vb = VBoxContainer.new()
	vb.add_theme_constant_override("separation", 8)
	panel.add_child(vb)

	var lbl = Label.new()
	lbl.text = "+ ADD PLACE (session-local, not persisted)"
	lbl.add_theme_font_size_override("font_size", DubSarTheme.SIZE_LABEL)
	lbl.add_theme_color_override("font_color", DubSarTheme.TEXT_DIM)
	vb.add_child(lbl)

	_add_name = LineEdit.new()
	_add_name.placeholder_text = "English name"
	vb.add_child(_add_name)

	_add_name_arabic = LineEdit.new()
	_add_name_arabic.placeholder_text = "الاسم بالعربية"
	vb.add_child(_add_name_arabic)

	var coord_row = HBoxContainer.new()
	coord_row.add_theme_constant_override("separation", 6)
	vb.add_child(coord_row)

	_add_lat = LineEdit.new()
	_add_lat.placeholder_text = "lat"
	_add_lat.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	coord_row.add_child(_add_lat)

	_add_lon = LineEdit.new()
	_add_lon.placeholder_text = "lon"
	_add_lon.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	coord_row.add_child(_add_lon)

	var btn_add = Button.new()
	btn_add.text = "+ Add to glossary"
	btn_add.add_theme_color_override("font_color", DubSarTheme.ACCENT_GREEN)
	btn_add.pressed.connect(_on_add_pressed)
	vb.add_child(btn_add)

	var status = Label.new()
	status.name = "AddStatus"
	status.autowrap_mode = TextServer.AUTOWRAP_WORD
	status.add_theme_font_size_override("font_size", DubSarTheme.SIZE_LABEL)
	status.add_theme_color_override("font_color", DubSarTheme.TEXT_FAINT)
	vb.add_child(status)

	return panel

func _update_gazetteer_count() -> void:
	_gazetteer_count_lbl.text = "%d place(s) in this session's glossary (seed + added)" % _gazetteer.size()

func _render_results_hint() -> void:
	_results_log.clear()
	_results_log.append_text("[color=#%s]Type a name (Arabic or English) and press Lookup. Real navi-translate-gdext calls, real adapa-recall TF-IDF/cosine ranking -- no canned results.[/color]" %
		DubSarTheme.TEXT_DIM.to_html(false))

func _on_lookup_pressed() -> void:
	var query := _query_edit.text.strip_edges()
	_results_log.clear()
	if query.is_empty():
		_results_log.append_text("[color=#%s]Type something to look up first.[/color]" % DubSarTheme.ACCENT_AMBER.to_html(false))
		return

	var places: Array = []
	for p in _gazetteer:
		places.append(p)

	var result: Dictionary = _bridge.lookup_by_name(query, places, 5)
	if not result.get("ok", false):
		_results_log.append_text("[color=#%s]Lookup FAILED: %s[/color]" % [DubSarTheme.ACCENT_RED.to_html(false), result.get("error", "?")])
		return

	var matches: Array = result.get("matches", [])
	if matches.is_empty():
		_results_log.append_text("[color=#%s]No matches -- real, honest empty result (no shared distinguishing terms found), not a fabricated \"not found\".[/color]" % DubSarTheme.TEXT_DIM.to_html(false))
		return

	_results_log.append_text("[b]%d match(es) for \"%s\":[/b]\n\n" % [matches.size(), query])
	for m in matches:
		var name: String = m.get("name", "?")
		var name_arabic: String = m.get("name_arabic", "?")
		var score: float = m.get("score", 0.0)
		var lat: float = m.get("lat", 0.0)
		var lon: float = m.get("lon", 0.0)
		_results_log.append_text("[color=#%s]● %s[/color]  /  %s\n    score=%.3f  (%.4f, %.4f)\n\n" %
			[DubSarTheme.ACCENT_CYAN.to_html(false), name, name_arabic, score, lat, lon])

func _on_add_pressed() -> void:
	var name := _add_name.text.strip_edges()
	var name_arabic := _add_name_arabic.text.strip_edges()

	var validation: Dictionary = _bridge.validate_place(name, name_arabic)
	if not validation.get("valid", false):
		_show_add_status("Both English and Arabic names are required (real EavStore::required_missing() check via ATTR_NAME/ATTR_NAME_ARABIC).", DubSarTheme.ACCENT_RED)
		return

	var lat := _add_lat.text.to_float()
	var lon := _add_lon.text.to_float()
	_gazetteer.append({"name": name, "name_arabic": name_arabic, "lat": lat, "lon": lon})
	_update_gazetteer_count()
	_show_add_status("Added \"%s\" / \"%s\" to this session's glossary." % [name, name_arabic], DubSarTheme.ACCENT_GREEN)

	_add_name.text = ""
	_add_name_arabic.text = ""
	_add_lat.text = ""
	_add_lon.text = ""

func _show_add_status(text: String, color: Color) -> void:
	var add_panels := find_children("AddStatus", "Label", true, false)
	if add_panels.size() > 0:
		var lbl: Label = add_panels[0]
		lbl.text = text
		lbl.add_theme_color_override("font_color", color)
