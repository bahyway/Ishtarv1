extends Control
class_name GilgameshMasterKey
# =============================================================================
# Gilgamesh Master Key -- BahyWay.Ecosystem v4.0
#
# The GUI for kupru's Architect Key ceremony (see crates/kupru/
# ARCHITECT_KEY_CEREMONY.md -- read that document before actually using
# this tool for real, not just this code comment). There is deliberately
# NO single master key ever saved to one file here: Split writes each
# share to a SEPARATE location you choose one at a time; Reconstruct
# requires gathering the threshold back together. The reconstructed key
# is held only in memory for this session and is never itself written
# to disk -- only minted architect passports (which you export
# explicitly) leave this tool.
#
# UI is a 5-step PAGER (TabContainer with its tab bar hidden), not one
# long vertical scroll -- a VM window that can't be resized freely
# otherwise forces constant scrolling to see any given step. Only the
# current step is visible; Previous/Next drive it.
#
# Step 5 is a personal, read-only LEDGER: every architect passport this
# tool mints gets logged locally (client label, date/time, realm,
# passport_id, expiry -- metadata only, NEVER the signing key itself,
# same "never store the whole key" principle as the rest of this
# ceremony) so you can later tell which passport belongs to which
# client. It is its OWN separate encrypted file with its OWN passphrase
# -- unrelated to the root key, its shares, or Sargon's vault.
# =============================================================================

const DubSarTheme = preload("res://scripts/dubsar_theme.gd")

const STEP_TITLES := [
	"STEP 1 — GENERATE (do this offline)",
	"STEP 2 — SPLIT (M-of-N Shamir shares)",
	"STEP 3 — RECONSTRUCT (break-glass only)",
	"STEP 4 — MINT ARCHITECT PASSPORT",
	"STEP 5 — LEDGER (your own record, view only)",
]

const LEDGER_PATH := "user://gilgamesh_ledger.dat"
# Fixed root phrase for the ledger's OWN key derivation domain -- not a
# secret (the ledger passphrase is the actual secret); keeps the
# ledger's KDF root distinct from any passport's own identity root.
const LEDGER_ROOT_PHRASE := "gilgamesh-ledger-root"

var _bridge: KupruBridge

# In-memory only -- never persisted directly by this tool.
var _current_signing_key: PackedByteArray = PackedByteArray()
var _current_verifying_key: PackedByteArray = PackedByteArray()
var _gathered_shares: Array[String] = [] # JSON strings

var _status_label: Label
var _status_info_btn: Button
var _last_error_detail: String = ""
var _key_status_label: LineEdit
var _threshold_spin: SpinBox
var _total_spin: SpinBox
var _shares_to_save: Array = [] # queued JSON strings awaiting a save dialog
# True once every share from the CURRENT key in memory has actually been
# saved (Step 2's full save-dialog chain completed), or the current key
# came from Reconstruct (Step 3) -- a reconstructed key was already split
# in a prior ceremony, so there's nothing to warn about unless the
# Architect chooses to re-split it, which sets this true again anyway.
# Reset to false on every fresh Generate, since that's a brand-new key
# with no shares of its own yet. Never a substitute for real distribution
# discipline -- just catches the "clicked Next past Split without
# thinking about it" case Step 2 -> 3 navigation makes easy to do by accident.
var _key_has_been_split: bool = false

# True once the CURRENT minted passport in _passport_output has actually
# been written to disk. Unlike _key_has_been_split (a soft confirmation --
# see that var's own comment for why), this one genuinely disables Next
# on Step 4: a minted passport is the entire deliverable of this
# ceremony, there's no legitimate "just testing" reason to leave Step 4
# with a real one unsaved, and unlike Split there's no verification gap
# this could give false confidence about -- either the file got written
# or it didn't. Reset to false every time a NEW passport is minted (a
# re-mint for a second client in the same session must be exported on
# its own, not ride on an earlier export).
var _passport_exported: bool = false
var _gathered_count_label: Label
var _identity_input: LineEdit
var _realm_input: LineEdit
var _subject_input: LineEdit
var _client_label_input: LineEdit
var _passport_output: TextEdit

var _ledger_key: PackedByteArray = PackedByteArray()
# The salt derive_key() returned when _ledger_key was minted -- persisted
# on disk (prepended to LEDGER_PATH, not secret) so a LATER session can
# re-derive the identical key via derive_key_with_salt. See the
# 2026-07-27 fix note on _on_ledger_unlock_confirmed for why this is required.
var _ledger_salt: PackedByteArray = PackedByteArray()
var _ledger_unlocked: bool = false
var _ledger_entries: Array = [] # each: {client_label, minted_at, realm, subject_label, passport_id, expires_at, privilege_level}
var _pending_ledger_entries: Array = [] # minted before the ledger was ever unlocked this session
var _ledger_list: ItemList
var _ledger_status_label: Label

var _tabs: TabContainer
var _step_indicator: Label
var _btn_prev: Button
var _btn_next: Button
var _current_step: int = 0

func _ready() -> void:
	get_tree().root.title = "BahyWay.Ecosystem v4.0 — Gilgamesh Master Key"
	_bridge = KupruBridge.new()
	DubSarTheme.apply_page_background(self)
	_build_ui()
	_show_step(0)

func _build_ui() -> void:
	var margin = MarginContainer.new()
	margin.set_anchors_preset(Control.PRESET_FULL_RECT)
	margin.add_theme_constant_override("margin_left", 24)
	margin.add_theme_constant_override("margin_top", 18)
	margin.add_theme_constant_override("margin_right", 24)
	margin.add_theme_constant_override("margin_bottom", 18)
	add_child(margin)

	var root_vb = VBoxContainer.new()
	root_vb.add_theme_constant_override("separation", 10)
	root_vb.size_flags_vertical = Control.SIZE_EXPAND_FILL
	margin.add_child(root_vb)

	var title = Label.new()
	title.text = "𒁾 Gilgamesh Master Key — Architect Ceremony"
	title.add_theme_font_size_override("font_size", DubSarTheme.SIZE_TITLE)
	title.add_theme_color_override("font_color", DubSarTheme.TEXT_PRIMARY)
	root_vb.add_child(title)

	var warning = Label.new()
	warning.text = "Read crates/kupru/ARCHITECT_KEY_CEREMONY.md before using this for a real root key. Never save two shares to the same location."
	warning.autowrap_mode = TextServer.AUTOWRAP_WORD
	warning.add_theme_font_size_override("font_size", DubSarTheme.SIZE_LABEL)
	warning.add_theme_color_override("font_color", DubSarTheme.ACCENT_AMBER)
	root_vb.add_child(warning)

	var status_row = HBoxContainer.new()
	status_row.add_theme_constant_override("separation", 8)
	root_vb.add_child(status_row)

	_status_label = Label.new()
	_status_label.text = "No key in memory."
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

	root_vb.add_child(HSeparator.new())

	# ── Step pager: only the current step is visible, no long scroll ──
	_tabs = TabContainer.new()
	_tabs.tabs_visible = false
	_tabs.size_flags_vertical = Control.SIZE_EXPAND_FILL
	root_vb.add_child(_tabs)

	_tabs.add_child(_build_step1())
	_tabs.add_child(_build_step2())
	_tabs.add_child(_build_step3())
	_tabs.add_child(_build_step4())
	_tabs.add_child(_build_step5())

	root_vb.add_child(HSeparator.new())

	# ── Nav bar ──────────────────────────────────────────────────────
	var nav = HBoxContainer.new()
	nav.add_theme_constant_override("separation", 12)
	root_vb.add_child(nav)

	_btn_prev = Button.new()
	_btn_prev.text = "< Previous"
	_btn_prev.pressed.connect(_on_prev_pressed)
	nav.add_child(_btn_prev)

	_step_indicator = Label.new()
	_step_indicator.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	_step_indicator.horizontal_alignment = HORIZONTAL_ALIGNMENT_CENTER
	_step_indicator.add_theme_color_override("font_color", DubSarTheme.ACCENT_CYAN)
	nav.add_child(_step_indicator)

	_btn_next = Button.new()
	_btn_next.text = "Next >"
	_btn_next.pressed.connect(_on_next_pressed)
	nav.add_child(_btn_next)

# Each step's content is wrapped in its own ScrollContainer -- belt and
# braces if a window is EVEN smaller than expected, that one step's
# content can still scroll internally without needing the whole ceremony
# laid out at once.
func _wrap_step(vb: VBoxContainer) -> ScrollContainer:
	var scroll = ScrollContainer.new()
	scroll.size_flags_vertical = Control.SIZE_EXPAND_FILL
	scroll.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	vb.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	scroll.add_child(vb)
	return scroll

func _build_step1() -> Control:
	var vb = VBoxContainer.new()
	vb.add_theme_constant_override("separation", 12)

	vb.add_child(_section_label(STEP_TITLES[0]))

	var explain = Label.new()
	explain.text = "Generates a brand-new Ed25519 root signing key, held ONLY in this session's memory -- it is never written to disk as a whole. Do this on an offline machine for a real ceremony. When you're ready, click Next below to split it into shares."
	explain.autowrap_mode = TextServer.AUTOWRAP_WORD
	explain.add_theme_font_size_override("font_size", DubSarTheme.SIZE_LABEL)
	explain.add_theme_color_override("font_color", DubSarTheme.TEXT_DIM)
	vb.add_child(explain)

	var btn_generate = Button.new()
	btn_generate.text = "▶ Generate New Root Key"
	btn_generate.add_theme_color_override("font_color", DubSarTheme.ACCENT_GREEN)
	btn_generate.pressed.connect(_on_generate_pressed)
	vb.add_child(btn_generate)

	vb.add_child(_field_label("VERIFYING KEY (public half -- safe to share, worth noting down)"))

	var key_row = HBoxContainer.new()
	key_row.add_theme_constant_override("separation", 8)
	vb.add_child(key_row)

	_key_status_label = LineEdit.new()
	_key_status_label.editable = false
	_key_status_label.text = "(none yet)"
	_key_status_label.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	key_row.add_child(_key_status_label)

	var btn_copy = Button.new()
	btn_copy.text = "Copy"
	btn_copy.pressed.connect(_on_copy_verifying_key_pressed)
	key_row.add_child(btn_copy)

	return _wrap_step(vb)

func _build_step2() -> Control:
	var vb = VBoxContainer.new()
	vb.add_theme_constant_override("separation", 12)

	vb.add_child(_section_label(STEP_TITLES[1]))

	var explain = Label.new()
	explain.text = "Splits the key currently in memory (from Step 1 or Step 3) into N encrypted share files using Shamir's Secret Sharing. Any M of them together reconstruct the key; fewer than M reveal nothing about it. After clicking Split, you'll get N separate \"Save\" dialogs in a row -- save each one to a GENUINELY different location or device (see ARCHITECT_KEY_CEREMONY.md). Never save two shares to the same place."
	explain.autowrap_mode = TextServer.AUTOWRAP_WORD
	explain.add_theme_font_size_override("font_size", DubSarTheme.SIZE_LABEL)
	explain.add_theme_color_override("font_color", DubSarTheme.TEXT_DIM)
	vb.add_child(explain)

	var split_row = HBoxContainer.new()
	split_row.add_theme_constant_override("separation", 12)
	vb.add_child(split_row)

	split_row.add_child(_field_label("Threshold (M)"))
	_threshold_spin = SpinBox.new()
	_threshold_spin.min_value = 2
	_threshold_spin.max_value = 20
	_threshold_spin.value = 3
	split_row.add_child(_threshold_spin)

	split_row.add_child(_field_label("Total shares (N)"))
	_total_spin = SpinBox.new()
	_total_spin.min_value = 2
	_total_spin.max_value = 20
	_total_spin.value = 5
	split_row.add_child(_total_spin)

	var btn_split = Button.new()
	btn_split.text = "Split & Save Shares…"
	btn_split.pressed.connect(_on_split_pressed)
	vb.add_child(btn_split)

	return _wrap_step(vb)

func _build_step3() -> Control:
	var vb = VBoxContainer.new()
	vb.add_theme_constant_override("separation", 12)

	vb.add_child(_section_label(STEP_TITLES[2]))

	var explain = Label.new()
	explain.text = "Break-glass only: gather at least the threshold (M) number of share files saved during Step 2, from wherever they were distributed. Click \"Add Share File…\" once per file -- select it, then press the dialog's Open button (double-clicking the file also works; a single click alone only highlights it). Once you've added enough, click Reconstruct below."
	explain.autowrap_mode = TextServer.AUTOWRAP_WORD
	explain.add_theme_font_size_override("font_size", DubSarTheme.SIZE_LABEL)
	explain.add_theme_color_override("font_color", DubSarTheme.TEXT_DIM)
	vb.add_child(explain)

	var btn_add_share = Button.new()
	btn_add_share.text = "Add Share File…"
	btn_add_share.pressed.connect(_on_add_share_pressed)
	vb.add_child(btn_add_share)

	_gathered_count_label = Label.new()
	_gathered_count_label.text = "0 shares gathered."
	_gathered_count_label.add_theme_color_override("font_color", DubSarTheme.TEXT_DIM)
	vb.add_child(_gathered_count_label)

	var btn_reconstruct = Button.new()
	btn_reconstruct.text = "▶ Reconstruct From Gathered Shares"
	btn_reconstruct.add_theme_color_override("font_color", DubSarTheme.ACCENT_GREEN)
	btn_reconstruct.pressed.connect(_on_reconstruct_pressed)
	vb.add_child(btn_reconstruct)

	return _wrap_step(vb)

func _build_step4() -> Control:
	var vb = VBoxContainer.new()
	vb.add_theme_constant_override("separation", 12)

	vb.add_child(_section_label(STEP_TITLES[3]))

	var explain = Label.new()
	explain.text = "Mints a fresh architect-privilege (level 7) passport from the key currently in memory (from Step 1 or Step 3)."
	explain.autowrap_mode = TextServer.AUTOWRAP_WORD
	explain.add_theme_font_size_override("font_size", DubSarTheme.SIZE_LABEL)
	explain.add_theme_color_override("font_color", DubSarTheme.TEXT_DIM)
	vb.add_child(explain)

	vb.add_child(_field_label("IDENTITY PHRASE -- needs >= 3 CONSONANTS, not just letters. a/e/i/o/u, spaces, and hyphens don't count, so a short name like \"nabu\" alone will fail. Use something like \"dubsar-nabu\" or \"sargon-kish\" instead."))
	_identity_input = LineEdit.new()
	_identity_input.placeholder_text = "e.g. dubsar-nabu"
	vb.add_child(_identity_input)

	vb.add_child(_field_label("REALM"))
	_realm_input = LineEdit.new()
	_realm_input.placeholder_text = "e.g. bahyway"
	vb.add_child(_realm_input)

	vb.add_child(_field_label("SUBJECT LABEL"))
	_subject_input = LineEdit.new()
	_subject_input.placeholder_text = "e.g. architect"
	vb.add_child(_subject_input)

	vb.add_child(_field_label("CLIENT LABEL (optional -- your own note, e.g. which client purchased this) -- recorded in your Ledger (Step 5), never inside the passport itself"))
	_client_label_input = LineEdit.new()
	_client_label_input.placeholder_text = "e.g. Acme Corp"
	vb.add_child(_client_label_input)

	var btn_mint = Button.new()
	btn_mint.text = "▶ Issue Architect Passport"
	btn_mint.add_theme_color_override("font_color", DubSarTheme.ACCENT_GREEN)
	btn_mint.pressed.connect(_on_mint_pressed)
	vb.add_child(btn_mint)

	_passport_output = TextEdit.new()
	_passport_output.custom_minimum_size = Vector2(0, 160)
	_passport_output.editable = false
	_passport_output.wrap_mode = TextEdit.LINE_WRAPPING_BOUNDARY
	vb.add_child(_passport_output)

	var btn_export = Button.new()
	btn_export.text = "Export Passport JSON…"
	btn_export.pressed.connect(_on_export_passport_pressed)
	vb.add_child(btn_export)

	return _wrap_step(vb)

func _build_step5() -> Control:
	var vb = VBoxContainer.new()
	vb.add_theme_constant_override("separation", 12)

	vb.add_child(_section_label(STEP_TITLES[4]))

	var explain = Label.new()
	explain.text = "Your own private record of every architect passport minted in this tool -- client label, date/time, realm, and expiry -- so you know which key belongs to which client later. Encrypted locally with its own separate passphrase; NEVER contains the signing key itself, only passport metadata. Nothing here is editable from this screen -- it's a log, not a vault you write to directly."
	explain.autowrap_mode = TextServer.AUTOWRAP_WORD
	explain.add_theme_font_size_override("font_size", DubSarTheme.SIZE_LABEL)
	explain.add_theme_color_override("font_color", DubSarTheme.TEXT_DIM)
	vb.add_child(explain)

	_ledger_status_label = Label.new()
	_ledger_status_label.text = "Ledger locked."
	_ledger_status_label.autowrap_mode = TextServer.AUTOWRAP_WORD
	_ledger_status_label.add_theme_color_override("font_color", DubSarTheme.TEXT_DIM)
	vb.add_child(_ledger_status_label)

	var btn_unlock = Button.new()
	btn_unlock.text = "Unlock Ledger"
	btn_unlock.pressed.connect(_on_unlock_ledger_pressed)
	vb.add_child(btn_unlock)

	_ledger_list = ItemList.new()
	_ledger_list.custom_minimum_size = Vector2(0, 240)
	_ledger_list.size_flags_vertical = Control.SIZE_EXPAND_FILL
	vb.add_child(_ledger_list)

	var btn_export = Button.new()
	btn_export.text = "Export Full Ledger JSON…"
	btn_export.pressed.connect(_on_export_ledger_pressed)
	vb.add_child(btn_export)

	return _wrap_step(vb)

func _section_label(text: String) -> Label:
	var l = Label.new()
	l.text = text
	l.add_theme_font_size_override("font_size", DubSarTheme.SIZE_LABEL)
	l.add_theme_color_override("font_color", DubSarTheme.ACCENT_CYAN)
	return l

func _field_label(text: String) -> Label:
	var l = Label.new()
	l.text = text
	l.add_theme_font_size_override("font_size", DubSarTheme.SIZE_LABEL)
	l.add_theme_color_override("font_color", DubSarTheme.TEXT_DIM)
	return l

# kupru's own errors are prefixed with Akkadian tags meant for logs, not
# end users (LEMNĪ = "bad/evil" = invalid input, DŪRU = "wall/barrier" =
# a Shamir threshold/share-count rule, NAKRU = "hostile" = a broken
# cryptographic seal) -- e.g. the raw string for a too-short identity
# phrase is literally "LEMNĪ (invalid input): LEMNĪ: < 3 consonants".
# Returns {summary, detail}: summary is a short line for the status bar;
# detail is a longer plain-language explanation (with the original raw
# message kept at the end, for anyone who wants the technical form too)
# shown by the ⓘ button. Anything unrecognized still gets a translated
# summary (there is no meaningful sentence to show as "the error" other
# than what kupru gave us) but its detail is just the raw text.
func _explain_kupru_error(raw: String) -> Dictionary:
	if raw.find("< 3 consonants") != -1 or raw.find("empty phrase") != -1:
		return {
			"summary": "Identity phrase needs at least 3 consonants.",
			"detail": "Vowels (a/e/i/o/u), spaces, and hyphens don't count toward the 3-consonant minimum. A short name like \"nabu\" alone has only 2 real consonants (n, b). Try something like \"dubsar-nabu\" or \"sargon-kish\" instead.\n\nOriginal message: %s" % raw,
		}

	var shares_re = RegEx.new()
	shares_re.compile("(\\d+) shares provided, but this split requires (\\d+)")
	var m = shares_re.search(raw)
	if m:
		var provided = m.get_string(1)
		var required = m.get_string(2)
		return {
			"summary": "Not enough shares yet -- %s of %s gathered." % [provided, required],
			"detail": "Reconstruction needs at least the threshold (M) chosen back in Step 2 -- for this split, that's %s. Only %s share(s) have been added so far. Click \"Add Share File…\" again to gather more from wherever the rest were saved, then try Reconstruct again.\n\nOriginal message: %s" % [required, provided, raw],
		}

	if raw.find("threshold must be >= 2") != -1:
		return {
			"summary": "Threshold (M) must be at least 2.",
			"detail": "A threshold of 1 would mean any single share reconstructs the whole key by itself, which defeats the entire point of splitting it. Raise the Threshold (M) spinner in Step 2 to 2 or higher.\n\nOriginal message: %s" % raw,
		}

	if raw.find("total_shares must be >= threshold") != -1:
		return {
			"summary": "Total shares (N) must be >= threshold (M).",
			"detail": "You can't require more shares to reconstruct than you're creating in total. Raise Total shares (N) in Step 2 to at least match Threshold (M).\n\nOriginal message: %s" % raw,
		}

	if raw.find("no shares provided") != -1:
		return {
			"summary": "No shares added yet.",
			"detail": "Click \"Add Share File…\" at least once before trying to Reconstruct.\n\nOriginal message: %s" % raw,
		}

	if raw.find("shares from different splits cannot be combined") != -1:
		return {
			"summary": "These shares are from different splits.",
			"detail": "Every share carries an internal split ID. Shares can only be combined if they all came from the SAME \"Split & Save Shares\" run -- mixing shares from two different ceremonies will never reconstruct anything. Double-check which files were gathered.\n\nOriginal message: %s" % raw,
		}

	return {"summary": raw, "detail": "Original message: %s" % raw}

# Convenience: sets the status label to "<prefix>: <friendly summary>",
# stores the fuller explanation, and reveals the ⓘ button so it can be
# read on demand instead of forcing a wall of text into the status bar.
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

# ── Pager navigation ───────────────────────────────────────────────

# Genuinely disables Next (not just a dismissable confirmation, unlike
# _confirm_skip_split) while on Step 4 with a minted passport that hasn't
# actually been written to disk yet -- see _passport_exported's own
# comment for why this one gets a hard gate and Split doesn't.
func _update_next_button_disabled() -> void:
	_btn_next.disabled = _current_step == 3 and not _passport_output.text.is_empty() and not _passport_exported

func _show_step(i: int) -> void:
	var previous = _current_step
	_current_step = clampi(i, 0, _tabs.get_tab_count() - 1)
	_tabs.current_tab = _current_step
	_step_indicator.text = "Step %d of %d" % [_current_step + 1, _tabs.get_tab_count()]
	_btn_prev.disabled = _current_step == 0
	var is_last = _current_step == _tabs.get_tab_count() - 1
	_update_next_button_disabled()
	_btn_next.text = "Finish" if is_last else "Next >"
	# First arrival at the Ledger step this session -- offer to unlock it
	# rather than making the user hunt for the button.
	if _current_step == 4 and previous != 4 and not _ledger_unlocked:
		_on_unlock_ledger_pressed()

func _on_prev_pressed() -> void:
	_show_step(_current_step - 1)

func _on_next_pressed() -> void:
	if _current_step == _tabs.get_tab_count() - 1:
		_confirm_finish()
		return
	# Leaving Step 2 (Split, index 1) forward with an unsplit key still in
	# memory -- easy to do by accident (two quick clicks), and the whole
	# point of the ceremony is that this key is never usable without
	# having been split first. A soft confirmation, not a hard block: a
	# real practice/test run legitimately wants to skip this every time,
	# and a share-count check couldn't verify genuinely-different storage
	# locations anyway -- see this file's own _key_has_been_split comment.
	if _current_step == 1 and not _key_has_been_split and not _current_signing_key.is_empty():
		_confirm_skip_split()
		return
	_show_step(_current_step + 1)

func _confirm_skip_split() -> void:
	var dialog = ConfirmationDialog.new()
	dialog.title = "Skip splitting into shares?"
	dialog.dialog_text = "You're proceeding without splitting this root key into shares. If this is a real ceremony, go back to Step 2 first -- this key won't be recoverable later otherwise. Especially if this is your first time of creating a Passport."
	add_child(dialog)
	dialog.confirmed.connect(func():
		dialog.queue_free()
		_show_step(_current_step + 1)
	)
	dialog.canceled.connect(func(): dialog.queue_free())
	dialog.popup_centered()

func _confirm_finish() -> void:
	var dialog = ConfirmationDialog.new()
	dialog.title = "Finish"
	dialog.dialog_text = "Close Gilgamesh Master Key? Anything still only in memory -- an unexported passport, an unsplit key, ungathered shares -- will be lost. Make sure you've saved what you need first."
	add_child(dialog)
	dialog.confirmed.connect(func(): get_tree().quit())
	dialog.canceled.connect(func(): dialog.queue_free())
	dialog.popup_centered()

# ── Step 1: generate ─────────────────────────────────────────────

func _on_generate_pressed() -> void:
	_status_info_btn.visible = false
	var kp = _bridge.generate_keypair()
	if not kp.get("ok", false):
		_show_error("Generation failed", kp.get("error", "?"))
		return
	_current_signing_key = kp["signing_key"]
	_current_verifying_key = kp["verifying_key"]
	_key_has_been_split = false
	_key_status_label.text = Marshalls.raw_to_base64(_current_verifying_key)
	_status_label.text = "New root key generated -- held in memory only. Proceed to Step 2."

func _on_copy_verifying_key_pressed() -> void:
	if _current_verifying_key.is_empty():
		_status_label.text = "Generate or reconstruct a key first."
		return
	DisplayServer.clipboard_set(Marshalls.raw_to_base64(_current_verifying_key))
	_status_label.text = "Verifying key copied to clipboard."

# ── Step 2: split ────────────────────────────────────────────────

func _on_split_pressed() -> void:
	_status_info_btn.visible = false
	if _current_signing_key.is_empty():
		_status_label.text = "Generate or reconstruct a key first."
		return
	var threshold = int(_threshold_spin.value)
	var total = int(_total_spin.value)
	var result = _bridge.split_architect_key(_current_signing_key, threshold, total)
	if not result.get("ok", false):
		_show_error("Split failed", result.get("error", "?"))
		return

	_shares_to_save = result["shares_json"].duplicate()
	_status_label.text = "Split into %d shares. Save each to a DIFFERENT location -- dialog will prompt %d times." % [total, total]
	_save_next_share()

func _save_next_share() -> void:
	if _shares_to_save.is_empty():
		_key_has_been_split = true
		_status_label.text = "All shares saved. Distribute them to genuinely different locations (see ceremony doc)."
		return
	var share_json: String = _shares_to_save.pop_front()
	var remaining = _shares_to_save.size()

	var share_index = int(_total_spin.value) - remaining
	var dialog = FileDialog.new()
	dialog.title = "Save share (%d more after this)" % remaining
	dialog.file_mode = FileDialog.FILE_MODE_SAVE_FILE
	dialog.access = FileDialog.ACCESS_FILESYSTEM
	dialog.add_filter("*.json", "Architect Key Share")
	# Deliberately distinct from the Passport export's default name below
	# -- these files look nothing alike in content, but similarly-named
	# JSON files saved to the same folder are easy to mix up otherwise
	# (this is exactly what happened importing a share into DubSar's
	# login instead of the actual exported Passport).
	dialog.current_file = "architect-key-SHARE-%d-of-%d.json" % [share_index, int(_total_spin.value)]
	add_child(dialog)
	dialog.file_selected.connect(func(path):
		var f = FileAccess.open(path, FileAccess.WRITE)
		f.store_string(share_json)
		f.close()
		dialog.queue_free()
		_save_next_share()
	)
	dialog.canceled.connect(func():
		dialog.queue_free()
		_status_label.text = "Share save cancelled -- %d share(s) still unsaved." % (remaining + 1)
	)
	dialog.popup_centered(Vector2i(600, 400))

# ── Step 3: reconstruct ──────────────────────────────────────────

func _on_add_share_pressed() -> void:
	var dialog = FileDialog.new()
	dialog.title = "Add a share file"
	dialog.file_mode = FileDialog.FILE_MODE_OPEN_FILE
	dialog.access = FileDialog.ACCESS_FILESYSTEM
	dialog.add_filter("*.json", "Architect Key Share")
	add_child(dialog)
	dialog.file_selected.connect(func(path):
		var f = FileAccess.open(path, FileAccess.READ)
		if f != null:
			_gathered_shares.append(f.get_as_text())
			f.close()
			_gathered_count_label.text = "%d share(s) gathered." % _gathered_shares.size()
			_status_label.text = "Added share from %s." % path
		else:
			var err_code = FileAccess.get_open_error()
			_status_label.text = "Could not read %s (error code %d) -- check the file exists and is readable, then try again." % [path, err_code]
		dialog.queue_free()
	)
	dialog.canceled.connect(func(): dialog.queue_free())
	dialog.popup_centered(Vector2i(600, 400))

func _on_reconstruct_pressed() -> void:
	_status_info_btn.visible = false
	if _gathered_shares.is_empty():
		_status_label.text = "Add shares first."
		return
	var arr: Array[String] = []
	for s in _gathered_shares:
		arr.append(s)
	var result = _bridge.reconstruct_architect_key(arr)
	if not result.get("ok", false):
		_show_error("Reconstruction failed", result.get("error", "?"))
		return
	_current_signing_key = result["signing_key"]
	_current_verifying_key = result["verifying_key"]
	# A reconstructed key was already split in a prior ceremony -- nothing
	# to warn about unless the Architect chooses to re-split it (Step 2's
	# own completion handler sets this true again regardless).
	_key_has_been_split = true
	_key_status_label.text = Marshalls.raw_to_base64(_current_verifying_key)
	_status_label.text = "Reconstructed from %d shares -- held in memory only. Proceed to Step 4." % _gathered_shares.size()

# ── Step 4: mint ─────────────────────────────────────────────────

func _on_mint_pressed() -> void:
	_status_info_btn.visible = false
	if _current_signing_key.is_empty():
		_status_label.text = "Generate or reconstruct a key first."
		return
	if _identity_input.text.is_empty() or _realm_input.text.is_empty() or _subject_input.text.is_empty():
		_status_label.text = "Fill in identity phrase, realm, and subject label first."
		return

	var result = _bridge.issue_architect_passport_local(
		_current_signing_key, _identity_input.text, _realm_input.text, _subject_input.text
	)
	if not result.get("ok", false):
		_show_error("Minting failed", result.get("error", "?"))
		return

	_passport_output.text = result["passport_json"]
	_passport_exported = false
	_update_next_button_disabled()
	_status_label.text = "Architect passport minted -- privilege level 7, still expires on the normal SATTATU_MAX schedule. Export it below before leaving this step."

	var summary = _bridge.passport_summary(result["passport_json"])
	if summary.get("ok", false):
		var entry = {
			"client_label": _client_label_input.text if not _client_label_input.text.is_empty() else "(unlabeled)",
			"minted_at": Time.get_unix_time_from_system(),
			"realm": summary.get("realm", "?"),
			"subject_label": _subject_input.text,
			"passport_id": summary.get("passport_id", "?"),
			"expires_at": summary.get("expires_at", 0),
			"privilege_level": summary.get("privilege_level", 0),
		}
		_append_to_ledger(entry)

func _on_export_passport_pressed() -> void:
	if _passport_output.text.is_empty():
		_status_label.text = "Mint a passport first."
		return
	var dialog = FileDialog.new()
	dialog.file_mode = FileDialog.FILE_MODE_SAVE_FILE
	dialog.access = FileDialog.ACCESS_FILESYSTEM
	dialog.add_filter("*.json", "Architect Passport JSON")
	# THIS is the file to pick when importing into DubSar's login --
	# named distinctly from the Shamir share files (see _save_next_share)
	# so the two are never confused in the same folder.
	dialog.current_file = "architect-PASSPORT-%s.json" % [_realm_input.text if not _realm_input.text.is_empty() else "unnamed"]
	add_child(dialog)
	dialog.file_selected.connect(func(path):
		var f = FileAccess.open(path, FileAccess.WRITE)
		f.store_string(_passport_output.text)
		f.close()
		_passport_exported = true
		_update_next_button_disabled()
		_status_label.text = "Exported to %s" % path
		dialog.queue_free()
	)
	dialog.popup_centered(Vector2i(600, 400))

# ── Step 5: ledger ─────────────────────────────────────────────────

func _append_to_ledger(entry: Dictionary) -> void:
	if _ledger_unlocked:
		_ledger_entries.append(entry)
		_save_ledger()
		_refresh_ledger_list()
	else:
		_pending_ledger_entries.append(entry)
		_status_label.text += " Unlock the Ledger (Step 5) to record this against a client."

func _on_unlock_ledger_pressed() -> void:
	if _ledger_unlocked:
		_ledger_status_label.text = "Already unlocked -- %d entr%s." % [
			_ledger_entries.size(), "y" if _ledger_entries.size() == 1 else "ies"
		]
		return
	var dialog = ConfirmationDialog.new()
	dialog.title = "Unlock Architect Ledger"
	dialog.dialog_close_on_escape = false
	var vb = VBoxContainer.new()
	var label = Label.new()
	label.text = "Ledger passphrase -- your own choice, separate from any vault or key passphrase. First use creates the ledger."
	label.autowrap_mode = TextServer.AUTOWRAP_WORD
	label.custom_minimum_size = Vector2(320, 0)
	vb.add_child(label)
	var input = LineEdit.new()
	input.secret = true
	input.custom_minimum_size = Vector2(320, 0)
	vb.add_child(input)
	dialog.add_child(vb)
	add_child(dialog)
	dialog.confirmed.connect(func(): _on_ledger_unlock_confirmed(dialog, input))
	dialog.canceled.connect(func(): dialog.queue_free())
	dialog.popup_centered()
	input.grab_focus()

func _on_ledger_unlock_confirmed(dialog: ConfirmationDialog, input: LineEdit) -> void:
	var passphrase = input.text
	if passphrase.is_empty():
		_ledger_status_label.text = "Passphrase cannot be empty. Click Unlock Ledger to try again."
		dialog.queue_free()
		return

	# FIXED (2026-07-27): derive_key() mints a fresh RANDOM salt on every
	# call (kupru::sargon_kdf::SargonKdf::new) -- deriving unconditionally
	# here, including for a ledger that already exists on disk, would
	# never match the key that encrypted it, and _load_ledger's decrypt
	# would fail for every correct passphrase, always (this ledger could
	# never actually be reopened across app restarts before this fix --
	# same bug as Sargon Passport Manager's own vault, caught live via
	# PB-261's round-trip check on DubSar's login vault). An existing
	# ledger's salt is read back from the file itself (prepended, not
	# secret) and re-derived via derive_key_with_salt; only a genuinely
	# new ledger mints a fresh salt.
	var opened_ok = true
	if FileAccess.file_exists(LEDGER_PATH):
		opened_ok = _load_ledger(passphrase)
	else:
		var derived = _bridge.derive_key(LEDGER_ROOT_PHRASE, passphrase)
		if not derived.get("ok", false):
			_ledger_status_label.text = "Key derivation failed: %s" % _explain_kupru_error(derived.get("error", "?")).summary
			dialog.queue_free()
			return
		_ledger_key = derived["key"]
		_ledger_salt = derived["salt"]
		_ledger_entries = []
		_ledger_status_label.text = "New ledger -- will be saved once an entry is recorded."

	dialog.queue_free()
	if not opened_ok:
		# Wrong passphrase or corrupt file -- do NOT mark unlocked, and do
		# NOT touch any pending entries. Click Unlock Ledger to retry;
		# nothing is lost either way.
		return

	_ledger_unlocked = true
	if not _pending_ledger_entries.is_empty():
		for e in _pending_ledger_entries:
			_ledger_entries.append(e)
		_pending_ledger_entries.clear()
		_save_ledger()
	_refresh_ledger_list()

func _load_ledger(passphrase: String) -> bool:
	var f = FileAccess.open(LEDGER_PATH, FileAccess.READ)
	if f == null:
		_ledger_status_label.text = "Could not open ledger file."
		return false
	var salt: PackedByteArray = f.get_buffer(32)
	var sealed: PackedByteArray = f.get_buffer(f.get_length() - 32)
	f.close()

	var derived = _bridge.derive_key_with_salt(LEDGER_ROOT_PHRASE, passphrase, salt)
	if not derived.get("ok", false):
		_ledger_status_label.text = "Key derivation failed: %s" % _explain_kupru_error(derived.get("error", "?")).summary
		return false
	_ledger_key = derived["key"]
	_ledger_salt = salt

	var result = _bridge.decrypt_vault_blob(_ledger_key, sealed)
	if not result.get("ok", false):
		_ledger_status_label.text = "WRONG PASSPHRASE, or the ledger file is corrupted (decrypt failed)."
		return false

	var plaintext_bytes: PackedByteArray = result["plaintext"]
	var json_text = plaintext_bytes.get_string_from_utf8()
	var parsed = JSON.parse_string(json_text)
	_ledger_entries = parsed if parsed is Array else []
	_ledger_status_label.text = "Ledger unlocked -- %d entr%s." % [
		_ledger_entries.size(), "y" if _ledger_entries.size() == 1 else "ies"
	]
	return true

## File layout: 32 raw salt bytes, then the ChaCha20-Poly1305 sealed blob
## (nonce+ciphertext+tag). The salt is not secret (kupru's own doc
## comment says so) -- it MUST be stored plainly so a later session's
## unlock can re-derive the identical key.
func _save_ledger() -> void:
	var json_text = JSON.stringify(_ledger_entries)
	var plaintext = json_text.to_utf8_buffer()
	var result = _bridge.encrypt_vault_blob(_ledger_key, plaintext)
	if not result.get("ok", false):
		_ledger_status_label.text = "Ledger encrypt failed: %s" % _explain_kupru_error(result.get("error", "?")).summary
		return
	var f = FileAccess.open(LEDGER_PATH, FileAccess.WRITE)
	f.store_buffer(_ledger_salt)
	f.store_buffer(result["sealed"])
	f.close()
	_ledger_status_label.text = "Ledger saved -- %d entr%s." % [
		_ledger_entries.size(), "y" if _ledger_entries.size() == 1 else "ies"
	]

func _refresh_ledger_list() -> void:
	_ledger_list.clear()
	# Newest first -- the most recently minted client credential is the
	# one you're most likely checking on.
	# FIXED 2026-08-02, live: dates were already shown here, but with no
	# computed status -- a real, time-sensitive gap given passports expire
	# in ~54h (SATTATU_MAX). This tab already IS the "show existing
	# passports history" view the Architect asked for (a matching toggle
	# button was added to Sargon's vault list instead, since Sargon has no
	# separate history tab of its own) -- just missing the status word.
	var now := Time.get_unix_time_from_system()
	for i in range(_ledger_entries.size() - 1, -1, -1):
		var entry = _ledger_entries[i]
		var expires_at: int = int(entry.get("expires_at", 0))
		var minted = Time.get_datetime_string_from_unix_time(int(entry.get("minted_at", 0)), true)
		var expires = Time.get_datetime_string_from_unix_time(expires_at, true)
		var status: String
		if now >= expires_at:
			status = "EXPIRED"
		elif expires_at - now <= (expires_at - int(entry.get("minted_at", 0))) * 0.2:
			# Same 80%-elapsed threshold SargonPassport::renewal_due() uses,
			# recomputed here since the Ledger stores only the summary
			# fields, not the full passport to call renewal_due() on.
			status = "Renew soon"
		else:
			status = "Valid"
		var text := "%s — %s (%s) — level %d — minted %s — expires %s — %s" % [
			entry.get("client_label", "?"), entry.get("realm", "?"), entry.get("subject_label", "?"),
			entry.get("privilege_level", 0), minted, expires, status
		]
		_ledger_list.add_item(text)

func _on_export_ledger_pressed() -> void:
	if _ledger_entries.is_empty():
		_ledger_status_label.text = "Nothing to export yet."
		return
	var dialog = FileDialog.new()
	dialog.file_mode = FileDialog.FILE_MODE_SAVE_FILE
	dialog.access = FileDialog.ACCESS_FILESYSTEM
	dialog.add_filter("*.json", "Architect Ledger JSON")
	# FIXED 2026-08-02, live: this dialog had NO suggested filename at all
	# (unlike the Passport/Share exports below, which deliberately do),
	# so a user naturally typed something like "Gilgamesh_MasterKy.json"
	# -- which reads as if it IS the master key/passport, then fails
	# DubSar's import with a "missing field `created_at`" error, since a
	# ledger entry has none of a real Passport's quppu/kupru/naru/istar
	# structure. This default makes it unmistakably a log, not a credential.
	dialog.current_file = "gilgamesh-ledger-record-%s.json" % Time.get_date_string_from_system()
	add_child(dialog)
	dialog.file_selected.connect(func(path):
		var f = FileAccess.open(path, FileAccess.WRITE)
		f.store_string(JSON.stringify(_ledger_entries, "\t"))
		f.close()
		_ledger_status_label.text = "Exported to %s -- this is your own record log, NOT a Passport. It will not work if imported into DubSar's login." % path
		dialog.queue_free()
	)
	dialog.canceled.connect(func(): dialog.queue_free())
	dialog.popup_centered(Vector2i(600, 400))
