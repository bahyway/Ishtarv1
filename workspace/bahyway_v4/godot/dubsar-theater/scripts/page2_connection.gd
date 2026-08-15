extends MarginContainer

signal connection_changed(data: Dictionary)

@onready var host_input: LineEdit = $VBoxContainer/FormRows/HostRow/HostInput
@onready var port_input: LineEdit = $VBoxContainer/FormRows/PortRow/PortInput
@onready var scope_input: LineEdit = $VBoxContainer/DbNameRow/DbNameInput
@onready var timeout_input: SpinBox = $VBoxContainer/TimeoutRow/TimeoutSpin
@onready var save_check: CheckBox = $VBoxContainer/SaveCheck
# Note: no SSL checkbox — transport is AkkadiCipherEngine (ChaCha20-Poly1305),
# always on. Sovereignty is not a checkbox.

func _ready():
	host_input.text_changed.connect(_on_data_changed)
	port_input.text_changed.connect(_on_data_changed)
	scope_input.text_changed.connect(_on_data_changed)
	timeout_input.value_changed.connect(_on_data_changed)
	save_check.toggled.connect(_on_data_changed)

func set_port(port: String):
	port_input.text = port
	_on_data_changed()

func _on_data_changed(_value = null):
	connection_changed.emit(get_data())

func get_data() -> Dictionary:
	return {
		"host": host_input.text,
		"port": port_input.text,
		"scope": scope_input.text,
		"timeout": int(timeout_input.value),
		"save_profile": save_check.button_pressed
	}
