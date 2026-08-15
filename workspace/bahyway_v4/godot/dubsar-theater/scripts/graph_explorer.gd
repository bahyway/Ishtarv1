extends Control
# =============================================================================
# Graph Explorer — PB-218/219 (2026-07-20)
#
# v1 scope, agreed with the Architect before building: a STATIC graph from
# one query at a time -- nodes = documents/sections, edges = the real
# link.target/link.description particles `crates/enkiddb/src/emitter.rs`
# already mints (emit_link, reused by both emit_sections' "section-of" and
# emit_concept_mentions' "mentioned-in"). Choose a node to see its full
# projected attrs and re-expand the graph from it -- no live/saved
# investigations yet.
#
# Real constraint this design works around: enkidb-readnode's ReadNode can
# only serve a WHERE clause with a leading exact-equality condition (see
# crates/enkidb-readnode/src/readnode.rs::indexable_condition) -- there is
# no "list everything" or "get by kaki" query. So a node's *own* row is
# always found by an exact meta.title match, and an edge's far end (only
# known as a raw link.target KAKI hex) is labeled using the title Sumerian
# convention `emit_sections` already bakes in: a section's own title is
# literally "<parent_title> § <header>", so the parent's title is
# recoverable by splitting on " § " without a second round trip -- clicking
# that stub node re-queries by the recovered title to pull its real row in.
#
# This only surfaces edges HeptaScript can express in one WHAT projection,
# so one QUERY per Expand click is enough: WHAT already asks for
# meta.title/meta.collection/link.target/link.description together, and
# HeptaScript silently omits attrs a row doesn't have (see
# crates/heptascript/src/engine.rs::project_what) rather than erroring, so
# a plain document (no link.* of its own) and a section/mention (has both)
# both come back from the same query shape.
#
# PB-218 discovered and fixed the bug that made this possible at all:
# enkiddb-read-server's fmt_val() was serializing link.target's
# AkkValue::KakiPk via Rust's derived Debug format instead of hex --
# live-verified fixed before any of this UI was written.
#
# Reuses SumuUkinClient (scripts/sumuukin_client.gd), the same real
# single-frame-then-0-sentinel TCP client the HeptaScript editor already
# uses against enkiddb-read-server -- no new wire protocol code here.
#
# UPDATED 2026-08-02, live: v1's "one QUERY per Expand" scope above turned
# out to be exactly the gap that made anu_governor::pb_catalog's location
# nodes look empty -- a location can have hundreds of "contains" edges, but
# a single meta.title match only ever returns THAT ONE entity's own folded
# current-state row (heptascript::engine::apply_entry_to_map is last-write-
# wins per attribute on one entity), never the many separate edge entities
# pointing at it. Expand now fires a SECOND query,
# `WHERE E[link.source_title] = "<title>"`, which finds every dedicated
# edge entity `enkiddb::WriteNode::mint_link_edge` minted with that exact
# title as its source -- see _query_outgoing_edges/_on_edges_complete
# below. Each edge carries its target's real title directly
# (link.target_title), so the newly-found node never needs the old
# stub/" § "-guessing trick to be labeled correctly.
# =============================================================================

const SumuUkinClient = preload("res://scripts/sumuukin_client.gd")
const SECTION_SEP := " § "  # matches DocumentEmitter::emit_sections exactly

@onready var host_input: LineEdit = %HostInput
@onready var title_input: LineEdit = %TitleInput
@onready var btn_search: Button = %BtnSearch
@onready var btn_clear: Button = %BtnClear
@onready var graph: GraphEdit = %Graph
@onready var details: RichTextLabel = %Details
@onready var status_label: Label = %Status

var _nodes: Dictionary = {}       # kaki_hex -> GraphNode
var _node_attrs: Dictionary = {}  # kaki_hex -> Array of [key, value] pairs
var _layout_next: int = 0

func open() -> void:
	visible = true
	title_input.grab_focus()

func close() -> void:
	visible = false

func _ready():
	visible = false
	btn_search.pressed.connect(_on_search_pressed)
	btn_clear.pressed.connect(_on_clear_pressed)
	title_input.text_submitted.connect(func(_t): _on_search_pressed())
	graph.node_selected.connect(_on_node_selected)
	# FIXED 2026-08-02, live: this and _on_clear_pressed's reset text below
	# both leaked internal HeptaScript query syntax ("WHO T.E") into
	# user-facing copy -- confirmed live, a real user typed "WHO T.E
	# Nergal" into the title field itself, reading it as an instruction
	# rather than an explanation of what Expand does under the hood.
	# Plain instructions only here now, with a concrete real example.
	_status("Type the EXACT title of something already in EnkiDDB (not a query) -- e.g. \"Playbook catalog location: in-repo/playbooks\" -- then press Expand.")

func _input(event: InputEvent) -> void:
	if not visible:
		return
	if event is InputEventKey and event.pressed and event.keycode == KEY_ESCAPE:
		close()
		get_viewport().set_input_as_handled()

func _on_clear_pressed() -> void:
	for gn in _nodes.values():
		gn.queue_free()
	_nodes.clear()
	_node_attrs.clear()
	_layout_next = 0
	details.text = "Type an exact title above and press Expand to see it here, then click any connected node to follow its links."
	_status("Cleared.")

func _on_search_pressed() -> void:
	var title := title_input.text.strip_edges()
	if title.is_empty():
		_status("Enter a meta.title first.")
		return
	var host := host_input.text.strip_edges()
	if host.is_empty():
		# FIXED 2026-08-02, live: was "192.168.122.107" -- a stale
		# eriduous-vdi-era libvirt IP. This host and DubSar Theater are
		# now both local to the same bare-metal machine (see PB-290),
		# so 127.0.0.1 is the correct default, not a remote VM address.
		host = "127.0.0.1"
	var escaped := title.replace("\"", "\\\"")
	var query_text := "QUERY:WHO T.E\nWHAT E[meta.title, meta.collection, link.target, link.description]\nWHERE E[meta.title] = \"%s\"" % escaped

	btn_search.disabled = true
	# FIXED 2026-08-02, live: was hardcoded to port 7102, the stray
	# default enkiddb-read-server itself moved away from on 2026-07-31
	# (corrected to 7007, its own doc comment explains why). Graph
	# Explorer's port was never updated to match, so it could never
	# reach a real, current enkiddb-read-server regardless of host.
	_status("Querying EnkiDDB %s:7007 for meta.title = \"%s\" ..." % [host, title])

	var thread := Thread.new()
	thread.start(func():
		var result = SumuUkinClient.route(
			[{"engine": "EnkiDDB", "host": host, "port": 7007}],
			query_text,
			SumuUkinClient.FanOutPolicy.FIRST_ONLY,
		)
		call_deferred("_on_search_complete", result, thread, title, host)
	)

func _on_search_complete(result: Dictionary, thread: Thread, title: String, host: String) -> void:
	thread.wait_to_finish()

	if result.per_target.is_empty() or not result.per_target[0].ok:
		var err: String = result.per_target[0].error if not result.per_target.is_empty() else "no target contacted"
		btn_search.disabled = false
		_status("[ERROR] %s" % err)
		return

	var rows: Array = result.per_target[0].rows
	if rows.is_empty():
		btn_search.disabled = false
		_status("No entity with that exact meta.title. HeptaScript title matches are exact, not substring.")
		return

	for row in rows:
		_ingest_row(row)
	var source_kaki: String = str(rows[0].get("kaki", ""))
	_query_outgoing_edges(title, host, source_kaki, rows.size())

# FIXED 2026-08-02, live: a title match alone only ever showed the ONE row
# for that exact title -- e.g. a playbook-catalog location entity querying
# as "2 node(s) total" no matter how many playbooks were actually catalogued
# there. Root cause: every "contains"/"found-in" edge used to be written as
# a repeated link.target/link.description particle on the SAME shared
# location entity, and heptascript::engine::apply_entry_to_map folds an
# entity's whole history into a single-slot (attr_hash -> value) map before
# projecting -- so only the LAST edge written ever survived a query. Fixed
# at the source in enkiddb::WriteNode::mint_link_edge (anu_governor::
# pb_catalog now calls it): every edge is its OWN dedicated entity, carrying
# a plain-Text link.source_title/link.target_title pair alongside the real
# KakiPk link.source/link.target (which can never be compared in a WHERE
# clause at all -- heptascript::engine::compare_val has no AkkValue::KakiPk
# arm). This second query finds every one of those edge entities sourced
# from `title`, however many there are.
func _query_outgoing_edges(title: String, host: String, source_kaki: String, found_count: int) -> void:
	var escaped := title.replace("\"", "\\\"")
	var edge_query := "QUERY:WHO T.E\nWHAT E[link.target, link.target_title, link.description]\nWHERE E[link.source_title] = \"%s\"" % escaped
	var thread := Thread.new()
	thread.start(func():
		var result = SumuUkinClient.route(
			[{"engine": "EnkiDDB", "host": host, "port": 7007}],
			edge_query,
			SumuUkinClient.FanOutPolicy.FIRST_ONLY,
		)
		call_deferred("_on_edges_complete", result, thread, source_kaki, found_count)
	)

func _on_edges_complete(result: Dictionary, thread: Thread, source_kaki: String, found_count: int) -> void:
	thread.wait_to_finish()
	btn_search.disabled = false

	var linked := 0
	if not result.per_target.is_empty() and result.per_target[0].ok:
		for row in result.per_target[0].rows:
			var target_kaki: String = _attr(row, "link.target")
			var target_title: String = _attr(row, "link.target_title")
			var desc: String = _attr(row, "link.description")
			if target_kaki.is_empty() or target_kaki == source_kaki or not _nodes.has(source_kaki):
				continue
			if not _nodes.has(target_kaki):
				var label := target_title if not target_title.is_empty() else ("kaki:" + target_kaki.substr(0, 12) + "…")
				_upsert_node(target_kaki, label, [])
			_connect(target_kaki, source_kaki, desc)
			linked += 1
	_status("Expanded: %d row(s) matched the title, %d linked node(s) found, %d node(s) total." % [found_count, linked, _nodes.size()])

# One matched entity -> its own node, plus (if it carries link.target/
# link.description, i.e. it's a section or concept-mention) an edge to its
# parent, added as a stub node if the parent hasn't been queried directly
# yet. See file header for why the parent's title is recoverable from the
# child's own title rather than a second query.
func _ingest_row(row: Dictionary) -> void:
	var kaki: String = str(row.get("kaki", ""))
	if kaki.is_empty():
		return
	var title := _attr(row, "meta.title")
	var link_target := _attr(row, "link.target")
	var link_desc := _attr(row, "link.description")

	_upsert_node(kaki, title if not title.is_empty() else kaki, row.get("attrs", []))

	if not link_target.is_empty():
		var parent_title := ""
		if title.find(SECTION_SEP) != -1:
			parent_title = title.split(SECTION_SEP)[0]
		var parent_label := parent_title if not parent_title.is_empty() else ("kaki:" + link_target.substr(0, 12) + "…")
		if not _nodes.has(link_target):
			_upsert_node(link_target, parent_label, [])
		_connect(kaki, link_target, link_desc)

func _attr(row: Dictionary, key: String) -> String:
	for pair in row.get("attrs", []):
		if pair is Array and pair.size() >= 2 and pair[0] == key:
			return str(pair[1])
	return ""

func _upsert_node(kaki: String, label: String, attrs: Array) -> void:
	if _nodes.has(kaki):
		var gn: GraphNode = _nodes[kaki]
		gn.title = label
		if not attrs.is_empty():
			_node_attrs[kaki] = attrs
		return

	var gn := GraphNode.new()
	gn.title = label
	gn.name = "N_" + kaki.substr(0, 24)
	gn.position_offset = _next_position()
	gn.resizable = false
	gn.custom_minimum_size = Vector2(200, 0)

	var lbl := Label.new()
	lbl.text = kaki.substr(0, 16) + "…"
	lbl.add_theme_font_size_override("font_size", 10)
	gn.add_child(lbl)

	gn.set_slot(0, true, 0, Color(0.937, 0.624, 0.153), true, 0, Color(0.114, 0.620, 0.459))

	graph.add_child(gn)
	_nodes[kaki] = gn
	_node_attrs[kaki] = attrs

func _connect(child_kaki: String, parent_kaki: String, label: String) -> void:
	var from_node: GraphNode = _nodes[child_kaki]
	var to_node: GraphNode = _nodes[parent_kaki]
	for c in graph.get_connection_list():
		if c["from_node"] == from_node.name and c["to_node"] == to_node.name:
			return
	graph.connect_node(from_node.name, 0, to_node.name, 0)
	if not label.is_empty():
		_status("Linked %s -> %s (%s)" % [from_node.title, to_node.title, label])

# Grid layout, filled column-major so a chain of expansions reads roughly
# left-to-right in the order it was discovered -- no force-directed layout
# in v1, deliberately: with an exact-title-match entry point the graph
# stays small (one query's worth of rows plus their immediate parents),
# so a grid is legible without the complexity of a real layout algorithm.
func _next_position() -> Vector2:
	var col := _layout_next % 5
	var row := _layout_next / 5
	_layout_next += 1
	return Vector2(col * 260, row * 160)

func _on_node_selected(node: GraphNode) -> void:
	var kaki := ""
	for k in _nodes:
		if _nodes[k] == node:
			kaki = k
			break
	if kaki.is_empty():
		return

	title_input.text = node.title
	var attrs: Array = _node_attrs.get(kaki, [])
	var body := "[b]%s[/b]\nkaki: %s\n\n" % [node.title.xml_escape(), kaki]
	if attrs.is_empty():
		body += "[i]Stub node -- only known via a link.target reference.\nPress Expand to fetch its real attrs.[/i]"
	else:
		for pair in attrs:
			if pair is Array and pair.size() >= 2:
				body += "[color=#9f9fb8]%s[/color] = %s\n" % [str(pair[0]).xml_escape(), str(pair[1]).xml_escape()]
	details.text = body

func _status(msg: String) -> void:
	status_label.text = msg
