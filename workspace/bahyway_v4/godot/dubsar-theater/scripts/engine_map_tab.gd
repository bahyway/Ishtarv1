extends MarginContainer
# =============================================================================
# Engine Map tab — 2026-07-24, the tool the Architect asked for: how every
# built engine, crate, and playbook in this workspace fits together, shown
# inside DubSar IDE.
#
# Grounded, not hand-maintained: every row comes from a REAL scan of this
# workspace's crates/*/Cargo.toml and bin/*/Cargo.toml files at the moment
# the tab opens (enkimdb-registry-gd -> enkimdb-registry::scan_workspace),
# so this list can never drift out of sync with what's actually built —
# unlike a static doc that goes stale the moment a new crate lands.
# =============================================================================

const DubSarTheme = preload("res://scripts/dubsar_theme.gd")

var _bridge: EnkiMdbRegistryBridge
var _workspace_root: String
var _search_edit: LineEdit
var _count_lbl: Label
var _results_vbox: VBoxContainer

func _ready() -> void:
	_bridge = EnkiMdbRegistryBridge.new()
	# This project lives at <workspace_root>/godot/dubsar-theater/ ; the
	# real crates/ and bin/ trees this scan needs are two levels up.
	_workspace_root = ProjectSettings.globalize_path("res://../..")
	_build_ui()
	_run_scan()

func _build_ui() -> void:
	var root_vb = VBoxContainer.new()
	root_vb.add_theme_constant_override("separation", 10)
	add_child(root_vb)

	var title = Label.new()
	title.text = "𒁾  Engine Map — every real crate/binary in this workspace"
	title.add_theme_font_size_override("font_size", DubSarTheme.SIZE_TITLE)
	title.add_theme_color_override("font_color", DubSarTheme.TEXT_PRIMARY)
	root_vb.add_child(title)

	var subtitle = Label.new()
	subtitle.text = "Live filesystem scan, not a maintained list — this tab can never drift out of sync with what's actually built."
	subtitle.autowrap_mode = TextServer.AUTOWRAP_WORD
	subtitle.add_theme_font_size_override("font_size", DubSarTheme.SIZE_LABEL)
	subtitle.add_theme_color_override("font_color", DubSarTheme.TEXT_DIM)
	root_vb.add_child(subtitle)

	var toolbar = HBoxContainer.new()
	toolbar.add_theme_constant_override("separation", 8)
	root_vb.add_child(toolbar)

	_search_edit = LineEdit.new()
	_search_edit.placeholder_text = "Search by name or description…"
	_search_edit.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	_search_edit.text_changed.connect(func(_t): _run_scan())
	toolbar.add_child(_search_edit)

	var btn_refresh = Button.new()
	btn_refresh.text = "⟳ Rescan"
	btn_refresh.pressed.connect(_run_scan)
	toolbar.add_child(btn_refresh)

	_count_lbl = Label.new()
	_count_lbl.add_theme_font_size_override("font_size", DubSarTheme.SIZE_LABEL)
	_count_lbl.add_theme_color_override("font_color", DubSarTheme.TEXT_FAINT)
	root_vb.add_child(_count_lbl)

	var scroll = ScrollContainer.new()
	scroll.size_flags_vertical = Control.SIZE_EXPAND_FILL
	root_vb.add_child(scroll)

	_results_vbox = VBoxContainer.new()
	_results_vbox.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	_results_vbox.add_theme_constant_override("separation", 4)
	scroll.add_child(_results_vbox)

func _run_scan() -> void:
	for child in _results_vbox.get_children():
		child.queue_free()

	var query := _search_edit.text.strip_edges() if _search_edit else ""
	var result: Dictionary = _bridge.search(_workspace_root, query) if not query.is_empty() \
		else _bridge.scan(_workspace_root)

	if not result.get("ok", false):
		_count_lbl.text = "Scan failed."
		return

	var engines: Array = result.get("engines", [])
	_count_lbl.text = "%d artifact(s) found under %s" % [engines.size(), _workspace_root]

	if engines.is_empty():
		var empty_lbl = Label.new()
		empty_lbl.text = "No matches." if not query.is_empty() else "No Cargo.toml files found — is workspace_root correct?"
		empty_lbl.add_theme_color_override("font_color", DubSarTheme.TEXT_DIM)
		_results_vbox.add_child(empty_lbl)
		return

	for e in engines:
		_results_vbox.add_child(_row(e))

func _row(e: Dictionary) -> PanelContainer:
	var panel = PanelContainer.new()
	panel.add_theme_stylebox_override("panel", DubSarTheme.card_stylebox(6, DubSarTheme.BORDER))

	var hb = HBoxContainer.new()
	hb.add_theme_constant_override("separation", 10)
	panel.add_child(hb)

	var kind: String = e.get("kind", "crate")
	var kind_pill = DubSarTheme.pill(kind.to_upper(), DubSarTheme.ACCENT_CYAN if kind == "crate" else DubSarTheme.ACCENT_GREEN)
	hb.add_child(kind_pill)

	var name_lbl = Label.new()
	name_lbl.text = str(e.get("name", "?"))
	name_lbl.custom_minimum_size = Vector2(220, 0)
	name_lbl.add_theme_font_size_override("font_size", DubSarTheme.SIZE_BODY)
	name_lbl.add_theme_color_override("font_color", DubSarTheme.TEXT_PRIMARY)
	hb.add_child(name_lbl)

	var version_lbl = Label.new()
	version_lbl.text = "v%s" % str(e.get("version", "?"))
	version_lbl.custom_minimum_size = Vector2(70, 0)
	version_lbl.add_theme_font_size_override("font_size", DubSarTheme.SIZE_LABEL)
	version_lbl.add_theme_color_override("font_color", DubSarTheme.ACCENT_AMBER)
	hb.add_child(version_lbl)

	var desc_lbl = Label.new()
	desc_lbl.text = str(e.get("description", ""))
	desc_lbl.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	desc_lbl.clip_text = true
	desc_lbl.add_theme_font_size_override("font_size", DubSarTheme.SIZE_LABEL)
	desc_lbl.add_theme_color_override("font_color", DubSarTheme.TEXT_DIM)
	hb.add_child(desc_lbl)

	return panel
