extends MarginContainer
# =============================================================================
# Glossary tab — 2026-07-25, Phase 2 of the naming-registry work: lets a
# new team member or an authorized client look up every Akkadian/Sumerian
# name used across this ecosystem (deity, city, king-era, or generic term),
# which domain(s) it plays a role in, and — where one exists — a real
# excerpt of the source file backing it.
#
# Grounded, not hand-maintained: every row comes from naming-registry-gd's
# real seed() catalog, not strings typed into this script. source_doc is
# shown to everyone (a governing-law tablet citation, not a source file).
#
# 2026-07-25 Phase 3: crate_path/snippet are now gated on the session's
# real kupru passport privilege_level (SessionIdentity, populated by
# login.gd's mandatory gate). The redaction is enforced in
# naming-registry-gd itself (NamingRegistryBridge.is_internal /
# every method's `internal` param) -- this script never receives
# crate_path or reads a snippet for a non-internal session; it isn't
# merely choosing not to render fields it was handed anyway. HONEST
# SCOPE: kupru only defines two concrete tiers today (gardener=1,
# architect=7); this draws the line at architect-tier because no
# intermediate "team member" tier exists in kupru yet (see
# naming-registry-gd's own module doc comment for the same note).
# =============================================================================

const DubSarTheme = preload("res://scripts/dubsar_theme.gd")
const SessionIdentity = preload("res://scripts/session_identity.gd")

const CATEGORY_COLOR := {
	"Deity": DubSarTheme.ACCENT_PURPLE,
	"City": DubSarTheme.ACCENT_TEAL,
	"KingOrEra": DubSarTheme.ACCENT_AMBER,
	"GenericTerm": DubSarTheme.ACCENT_CYAN,
}
const STATUS_COLOR := {
	"SealedByLaw": DubSarTheme.ACCENT_GREEN,
	"ExistingUnverified": DubSarTheme.STATE_FUZZY,
	"Reserved": DubSarTheme.TEXT_FAINT,
}
const CATEGORIES := ["All", "Deity", "City", "KingOrEra", "GenericTerm"]
const ROLES := ["All", "Engine", "Calculus", "AlertClass", "Pattern", "Agent", "Suite", "Tool", "Protocol", "IndexStack", "Language"]

var _bridge: NamingRegistryBridge
var _workspace_root: String
var _is_internal: bool
var _search_edit: LineEdit
var _category_option: OptionButton
var _role_option: OptionButton
var _count_lbl: Label
var _view_mode_lbl: Label
var _results_vbox: VBoxContainer

func _ready() -> void:
	_bridge = NamingRegistryBridge.new()
	# This project lives at <workspace_root>/godot/dubsar-theater/ ; the
	# real crates/ tree snippet() needs to read from is two levels up.
	_workspace_root = ProjectSettings.globalize_path("res://../..")
	# SessionIdentity is populated once by login.gd's mandatory gate
	# before any Theater scene (this tab included) is ever reachable --
	# .valid is false only if something bypassed that gate, which the
	# redaction must treat as the least-privileged case, not fail open.
	_is_internal = SessionIdentity.valid and _bridge.is_internal(SessionIdentity.privilege_level)
	_build_ui()
	_run_query()

func _build_ui() -> void:
	var root_vb = VBoxContainer.new()
	root_vb.add_theme_constant_override("separation", 10)
	add_child(root_vb)

	var title = Label.new()
	title.text = "𒁾  Glossary — Akkadian & Sumerian names in this ecosystem"
	title.add_theme_font_size_override("font_size", DubSarTheme.SIZE_TITLE)
	title.add_theme_color_override("font_color", DubSarTheme.TEXT_PRIMARY)
	root_vb.add_child(title)

	var subtitle = Label.new()
	subtitle.text = "Real catalog (naming-registry), not a hand-typed list. Reserved/unverified names are shown honestly, not hidden."
	subtitle.autowrap_mode = TextServer.AUTOWRAP_WORD
	subtitle.add_theme_font_size_override("font_size", DubSarTheme.SIZE_LABEL)
	subtitle.add_theme_color_override("font_color", DubSarTheme.TEXT_DIM)
	root_vb.add_child(subtitle)

	_view_mode_lbl = Label.new()
	if _is_internal:
		_view_mode_lbl.text = "🔓 Internal view — source snippets and crate paths visible (architect-tier passport)."
		_view_mode_lbl.add_theme_color_override("font_color", DubSarTheme.ACCENT_GREEN)
	else:
		_view_mode_lbl.text = "🔒 Client view — names, categories, and roles only. Source snippets and crate paths require an internal (architect-tier) passport."
		_view_mode_lbl.add_theme_color_override("font_color", DubSarTheme.ACCENT_AMBER)
	_view_mode_lbl.autowrap_mode = TextServer.AUTOWRAP_WORD
	_view_mode_lbl.add_theme_font_size_override("font_size", DubSarTheme.SIZE_LABEL)
	root_vb.add_child(_view_mode_lbl)

	var toolbar = HBoxContainer.new()
	toolbar.add_theme_constant_override("separation", 8)
	root_vb.add_child(toolbar)

	_search_edit = LineEdit.new()
	_search_edit.placeholder_text = "Search name, blurb, or domain…"
	_search_edit.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	_search_edit.text_changed.connect(func(_t): _run_query())
	toolbar.add_child(_search_edit)

	_category_option = OptionButton.new()
	for c in CATEGORIES:
		_category_option.add_item(c)
	_category_option.item_selected.connect(func(_i): _run_query())
	toolbar.add_child(_category_option)

	_role_option = OptionButton.new()
	for r in ROLES:
		_role_option.add_item(r)
	_role_option.item_selected.connect(func(_i): _run_query())
	toolbar.add_child(_role_option)

	var btn_refresh = Button.new()
	btn_refresh.text = "⟳ Reset filters"
	btn_refresh.pressed.connect(func():
		_search_edit.text = ""
		_category_option.selected = 0
		_role_option.selected = 0
		_run_query()
	)
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
	_results_vbox.add_theme_constant_override("separation", 6)
	scroll.add_child(_results_vbox)

func _run_query() -> void:
	for child in _results_vbox.get_children():
		child.queue_free()

	var query := _search_edit.text.strip_edges() if _search_edit else ""
	var category: String = CATEGORIES[_category_option.selected] if _category_option else "All"
	var role: String = ROLES[_role_option.selected] if _role_option else "All"

	# Start from search (or "all"), then narrow by category/role client-side
	# -- three independent filters compose more simply this way than a
	# three-argument bridge call, and the entry count here is small (~tens),
	# so the extra pass costs nothing.
	var result: Dictionary = _bridge.search(query, _is_internal) if not query.is_empty() else _bridge.all(_is_internal)
	if not result.get("ok", false):
		_count_lbl.text = "Query failed."
		return

	var entries: Array = result.get("entries", [])
	if category != "All":
		entries = entries.filter(func(e): return e.get("mytho_category", "") == category)
	if role != "All":
		entries = entries.filter(func(e): return e.get("system_role", "") == role)

	_count_lbl.text = "%d name(s)" % entries.size()

	if entries.is_empty():
		var empty_lbl = Label.new()
		empty_lbl.text = "No matches."
		empty_lbl.add_theme_color_override("font_color", DubSarTheme.TEXT_DIM)
		_results_vbox.add_child(empty_lbl)
		return

	for e in entries:
		_results_vbox.add_child(_row(e))

func _row(e: Dictionary) -> PanelContainer:
	var panel = PanelContainer.new()
	panel.add_theme_stylebox_override("panel", DubSarTheme.card_stylebox(6, DubSarTheme.BORDER))

	var outer_vb = VBoxContainer.new()
	outer_vb.add_theme_constant_override("separation", 4)
	panel.add_child(outer_vb)

	var hb = HBoxContainer.new()
	hb.add_theme_constant_override("separation", 10)
	outer_vb.add_child(hb)

	var name: String = str(e.get("name", "?"))
	var category: String = str(e.get("mytho_category", ""))
	var role: String = str(e.get("system_role", ""))
	var status: String = str(e.get("status", ""))

	var name_lbl = Label.new()
	name_lbl.text = name
	name_lbl.custom_minimum_size = Vector2(160, 0)
	name_lbl.add_theme_font_size_override("font_size", DubSarTheme.SIZE_BODY)
	name_lbl.add_theme_color_override("font_color", DubSarTheme.TEXT_PRIMARY)
	hb.add_child(name_lbl)

	hb.add_child(DubSarTheme.pill(category, CATEGORY_COLOR.get(category, DubSarTheme.TEXT_DIM)))
	hb.add_child(DubSarTheme.pill(role, DubSarTheme.ACCENT_CYAN))
	hb.add_child(DubSarTheme.pill(status, STATUS_COLOR.get(status, DubSarTheme.TEXT_DIM)))

	var domain_tags: Array = e.get("domain_tags", [])
	if not domain_tags.is_empty():
		var domain_lbl = Label.new()
		domain_lbl.text = "domains: " + ", ".join(domain_tags)
		domain_lbl.add_theme_font_size_override("font_size", DubSarTheme.SIZE_LABEL)
		domain_lbl.add_theme_color_override("font_color", DubSarTheme.TEXT_FAINT)
		hb.add_child(domain_lbl)

	var spacer = Control.new()
	spacer.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	hb.add_child(spacer)

	var btn_snippet = Button.new()
	btn_snippet.text = "Snippet" if _is_internal else "🔒 Snippet"
	hb.add_child(btn_snippet)

	var blurb_lbl = Label.new()
	blurb_lbl.text = str(e.get("blurb", ""))
	blurb_lbl.autowrap_mode = TextServer.AUTOWRAP_WORD
	blurb_lbl.add_theme_font_size_override("font_size", DubSarTheme.SIZE_LABEL)
	blurb_lbl.add_theme_color_override("font_color", DubSarTheme.TEXT_DIM)
	outer_vb.add_child(blurb_lbl)

	var source_doc: String = str(e.get("source_doc", ""))
	if not source_doc.is_empty():
		var source_lbl = Label.new()
		source_lbl.text = "source: " + source_doc
		source_lbl.add_theme_font_size_override("font_size", DubSarTheme.SIZE_LABEL)
		source_lbl.add_theme_color_override("font_color", DubSarTheme.TEXT_FAINT)
		outer_vb.add_child(source_lbl)

	var snippet_box = TextEdit.new()
	snippet_box.visible = false
	snippet_box.editable = false
	snippet_box.custom_minimum_size = Vector2(0, 160)
	snippet_box.wrap_mode = TextEdit.LINE_WRAPPING_NONE
	snippet_box.add_theme_font_size_override("font_size", DubSarTheme.SIZE_CODE)
	outer_vb.add_child(snippet_box)

	var loaded := false
	btn_snippet.pressed.connect(func():
		snippet_box.visible = not snippet_box.visible
		if snippet_box.visible and not loaded:
			loaded = true
			var res: Dictionary = _bridge.snippet(name, _workspace_root, 60, _is_internal)
			if res.get("ok", false):
				snippet_box.text = "# %s\n\n%s" % [str(res.get("crate_path", "")), str(res.get("snippet", ""))]
			else:
				snippet_box.text = "(no snippet)  " + str(res.get("error", "unknown error"))
	)

	return panel
