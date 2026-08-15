extends Control
class_name DubSarPdmLogin
# =============================================================================
# DubSarPdmLogin — 2026-07-24, the mandatory DubSar PDM entry gate.
#
# Adapted from dubsar-theater's login.gd: same real cryptographic gate
# (kupru-gdext's KupruBridge, the same GDExtension the standalone Sargon
# Passport Manager / Gilgamesh Master Key / DubSar IDE all use), same
# Import-Passport-then-log-in flow, same per-identity-encrypted local
# vault (user://dubsar_pdm_login_vault.dat -- a SEPARATE vault file from
# DubSar IDE's, since these are two independent apps with two independent
# OS-level user:// data directories; a Passport imported into one does
# NOT automatically appear in the other -- import it again here).
#
# Two deliberate simplifications versus dubsar-theater's login.gd:
#   1. Root is a plain Control, not a Node3D with a GPUParticles3D BIGRING
#      backdrop -- that backdrop is decorative, and hand-authoring a
#      particle scene's .tscn resource format correctly without a running
#      Godot editor to verify it visually is a real risk not worth taking
#      for a first pass. Add it later, in the editor, if wanted.
#   2. No EnkiEngines dependency -- that file's PASSPORT_TYPES constant
#      is inlined below directly rather than pulling in 48 lines of
#      EnkiDB-family port numbers a PDM math tool has no use for.
#
# HONEST LIMIT: this verifies WHO presented a genuine, non-expired,
# unbroken-seal Passport, and records its privilege_level in
# SessionIdentity for the rest of the session -- but nothing in DubSar
# PDM yet READS that to gate or restrict any feature. Same honest limit
# dubsar-theater's login.gd already states about itself.
# =============================================================================

const DubSarTheme = preload("res://scripts/dubsar_theme.gd")
const AkkadiSafeBridge = preload("res://scripts/akkadi_safe_bridge.gd")
const SessionIdentity = preload("res://scripts/session_identity.gd")

const PASSPORT_TYPES: Array[String] = ["Gilgamesh Passport", "Sargon Passport"]

const PDM_WORKSPACE_SCENE := "res://scenes/pdm_workspace.tscn"
const LOGIN_VAULT_PATH := "user://dubsar_pdm_login_vault.dat"

var _bridge: KupruBridge

var _passport_choice: OptionButton
var _identity_input: LineEdit
var _passphrase_input: LineEdit
var _remember_check: CheckBox
var _error_label: Label
var _btn_login: Button
var _btn_import: Button

# Set by _on_import_file_selected(), consumed once the import-details
# dialog is confirmed.
var _pending_import_json: String = ""

func _ready() -> void:
	get_tree().root.title = "BahyWay.Ecosystem v4.0 — DubSar PDM"
	DubSarTheme.apply_page_background(self)
	_bridge = KupruBridge.new()
	_build_ui()
	_identity_input.grab_focus()

func _build_ui() -> void:
	var center = CenterContainer.new()
	center.set_anchors_preset(Control.PRESET_FULL_RECT)
	add_child(center)

	var card = PanelContainer.new()
	card.custom_minimum_size = Vector2(420, 0)
	card.add_theme_stylebox_override("panel", DubSarTheme.card_stylebox(16))
	center.add_child(card)

	var vb = VBoxContainer.new()
	vb.add_theme_constant_override("separation", 14)
	card.add_child(vb)

	var title_row = HBoxContainer.new()
	vb.add_child(title_row)

	var title = Label.new()
	title.text = "𒁾 DubSar PDM — Sovereign Login"
	title.horizontal_alignment = HORIZONTAL_ALIGNMENT_CENTER
	title.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	title.add_theme_font_size_override("font_size", DubSarTheme.SIZE_TITLE)
	title.add_theme_color_override("font_color", DubSarTheme.TEXT_PRIMARY)
	title_row.add_child(title)

	var btn_quit = Button.new()
	btn_quit.text = "✕"
	btn_quit.tooltip_text = "Quit DubSar PDM"
	btn_quit.flat = true
	btn_quit.add_theme_color_override("font_color", DubSarTheme.TEXT_DIM)
	btn_quit.add_theme_color_override("font_hover_color", DubSarTheme.ACCENT_RED)
	btn_quit.add_theme_font_size_override("font_size", DubSarTheme.SIZE_TITLE)
	btn_quit.pressed.connect(func(): get_tree().quit())
	title_row.add_child(btn_quit)

	var subtitle = Label.new()
	subtitle.text = "BahyWay.Ecosystem v4.0 — present your sovereign Passport to continue"
	subtitle.horizontal_alignment = HORIZONTAL_ALIGNMENT_CENTER
	subtitle.autowrap_mode = TextServer.AUTOWRAP_WORD
	subtitle.add_theme_font_size_override("font_size", DubSarTheme.SIZE_LABEL)
	subtitle.add_theme_color_override("font_color", DubSarTheme.TEXT_DIM)
	vb.add_child(subtitle)

	vb.add_child(HSeparator.new())

	# ── Passport type ────────────────────────────────────────────────
	var passport_label = _field_label("PASSPORT")
	vb.add_child(passport_label)
	_passport_choice = OptionButton.new()
	for p in PASSPORT_TYPES:
		_passport_choice.add_item(p)
	_passport_choice.select(0)
	vb.add_child(_passport_choice)

	# ── Identity ─────────────────────────────────────────────────────
	vb.add_child(_field_label("IDENTITY"))
	_identity_input = LineEdit.new()
	_identity_input.placeholder_text = "e.g. bahyway"
	vb.add_child(_identity_input)

	# ── Passphrase ───────────────────────────────────────────────────
	vb.add_child(_field_label("KEY UNLOCK PASSPHRASE"))
	_passphrase_input = LineEdit.new()
	_passphrase_input.secret = true
	_passphrase_input.placeholder_text = "passphrase"
	vb.add_child(_passphrase_input)

	_remember_check = CheckBox.new()
	_remember_check.text = "Remember this Passport (AkkadiSafeEngine vault)"
	vb.add_child(_remember_check)

	_error_label = Label.new()
	_error_label.visible = false
	_error_label.autowrap_mode = TextServer.AUTOWRAP_WORD
	_error_label.add_theme_color_override("font_color", DubSarTheme.ACCENT_RED)
	_error_label.add_theme_font_size_override("font_size", DubSarTheme.SIZE_LABEL)
	vb.add_child(_error_label)

	_btn_login = Button.new()
	_btn_login.text = "▶ Enter DubSar PDM"
	_btn_login.add_theme_color_override("font_color", DubSarTheme.ACCENT_GREEN)
	vb.add_child(_btn_login)

	_btn_import = Button.new()
	_btn_import.text = "Import Passport…"
	_btn_import.tooltip_text = "Register a Passport JSON exported from Sargon Passport Manager or Gilgamesh Master Key, so it can be used to log in here."
	_btn_import.flat = true
	vb.add_child(_btn_import)

	_btn_login.pressed.connect(_on_login_pressed)
	_btn_import.pressed.connect(_on_import_pressed)
	_identity_input.text_submitted.connect(func(_t): _on_login_pressed())
	_passphrase_input.text_submitted.connect(func(_t): _on_login_pressed())

func _field_label(text: String) -> Label:
	var l = Label.new()
	l.text = text
	l.add_theme_font_size_override("font_size", DubSarTheme.SIZE_LABEL)
	l.add_theme_color_override("font_color", DubSarTheme.TEXT_DIM)
	return l

func _on_login_pressed() -> void:
	var identity := _identity_input.text.strip_edges()
	var passphrase := _passphrase_input.text

	if identity.is_empty():
		_show_error("Please enter your sovereign identity.")
		return
	if passphrase.is_empty():
		_show_error("Please enter your key unlock passphrase.")
		return

	var passport_type: String = _passport_choice.get_item_text(_passport_choice.selected)

	var vault := _load_login_vault()
	var record = _find_vault_record(vault, identity, passport_type)
	if record == null:
		_show_error("No %s imported for \"%s\" yet. Use \"Import Passport…\" below first." % [passport_type, identity])
		return

	# FIXED (2026-07-27): derive_key() mints a fresh RANDOM salt on every
	# call by design (kupru::sargon_kdf::SargonKdf::new) -- re-deriving
	# here without the salt from import time would produce a key that
	# can never match the one used to encrypt, and decrypt_vault_blob
	# would fail for every correct passphrase, always. Same bug, same
	# fix, as dubsar-theater's login.gd (caught live via PB-261's
	# round-trip check).
	var salt: PackedByteArray = Marshalls.base64_to_raw(record["salt_b64"])
	var derived = _bridge.derive_key_with_salt(_login_key_root(identity, passport_type), passphrase, salt)
	if not derived.get("ok", false):
		_show_error("Key derivation failed: %s" % derived.get("error", "?"))
		return

	var sealed: PackedByteArray = Marshalls.base64_to_raw(record["sealed_b64"])
	var decrypted = _bridge.decrypt_vault_blob(derived["key"], sealed)
	if not decrypted.get("ok", false):
		_show_error("Wrong passphrase for \"%s\" (%s)." % [identity, passport_type])
		return

	var passport_json: String = decrypted["plaintext"].get_string_from_utf8()
	var check = _bridge.verify_passport(passport_json)
	if not check.get("ok", false):
		_show_error("Stored Passport is corrupted: %s" % check.get("error", "?"))
		return
	if not check.get("valid", false):
		_show_error("Passport seal invalid or expired: %s" % check.get("error", "seal check failed"))
		return

	var summary = _bridge.passport_summary(passport_json)
	SessionIdentity.set_from_verified(identity, passport_type, summary)

	var seal := AkkadiSafeBridge.seal_profile({"identity": identity, "passport_type": passport_type})
	if _remember_check.button_pressed:
		var res := AkkadiSafeBridge.store_secret(identity, passphrase)
		if res != AkkadiSafeBridge.Result.OK:
			push_warning("AkkadiSafeEngine vault unavailable — passphrase NOT persisted (lawful refusal).")

	print("[DubSarPdmLogin] Passport VERIFIED: %s  identity=%s  privilege_level=%d  seal=%s" % [
		passport_type, identity, summary.get("privilege_level", 0), seal
	])
	get_tree().change_scene_to_file(PDM_WORKSPACE_SCENE)

func _show_error(message: String) -> void:
	_error_label.text = message
	_error_label.visible = true

# ── Passport import (identical flow to dubsar-theater's login.gd) ────

func _on_import_pressed() -> void:
	var dialog = FileDialog.new()
	dialog.title = "Select an exported Passport JSON"
	dialog.file_mode = FileDialog.FILE_MODE_OPEN_FILE
	dialog.access = FileDialog.ACCESS_FILESYSTEM
	dialog.add_filter("*.json", "Passport JSON")
	add_child(dialog)
	dialog.file_selected.connect(func(path): _on_import_file_selected(dialog, path))
	dialog.canceled.connect(func(): dialog.queue_free())
	dialog.popup_centered(Vector2i(600, 400))

func _on_import_file_selected(dialog: FileDialog, path: String) -> void:
	dialog.queue_free()
	var f = FileAccess.open(path, FileAccess.READ)
	if f == null:
		_show_error("Could not read %s (error code %d)." % [path, FileAccess.get_open_error()])
		return
	var passport_json = f.get_as_text()
	f.close()

	var check = _bridge.verify_passport(passport_json)
	if not check.get("ok", false):
		_show_error("That file isn't valid passport JSON: %s" % check.get("error", "?"))
		return
	if not check.get("valid", false):
		_show_error("That Passport's seal is invalid or it's expired -- won't import: %s" % check.get("error", "seal check failed"))
		return

	_pending_import_json = passport_json
	_show_import_details_dialog()

func _show_import_details_dialog() -> void:
	var dialog = ConfirmationDialog.new()
	dialog.title = "Register this Passport with DubSar PDM"
	dialog.dialog_close_on_escape = false

	var vb = VBoxContainer.new()
	vb.add_theme_constant_override("separation", 8)

	vb.add_child(_field_label("IDENTITY (what you'll type to log in)"))
	var identity_input = LineEdit.new()
	identity_input.text = _identity_input.text.strip_edges()
	identity_input.custom_minimum_size = Vector2(320, 0)
	vb.add_child(identity_input)

	vb.add_child(_field_label("PASSPORT TYPE"))
	var type_choice = OptionButton.new()
	for p in PASSPORT_TYPES:
		type_choice.add_item(p)
	type_choice.select(_passport_choice.selected)
	vb.add_child(type_choice)

	vb.add_child(_field_label("NEW DubSar PDM login passphrase (your own choice -- does not need to match any passphrase used in Sargon/Gilgamesh/DubSar IDE)"))
	var pass_input = LineEdit.new()
	pass_input.secret = true
	pass_input.custom_minimum_size = Vector2(320, 0)
	vb.add_child(pass_input)

	vb.add_child(_field_label("CONFIRM PASSPHRASE"))
	var pass_confirm_input = LineEdit.new()
	pass_confirm_input.secret = true
	pass_confirm_input.custom_minimum_size = Vector2(320, 0)
	vb.add_child(pass_confirm_input)

	dialog.add_child(vb)
	add_child(dialog)
	dialog.confirmed.connect(func():
		_on_import_details_confirmed(dialog, identity_input, type_choice, pass_input, pass_confirm_input)
	)
	dialog.canceled.connect(func():
		_pending_import_json = ""
		dialog.queue_free()
	)
	dialog.popup_centered()
	identity_input.grab_focus()

func _on_import_details_confirmed(
	dialog: ConfirmationDialog, identity_input: LineEdit, type_choice: OptionButton,
	pass_input: LineEdit, pass_confirm_input: LineEdit
) -> void:
	var identity := identity_input.text.strip_edges()
	var passport_type: String = type_choice.get_item_text(type_choice.selected)
	var passphrase := pass_input.text

	if identity.is_empty():
		_show_error("Identity cannot be empty. Click Import Passport… to try again.")
		dialog.queue_free()
		_pending_import_json = ""
		return
	if passphrase.is_empty() or passphrase != pass_confirm_input.text:
		_show_error("Passphrases were empty or didn't match. Click Import Passport… to try again.")
		dialog.queue_free()
		_pending_import_json = ""
		return

	var derived = _bridge.derive_key(_login_key_root(identity, passport_type), passphrase)
	if not derived.get("ok", false):
		_show_error("Key derivation failed: %s" % derived.get("error", "?"))
		dialog.queue_free()
		_pending_import_json = ""
		return

	var sealed = _bridge.encrypt_vault_blob(derived["key"], _pending_import_json.to_utf8_buffer())
	if not sealed.get("ok", false):
		_show_error("Encrypt failed: %s" % sealed.get("error", "?"))
		dialog.queue_free()
		_pending_import_json = ""
		return

	var vault := _load_login_vault()
	var existing = _find_vault_record(vault, identity, passport_type)
	var new_record = {
		"identity": identity,
		"passport_type": passport_type,
		"sealed_b64": Marshalls.raw_to_base64(sealed["sealed"]),
		# FIXED (2026-07-27): must be persisted so a later login can
		# re-derive the same key via derive_key_with_salt. See
		# _on_login_pressed for the matching fix.
		"salt_b64": Marshalls.raw_to_base64(derived["salt"]),
	}
	if existing != null:
		vault[vault.find(existing)] = new_record
	else:
		vault.append(new_record)
	_save_login_vault(vault)

	_pending_import_json = ""
	dialog.queue_free()
	_identity_input.text = identity
	_passport_choice.select(type_choice.selected)
	_error_label.visible = false
	print("[DubSarPdmLogin] Passport imported for identity=%s type=%s" % [identity, passport_type])

# ── Login vault I/O ──────────────────────────────────────────────────

func _login_key_root(identity: String, passport_type: String) -> String:
	return "dubsar-pdm-login-%s-%s" % [identity.to_lower(), passport_type.to_lower().replace(" ", "-")]

func _load_login_vault() -> Array:
	if not FileAccess.file_exists(LOGIN_VAULT_PATH):
		return []
	var f = FileAccess.open(LOGIN_VAULT_PATH, FileAccess.READ)
	if f == null:
		return []
	var text = f.get_as_text()
	f.close()
	var parsed = JSON.parse_string(text)
	return parsed if parsed is Array else []

func _save_login_vault(vault: Array) -> void:
	var f = FileAccess.open(LOGIN_VAULT_PATH, FileAccess.WRITE)
	f.store_string(JSON.stringify(vault))
	f.close()

func _find_vault_record(vault: Array, identity: String, passport_type: String):
	for record in vault:
		if record.get("identity", "") == identity and record.get("passport_type", "") == passport_type:
			return record
	return null
