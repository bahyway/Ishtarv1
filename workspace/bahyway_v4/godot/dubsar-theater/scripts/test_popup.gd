extends Window

signal test_completed(success: bool, message: String)

@onready var status_label: Label = $MarginContainer/VBoxContainer/StatusLabel
@onready var loading_panel: PanelContainer = $MarginContainer/VBoxContainer/LoadingPanel
@onready var success_panel: PanelContainer = $MarginContainer/VBoxContainer/SuccessPanel
@onready var error_panel: PanelContainer = $MarginContainer/VBoxContainer/ErrorPanel
@onready var details_label: Label = $MarginContainer/VBoxContainer/DetailsPanel/DetailsLabel
@onready var btn_close: Button = $MarginContainer/VBoxContainer/ButtonRow/BtnClose
@onready var btn_continue: Button = $MarginContainer/VBoxContainer/ButtonRow/BtnContinue
@onready var spinner: TextureRect = $MarginContainer/VBoxContainer/LoadingPanel/HBoxContainer/Spinner

const ConnectionTester = preload("res://scripts/connection_tester.gd")

var test_data: Dictionary = {}
var tester: Node = null
var last_success: bool = false
var last_message: String = ""

func _ready():
	btn_close.pressed.connect(_on_close)
	btn_continue.pressed.connect(_on_continue)
	close_requested.connect(_on_close)
	about_to_popup.connect(_on_about_to_show)

	tester = ConnectionTester.new()
	add_child(tester)
	tester.test_finished.connect(_on_test_finished)

func setup_test(data: Dictionary):
	test_data = data.duplicate()

func _on_about_to_show():
	_reset_ui()
	_start_test()

func _reset_ui():
	loading_panel.visible = true
	success_panel.visible = false
	error_panel.visible = false
	btn_continue.visible = false
	btn_close.text = "Cancel"

	# CORRECTED 2026-07-10: enkidb-query-server has no handshake frame,
	# so only TCP reachability is verified today — stated honestly rather
	# than claiming a protocol check that doesn't exist server-side.
	details_label.text = "Host: %s\nPort: %s\nTransport: AkkadiCipherEngine — always on\nCheck performed: TCP reachability only\nSeal verify: PENDING BRIDGE\nUrNammu attestation: PENDING BRIDGE\nABAC role probe: PENDING BRIDGE" % [
		test_data.get("host", "—"),
		test_data.get("port", "—")
	]

	status_label.text = "Probing %s..." % test_data.get("database", "—")

func _start_test():
	var tween = create_tween().set_loops()
	tween.tween_property(spinner, "rotation", TAU, 0.8)
	set_meta("spin_tween", tween)

	tester.test_connection(
		test_data.get("host", "localhost"),
		int(test_data.get("port", "0")),
		int(test_data.get("timeout", 30)) * 1000
	)

func _on_test_finished(success: bool, message: String):
	last_success = success
	last_message = message

	loading_panel.visible = false
	var tween = get_meta("spin_tween", null)
	if tween:
		tween.kill()

	if success:
		success_panel.visible = true
		btn_continue.visible = true
		btn_close.text = "Close"
		status_label.text = message
	else:
		error_panel.visible = true
		btn_close.text = "Close"
		status_label.text = "Connection failed"
		var error_msg = error_panel.get_node("HBoxContainer/ErrorMessage") as Label
		error_msg.text = "%s (%s:%s). Check that the Enki node is running." % [
			message,
			test_data.get("host", ""),
			test_data.get("port", "")
		]

func _on_close():
	if tester:
		tester.cancel_test()
	hide()

func _on_continue():
	test_completed.emit(last_success, last_message)
	hide()
