# enkidb_wizard.gd — Clean EnkiDB Connection Wizard
# Step 1: Engine picker | Step 2: Connect
# Zero external dependencies — pure GDScript only
# DUB.SAR 𒁾  BahyWay.Ecosystem v4.0
extends CanvasLayer

signal connection_confirmed(profile: Dictionary)
signal wizard_cancelled

const C_BG   = Color(0.063, 0.071, 0.094, 0.97)
const C_GOLD = Color(0.784, 0.627, 0.157, 1.00)
const C_TEAL = Color(0.235, 0.706, 0.471, 1.00)
const C_RED  = Color(0.706, 0.235, 0.275, 1.00)
const C_DIM  = Color(0.392, 0.431, 0.510, 1.00)
const C_TEXT = Color(0.900, 0.900, 0.900, 1.00)

const ENGINES = [
	{"id":"EnkiDB",  "glyph":"𒆳", "port":7001, "desc":"Core CQRS sovereign store"},
	{"id":"EnkiDW",  "glyph":"𒀭", "port":7002, "desc":"Data warehouse / ETL"},
	{"id":"EnkiSDB", "glyph":"𒌓", "port":7003, "desc":"Staging database"},
	{"id":"EnkiODB", "glyph":"𒌋", "port":7004, "desc":"Operation database"},
	{"id":"EnkiQDB", "glyph":"𒁾", "port":7005, "desc":"Quarantine database"},
]

var _selected: Dictionary = {}
var _host_field: LineEdit
var _port_field: LineEdit
var _status_lbl: Label
var _step1: VBoxContainer
var _step2: VBoxContainer

func _ready() -> void:
	layer = 100
	visible = false
	_build()

func open() -> void:
	visible = true
	_step1.visible = true
	_step2.visible = false

func close() -> void:
	visible = false

func _build() -> void:
	var bg = ColorRect.new()
	bg.color = C_BG
	bg.set_anchors_preset(Control.PRESET_FULL_RECT)
	add_child(bg)

	var panel = PanelContainer.new()
	panel.set_anchors_preset(Control.PRESET_CENTER)
	panel.custom_minimum_size = Vector2(580, 480)
	panel.position = Vector2(-290, -240)
	add_child(panel)

	var vb = VBoxContainer.new()
	vb.add_theme_constant_override("separation", 14)
	panel.add_child(vb)

	var title = Label.new()
	title.text = "𒁾  EnkiDB Connection Wizard"
	title.add_theme_font_size_override("font_size", 20)
	title.add_theme_color_override("font_color", C_GOLD)
	title.horizontal_alignment = HORIZONTAL_ALIGNMENT_CENTER
	vb.add_child(title)

	vb.add_child(HSeparator.new())

	# ── Step 1 ──────────────────────────────────────────────
	_step1 = VBoxContainer.new()
	_step1.add_theme_constant_override("separation", 10)
	vb.add_child(_step1)

	var s1title = Label.new()
	s1title.text = "Step 1 — Select EnkiDB Engine:"
	s1title.add_theme_font_size_override("font_size", 13)
	s1title.add_theme_color_override("font_color", C_GOLD)
	_step1.add_child(s1title)

	for eng in ENGINES:
		var btn = Button.new()
		btn.text = eng["glyph"] + "  " + eng["id"] + "   —   " + eng["desc"]
		btn.add_theme_font_size_override("font_size", 13)
		btn.custom_minimum_size = Vector2(540, 46)
		var e = eng
		btn.pressed.connect(func():
			_selected = e
			_port_field.text = str(e["port"])
			_status_lbl.text = "Engine: " + e["id"] + " selected"
			_status_lbl.add_theme_color_override("font_color", C_GOLD)
			_step1.visible = false
			_step2.visible = true
		)
		_step1.add_child(btn)

	# ── Step 2 ──────────────────────────────────────────────
	_step2 = VBoxContainer.new()
	_step2.add_theme_constant_override("separation", 12)
	_step2.visible = false
	vb.add_child(_step2)

	var s2title = Label.new()
	s2title.text = "Step 2 — Connection Details:"
	s2title.add_theme_font_size_override("font_size", 13)
	s2title.add_theme_color_override("font_color", C_GOLD)
	_step2.add_child(s2title)

	var host_row = HBoxContainer.new()
	var hl = Label.new()
	hl.text = "Host:"
	hl.custom_minimum_size = Vector2(70, 0)
	hl.add_theme_color_override("font_color", C_TEXT)
	host_row.add_child(hl)
	_host_field = LineEdit.new()
	_host_field.text = "192.168.122.107"
	_host_field.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	_host_field.add_theme_font_size_override("font_size", 13)
	host_row.add_child(_host_field)
	_step2.add_child(host_row)

	var port_row = HBoxContainer.new()
	var pl = Label.new()
	pl.text = "Port:"
	pl.custom_minimum_size = Vector2(70, 0)
	pl.add_theme_color_override("font_color", C_TEXT)
	port_row.add_child(pl)
	_port_field = LineEdit.new()
	_port_field.text = "7001"
	_port_field.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	_port_field.add_theme_font_size_override("font_size", 13)
	port_row.add_child(_port_field)
	_step2.add_child(port_row)

	_status_lbl = Label.new()
	_status_lbl.text = ""
	_status_lbl.add_theme_font_size_override("font_size", 12)
	_status_lbl.add_theme_color_override("font_color", C_TEAL)
	_status_lbl.autowrap_mode = TextServer.AUTOWRAP_WORD
	_step2.add_child(_status_lbl)

	var btn_row = HBoxContainer.new()
	btn_row.add_theme_constant_override("separation", 10)

	var back_btn = Button.new()
	back_btn.text = "◀ Back"
	back_btn.pressed.connect(func():
		_step1.visible = true
		_step2.visible = false
	)
	btn_row.add_child(back_btn)

	var sp = Control.new()
	sp.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	btn_row.add_child(sp)

	var cancel_btn = Button.new()
	cancel_btn.text = "✕ Cancel"
	cancel_btn.pressed.connect(func():
		emit_signal("wizard_cancelled")
		close()
	)
	btn_row.add_child(cancel_btn)

	var conn_btn = Button.new()
	conn_btn.text = "▶ Connect"
	conn_btn.pressed.connect(_do_connect)
	btn_row.add_child(conn_btn)

	_step2.add_child(btn_row)

func _do_connect() -> void:
	var host = _host_field.text
	var port = int(_port_field.text)
	_status_lbl.text = "⟳ Connecting to %s:%d ..." % [host, port]
	_status_lbl.add_theme_color_override("font_color", C_TEAL)

	var peer = StreamPeerTCP.new()
	peer.connect_to_host(host, port)
	await get_tree().create_timer(1.5).timeout

	var status = peer.get_status()
	peer.disconnect_from_host()

	if status == StreamPeerTCP.STATUS_CONNECTED:
		_status_lbl.text = "✓ Connected to %s:%d" % [host, port]
		_status_lbl.add_theme_color_override("font_color", C_TEAL)
		await get_tree().create_timer(0.5).timeout
		emit_signal("connection_confirmed", {
			"host":   host,
			"port":   port,
			"engine": _selected.get("id", "EnkiDB"),
		})
		close()
	else:
		_status_lbl.text = "✗ Cannot reach %s:%d" % [host, port]
		_status_lbl.add_theme_color_override("font_color", C_RED)
