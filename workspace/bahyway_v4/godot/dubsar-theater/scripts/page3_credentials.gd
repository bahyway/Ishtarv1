extends MarginContainer

@onready var identity_input: LineEdit = $VBoxContainer/UsernameRow/UsernameInput
@onready var passport_display: LineEdit = $VBoxContainer/PasswordRow/PasswordInput
@onready var save_creds_check: CheckBox = $VBoxContainer/SaveCredsCheck
@onready var db_display: Label = $VBoxContainer/ChosenDBPanel/VBoxContainer/DbDisplay

func _ready():
	identity_input.editable = false
	passport_display.editable = false
	passport_display.secret = false

func set_database_display(db_name: String, desc: String):
	db_display.text = db_name + " — " + desc

func set_credentials(passport_type: String, identity: String, save: bool):
	identity_input.text = identity
	passport_display.text = passport_type + " · key unlocked"
	save_creds_check.button_pressed = save
