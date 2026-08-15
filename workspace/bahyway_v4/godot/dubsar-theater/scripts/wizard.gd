extends Control

const DubSarTheme = preload("res://scripts/dubsar_theme.gd")

# Enki Database Connector Wizard - Main Controller (WIZ-001 conformant)
# Handles wizard flow, page navigation, and data collection.
#
# CONSOLIDATION 2026-07-10: this is now the ONLY EnkiDB connector wizard
# in dubsar-theater. The standalone enkidb_wizard.gd/.tscn (5 engines,
# ports 7001-7005, no crypto) and dubsar_proof.gd's inline
# _show_wizard()/_wizard_connect() are retired in its favour. Port
# numbering was reconciled to preserve the numbers enkidb_wizard.gd
# already had live (EnkiDB=7001..EnkiQDB=7005), extended with the two
# engines it never had (EnkiMDB=7006, EnkiDDB=7007) — see enki_engines.gd.

signal wizard_completed(connection_data: Dictionary)
signal connection_tested(success: bool, message: String)

@onready var tab_container: TabContainer = $MarginContainer/VBoxContainer/TabContainer
@onready var step_dots: HBoxContainer = $MarginContainer/VBoxContainer/StepIndicator/StepDots
@onready var step_labels: HBoxContainer = $MarginContainer/VBoxContainer/StepIndicator/StepLabels
# FIXED (2026-07-27): the real path (confirmed against wizard.tscn's
# own node tree) is Footer/MarginContainer/HBoxContainer/BtnPrev --
# these were missing the MarginContainer/HBoxContainer wrapper the
# Footer panel actually uses, so both onready vars resolved to null.
# That null crashed _setup_signals() at the very first line
# (btn_prev.pressed.connect), which aborted the REST of that
# function -- meaning page1/page2 data-capture signals,
# credentials_popup.credentials_confirmed, and
# test_popup.test_completed never got connected either. This is the
# actual root cause of the Test Connection popup showing frozen
# placeholder text with blank Host/Port: the wizard's own Next/Test
# Connection button never had a working handler wired to it in the
# first place, so real form data never reached connection_data.
@onready var btn_prev: Button = $MarginContainer/VBoxContainer/Footer/MarginContainer/HBoxContainer/BtnPrev
@onready var btn_next: Button = $MarginContainer/VBoxContainer/Footer/MarginContainer/HBoxContainer/BtnNext
@onready var credentials_popup: Window = $CredentialsPopup
@onready var test_popup: Window = $TestPopup

var pages: Array[Control] = []
var current_page: int = 0
var total_pages: int = 4

# Connection data collected across all pages.
# Transport is AkkadiCipherEngine (ChaCha20-Poly1305), ALWAYS ON — there is
# deliberately no ssl/tls field: sovereignty is not a checkbox.
var connection_data: Dictionary = {
	"database": "",
	"host": "localhost",
	"port": "",
	"scope": "",
	"timeout": 30,
	"save_profile": false,
	"passport_type": "",
	"identity": "",
	"passphrase": "",
	"save_credentials": false
}

func _ready():
	visible = false
	_setup_pages()
	_setup_signals()
	_update_ui()
	_update_step_indicator()

# Open/close so callers (e.g. theater_controller.gd's
# open_enkidb_wizard()) can toggle this like the previous CanvasLayer
# wizard did, even though this one is a plain Control.
func open() -> void:
	visible = true
	current_page = 0
	_update_ui()
	_update_step_indicator()

func close() -> void:
	visible = false

func _setup_pages():
	for i in range(tab_container.get_child_count()):
		var page = tab_container.get_child(i)
		pages.append(page)
	tab_container.tabs_visible = false

func _setup_signals():
	btn_prev.pressed.connect(_on_prev_pressed)
	btn_next.pressed.connect(_on_next_pressed)

	var page1 = pages[0]  # DB Choice
	if page1.has_signal("database_chosen"):
		page1.database_chosen.connect(_on_database_chosen)

	var page2 = pages[1]  # Connection
	if page2.has_signal("connection_changed"):
		page2.connection_changed.connect(_on_connection_changed)

	credentials_popup.credentials_confirmed.connect(_on_credentials_confirmed)
	test_popup.test_completed.connect(_on_test_completed)

func _on_prev_pressed():
	if current_page > 0:
		current_page -= 1
		_update_ui()
		_update_step_indicator()

func _on_next_pressed():
	if not _validate_current_page():
		return

	if current_page == 2:
		_show_credentials_popup()
		return

	if current_page == 3:
		_show_test_popup()
		return

	if current_page < total_pages - 1:
		current_page += 1
		_update_ui()
		_update_step_indicator()

func _validate_current_page() -> bool:
	match current_page:
		0:
			if connection_data.database.is_empty():
				_show_error("Please choose an Enki database engine.")
				return false
		1:
			if connection_data.host.is_empty():
				_show_error("Please enter a host address.")
				return false
			if connection_data.port.is_empty():
				_show_error("Please enter a HEPT port.")
				return false
	return true

func _update_ui():
	tab_container.current_tab = current_page
	btn_prev.visible = current_page > 0
	match current_page:
		0: btn_next.text = "Next →"
		1: btn_next.text = "Next →"
		2: btn_next.text = "🔐 Present Passport →"
		3:
			btn_next.text = "🧪 Test Connection"
			btn_next.add_theme_color_override("font_color", DubSarTheme.ACCENT_GREEN)

func _update_step_indicator():
	for i in range(step_dots.get_child_count()):
		var dot = step_dots.get_child(i) as ColorRect
		if i < current_page:
			dot.color = DubSarTheme.ACCENT_GREEN
		elif i == current_page:
			dot.color = DubSarTheme.ACCENT_CYAN
		else:
			dot.color = DubSarTheme.BORDER

	for i in range(step_labels.get_child_count()):
		var label = step_labels.get_child(i) as Label
		if i <= current_page:
			label.modulate = DubSarTheme.TEXT_PRIMARY
		else:
			label.modulate = DubSarTheme.TEXT_DIM

# ===== DATABASE CHOICE =====
func _on_database_chosen(db_name: String):
	connection_data.database = db_name
	connection_data.port = EnkiEngines.TABLE[db_name]["port"]

	var conn_page = pages[1]
	if conn_page.has_method("set_port"):
		conn_page.set_port(connection_data.port)

	var creds_page = pages[2]
	if creds_page.has_method("set_database_display"):
		creds_page.set_database_display(db_name, EnkiEngines.TABLE[db_name]["desc"])

# ===== CONNECTION DETAILS =====
func _on_connection_changed(data: Dictionary):
	connection_data.host = data.get("host", "localhost")
	connection_data.port = data.get("port", "")
	connection_data.scope = data.get("scope", "")
	connection_data.timeout = data.get("timeout", 30)
	connection_data.save_profile = data.get("save_profile", false)

# ===== CREDENTIALS POPUP =====
func _show_credentials_popup():
	credentials_popup.popup_centered()

func _on_credentials_confirmed(passport_type: String, identity: String, passphrase: String, save: bool):
	# WIZ-001: EnkiMDB is read-only at runtime — a write-role passport
	# request against it must be refused at the wizard level.
	# FIXED (2026-07-27): this used to key off EnkiEngines.TABLE's
	# "read_only" field, which is true for ALL SEVEN engines (it marks
	# "this table only tracks Read Node ports", not "this database is
	# under the Nergal/ADAD law") -- so this refused a Sargon Passport
	# against every engine, not just EnkiMDB. gilgamesh_required is the
	# field that's actually scoped to the law (true only for EnkiMDB).
	if EnkiEngines.TABLE.get(connection_data.database, {}).get("gilgamesh_required", false) \
			and passport_type == "Sargon Passport":
		_show_error("EnkiMDB is read-only at runtime (Nergal/ADAD law).\nUse a Gilgamesh Passport for read access.")
		return

	connection_data.passport_type = passport_type
	connection_data.identity = identity
	connection_data.passphrase = passphrase
	connection_data.save_credentials = save

	var creds_page = pages[2]
	if creds_page.has_method("set_credentials"):
		creds_page.set_credentials(passport_type, identity, save)

	current_page = 3
	_update_ui()
	_update_step_indicator()

	var summary_page = pages[3]
	if summary_page.has_method("populate_summary"):
		summary_page.populate_summary(connection_data)

# ===== TEST CONNECTION POPUP =====
func _show_test_popup():
	test_popup.setup_test(connection_data)
	test_popup.popup_centered()

func _on_test_completed(success: bool, message: String):
	connection_tested.emit(success, message)
	if success:
		_save_connection()
	else:
		_show_error("Connection test failed: " + message)

func _save_connection():
	wizard_completed.emit(connection_data.duplicate())
	btn_next.text = "✅ Connection Saved!"
	btn_next.disabled = true
	_persist_connection()
	await get_tree().create_timer(1.2).timeout
	close()
	_reset_for_reopen()

func _reset_for_reopen():
	current_page = 0
	btn_next.disabled = false
	_update_ui()
	_update_step_indicator()

func _persist_connection():
	var config = ConfigFile.new()
	var path = "user://enki_connections.cfg"
	config.load(path)

	var key = connection_data.database + "_" + str(Time.get_unix_time_from_system())
	for prop in connection_data:
		if prop != "passphrase":  # secrets never touch the profile file
			config.set_value(key, prop, connection_data[prop])

	# Profile sealing (Ed25519 AkkadianSeal) via the bridge — honest marker
	# until the bridge is wired; never a homebrew substitute.
	config.set_value(key, "akkadian_seal", AkkadiSafeBridge.seal_profile(connection_data))

	if connection_data.save_credentials and not connection_data.passphrase.is_empty():
		var res = AkkadiSafeBridge.store_secret(key, connection_data.passphrase)
		if res != AkkadiSafeBridge.Result.OK:
			config.set_value(key, "vault_status", "NOT_STORED_VAULT_UNAVAILABLE")
			push_warning("AkkadiSafeEngine vault unavailable — passphrase NOT persisted (lawful refusal).")

	config.save(path)

# ===== UTILITY =====
func _show_error(message: String):
	var dialog = AcceptDialog.new()
	dialog.title = "Validation Error"
	dialog.dialog_text = message
	dialog.ok_button_text = "OK"
	add_child(dialog)
	dialog.popup_centered()
	dialog.confirmed.connect(func(): dialog.queue_free())
