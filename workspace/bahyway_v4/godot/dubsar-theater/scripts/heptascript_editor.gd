extends Control
# =============================================================================
# HeptaScript Editor — upper CodeEdit (W5H2 query composition) + lower CLI
# log (execution trace) + NODE-target checkboxes + PDM tab.
#
# Wired to SumuUkinClient.route() (scripts/sumuukin_client.gd), the real
# multi-target parallel/serial/first-only dispatcher that speaks the same
# HEPT wire protocol as the real bin/enkidb-query-server. Every engine
# checkbox comes from EnkiEngines.TABLE (single source of truth, shared
# with the connector wizard) -- no duplicated engine list.
# =============================================================================

const SumuUkinClient = preload("res://scripts/sumuukin_client.gd")
const DubSarTheme = preload("res://scripts/dubsar_theme.gd")

@onready var host_input: LineEdit = %HostInput
@onready var node_targets_box: HBoxContainer = %NodeTargetsBox
@onready var policy_option: OptionButton = %PolicyOption
@onready var btn_run: Button = %BtnRun
@onready var query_edit: CodeEdit = %QueryEdit
@onready var log_edit: TextEdit = %LogEdit
@onready var main_tabs: TabContainer = %MainTabs
@onready var pdm_tab := $MarginContainer/VBoxContainer/MainTabs/PDM

var _target_checks: Dictionary = {}  # engine name -> CheckBox
var _all_check: CheckBox

func open() -> void:
	visible = true

func close() -> void:
	visible = false

func _ready():
	visible = false
	_build_node_target_checks()
	_setup_policy_dropdown()
	_setup_syntax_highlighting()
	btn_run.pressed.connect(_on_run_pressed)
	# PB-207: default text is a real command against EnkiDDB's actual wire
	# protocol (bin/enkiddb-read-server/src/main.rs) -- QUERY:/SEARCH: is a
	# literal prefix that server requires, unlike the bare-HeptaScript
	# EnkiDB (port 7001) target. Swap to SEARCH:5:<phrase> to test semantic
	# search instead of a structured query.
	query_edit.text = "QUERY:WHO T.E\nWHAT E[meta.title]\nWHERE E[meta.collection] = \"components\""
	_log("HeptaScript editor ready. Defaulting to EnkiDDB only (host:%s port:%s) -- check other NODE boxes to add targets." % [host_input.text, EnkiEngines.TABLE["EnkiDDB"]["port"]])
	_log("Try: QUERY:WHO T.E ... (structured) or SEARCH:5:<phrase> (semantic/vector search, ranked by score).")
	if pdm_tab and pdm_tab.has_method("bind_run_callback"):
		pdm_tab.bind_run_callback(_run_query_text)

func _build_node_target_checks():
	_all_check = CheckBox.new()
	_all_check.text = "ALL"
	# PB-207: default OFF, not ON -- only EnkiDDB is a real, live-deployed
	# target as of this session; checking ALL by default would fan out to
	# six engines with no server running and read as broken on first open.
	_all_check.button_pressed = false
	_all_check.toggled.connect(_on_all_toggled)
	node_targets_box.add_child(_all_check)

	for engine_name in EnkiEngines.TABLE.keys():
		var cb = CheckBox.new()
		cb.text = engine_name
		cb.button_pressed = (engine_name == "EnkiDDB")  # PB-207: EnkiDDB is the real target today
		cb.disabled = false
		cb.toggled.connect(_on_target_toggled)
		node_targets_box.add_child(cb)
		_target_checks[engine_name] = cb

func _on_all_toggled(pressed: bool):
	for cb in _target_checks.values():
		cb.disabled = pressed
		if pressed:
			cb.button_pressed = false

func _on_target_toggled(_pressed: bool):
	pass  # individual choice; ALL stays user-controlled separately

func _chosen_targets() -> Array:
	var host = host_input.text.strip_edges()
	if host.is_empty():
		# PB-212 (2026-07-19): enkidb-node-read, not eriduous-vdi -- PB-207
		# had moved this to eriduous-vdi (192.168.122.214) when that host
		# briefly ran every container; PB-212 split Write/Read back onto
		# their own real VMs, so QUERY/SEARCH go to the Read node again --
		# matching every other Godot script in this project
		# (dubsar_proof.gd, enkidb_tcp.gd, theater_3d.gd, etc.), which
		# never changed away from this IP in the first place.
		host = "192.168.122.112"
	var targets: Array = []
	var engines: Array = []
	if _all_check.button_pressed:
		engines = EnkiEngines.TABLE.keys()
	else:
		for engine_name in _target_checks:
			if _target_checks[engine_name].button_pressed:
				engines.append(engine_name)
	for engine_name in engines:
		var port = int(EnkiEngines.TABLE[engine_name]["port"])
		targets.append({"engine": engine_name, "host": host, "port": port})
	return targets

func _setup_policy_dropdown():
	policy_option.clear()
	policy_option.add_item("ALL PARALLEL", SumuUkinClient.FanOutPolicy.ALL_PARALLEL)
	policy_option.add_item("FIRST ONLY", SumuUkinClient.FanOutPolicy.FIRST_ONLY)
	policy_option.add_item("ALL SERIAL", SumuUkinClient.FanOutPolicy.ALL_SERIAL)
	policy_option.select(0)

func _setup_syntax_highlighting():
	var hl = CodeHighlighter.new()
	# W5H2 clause keywords (real, confirmed against crates/heptascript/src/token.rs)
	var w5h2 = ["WHO", "WHAT", "WHERE", "WHEN", "WHY", "HOW", "HOW_MUCH", "NODE",
				"TIER", "STATE", "NASH", "PATTERN", "LINEAGE", "GATE", "SATAMU",
				"ORBITAL", "ACROSS", "LIMIT", "ALL"]
	for kw in w5h2:
		hl.add_keyword_color(kw, DubSarTheme.ACCENT_CYAN)
	# Triple-O verbs (design vocabulary; not all tokenized today, harmless to highlight)
	var verbs = ["ORBIT", "EMIT", "PROVE", "SYNC", "WITNESS"]
	for kw in verbs:
		hl.add_keyword_color(kw, DubSarTheme.ACCENT_GREEN)
	# Real DbTarget names (crates/heptascript/src/query.rs DbTarget enum)
	var db_targets = ["EnkiDB", "EnkiDW", "EnkiSDB", "EnkiODB", "EnkiQDB",
					   "EnkiMDB", "EnkiDDB", "NARUDU"]
	for kw in db_targets:
		hl.add_keyword_color(kw, DubSarTheme.ACCENT_AMBER)
	hl.number_color = DubSarTheme.ACCENT_PURPLE
	hl.symbol_color = DubSarTheme.TEXT_DIM
	query_edit.syntax_highlighter = hl

func _on_run_pressed():
	_run_query_text(query_edit.text)

func _run_query_text(query_text: String) -> void:
	var targets = _chosen_targets()
	if targets.is_empty():
		_log("[ERROR] No target engine chosen.")
		return

	var policy: int = policy_option.get_item_id(policy_option.selected)
	_log("")
	_log(">>> RUN  targets=%s  policy=%s" % [
		targets.map(func(t): return t.engine),
		["ALL_PARALLEL", "FIRST_ONLY", "ALL_SERIAL"][policy]
	])
	btn_run.disabled = true

	var thread = Thread.new()
	thread.start(func():
		var result = SumuUkinClient.route(targets, query_text, policy)
		call_deferred("_on_run_complete", result, thread)
	)

func _on_run_complete(result: Dictionary, thread: Thread) -> void:
	thread.wait_to_finish()
	btn_run.disabled = false

	_log("--- contacted=%d failed=%d ---" % [result.targets_contacted, result.targets_failed])
	for pt in result.per_target:
		if pt.ok:
			_log("[OK]   %s %s:%s  %d row(s)  %dms" % [
				pt.engine, pt.host, pt.port, pt.rows.size(), pt.elapsed_ms])
			for row in pt.rows:
				_log("       " + _format_row(row))
		else:
			_log("[FAIL] %s %s:%s  %s  (%dms)" % [
				pt.engine, pt.host, pt.port, pt.error, pt.elapsed_ms])

	if pdm_tab and pdm_tab.has_method("on_query_result"):
		pdm_tab.on_query_result(result)

# PB-207: row counts alone don't answer "does SEARCH: actually bring back
# relevant results for a phrase" -- the row CONTENT has to be visible.
# Two real shapes come back from enkiddb-read-server (bin/enkiddb-read-server/
# src/main.rs): QUERY: rows carry {"kaki","attrs":[[k,v],...]}; SEARCH: hits
# carry {"kaki","score","text"} (no "attrs" at all) -- distinguished here by
# which key is present, not by which command was typed, so this works
# regardless of NODE target or fan-out policy.
func _format_row(row: Dictionary) -> String:
	if row.has("score"):
		var text: String = str(row.get("text", ""))
		var snippet: String = text if text.length() <= 120 else text.substr(0, 117) + "..."
		return "score=%.4f  %s" % [float(row.get("score", 0.0)), snippet]
	elif row.has("attrs"):
		var kaki: String = str(row.get("kaki", "?"))
		var attrs_text := ""
		for pair in row.get("attrs", []):
			if pair is Array and pair.size() >= 2:
				if not attrs_text.is_empty():
					attrs_text += "  "
				attrs_text += "%s=%s" % [pair[0], pair[1]]
		return "kaki=%s  %s" % [kaki.substr(0, 12), attrs_text]
	else:
		return str(row)

func _log(msg: String) -> void:
	log_edit.text += msg + "\n"
	log_edit.scroll_vertical = log_edit.get_line_count()
