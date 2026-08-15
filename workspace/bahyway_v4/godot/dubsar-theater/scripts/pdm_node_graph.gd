class_name PdmNodeGraph
extends GraphEdit
# =============================================================================
# PDM Node Graph — 2026-07-21, Houdini-style procedural front-end for .akk.
#
# HONEST SCOPE: this graph compiles to real .akk SOURCE TEXT, which then
# runs through the exact same, already-proven AkkadiCompiler.compile()
# (scripts/akkadi_compiler.gd) pdm_tab.gd's text mode has always used — no
# second/divergent compiler exists here. The graph is a visual way to
# assemble the same 5-clause grammar AkkadiCompiler already supports
# (MODEL / NODE / WHO / ATTR / LIMIT); nothing about query execution
# changes.
#
# SCHEMA_DISCOVERY (added 2026-07-31) is the Conceptual-tab node this file
# used to defer: a real relationship-based complex-builder for DubSar PDM
# schema discovery, backed by crates/pdm-discovery + bin/pdm-discover (a
# v1 exact-value-overlap join-key heuristic feeding
# bahyway-algebra::persistence::clique_complex_persistence — see that
# crate's own module doc for its named limitations: exact-match only, no
# renamed/reformatted-key or semantic matching). Still no GDExtension
# bridge into Godot exists (confirmed absent, same as always — see
# akkadi_compiler.gd's own header for the same gap on HeptaScript
# emission), so this node shells out to the compiled `pdm-discover` CLI
# binary via OS.execute rather than linking the Rust crate directly: a
# real, working call to the actual discovery code, not a client-side
# reimplementation that could drift from it. NOT exercised under a live
# Godot runtime in this session (none available in this sandbox);
# reviewed statically only — verify by hand before relying on it.
#
# Deliberately still NOT included, and not faked with placeholder math: a
# GA (Clifford algebra) node. It has a real, working, tested Rust
# implementation (bahyway-algebra::clifford — Cl(7,0), 128-blade
# Multivector, geometric()/wedge()/grade()) but no bridge into Godot and
# no CLI front end yet either — genuine follow-up work, not built today.
#
# Node topology contract (this is what the wires mean):
#   MODEL  --(port 0)--> TARGET --(port 0)--> [nothing required downstream]
#   ENTITY (WHO) is the one mandatory node. Its OWN input port (row 0) is
#   purely informational (not required to be wired) — MODEL/TARGET are
#   resolved by simply being present in the graph (a query has at most one
#   of each), matching AkkadiCompiler's own grammar where NODE/MODEL are
#   global clauses, not per-entity ones.
#   ENTITY exposes two OUTPUT ports that DO matter and must be wired:
#     port "attrs" — every ATTR node connected here becomes a WHAT
#                    projection field, in connection order.
#     port "limit" — a LIMIT node connected here supplies HOW_MUCH LIMIT.
#   Unconnected ATTR/LIMIT nodes are ignored (dangling scratch nodes are
#   allowed, matching Houdini's own "unwired nodes just don't cook").
# =============================================================================

const DubSarTheme = preload("res://scripts/dubsar_theme.gd")

enum NodeKind { MODEL, TARGET, ENTITY, ATTR, LIMIT, SCHEMA_DISCOVERY }

const KIND_NAMES := {
	NodeKind.MODEL: "Model",
	NodeKind.TARGET: "Target (NODE)",
	NodeKind.ENTITY: "Entity (WHO)",
	NodeKind.ATTR: "Attribute (ATTR)",
	NodeKind.LIMIT: "Limit",
	NodeKind.SCHEMA_DISCOVERY: "Schema Discovery (PDM v1)",
}

# `pdm-discover` must be built (`cargo build -p pdm-discover` from the
# workspace root) and reachable on PATH; edit this to an absolute path if
# it isn't. See the SCHEMA_DISCOVERY node comment above for what this
# calls and why a subprocess, not a GDExtension link.
const PDM_DISCOVER_BIN := "pdm-discover"

var PORT_COLOR_IN: Color = DubSarTheme.ACCENT_AMBER
var PORT_COLOR_OUT: Color = DubSarTheme.ACCENT_TEAL

var _node_kind: Dictionary = {}    # node name -> NodeKind
var _node_fields: Dictionary = {}  # node name -> Dictionary of field controls
var _next_id: int = 0
var _layout_next: int = 0

func _ready() -> void:
	right_disconnects = true
	connection_request.connect(_on_connection_request)
	disconnection_request.connect(_on_disconnection_request)
	DubSarTheme.apply_page_background(self)

func _on_connection_request(from_node: StringName, from_port: int, to_node: StringName, to_port: int) -> void:
	for c in get_connection_list():
		if c["from_node"] == from_node and c["from_port"] == from_port \
				and c["to_node"] == to_node and c["to_port"] == to_port:
			return
	connect_node(from_node, from_port, to_node, to_port)

func _on_disconnection_request(from_node: StringName, from_port: int, to_node: StringName, to_port: int) -> void:
	disconnect_node(from_node, from_port, to_node, to_port)

func set_field_text(gn: GraphNode, field: String, value: String) -> void:
	var fields: Dictionary = _node_fields.get(gn.name, {})
	if fields.has(field) and fields[field] is LineEdit:
		(fields[field] as LineEdit).text = value

func set_target_checked(gn: GraphNode, engine_name: String, checked: bool) -> void:
	var fields: Dictionary = _node_fields.get(gn.name, {})
	for cb in fields.get("targets", []):
		if (cb as CheckBox).text == engine_name:
			(cb as CheckBox).button_pressed = checked

func clear_graph() -> void:
	for child in get_children():
		if child is GraphNode:
			child.queue_free()
	_node_kind.clear()
	_node_fields.clear()
	_layout_next = 0

func add_node_of_kind(kind: NodeKind) -> GraphNode:
	var gn := _make_node(kind)
	gn.position_offset = (scroll_offset + _next_position()) / zoom
	add_child(gn)
	return gn

func _next_position() -> Vector2:
	var col := _layout_next % 3
	var row := _layout_next / 3
	_layout_next += 1
	return Vector2(col * 260 + 40, row * 180 + 40)

func _make_node(kind: NodeKind) -> GraphNode:
	_next_id += 1
	var gn := GraphNode.new()
	gn.name = "PDM_%d_%s" % [_next_id, NodeKind.keys()[kind]]
	gn.title = KIND_NAMES[kind]
	gn.resizable = false
	gn.custom_minimum_size = Vector2(210, 0)
	_node_kind[gn.name] = kind
	_node_fields[gn.name] = {}

	var panel_sb = DubSarTheme.card_stylebox(6)
	gn.add_theme_stylebox_override("panel", panel_sb)
	var titlebar_sb = DubSarTheme.header_stylebox()
	titlebar_sb.set_corner_radius_all(0)
	titlebar_sb.corner_radius_top_left = 6
	titlebar_sb.corner_radius_top_right = 6
	gn.add_theme_stylebox_override("titlebar", titlebar_sb)

	match kind:
		NodeKind.MODEL:
			_row_label(gn, "→ target/entity", false, true)
			_node_fields[gn.name]["name"] = _row_line_edit(gn, "name", "najaf_deaths")
		NodeKind.TARGET:
			_row_label(gn, "in ←", true, false)
			var box := VBoxContainer.new()
			gn.add_child(box)
			var checks: Array = []
			for engine_name in EnkiEngines.TABLE.keys():
				var cb := CheckBox.new()
				cb.text = engine_name
				box.add_child(cb)
				checks.append(cb)
			_node_fields[gn.name]["targets"] = checks
			_row_label(gn, "→ entity", false, true)
		NodeKind.ENTITY:
			_row_label(gn, "in ← (informational)", true, false)
			_node_fields[gn.name]["tribe"] = _row_line_edit(gn, "Tribe", "Tribes")
			_node_fields[gn.name]["var"] = _row_line_edit(gn, "Var", "E")
			_row_label(gn, "attrs →", false, true)
			_row_label(gn, "limit →", false, true)
		NodeKind.ATTR:
			_row_label(gn, "in ←", true, false)
			_node_fields[gn.name]["attr"] = _row_line_edit(gn, "attr.path", "meta.title")
		NodeKind.LIMIT:
			_row_label(gn, "in ←", true, false)
			_node_fields[gn.name]["limit"] = _row_line_edit(gn, "n", "100")
		NodeKind.SCHEMA_DISCOVERY:
			_row_label(gn, "in ← (informational)", true, false)
			_node_fields[gn.name]["tables"] = _row_line_edit(
				gn, "tables", "customers=res://data/customers.csv,orders=res://data/orders.csv"
			)
			var run_btn := Button.new()
			run_btn.text = "▶ Discover Schema"
			gn.add_child(run_btn)
			var run_idx := gn.get_child_count() - 1
			gn.set_slot(run_idx, false, 0, PORT_COLOR_IN, false, 0, PORT_COLOR_OUT)
			var out_label := Label.new()
			out_label.text = "(not run yet)"
			out_label.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
			out_label.add_theme_font_size_override("font_size", 10)
			gn.add_child(out_label)
			var out_idx := gn.get_child_count() - 1
			gn.set_slot(out_idx, false, 0, PORT_COLOR_IN, false, 0, PORT_COLOR_OUT)
			_node_fields[gn.name]["result_label"] = out_label
			var captured_name: StringName = gn.name
			run_btn.pressed.connect(func(): _run_schema_discovery(captured_name))

	return gn

# Runs `pdm-discover <table=path.csv> ...` as a subprocess and parses its
# real JSON stdout (see this file's own header comment for the honest
# scope: v1 exact-value-overlap heuristic, subprocess not GDExtension).
# The parsed proposal is stashed on the node's own field dict under
# "proposal" for a later stage (the Logical-tab review UI) to read; this
# node itself only detects and displays, it never applies anything.
func _run_schema_discovery(node_name: StringName) -> void:
	var fields: Dictionary = _node_fields.get(node_name, {})
	var tables_field: LineEdit = fields.get("tables")
	var result_label: Label = fields.get("result_label")
	if tables_field == null or result_label == null:
		return
	var spec: String = tables_field.text.strip_edges()
	var args: Array = []
	for part in spec.split(","):
		var p: String = part.strip_edges()
		if not p.is_empty():
			args.append(p)
	if args.is_empty():
		result_label.text = "[ERROR] no tables configured"
		return

	var output: Array = []
	var exit_code := OS.execute(PDM_DISCOVER_BIN, args, output, true)
	if exit_code != 0:
		result_label.text = "[ERROR] pdm-discover exited %d: %s" % [exit_code, "\n".join(output)]
		return
	var stdout: String = "\n".join(output).strip_edges()
	var parsed = JSON.parse_string(stdout)
	if parsed == null or not (parsed is Dictionary):
		result_label.text = "[ERROR] could not parse pdm-discover output: %s" % stdout
		return

	fields["proposal"] = parsed
	var tables_out: Array = parsed.get("tables", [])
	var rels_out: Array = parsed.get("relationships", [])
	result_label.text = "%d table(s), %d relationship(s), %d void(s)" % [
		tables_out.size(), rels_out.size(), int(parsed.get("void_count", 0))
	]

# Every SCHEMA_DISCOVERY node's parsed pdm-discover proposal (empty for a
# node that hasn't been run yet). Public so the Logical tab (pdm_tab.gd)
# can pull discovered proposals to review without reaching into this
# graph's own private `_node_fields`.
func get_schema_discovery_proposals() -> Array:
	var out: Array = []
	for node_name in _node_kind:
		if _node_kind[node_name] != NodeKind.SCHEMA_DISCOVERY:
			continue
		var proposal = _node_fields.get(node_name, {}).get("proposal")
		if proposal is Dictionary:
			out.append(proposal)
	return out

# Adds a row to `gn` and configures its connection slot (the row's index
# among gn's children is what set_slot's `idx` refers to, per Godot's
# GraphNode API — same convention graph_explorer.gd already uses).
func _row_label(gn: GraphNode, text: String, port_in: bool, port_out: bool) -> void:
	var lbl := Label.new()
	lbl.text = text
	lbl.add_theme_font_size_override("font_size", 11)
	gn.add_child(lbl)
	var idx := gn.get_child_count() - 1
	gn.set_slot(idx, port_in, 0, PORT_COLOR_IN, port_out, 0, PORT_COLOR_OUT)

func _row_line_edit(gn: GraphNode, label_text: String, placeholder: String) -> LineEdit:
	var row := HBoxContainer.new()
	gn.add_child(row)
	var idx := gn.get_child_count() - 1
	gn.set_slot(idx, false, 0, PORT_COLOR_IN, false, 0, PORT_COLOR_OUT)
	var lbl := Label.new()
	lbl.text = label_text
	lbl.custom_minimum_size = Vector2(40, 0)
	lbl.add_theme_font_size_override("font_size", 10)
	row.add_child(lbl)
	var edit := LineEdit.new()
	edit.placeholder_text = placeholder
	edit.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	row.add_child(edit)
	return edit

# Row index of the ENTITY node's two output ports — fixed because
# _make_node always builds ENTITY's rows in the same order (see above:
# row 0 = in, row 1 = Tribe, row 2 = Var, row 3 = attrs out, row 4 = limit out).
const ENTITY_ATTRS_PORT := 3
const ENTITY_LIMIT_PORT := 4

class CompileResult:
	var ok: bool = true
	var akk_source: String = ""
	var errors: Array = []

func compile_to_akk() -> CompileResult:
	var result := CompileResult.new()

	var model_names: Array = []
	var target_names: Array = []
	var entity_names: Array = []
	for node_name in _node_kind:
		match _node_kind[node_name]:
			NodeKind.MODEL: model_names.append(node_name)
			NodeKind.TARGET: target_names.append(node_name)
			NodeKind.ENTITY: entity_names.append(node_name)

	if entity_names.size() != 1:
		result.errors.append("Exactly one Entity (WHO) node is required, found %d." % entity_names.size())
		result.ok = false
		return result
	var entity_name: String = entity_names[0]
	var tribe: String = (_node_fields[entity_name]["tribe"] as LineEdit).text.strip_edges()
	var var_name: String = (_node_fields[entity_name]["var"] as LineEdit).text.strip_edges()
	if tribe.is_empty() or var_name.is_empty():
		result.errors.append("Entity node needs both a Tribe and a Var name.")
		result.ok = false
		return result

	var lines: Array = []

	if model_names.size() > 1:
		result.errors.append("At most one Model node is allowed, found %d." % model_names.size())
		result.ok = false
		return result
	if model_names.size() == 1:
		var name: String = (_node_fields[model_names[0]]["name"] as LineEdit).text.strip_edges()
		if not name.is_empty():
			lines.append("MODEL " + name)

	if target_names.size() > 1:
		result.errors.append("At most one Target (NODE) node is allowed, found %d." % target_names.size())
		result.ok = false
		return result
	if target_names.size() == 1:
		var checks: Array = _node_fields[target_names[0]]["targets"]
		var picked: Array = []
		for cb in checks:
			if (cb as CheckBox).button_pressed:
				picked.append((cb as CheckBox).text)
		if not picked.is_empty():
			lines.append("NODE " + " | ".join(picked))

	lines.append("WHO %s.%s" % [tribe, var_name])

	# Entity's "attrs"/"limit" rows are OUTPUT ports (right side); ATTR/LIMIT
	# nodes' own single row is an INPUT port (left side) -- so the wire
	# direction is Entity -> Attr/Limit, i.e. from_node/from_port is the
	# entity side here, not the other way around.
	for c in get_connection_list():
		if c["from_node"] != entity_name or int(c["from_port"]) != ENTITY_ATTRS_PORT:
			continue
		var to_node: String = c["to_node"]
		if _node_kind.get(to_node, -1) != NodeKind.ATTR:
			continue
		var attr_path: String = (_node_fields[to_node]["attr"] as LineEdit).text.strip_edges()
		if not attr_path.is_empty():
			lines.append("ATTR " + attr_path)

	for c in get_connection_list():
		if c["from_node"] != entity_name or int(c["from_port"]) != ENTITY_LIMIT_PORT:
			continue
		var to_node: String = c["to_node"]
		if _node_kind.get(to_node, -1) != NodeKind.LIMIT:
			continue
		var n: String = (_node_fields[to_node]["limit"] as LineEdit).text.strip_edges()
		if n.is_valid_int():
			lines.append("LIMIT " + n)
		break  # only the first connected Limit node counts, same as AkkadiCompiler's single LIMIT clause

	result.akk_source = "\n".join(lines)
	return result
