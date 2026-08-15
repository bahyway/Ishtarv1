extends Control
class_name SargonPassportManager
# =============================================================================
# Sargon Passport Manager -- BahyWay.Ecosystem v4.0
#
# Standalone "kind-of-KeePass" vault for SargonPassport credentials,
# backed by kupru-gdext (real Argon2id + Ed25519 + ChaCha20-Poly1305)
# via the KupruBridge GDExtension class. This is deliberately separate
# from AkkadiSafeBridge.gd (the DubSar Theater's own stub, still
# hard-coded to "UNSEALED-DEV") -- this tool is where the real crypto
# actually runs.
#
# Vault file: user://sargon_vault.dat -- a ChaCha20-Poly1305-encrypted
# JSON blob. The encryption key is Argon2id-derived from a master vault
# passphrase chosen on first unlock; nothing is ever written to disk
# unencrypted. Losing the master passphrase means losing the vault --
# there is no recovery path here (that's what the separate Gilgamesh
# Master Key tool and its Shamir-split ceremony are for).
# =============================================================================

const DubSarTheme = preload("res://scripts/dubsar_theme.gd")
const VAULT_PATH := "user://sargon_vault.dat"
# Fixed root phrase for the vault's OWN key derivation domain -- not a
# secret (the master passphrase is the actual secret); this just keeps
# the vault's KDF root distinct from any passport's own identity root.
const VAULT_ROOT_PHRASE := "sargon-vault-root"

var _bridge: KupruBridge
var _vault_key: PackedByteArray = PackedByteArray()
# The salt derive_key() returned when _vault_key was minted -- persisted
# on disk (prepended to VAULT_PATH, not secret) so a LATER session can
# re-derive the identical key via derive_key_with_salt. See the 2026-07-27
# fix note on _on_unlock_confirmed for why this is required.
var _vault_salt: PackedByteArray = PackedByteArray()
var _entries: Array = [] # each: {signing_key_b64, verifying_key_b64, passport_json}

var _list: ItemList
# Toggled by "Show Existing Passports History" -- off by default (today's
# compact one-line-per-entry view), on shows minted/expiry dates and a
# computed Valid/Renew soon/EXPIRED status per entry. The underlying data
# (created_at/expires_at/renewal_due) was already tracked via
# passport_summary() -- this only makes it visible, no new bridge calls.
var _show_history_detail: bool = false
var _status_label: Label
var _status_info_btn: Button
var _last_error_detail: String = ""
var _identity_input: LineEdit
var _realm_input: LineEdit
var _subject_input: LineEdit
var _master_pass_input: LineEdit

func _ready() -> void:
	get_tree().root.title = "BahyWay.Ecosystem v4.0 — Sargon Passport Manager"
	_bridge = KupruBridge.new()
	DubSarTheme.apply_page_background(self)
	_build_ui()
	_prompt_unlock()

func _build_ui() -> void:
	var margin = MarginContainer.new()
	margin.set_anchors_preset(Control.PRESET_FULL_RECT)
	margin.add_theme_constant_override("margin_left", 24)
	margin.add_theme_constant_override("margin_top", 24)
	margin.add_theme_constant_override("margin_right", 24)
	margin.add_theme_constant_override("margin_bottom", 24)
	add_child(margin)

	var vb = VBoxContainer.new()
	vb.add_theme_constant_override("separation", 14)
	margin.add_child(vb)

	var title = Label.new()
	title.text = "𒁾 Sargon Passport Manager"
	title.add_theme_font_size_override("font_size", DubSarTheme.SIZE_TITLE)
	title.add_theme_color_override("font_color", DubSarTheme.TEXT_PRIMARY)
	vb.add_child(title)

	var status_row = HBoxContainer.new()
	status_row.add_theme_constant_override("separation", 8)
	vb.add_child(status_row)

	_status_label = Label.new()
	_status_label.text = "Locked."
	_status_label.autowrap_mode = TextServer.AUTOWRAP_WORD
	_status_label.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	_status_label.add_theme_color_override("font_color", DubSarTheme.TEXT_DIM)
	status_row.add_child(_status_label)

	_status_info_btn = Button.new()
	_status_info_btn.text = "ⓘ"
	_status_info_btn.tooltip_text = "What does this error mean?"
	_status_info_btn.visible = false
	_status_info_btn.pressed.connect(_on_status_info_pressed)
	status_row.add_child(_status_info_btn)

	vb.add_child(HSeparator.new())

	var body = HBoxContainer.new()
	body.add_theme_constant_override("separation", 20)
	body.size_flags_vertical = Control.SIZE_EXPAND_FILL
	vb.add_child(body)

	# ── Left: vault list ──────────────────────────────────────────
	var left = VBoxContainer.new()
	left.custom_minimum_size = Vector2(380, 0)
	left.size_flags_vertical = Control.SIZE_EXPAND_FILL
	body.add_child(left)

	var list_label = Label.new()
	list_label.text = "VAULT ENTRIES"
	list_label.add_theme_font_size_override("font_size", DubSarTheme.SIZE_LABEL)
	list_label.add_theme_color_override("font_color", DubSarTheme.TEXT_DIM)
	left.add_child(list_label)

	_list = ItemList.new()
	_list.size_flags_vertical = Control.SIZE_EXPAND_FILL
	left.add_child(_list)

	var list_buttons = HBoxContainer.new()
	left.add_child(list_buttons)

	var btn_verify = Button.new()
	btn_verify.text = "Verify Selected"
	btn_verify.pressed.connect(_on_verify_pressed)
	list_buttons.add_child(btn_verify)

	var btn_renew = Button.new()
	btn_renew.text = "Renew Selected"
	btn_renew.pressed.connect(_on_renew_pressed)
	list_buttons.add_child(btn_renew)

	var btn_export = Button.new()
	btn_export.text = "Export JSON"
	btn_export.pressed.connect(_on_export_pressed)
	list_buttons.add_child(btn_export)

	var btn_history = Button.new()
	btn_history.text = "🕐 Show Existing Passports History"
	btn_history.toggle_mode = true
	btn_history.pressed.connect(func():
		_show_history_detail = btn_history.button_pressed
		_refresh_list()
		if _show_history_detail:
			_maybe_flash_no_history()
	)
	list_buttons.add_child(btn_history)

	# ── Right: new passport form ─────────────────────────────────
	var right = VBoxContainer.new()
	right.add_theme_constant_override("separation", 10)
	right.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	body.add_child(right)

	var form_label = Label.new()
	form_label.text = "ISSUE NEW GARDENER PASSPORT"
	form_label.add_theme_font_size_override("font_size", DubSarTheme.SIZE_LABEL)
	form_label.add_theme_color_override("font_color", DubSarTheme.TEXT_DIM)
	right.add_child(form_label)

	right.add_child(_field_label("IDENTITY PHRASE -- needs >= 3 CONSONANTS, not just letters. Vowels (a/e/i/o/u), spaces, and hyphens don't count -- e.g. \"sargon-kish\", not a short name alone."))
	_identity_input = LineEdit.new()
	_identity_input.placeholder_text = "e.g. sargon-kish"
	right.add_child(_identity_input)

	right.add_child(_field_label("REALM"))
	_realm_input = LineEdit.new()
	_realm_input.placeholder_text = "e.g. bahyway"
	right.add_child(_realm_input)

	right.add_child(_field_label("SUBJECT LABEL"))
	_subject_input = LineEdit.new()
	_subject_input.placeholder_text = "e.g. stakeholder-01"
	right.add_child(_subject_input)

	var btn_issue = Button.new()
	btn_issue.text = "▶ Generate & Save to Vault"
	btn_issue.add_theme_color_override("font_color", DubSarTheme.ACCENT_GREEN)
	btn_issue.pressed.connect(_on_issue_pressed)
	right.add_child(btn_issue)

	var hint = Label.new()
	hint.text = "Each entry gets its OWN Ed25519 keypair. Renewal always reissues a fresh, non-expired passport with the same keypair -- kupru has no \"extend\" operation by design."
	hint.autowrap_mode = TextServer.AUTOWRAP_WORD
	hint.add_theme_font_size_override("font_size", DubSarTheme.SIZE_LABEL)
	hint.add_theme_color_override("font_color", DubSarTheme.TEXT_FAINT)
	right.add_child(hint)

func _field_label(text: String) -> Label:
	var l = Label.new()
	l.text = text
	l.add_theme_font_size_override("font_size", DubSarTheme.SIZE_LABEL)
	l.add_theme_color_override("font_color", DubSarTheme.TEXT_DIM)
	return l

# kupru's own errors are prefixed with Akkadian tags meant for logs, not
# end users -- e.g. the raw string for a too-short identity phrase is
# literally "LEMNĪ (invalid input): LEMNĪ: < 3 consonants". Returns
# {summary, detail}: summary is a short line for the status bar; detail
# is a longer plain-language explanation (with the original raw message
# kept at the end) shown by the ⓘ button.
func _explain_kupru_error(raw: String) -> Dictionary:
	if raw.find("< 3 consonants") != -1 or raw.find("empty phrase") != -1:
		return {
			"summary": "Identity phrase needs at least 3 consonants.",
			"detail": "Vowels (a/e/i/o/u), spaces, and hyphens don't count toward the 3-consonant minimum. Try \"sargon-kish\" or \"dubsar-nabu\" instead of a short name.\n\nOriginal message: %s" % raw,
		}
	return {"summary": raw, "detail": "Original message: %s" % raw}

# Sets the status label to "<prefix>: <friendly summary>", stores the
# fuller explanation, and reveals the ⓘ button so it can be read on
# demand instead of forcing a wall of text into the status bar.
func _show_error(prefix: String, raw: String) -> void:
	var explained = _explain_kupru_error(raw)
	_status_label.text = "%s: %s" % [prefix, explained.summary]
	_last_error_detail = explained.detail
	_status_info_btn.visible = true

func _on_status_info_pressed() -> void:
	var dialog = AcceptDialog.new()
	dialog.title = "What does this mean?"
	dialog.dialog_text = _last_error_detail
	add_child(dialog)
	dialog.confirmed.connect(func(): dialog.queue_free())
	dialog.canceled.connect(func(): dialog.queue_free())
	dialog.popup_centered(Vector2i(560, 280))

# ── Vault unlock ─────────────────────────────────────────────────

func _prompt_unlock() -> void:
	var dialog = ConfirmationDialog.new()
	dialog.title = "Unlock Sargon Vault"
	dialog.dialog_close_on_escape = false
	var vb = VBoxContainer.new()
	var label = Label.new()
	label.text = "Master vault passphrase:"
	vb.add_child(label)
	_master_pass_input = LineEdit.new()
	_master_pass_input.secret = true
	_master_pass_input.custom_minimum_size = Vector2(320, 0)
	vb.add_child(_master_pass_input)
	dialog.add_child(vb)
	add_child(dialog)
	dialog.confirmed.connect(func(): _on_unlock_confirmed(dialog))
	dialog.popup_centered()
	_master_pass_input.grab_focus()

func _on_unlock_confirmed(dialog: ConfirmationDialog) -> void:
	var passphrase = _master_pass_input.text
	if passphrase.is_empty():
		_status_label.text = "Passphrase cannot be empty. Try again."
		dialog.popup_centered()
		return

	# FIXED (2026-07-27): derive_key() mints a fresh RANDOM salt on every
	# call (kupru::sargon_kdf::SargonKdf::new) -- calling it plain here on
	# every unlock, including for a vault that already exists on disk,
	# would derive a DIFFERENT key than whatever encrypted that file, and
	# decrypt_vault_blob would fail for every correct passphrase, always
	# (this vault could never actually be reopened across app restarts
	# before this fix -- caught live via PB-261's round-trip check on the
	# same underlying bug in DubSar's own login vault). An existing
	# vault's salt is read back from the file itself (prepended, not
	# secret) and re-derived via derive_key_with_salt; only a genuinely
	# new vault mints a fresh salt.
	if FileAccess.file_exists(VAULT_PATH):
		if not _load_vault(passphrase):
			return
	else:
		var derived = _bridge.derive_key(VAULT_ROOT_PHRASE, passphrase)
		if not derived.get("ok", false):
			_status_label.text = "Key derivation failed: %s" % derived.get("error", "?")
			return
		_vault_key = derived["key"]
		_vault_salt = derived["salt"]
		_entries = []
		_status_label.text = "New vault -- unsaved until the first passport is issued."
	dialog.queue_free()
	_refresh_list()

# ── Vault I/O ────────────────────────────────────────────────────
#
# File layout: 32 raw salt bytes, then the ChaCha20-Poly1305 sealed blob
# (nonce+ciphertext+tag). The salt is not secret (kupru's own doc
# comment says so) -- it MUST be stored plainly so a later session's
# unlock can re-derive the identical key.

func _load_vault(passphrase: String) -> bool:
	var f = FileAccess.open(VAULT_PATH, FileAccess.READ)
	if f == null:
		_status_label.text = "Could not open vault file."
		return false
	var salt: PackedByteArray = f.get_buffer(32)
	var sealed: PackedByteArray = f.get_buffer(f.get_length() - 32)
	f.close()

	var derived = _bridge.derive_key_with_salt(VAULT_ROOT_PHRASE, passphrase, salt)
	if not derived.get("ok", false):
		_status_label.text = "Key derivation failed: %s" % derived.get("error", "?")
		return false
	_vault_key = derived["key"]
	_vault_salt = salt

	var result = _bridge.decrypt_vault_blob(_vault_key, sealed)
	if not result.get("ok", false):
		_status_label.text = "WRONG PASSPHRASE, or the vault file is corrupted (decrypt failed)."
		_entries = []
		return false

	var plaintext_bytes: PackedByteArray = result["plaintext"]
	var json_text = plaintext_bytes.get_string_from_utf8()
	var parsed = JSON.parse_string(json_text)
	_entries = parsed if parsed is Array else []
	_status_label.text = "Vault unlocked -- %d passport(s)." % _entries.size()
	return true

func _save_vault() -> void:
	var json_text = JSON.stringify(_entries)
	var plaintext = json_text.to_utf8_buffer()
	var result = _bridge.encrypt_vault_blob(_vault_key, plaintext)
	if not result.get("ok", false):
		_status_label.text = "Encrypt failed: %s" % result.get("error", "?")
		return
	var f = FileAccess.open(VAULT_PATH, FileAccess.WRITE)
	f.store_buffer(_vault_salt)
	f.store_buffer(result["sealed"])
	f.close()
	_status_label.text = "Vault saved -- %d passport(s)." % _entries.size()

# ── Actions ──────────────────────────────────────────────────────

func _on_issue_pressed() -> void:
	_status_info_btn.visible = false
	if _identity_input.text.is_empty() or _realm_input.text.is_empty() or _subject_input.text.is_empty():
		_status_label.text = "Fill in identity phrase, realm, and subject label first."
		return

	var kp = _bridge.generate_keypair()
	if not kp.get("ok", false):
		_show_error("Keypair generation failed", kp.get("error", "?"))
		return

	var issued = _bridge.issue_gardener_passport(
		kp["signing_key"], _identity_input.text, _realm_input.text, _subject_input.text
	)
	if not issued.get("ok", false):
		_show_error("Passport issuance failed", issued.get("error", "?"))
		return

	_entries.append({
		"signing_key_b64": Marshalls.raw_to_base64(kp["signing_key"]),
		"verifying_key_b64": Marshalls.raw_to_base64(kp["verifying_key"]),
		"passport_json": issued["passport_json"],
		# The passport itself never stores this in plaintext -- it's
		# SHA3-hashed straight into subject_kaki at issuance (see
		# kupru-gdext's issue_passport_inner). Kept HERE, vault-side
		# only, purely so the list can tell entries apart.
		"subject_label": _subject_input.text,
	})
	_save_vault()
	_refresh_list()
	_identity_input.text = ""
	_realm_input.text = ""
	_subject_input.text = ""

func _selected_entry_index() -> int:
	var sel = _list.get_selected_items()
	return sel[0] if not sel.is_empty() else -1

func _on_verify_pressed() -> void:
	var idx = _selected_entry_index()
	if idx < 0:
		_status_label.text = "Select an entry first."
		return
	var result = _bridge.verify_passport(_entries[idx]["passport_json"])
	if result.get("valid", false):
		_status_label.text = "✓ Seal valid, not expired."
	else:
		_status_label.text = "✗ INVALID: %s" % result.get("error", "seal check failed")

func _on_renew_pressed() -> void:
	_status_info_btn.visible = false
	var idx = _selected_entry_index()
	if idx < 0:
		_status_label.text = "Select an entry first."
		return
	var entry = _entries[idx]
	var signing_key = Marshalls.base64_to_raw(entry["signing_key_b64"])

	var old = JSON.parse_string(entry["passport_json"])
	if old == null:
		_status_label.text = "Could not parse stored passport JSON."
		return
	var realm: String = old["naru"]["realm"]
	# Renewal always reissues -- kupru has no "extend" by design (see
	# ARCHITECT_KEY_CEREMONY.md / passport.rs's renewal_due() header).
	# The original identity phrase isn't stored (only its commitment
	# is, inside linguistic_proof) -- ask for it again for renewal.
	if _identity_input.text.is_empty() or _subject_input.text.is_empty():
		_status_label.text = "Enter the SAME identity phrase and subject label used originally, then press Renew again."
		return

	var reissued = _bridge.issue_gardener_passport(
		signing_key, _identity_input.text, realm, _subject_input.text
	)
	if not reissued.get("ok", false):
		_show_error("Renewal failed", reissued.get("error", "?"))
		return
	entry["passport_json"] = reissued["passport_json"]
	_entries[idx] = entry
	_save_vault()
	_refresh_list()
	_status_label.text = "Renewed."

func _on_export_pressed() -> void:
	var idx = _selected_entry_index()
	if idx < 0:
		_status_label.text = "Select an entry first."
		return
	var passport_json: String = _entries[idx]["passport_json"]

	var dialog = FileDialog.new()
	dialog.file_mode = FileDialog.FILE_MODE_SAVE_FILE
	dialog.access = FileDialog.ACCESS_FILESYSTEM
	dialog.add_filter("*.json", "Passport JSON")
	# THIS is the file to pick when importing into DubSar's login.
	dialog.current_file = "sargon-PASSPORT-%s.json" % [_entries[idx].get("subject_label", "unnamed")]
	add_child(dialog)
	dialog.file_selected.connect(func(path):
		var f = FileAccess.open(path, FileAccess.WRITE)
		f.store_string(passport_json)
		f.close()
		_status_label.text = "Exported to %s" % path
		dialog.queue_free()
	)
	dialog.popup_centered(Vector2i(600, 400))

# FIXED 2026-08-02, live: pressing "Show Existing Passports History" on a
# vault with zero entries left the list silently empty, with nothing on
# screen actually saying so -- a user with a still-locked vault (the
# persistent "Locked." header text is easy to miss) had no way to tell
# that apart from a real, unlocked, genuinely-empty vault. Both cases now
# get an explicit popup AND a status-bar line the moment the toggle is
# switched on, worded differently so the two are never confused with each
# other.
func _maybe_flash_no_history() -> void:
	if not _entries.is_empty():
		return
	var msg: String
	if _vault_key.is_empty():
		msg = "Vault is locked -- unlock it first (master passphrase) to see any passport history."
	else:
		msg = "No history of any passport yet -- this vault has never had one issued."
	_status_label.text = msg
	var dialog = AcceptDialog.new()
	dialog.title = "Passport History"
	dialog.dialog_text = msg
	add_child(dialog)
	dialog.confirmed.connect(func(): dialog.queue_free())
	dialog.canceled.connect(func(): dialog.queue_free())
	dialog.popup_centered(Vector2i(420, 160))

func _refresh_list() -> void:
	_list.clear()
	for entry in _entries:
		var summary = _bridge.passport_summary(entry["passport_json"])
		var text := "?"
		if summary.get("ok", false):
			var label: String = entry.get("subject_label", "")
			var label_part = " (%s)" % label if not label.is_empty() else ""
			if _show_history_detail:
				text = _history_detail_line(summary, label_part)
			else:
				var renew_flag = "  [RENEW DUE]" if summary.get("renewal_due", false) else ""
				text = "%s%s -- level %d%s" % [
					summary.get("realm", "?"), label_part, summary.get("privilege_level", 0), renew_flag
				]
		_list.add_item(text)

# One line for the "Show Existing Passports History" view: real minted/
# expiry dates plus a computed status, distinguishing an actually-expired
# passport from one merely approaching renewal -- today's compact view
# only ever showed a bare [RENEW DUE] flag with no dates at all.
func _history_detail_line(summary: Dictionary, label_part: String) -> String:
	var now := Time.get_unix_time_from_system()
	var expires_at: int = summary.get("expires_at", 0)
	var created_at: int = summary.get("created_at", 0)
	var status: String
	if now >= expires_at:
		status = "EXPIRED"
	elif summary.get("renewal_due", false):
		status = "Renew soon"
	else:
		status = "Valid"
	var created_str = Time.get_datetime_string_from_unix_time(created_at)
	var expires_str = Time.get_datetime_string_from_unix_time(expires_at)
	return "%s%s -- level %d -- minted %s -- expires %s -- %s" % [
		summary.get("realm", "?"), label_part, summary.get("privilege_level", 0),
		created_str, expires_str, status
	]
