extends PanelContainer
class_name DocumentExplorer
# =============================================================================
# DocumentExplorer — New DubSar Theater, 2026-07-21.
#
# The tree-sidebar + content + related-links document browser from the
# Architect's own reference screenshots. New scene -- no prior
# "Document Explorer" existed in this project (graph_explorer.gd is a
# different thing, a GraphEdit node-link view). Built against EnkiDDB
# (Tigris) specifically, since it's the one real document-shaped type
# (meta.title/meta.collection/body.summary/body.text/link.target/
# link.description -- see docs/ARCHITECT_DESIGN_ENKIDDB_ENKIMDB_PODMAN.md
# §3 for the real EAV schema this reads).
#
# Real data only, two honest omissions from the reference mockup rather
# than faked equivalents:
#   - No "KAKI VECTOR" readout. The mockup shows a 7-float vector per
#     document; nothing in EnkiDDB's real schema or HeptaScript's own
#     projection produces a per-document 7D coordinate today (GeoLaw-01's
#     E7 lattice orbital zones are real, crates/bahyway-algebra, but nothing
#     wires a document's KAKI through that machinery yet). Shows the
#     document's real 16-byte KAKI (hex) instead -- smaller claim, still
#     true.
#   - No "7D KAKI Orbit" mini radar widget, for the same reason -- nothing
#     computes a real per-document orbital position to plot. Omitted
#     rather than decorated with placeholder numbers.
# Related Links IS real: link.target (raw KAKI hex) + link.description
# (free text set by the emitter -- crates/enkiddb/src/emitter.rs::
# emit_link, reused by emit_sections' "section-of" and
# emit_concept_mentions' "mentioned-in") are real particles on real
# documents, shown as-is, not relabeled or guessed at.
#
# Tree population and full-content fetch both go through the real,
# now-fixed SumuUkinClient binary-wire client. The tree query is
# WHERE-less (`WHO T.E WHAT E[meta.title, meta.collection] HOW_MUCH LIMIT
# 200`) -- valid because every real Read Node now runs CachedReadNode,
# whose full-scan fallback answers WHERE-less queries directly (proven
# this session, cached_serves_queries_readnode_would_reject test); the
# older ReadNode engine graph_explorer.gd's own header comment describes
# ("no WHERE, no list-everything") is not what any deployed Read Node
# actually runs anymore.
# =============================================================================

const SumuUkinClient = preload("res://scripts/sumuukin_client.gd")
const DubSarTheme = preload("res://scripts/dubsar_theme.gd")

const HOST := "192.168.122.107"
const PORT := 7102  # enkiddb-read-server (Tigris)
const TREE_LIMIT := 200

var _search_edit: LineEdit
var _tree: Tree
var _tree_root: TreeItem
var _collection_items: Dictionary = {}  # collection name -> TreeItem
var _title_by_tree_id: Dictionary = {}  # TreeItem instance id -> meta.title
var _content_root: VBoxContainer
var _status_label: Label
var _busy := false

func _ready() -> void:
	_build()
	_load_tree()

func _unhandled_input(event: InputEvent) -> void:
	if event is InputEventKey and event.pressed and not event.echo and event.keycode == KEY_ESCAPE:
		get_tree().change_scene_to_file("res://scenes/main_theater.tscn")

func _build() -> void:
	var sb = StyleBoxFlat.new()
	sb.bg_color = DubSarTheme.BG
	add_theme_stylebox_override("panel", sb)

	var root = VBoxContainer.new()
	root.add_theme_constant_override("separation", 0)
	add_child(root)

	root.add_child(_build_header())

	var hsplit = HSplitContainer.new()
	hsplit.size_flags_vertical = Control.SIZE_EXPAND_FILL
	hsplit.split_offset = 320
	root.add_child(hsplit)

	hsplit.add_child(_build_tree_panel())

	var scroll = ScrollContainer.new()
	scroll.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	hsplit.add_child(scroll)

	_content_root = VBoxContainer.new()
	_content_root.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	_content_root.add_theme_constant_override("separation", 14)
	var margin = MarginContainer.new()
	margin.add_theme_constant_override("margin_left", 20)
	margin.add_theme_constant_override("margin_top", 16)
	margin.add_theme_constant_override("margin_right", 20)
	margin.add_theme_constant_override("margin_bottom", 20)
	margin.add_child(_content_root)
	scroll.add_child(margin)

	_show_placeholder("Choose a document from the tree, or search above.")

func _build_header() -> PanelContainer:
	var header = PanelContainer.new()
	header.add_theme_stylebox_override("panel", DubSarTheme.header_stylebox())

	var hbox = HBoxContainer.new()
	hbox.add_theme_constant_override("separation", 12)
	header.add_child(hbox)

	var title = Label.new()
	title.text = "🔍 EnkiDDB Document Explorer"
	title.add_theme_font_size_override("font_size", DubSarTheme.SIZE_TITLE)
	title.add_theme_color_override("font_color", DubSarTheme.ACCENT_TEAL)
	hbox.add_child(title)

	_search_edit = LineEdit.new()
	_search_edit.placeholder_text = "Search term or expression (real SEARCH: against EnkiDDB's RAG)..."
	_search_edit.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	_search_edit.text_submitted.connect(func(_t): _run_search())
	hbox.add_child(_search_edit)

	var btn_search = Button.new()
	btn_search.text = "🔍 Search"
	btn_search.pressed.connect(_run_search)
	hbox.add_child(btn_search)

	var btn_refresh_tree = Button.new()
	btn_refresh_tree.text = "⟳ Reload tree"
	btn_refresh_tree.pressed.connect(_load_tree)
	hbox.add_child(btn_refresh_tree)

	return header

func _build_tree_panel() -> PanelContainer:
	var panel = PanelContainer.new()
	panel.custom_minimum_size = Vector2(280, 0)
	var sb = StyleBoxFlat.new()
	sb.bg_color = DubSarTheme.BG_ELEVATED
	sb.border_width_right = 1
	sb.border_color = DubSarTheme.BORDER
	sb.content_margin_left = 8
	sb.content_margin_top = 8
	sb.content_margin_right = 8
	sb.content_margin_bottom = 8
	panel.add_theme_stylebox_override("panel", sb)

	var vbox = VBoxContainer.new()
	panel.add_child(vbox)

	var lbl = Label.new()
	lbl.text = "📂 DOCUMENT TREE"
	lbl.add_theme_font_size_override("font_size", DubSarTheme.SIZE_LABEL)
	lbl.add_theme_color_override("font_color", DubSarTheme.TEXT_DIM)
	vbox.add_child(lbl)

	_tree = Tree.new()
	_tree.size_flags_vertical = Control.SIZE_EXPAND_FILL
	_tree.hide_root = true
	_tree.item_selected.connect(_on_tree_item_selected)
	vbox.add_child(_tree)

	_status_label = Label.new()
	_status_label.text = "Loading…"
	_status_label.add_theme_font_size_override("font_size", DubSarTheme.SIZE_LABEL)
	_status_label.add_theme_color_override("font_color", DubSarTheme.TEXT_FAINT)
	_status_label.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	vbox.add_child(_status_label)

	return panel

# ── Tree population (real, WHERE-less enumeration) ──────────────────────

func _load_tree() -> void:
	if _busy:
		return
	_busy = true
	_status_label.text = "Loading document tree…"

	var query = "WHO T.E\nWHAT E[meta.title, meta.collection]\nHOW_MUCH LIMIT %d" % TREE_LIMIT
	var target = {"engine": "EnkiDDB", "host": HOST, "port": PORT}

	var thread = Thread.new()
	thread.start(func():
		var result = SumuUkinClient.route([target], query, SumuUkinClient.FanOutPolicy.FIRST_ONLY)
		call_deferred("_on_tree_loaded", result, thread)
	)

func _on_tree_loaded(result: Dictionary, thread: Thread) -> void:
	thread.wait_to_finish()
	_busy = false

	_tree.clear()
	_collection_items.clear()
	_title_by_tree_id.clear()
	_tree_root = _tree.create_item()

	var pt = result.per_target[0] if not result.per_target.is_empty() else {"ok": false, "rows": [], "error": "no target contacted"}
	if not pt.ok:
		_status_label.text = "[ERROR] %s" % pt.error
		return

	var rows: Array = pt.rows
	rows.sort_custom(func(a, b): return _attr(a, "meta.title") < _attr(b, "meta.title"))

	var count := 0
	for row in rows:
		var title: String = _attr(row, "meta.title")
		var collection: String = _attr(row, "meta.collection")
		if title.is_empty():
			continue
		var coll_key = collection if not collection.is_empty() else "(uncategorized)"
		if not _collection_items.has(coll_key):
			var coll_item = _tree.create_item(_tree_root)
			coll_item.set_text(0, "📁 " + coll_key)
			coll_item.set_selectable(0, false)
			_collection_items[coll_key] = coll_item
		var doc_item = _tree.create_item(_collection_items[coll_key])
		doc_item.set_text(0, "📄 " + title)
		_title_by_tree_id[doc_item.get_instance_id()] = title
		count += 1

	_status_label.text = "%d document(s) loaded." % count

func _on_tree_item_selected() -> void:
	var item := _tree.get_selected()
	if item == null:
		return
	var title = _title_by_tree_id.get(item.get_instance_id(), "")
	if title.is_empty():
		return
	_load_document(title)

# ── Search (real SEARCH:, EnkiDDB's RAG) ─────────────────────────────────

func _run_search() -> void:
	var phrase := _search_edit.text.strip_edges()
	if phrase.is_empty() or _busy:
		return
	_busy = true
	_show_placeholder("Searching…")

	var command = "SEARCH:5:%s" % phrase
	var target = {"engine": "EnkiDDB", "host": HOST, "port": PORT}
	var thread = Thread.new()
	thread.start(func():
		var result = SumuUkinClient.route([target], command, SumuUkinClient.FanOutPolicy.FIRST_ONLY)
		call_deferred("_on_search_complete", result, thread)
	)

func _on_search_complete(result: Dictionary, thread: Thread) -> void:
	thread.wait_to_finish()
	_busy = false

	var pt = result.per_target[0] if not result.per_target.is_empty() else {"ok": false, "rows": [], "error": "no target contacted"}
	if not pt.ok:
		_show_placeholder("[ERROR] %s" % pt.error)
		return
	if pt.rows.is_empty():
		_show_placeholder("No hits. Real semantic search, not a substring match -- try different wording.")
		return

	for child in _content_root.get_children():
		child.queue_free()

	var header = Label.new()
	header.text = "SEARCH RESULTS — real ranked hits, real scores"
	header.add_theme_font_size_override("font_size", DubSarTheme.SIZE_LABEL)
	header.add_theme_color_override("font_color", DubSarTheme.TEXT_DIM)
	_content_root.add_child(header)

	for hit in pt.rows:
		_content_root.add_child(_search_hit_card(hit))

func _search_hit_card(hit: Dictionary) -> PanelContainer:
	var card = PanelContainer.new()
	card.add_theme_stylebox_override("panel", DubSarTheme.card_stylebox())

	var vbox = VBoxContainer.new()
	vbox.add_theme_constant_override("separation", 4)
	card.add_child(vbox)

	var score_row = HBoxContainer.new()
	vbox.add_child(score_row)
	score_row.add_child(DubSarTheme.pill("score %.4f" % float(hit.get("score", 0.0)), DubSarTheme.ACCENT_TEAL))

	var text_lbl = Label.new()
	var text: String = str(hit.get("text", ""))
	text_lbl.text = text if text.length() <= 400 else text.substr(0, 397) + "…"
	text_lbl.add_theme_color_override("font_color", DubSarTheme.TEXT_PRIMARY)
	text_lbl.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	vbox.add_child(text_lbl)

	var kaki_lbl = Label.new()
	kaki_lbl.text = "kaki " + str(hit.get("kaki", "?")).left(16) + "…"
	kaki_lbl.add_theme_font_size_override("font_size", DubSarTheme.SIZE_LABEL)
	kaki_lbl.add_theme_color_override("font_color", DubSarTheme.TEXT_FAINT)
	vbox.add_child(kaki_lbl)

	return card

# ── Document content (real, exact meta.title match) ──────────────────────

func _load_document(title: String) -> void:
	if _busy:
		return
	_busy = true
	_show_placeholder("Loading \"%s\"…" % title)

	var escaped := title.replace("\"", "\\\"")
	var query = "WHO T.E\nWHAT E[meta.title, meta.collection, body.summary, body.text, link.target, link.description]\nWHERE E[meta.title] = \"%s\"" % escaped
	var target = {"engine": "EnkiDDB", "host": HOST, "port": PORT}
	var thread = Thread.new()
	thread.start(func():
		var result = SumuUkinClient.route([target], query, SumuUkinClient.FanOutPolicy.FIRST_ONLY)
		call_deferred("_on_document_loaded", result, thread)
	)

func _on_document_loaded(result: Dictionary, thread: Thread) -> void:
	thread.wait_to_finish()
	_busy = false

	var pt = result.per_target[0] if not result.per_target.is_empty() else {"ok": false, "rows": [], "error": "no target contacted"}
	if not pt.ok:
		_show_placeholder("[ERROR] %s" % pt.error)
		return
	if pt.rows.is_empty():
		_show_placeholder("No document found with that exact title.")
		return

	_render_document(pt.rows[0])

func _render_document(row: Dictionary) -> void:
	for child in _content_root.get_children():
		child.queue_free()

	var title: String = _attr(row, "meta.title")
	var collection: String = _attr(row, "meta.collection")
	var kaki: String = str(row.get("kaki", "?"))

	var title_lbl = Label.new()
	title_lbl.text = title
	title_lbl.add_theme_font_size_override("font_size", DubSarTheme.SIZE_TITLE + 6)
	title_lbl.add_theme_color_override("font_color", DubSarTheme.TEXT_PRIMARY)
	_content_root.add_child(title_lbl)

	var badge_row = HBoxContainer.new()
	badge_row.add_theme_constant_override("separation", 8)
	_content_root.add_child(badge_row)
	badge_row.add_child(DubSarTheme.pill("kaki " + kaki.left(12) + "…", DubSarTheme.TEXT_DIM))
	if not collection.is_empty():
		badge_row.add_child(DubSarTheme.pill(collection, DubSarTheme.ACCENT_TEAL))

	var content_card = PanelContainer.new()
	content_card.add_theme_stylebox_override("panel", DubSarTheme.card_stylebox())
	_content_root.add_child(content_card)

	var content_vbox = VBoxContainer.new()
	content_vbox.add_theme_constant_override("separation", 8)
	content_card.add_child(content_vbox)

	var content_label = Label.new()
	content_label.text = "CONTENT"
	content_label.add_theme_font_size_override("font_size", DubSarTheme.SIZE_LABEL)
	content_label.add_theme_color_override("font_color", DubSarTheme.TEXT_DIM)
	content_vbox.add_child(content_label)

	var body: String = _attr(row, "body.text")
	if body.is_empty():
		body = _attr(row, "body.summary")
	if body.is_empty():
		body = "(no body.text/body.summary attribute on this row)"
	var body_lbl = Label.new()
	body_lbl.text = body
	body_lbl.add_theme_color_override("font_color", DubSarTheme.TEXT_PRIMARY)
	body_lbl.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	content_vbox.add_child(body_lbl)

	# ── Related Links — real link.target/link.description particles ──
	#
	# Honest limit, not a UI shortcut: heptascript::engine::get_val() finds
	# the FIRST EAV entry matching a requested attribute name (Vec::find),
	# even though a document's real EavMap can carry several link.target/
	# link.description particles (one per outgoing link). One WHAT
	# projection can only ever surface one of them -- the same real
	# constraint graph_explorer.gd's own header documents and works around
	# by expanding one relationship at a time. Shown as exactly what it
	# is: the first related link this query found, not "the" related link.
	var target_kaki: String = _attr(row, "link.target")
	var desc: String = _attr(row, "link.description")
	if not target_kaki.is_empty():
		var links_label = Label.new()
		links_label.text = "🔗 RELATED LINK (first match -- see Graph Explorer, Ctrl+K, to walk the rest)"
		links_label.add_theme_font_size_override("font_size", DubSarTheme.SIZE_LABEL)
		links_label.add_theme_color_override("font_color", DubSarTheme.TEXT_DIM)
		_content_root.add_child(links_label)

		_content_root.add_child(DubSarTheme.accent_row(
			desc if not desc.is_empty() else "(no description)",
			"kaki " + target_kaki.left(16) + "…",
			"",
			DubSarTheme.ACCENT_TEAL,
		))

func _show_placeholder(text: String) -> void:
	for child in _content_root.get_children():
		child.queue_free()
	var lbl = Label.new()
	lbl.text = text
	lbl.add_theme_color_override("font_color", DubSarTheme.TEXT_DIM)
	lbl.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	_content_root.add_child(lbl)

# ── Attr helpers (row.attrs is an Array of [name, value] pairs) ──────────

func _attr(row: Dictionary, name: String) -> String:
	for pair in row.get("attrs", []):
		if pair is Array and pair.size() >= 2 and str(pair[0]) == name:
			return str(pair[1])
	return ""
