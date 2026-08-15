extends Window

signal credentials_confirmed(passport_type: String, identity: String, passphrase: String, save: bool)

@onready var passport_choice: OptionButton = $MarginContainer/VBoxContainer/PassportRow/PassportChoice
@onready var identity_input: LineEdit = $MarginContainer/VBoxContainer/UsernameRow/UsernameInput
@onready var passphrase_input: LineEdit = $MarginContainer/VBoxContainer/PasswordRow/PasswordInput
@onready var save_check: CheckBox = $MarginContainer/VBoxContainer/SaveCheck
@onready var btn_confirm: Button = $MarginContainer/VBoxContainer/ButtonRow/BtnConfirm
@onready var btn_cancel: Button = $MarginContainer/VBoxContainer/ButtonRow/BtnCancel

func _ready():
	btn_confirm.pressed.connect(_on_confirm)
	btn_cancel.pressed.connect(_on_cancel)
	close_requested.connect(_on_cancel)
	about_to_popup.connect(_reset_fields)

	passport_choice.clear()
	for p in EnkiEngines.PASSPORT_TYPES:
		passport_choice.add_item(p)

func _reset_fields():
	passport_choice.select(0)
	identity_input.text = ""
	passphrase_input.text = ""
	save_check.button_pressed = false
	identity_input.grab_focus()

func _on_confirm():
	var identity = identity_input.text.strip_edges()
	var passphrase = passphrase_input.text

	if identity.is_empty():
		_show_error("Please enter the passport holder identity.")
		return
	if passphrase.is_empty():
		_show_error("Please enter the key unlock passphrase.")
		return

	var passport_type = passport_choice.get_item_text(passport_choice.selected)
	credentials_confirmed.emit(passport_type, identity, passphrase, save_check.button_pressed)
	hide()

func _on_cancel():
	hide()

func _show_error(message: String):
	var dialog = AcceptDialog.new()
	dialog.title = "Error"
	dialog.dialog_text = message
	dialog.ok_button_text = "OK"
	add_child(dialog)
	dialog.popup_centered()
	dialog.confirmed.connect(func(): dialog.queue_free())
