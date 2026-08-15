extends MarginContainer
# =============================================================================
# PDM (Particles Data Modeling) tab — 2026-07-10
#
# Lets a stakeholder write a minimal .akk model, compile it (via
# AkkadiCompiler, scripts/akkadi_compiler.gd) into real HeptaScript text,
# and run it through the parent HeptaScript editor's dispatch path
# (bind_run_callback). Query results come back via on_query_result() and
# are shown as a particle/attribute model grid, classified GEM/ORBIT/FUZZY
# using the same B11-from-EAV-attrs rule already established and working
# in dubsar_proof.gd -- never from raw KAKI bytes.
#
# No fictional "EnkiSDB test schema" system: running against EnkiSDB (or
# any other engine) just means checking its box in the parent editor's
# NODE target list -- EnkiSDB has no special schema concept in the real
# repo (confirmed 2026-07-10), so none is invented here.
#
# 2026-07-21: added a "Node Graph" sub-tab (scripts/pdm_node_graph.gd) as
# a Houdini-style visual front-end for the SAME .akk grammar -- it compiles
# a graph of Model/Target/Entity/Attribute/Limit nodes down to real .akk
# source text and writes it into akk_edit.text, so it flows through this
# file's own already-tested _compile()/_on_compile_run_pressed() path
# unchanged. See pdm_node_graph.gd's header for why a GA (Clifford
# algebra) node type is deliberately NOT included yet (no GDExtension
# bridge exists to the real Rust implementation).
#
# 2026-07-31: added "Logical" and "Physical" sub-tabs -- the Conceptual/
# Logical/Physical split for PDM schema discovery, mirroring ERwin/
# Archimate's data-modeling layers:
#   Conceptual = wiring which relationships feed discovery (the Node
#     Graph tab's new SCHEMA_DISCOVERY node type, see pdm_node_graph.gd).
#   Logical    = this tab's own review/approve step: a stakeholder checks
#     off which of pdm-discover's candidate relationships to accept
#     (never auto-applied -- see pdm-discovery's own module doc for why
#     its v1 exact-match heuristic needs human review).
#   Physical   = compiles the APPROVED proposal into real .akk source,
#     reusing the exact same AkkadiCompiler.compile() this file has
#     always used. HONEST SCOPE: AkkadiCompiler's grammar supports one
#     entity (one WHO clause) per query -- there is no real multi-table
#     JOIN query capability in HeptaScript today, so this emits one .akk
#     block per approved table (loaded into the Text tab one at a time),
#     not a fabricated join syntax the server can't actually execute.
# NOT exercised under a live Godot runtime in this session (none
# available in this sandbox); reviewed statically only.
# =============================================================================

const AkkadiCompiler = preload("res://scripts/akkadi_compiler.gd")
const DubSarTheme = preload("res://scripts/dubsar_theme.gd")
const PdmNodeGraph = preload("res://scripts/pdm_node_graph.gd")

# 2026-07-21: restyled onto DubSarTheme's shared palette -- `var`, not
# `const`, to avoid relying on unverified cross-script const-from-const
# compile-time resolution in GDScript 4.3 (same reasoning as
# page4_summary.gd's LABEL_COLOR/VALUE_COLOR). Names kept as-is so the
# ~20 call sites below didn't need touching, just their source values.
var C_BG:    Color = DubSarTheme.PANEL_BG
var C_TEXT:  Color = DubSarTheme.TEXT_PRIMARY
var C_DIM:   Color = DubSarTheme.TEXT_DIM
var C_GOLD:  Color = DubSarTheme.ACCENT_AMBER
var C_GEM:   Color = DubSarTheme.ACCENT_GREEN  # B11 >= 200
var C_ORBIT: Color = DubSarTheme.ACCENT_AMBER  # 128 <= B11 < 200
var C_FUZZY: Color = DubSarTheme.ACCENT_RED    # B11 < 128

var _run_callback: Callable = Callable()

var akk_edit: TextEdit
var hepta_preview: Label
var compile_errors: Label
var model_grid: VBoxContainer
var summary_label: Label
var node_graph: PdmNodeGraph
var graph_status: Label

# ── Logical/Physical tab state ──────────────────────────────────────────
var logical_status: Label
var logical_list: VBoxContainer
var _logical_checks: Array = []       # Array[{"rel": Dictionary, "check": CheckBox}]
var _logical_source_proposal: Dictionary = {}
var _approved_proposal: Dictionary = {}

var physical_status: Label
var physical_table_picker: OptionButton

const DEFAULT_AKK := """MODEL najaf_deaths
NODE EnkiSDB
WHO Tribes.E
ATTR record.no
ATTR person.name_arabic
ATTR death.date
ATTR b11
LIMIT 1000
"""

func _ready():
	_build()

func bind_run_callback(cb: Callable) -> void:
	_run_callback = cb

func _build():
	var hsplit = HSplitContainer.new()
	hsplit.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	hsplit.size_flags_vertical = Control.SIZE_EXPAND_FILL
	add_child(hsplit)

	# ── LEFT: .akk source + compile controls ──────────────────────────────
	var left = VBoxContainer.new()
	left.custom_minimum_size = Vector2(360, 0)
	left.add_theme_constant_override("separation", 8)
	hsplit.add_child(left)

	var akk_label = Label.new()
	akk_label.text = "PARTICLE DATA MODEL (.akk)"
	akk_label.add_theme_font_size_override("font_size", 12)
	akk_label.add_theme_color_override("font_color", C_DIM)
	left.add_child(akk_label)

	var source_tabs = TabContainer.new()
	source_tabs.custom_minimum_size = Vector2(0, 260)
	source_tabs.size_flags_vertical = Control.SIZE_EXPAND_FILL
	left.add_child(source_tabs)

	var text_tab = VBoxContainer.new()
	text_tab.name = "Text"
	text_tab.size_flags_vertical = Control.SIZE_EXPAND_FILL
	source_tabs.add_child(text_tab)

	akk_edit = TextEdit.new()
	akk_edit.size_flags_vertical = Control.SIZE_EXPAND_FILL
	akk_edit.text = DEFAULT_AKK
	text_tab.add_child(akk_edit)

	var graph_tab = VBoxContainer.new()
	graph_tab.name = "Node Graph"
	graph_tab.size_flags_vertical = Control.SIZE_EXPAND_FILL
	source_tabs.add_child(graph_tab)
	_build_node_graph_tab(graph_tab, source_tabs, text_tab)

	var logical_tab = VBoxContainer.new()
	logical_tab.name = "Logical"
	logical_tab.size_flags_vertical = Control.SIZE_EXPAND_FILL
	source_tabs.add_child(logical_tab)
	_build_logical_tab(logical_tab)

	var physical_tab = VBoxContainer.new()
	physical_tab.name = "Physical"
	physical_tab.size_flags_vertical = Control.SIZE_EXPAND_FILL
	source_tabs.add_child(physical_tab)
	_build_physical_tab(physical_tab, source_tabs, text_tab)

	var btn_row = HBoxContainer.new()
	btn_row.add_theme_constant_override("separation", 8)
	left.add_child(btn_row)

	var btn_compile = Button.new()
	btn_compile.text = "⚙ Compile"
	btn_compile.pressed.connect(_on_compile_pressed)
	btn_row.add_child(btn_compile)

	var btn_compile_run = Button.new()
	btn_compile_run.text = "⚙ Compile & Run →"
	btn_compile_run.pressed.connect(_on_compile_run_pressed)
	btn_row.add_child(btn_compile_run)

	var preview_label = Label.new()
	preview_label.text = "COMPILED HEPTASCRIPT"
	preview_label.add_theme_font_size_override("font_size", 12)
	preview_label.add_theme_color_override("font_color", C_DIM)
	left.add_child(preview_label)

	var preview_panel = PanelContainer.new()
	var style = StyleBoxFlat.new()
	style.bg_color = C_BG
	style.corner_radius_top_left = 4
	style.corner_radius_top_right = 4
	style.corner_radius_bottom_left = 4
	style.corner_radius_bottom_right = 4
	style.content_margin_left = 10
	style.content_margin_top = 8
	style.content_margin_right = 10
	style.content_margin_bottom = 8
	preview_panel.add_theme_stylebox_override("panel", style)
	left.add_child(preview_panel)

	hepta_preview = Label.new()
	hepta_preview.text = "—"
	hepta_preview.add_theme_color_override("font_color", C_GOLD)
	hepta_preview.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	preview_panel.add_child(hepta_preview)

	compile_errors = Label.new()
	compile_errors.add_theme_color_override("font_color", C_FUZZY)
	compile_errors.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	left.add_child(compile_errors)

	# ── RIGHT: model / results grid ────────────────────────────────────────
	var right = VBoxContainer.new()
	right.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	right.add_theme_constant_override("separation", 8)
	hsplit.add_child(right)

	var model_label = Label.new()
	model_label.text = "MODEL — particles returned, classified by B11 (EAV, not KAKI)"
	model_label.add_theme_font_size_override("font_size", 12)
	model_label.add_theme_color_override("font_color", C_DIM)
	right.add_child(model_label)

	summary_label = Label.new()
	summary_label.text = "No results yet — compile and run a model."
	summary_label.add_theme_color_override("font_color", C_TEXT)
	right.add_child(summary_label)

	var scroll = ScrollContainer.new()
	scroll.size_flags_vertical = Control.SIZE_EXPAND_FILL
	right.add_child(scroll)

	model_grid = VBoxContainer.new()
	model_grid.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	model_grid.add_theme_constant_override("separation", 4)
	scroll.add_child(model_grid)

func _build_node_graph_tab(graph_tab: VBoxContainer, source_tabs: TabContainer, text_tab: VBoxContainer) -> void:
	var toolbar = HBoxContainer.new()
	toolbar.add_theme_constant_override("separation", 6)
	graph_tab.add_child(toolbar)

	var add_menu = MenuButton.new()
	add_menu.text = "+ Add Node"
	var popup = add_menu.get_popup()
	for kind in PdmNodeGraph.NodeKind.values():
		popup.add_item(PdmNodeGraph.KIND_NAMES[kind], kind)
	popup.id_pressed.connect(func(id: int): node_graph.add_node_of_kind(id))
	toolbar.add_child(add_menu)

	var btn_clear = Button.new()
	btn_clear.text = "🗑 Clear Graph"
	btn_clear.pressed.connect(func(): node_graph.clear_graph())
	toolbar.add_child(btn_clear)

	var btn_to_akk = Button.new()
	btn_to_akk.text = "⇒ Compile Graph to .akk"
	btn_to_akk.pressed.connect(func(): _on_graph_to_akk_pressed(source_tabs, text_tab))
	toolbar.add_child(btn_to_akk)

	graph_status = Label.new()
	graph_status.text = "Wire Attribute/Limit nodes into the Entity node's attrs/limit ports, then Compile Graph to .akk."
	graph_status.add_theme_font_size_override("font_size", 11)
	graph_status.add_theme_color_override("font_color", C_DIM)
	graph_status.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	graph_tab.add_child(graph_status)

	node_graph = PdmNodeGraph.new()
	node_graph.size_flags_vertical = Control.SIZE_EXPAND_FILL
	node_graph.right_disconnects = true
	graph_tab.add_child(node_graph)

	# Seed a starter graph matching DEFAULT_AKK so the tab isn't empty on first
	# open. Model/Target don't need wiring to Entity (see pdm_node_graph.gd's
	# topology contract -- they're resolved by presence, not connection);
	# only Attr/Limit -> Entity's attrs/limit ports are real, required wires.
	var model_n = node_graph.add_node_of_kind(PdmNodeGraph.NodeKind.MODEL)
	node_graph.set_field_text(model_n, "name", "najaf_deaths")

	var target_n = node_graph.add_node_of_kind(PdmNodeGraph.NodeKind.TARGET)
	node_graph.set_target_checked(target_n, "EnkiSDB", true)

	var entity_n = node_graph.add_node_of_kind(PdmNodeGraph.NodeKind.ENTITY)
	node_graph.set_field_text(entity_n, "tribe", "Tribes")
	node_graph.set_field_text(entity_n, "var", "E")

	var attr_n = node_graph.add_node_of_kind(PdmNodeGraph.NodeKind.ATTR)
	node_graph.set_field_text(attr_n, "attr", "record.no")
	node_graph.connect_node(entity_n.name, PdmNodeGraph.ENTITY_ATTRS_PORT, attr_n.name, 0)

	var limit_n = node_graph.add_node_of_kind(PdmNodeGraph.NodeKind.LIMIT)
	node_graph.set_field_text(limit_n, "limit", "1000")
	node_graph.connect_node(entity_n.name, PdmNodeGraph.ENTITY_LIMIT_PORT, limit_n.name, 0)

func _on_graph_to_akk_pressed(source_tabs: TabContainer, text_tab: VBoxContainer) -> void:
	var result = node_graph.compile_to_akk()
	if not result.ok:
		graph_status.text = "[ERROR] " + " ".join(result.errors)
		graph_status.add_theme_color_override("font_color", C_FUZZY)
		return
	akk_edit.text = result.akk_source
	graph_status.text = "Compiled to .akk -- switched to Text tab. Now press Compile / Compile & Run."
	graph_status.add_theme_color_override("font_color", C_TEXT)
	source_tabs.current_tab = text_tab.get_index()

# ── Logical tab: review/approve discovered candidate relationships ──────
# Reads whatever the Node Graph tab's SCHEMA_DISCOVERY node(s) have
# already run (see pdm_node_graph.gd::get_schema_discovery_proposals()).
# Nothing here mutates any real schema -- "Approve Selected" only builds
# an in-memory _approved_proposal for the Physical tab to compile from.
func _build_logical_tab(tab: VBoxContainer) -> void:
	var toolbar = HBoxContainer.new()
	toolbar.add_theme_constant_override("separation", 6)
	tab.add_child(toolbar)

	var btn_load = Button.new()
	btn_load.text = "🔄 Load Proposal from Node Graph"
	btn_load.pressed.connect(_on_logical_load_pressed)
	toolbar.add_child(btn_load)

	var btn_approve = Button.new()
	btn_approve.text = "✅ Approve Selected"
	btn_approve.pressed.connect(_on_logical_approve_pressed)
	toolbar.add_child(btn_approve)

	logical_status = Label.new()
	logical_status.text = "Run a Schema Discovery node in the Node Graph tab, then Load Proposal here to review it."
	logical_status.add_theme_font_size_override("font_size", 11)
	logical_status.add_theme_color_override("font_color", C_DIM)
	logical_status.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	tab.add_child(logical_status)

	var scroll = ScrollContainer.new()
	scroll.size_flags_vertical = Control.SIZE_EXPAND_FILL
	tab.add_child(scroll)
	logical_list = VBoxContainer.new()
	logical_list.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	logical_list.add_theme_constant_override("separation", 4)
	scroll.add_child(logical_list)

func _on_logical_load_pressed() -> void:
	var proposals: Array = node_graph.get_schema_discovery_proposals()
	if proposals.is_empty():
		logical_status.text = "[ERROR] No Schema Discovery node has produced a proposal yet -- add one in the Node Graph tab and press Discover Schema."
		logical_status.add_theme_color_override("font_color", C_FUZZY)
		return
	# v1: review one discovered proposal at a time -- the most recently
	# added Schema Discovery node's. Reviewing/merging multiple discovery
	# runs at once is real follow-up work, not built here.
	_logical_source_proposal = proposals[proposals.size() - 1]

	for child in logical_list.get_children():
		child.queue_free()
	_logical_checks.clear()

	var rels: Array = _logical_source_proposal.get("relationships", [])
	if rels.is_empty():
		logical_status.text = "Proposal loaded: 0 candidate relationships detected."
	else:
		logical_status.text = "Proposal loaded: %d candidate relationship(s) -- review and Approve Selected." % rels.size()
	logical_status.add_theme_color_override("font_color", C_TEXT)

	for rel in rels:
		var cb = CheckBox.new()
		cb.button_pressed = true  # default: reviewed candidates start selected, stakeholder unchecks false positives
		cb.text = "%s.%s  <->  %s.%s   (confidence %.2f)" % [
			str(rel.get("from_table", "?")), str(rel.get("from_column", "?")),
			str(rel.get("to_table", "?")), str(rel.get("to_column", "?")),
			float(rel.get("overlap_confidence", 0.0)),
		]
		logical_list.add_child(cb)
		_logical_checks.append({"rel": rel, "check": cb})

func _on_logical_approve_pressed() -> void:
	if _logical_source_proposal.is_empty():
		logical_status.text = "[ERROR] Load a proposal before approving."
		logical_status.add_theme_color_override("font_color", C_FUZZY)
		return
	var approved_rels: Array = []
	for entry in _logical_checks:
		if (entry["check"] as CheckBox).button_pressed:
			approved_rels.append(entry["rel"])
	_approved_proposal = {
		"tables": _logical_source_proposal.get("tables", []),
		"relationships": approved_rels,
	}
	logical_status.text = "Approved %d of %d relationship(s). Switch to the Physical tab to compile." % [
		approved_rels.size(), _logical_checks.size()
	]
	logical_status.add_theme_color_override("font_color", C_TEXT)
	_refresh_physical_picker()

# ── Physical tab: compile the APPROVED proposal into real .akk source ───
# See this file's own header for why this emits one .akk block per
# approved table rather than a fabricated multi-table join query.
func _build_physical_tab(tab: VBoxContainer, source_tabs: TabContainer, text_tab: VBoxContainer) -> void:
	var info = Label.new()
	info.text = "Compiles the Logical tab's APPROVED tables into real .akk -- one block per table (this compiler supports one entity per query; loads into the Text tab for Compile/Compile & Run)."
	info.add_theme_font_size_override("font_size", 11)
	info.add_theme_color_override("font_color", C_DIM)
	info.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	tab.add_child(info)

	var row = HBoxContainer.new()
	row.add_theme_constant_override("separation", 6)
	tab.add_child(row)

	physical_table_picker = OptionButton.new()
	physical_table_picker.disabled = true
	row.add_child(physical_table_picker)

	var btn_load = Button.new()
	btn_load.text = "⇒ Load Table into Text Tab"
	btn_load.pressed.connect(func(): _on_physical_load_pressed(source_tabs, text_tab))
	row.add_child(btn_load)

	physical_status = Label.new()
	physical_status.text = "Approve relationships in the Logical tab first."
	physical_status.add_theme_font_size_override("font_size", 11)
	physical_status.add_theme_color_override("font_color", C_DIM)
	physical_status.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	tab.add_child(physical_status)

func _refresh_physical_picker() -> void:
	physical_table_picker.clear()
	var tables: Array = _approved_proposal.get("tables", [])
	for t in tables:
		physical_table_picker.add_item(str(t))
	physical_table_picker.disabled = tables.is_empty()
	physical_status.text = "%d approved table(s) ready to compile." % tables.size()
	physical_status.add_theme_color_override("font_color", C_TEXT)

# Only column names that appeared in an APPROVED relationship touching
# this table are known here -- pdm-discover's JSON output does not carry
# a full column enumeration per table (only relationship columns), so
# any other real attribute the stakeholder wants must still be added by
# hand in the Text tab. Named limitation, not silently papered over.
func _attrs_for_table(table_name: String) -> Array:
	var attrs: Array = []
	for rel in _approved_proposal.get("relationships", []):
		if str(rel.get("from_table", "")) == table_name:
			var c: String = str(rel.get("from_column", ""))
			if not c.is_empty() and not attrs.has(c):
				attrs.append(c)
		if str(rel.get("to_table", "")) == table_name:
			var c: String = str(rel.get("to_column", ""))
			if not c.is_empty() and not attrs.has(c):
				attrs.append(c)
	return attrs

func _on_physical_load_pressed(source_tabs: TabContainer, text_tab: VBoxContainer) -> void:
	if physical_table_picker.item_count == 0:
		physical_status.text = "[ERROR] No approved tables -- go back to the Logical tab."
		physical_status.add_theme_color_override("font_color", C_FUZZY)
		return
	var table_name: String = physical_table_picker.get_item_text(physical_table_picker.selected)
	var attrs: Array = _attrs_for_table(table_name)

	var lines: Array = []
	lines.append("MODEL %s_discovered" % table_name)
	lines.append("WHO %s.E" % table_name)
	for a in attrs:
		lines.append("ATTR " + a)
	lines.append("LIMIT 1000")
	akk_edit.text = "\n".join(lines)

	physical_status.text = "Loaded %s (%d known attribute(s) from approved relationships) into the Text tab. Compile / Compile & Run there." % [table_name, attrs.size()]
	physical_status.add_theme_color_override("font_color", C_TEXT)
	source_tabs.current_tab = text_tab.get_index()

func _on_compile_pressed():
	_compile()

func _on_compile_run_pressed():
	var akk_result = _compile()
	if akk_result.ok and _run_callback.is_valid():
		_run_callback.call(akk_result.heptascript)

func _compile() -> AkkadiCompiler.CompileResult:
	var result = AkkadiCompiler.compile(akk_edit.text)
	if result.ok:
		hepta_preview.text = result.heptascript
		compile_errors.text = ""
	else:
		hepta_preview.text = "—"
		var msg = ""
		for e in result.errors:
			msg += "• " + e + "\n"
		compile_errors.text = msg.strip_edges()
	return result

func on_query_result(fanout_result: Dictionary) -> void:
	for child in model_grid.get_children():
		child.queue_free()

	var all_rows: Array = []
	for pt in fanout_result.per_target:
		if pt.ok:
			for row in pt.rows:
				all_rows.append({"row": row, "engine": pt.engine})

	if all_rows.is_empty():
		summary_label.text = "No rows returned (contacted=%d failed=%d)." % [
			fanout_result.targets_contacted, fanout_result.targets_failed]
		return

	var gem = 0
	var orb = 0
	var fuzzy = 0
	for entry in all_rows:
		var b11 = _b11_from_attrs(entry.row.get("attrs", []))
		if b11 >= 200:
			gem += 1
		elif b11 >= 128:
			orb += 1
		else:
			fuzzy += 1
		model_grid.add_child(_row_widget(entry.row, entry.engine, b11))

	summary_label.text = "%d particle(s)  GEM:%d  ORBIT:%d  FUZZY:%d" % [
		all_rows.size(), gem, orb, fuzzy]

# Identical rule to dubsar_proof.gd's _b11_from_attrs(): B11 comes from an
# explicit "b11" EAV attribute in the WHAT projection, never from raw KAKI
# bytes. Defaults to 128 (mid-tier/ORBIT) if the model didn't ATTR b11.
func _b11_from_attrs(attrs: Array) -> int:
	for pair in attrs:
		if pair is Array and pair.size() >= 2 and str(pair[0]) == "b11":
			return int(str(pair[1]).to_int())
	return 128

func _row_widget(row: Dictionary, engine: String, b11: int) -> PanelContainer:
	var panel = PanelContainer.new()
	var style = StyleBoxFlat.new()
	style.bg_color = DubSarTheme.PANEL_BG_HOVER
	style.corner_radius_top_left = 6
	style.corner_radius_top_right = 6
	style.corner_radius_bottom_left = 6
	style.corner_radius_bottom_right = 6
	style.content_margin_left = 10
	style.content_margin_top = 6
	style.content_margin_right = 10
	style.content_margin_bottom = 6
	panel.add_theme_stylebox_override("panel", style)

	var hbox = HBoxContainer.new()
	hbox.add_theme_constant_override("separation", 10)
	panel.add_child(hbox)

	var dot = ColorRect.new()
	dot.custom_minimum_size = Vector2(10, 10)
	if b11 >= 200:
		dot.color = C_GEM
	elif b11 >= 128:
		dot.color = C_ORBIT
	else:
		dot.color = C_FUZZY
	hbox.add_child(dot)

	var engine_lbl = Label.new()
	engine_lbl.text = engine
	engine_lbl.custom_minimum_size = Vector2(70, 0)
	engine_lbl.add_theme_color_override("font_color", C_DIM)
	hbox.add_child(engine_lbl)

	var kaki_lbl = Label.new()
	kaki_lbl.text = str(row.get("kaki", "—"))
	kaki_lbl.custom_minimum_size = Vector2(140, 0)
	kaki_lbl.add_theme_color_override("font_color", C_TEXT)
	hbox.add_child(kaki_lbl)

	var attrs_text = ""
	for pair in row.get("attrs", []):
		if pair is Array and pair.size() >= 2:
			if not attrs_text.is_empty():
				attrs_text += "  "
			attrs_text += "%s=%s" % [pair[0], pair[1]]
	var attrs_lbl = Label.new()
	attrs_lbl.text = attrs_text
	attrs_lbl.add_theme_color_override("font_color", C_TEXT)
	attrs_lbl.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	attrs_lbl.autowrap_mode = TextServer.AUTOWRAP_OFF
	attrs_lbl.clip_text = true
	hbox.add_child(attrs_lbl)

	return panel
