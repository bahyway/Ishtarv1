extends SceneTree
# ISIMUD headless bootstrap mint — 2026-07-27, the Architect's own request.
#
# Mints a low-privilege, short-lived Sargon ("gardener") Passport with a
# fixed bootstrap identity and imports it straight into DubSar's own login
# vault (user://dubsar_login_vault.dat) — with ZERO human clicking through
# a Godot GUI window. Exists so IsimudEngine's first-time-run sweep can
# smoke-test the mint -> import -> decrypt -> verify_passport() round trip
# for real, end to end, unattended.
#
# WHAT THIS DELIBERATELY DOES NOT DO: mint a Gilgamesh ("architect",
# privilege 7) Passport. That ceremony requires a human-typed identity
# phrase and (for a from-scratch key) a real Shamir M-of-N share split --
# by design, per docs/15_howto/generate_and_use_a_sovereign_passport.md.
# This script only ever calls issue_gardener_passport (privilege 1), the
# same call Sargon Passport Manager's own GUI uses for its lowest tier.
# There is no flag, no override, no "just this once" path to architect
# level here on purpose.
#
# This is NOT a secrets-management tool. The bootstrap identity and
# passphrase below are fixed and intentionally not secret -- this mints
# a disposable, 54-hour smoke-test credential, the same SATTATU_MAX
# expiry every other Passport in this ecosystem gets. Nothing it unlocks
# is meant to hold real privilege; re-run this any time it expires, same
# as any human would re-mint a Sargon Passport by hand.
#
# USAGE (must run INSIDE the dubsar-theater project so `user://` resolves
# to DubSar's own vault, and so KupruBridge is the copy already wired in
# by PB-226/PB-261's own build+copy step):
#   godot --headless --path workspace/bahyway_v4/godot/dubsar-theater \
#     --script res://scripts/isimud_bootstrap_mint.gd

const BOOTSTRAP_IDENTITY := "isimud-bootstrap"
const BOOTSTRAP_REALM := "IsimudEngine"
const BOOTSTRAP_SUBJECT := "first-time-run smoke test"
const BOOTSTRAP_PASSPORT_TYPE := "Sargon Passport"
const BOOTSTRAP_PASSPHRASE := "isimud-bootstrap-smoke-test"
const LOGIN_VAULT_PATH := "user://dubsar_login_vault.dat"
const EXPORT_PATH := "/tmp/isimud_bootstrap_passport.json"

# Identical to login.gd's own _login_key_root() -- not reimplemented from
# scratch, copied verbatim so the derived key is byte-for-byte the same
# key login.gd itself would compute for this identity+type at login time.
func _login_key_root(identity: String, passport_type: String) -> String:
	return "dubsar-login-%s-%s" % [identity.to_lower(), passport_type.to_lower().replace(" ", "-")]

func _fail(step: String, err: String) -> void:
	printerr("ISIMUD-BOOTSTRAP FAIL [%s]: %s" % [step, err])
	quit(1)

func _initialize() -> void:
	var bridge = KupruBridge.new()

	# ── 1. Mint a fresh gardener (privilege 1) Passport ────────────────
	var kp = bridge.generate_keypair()
	if not kp.get("ok", false):
		_fail("generate_keypair", kp.get("error", "?"))
		return

	var minted = bridge.issue_gardener_passport(
		kp["signing_key"], BOOTSTRAP_IDENTITY, BOOTSTRAP_REALM, BOOTSTRAP_SUBJECT
	)
	if not minted.get("ok", false):
		_fail("issue_gardener_passport", minted.get("error", "?"))
		return
	var passport_json: String = minted["passport_json"]

	var ef = FileAccess.open(EXPORT_PATH, FileAccess.WRITE)
	ef.store_string(passport_json)
	ef.close()

	# ── 2. Encrypt + write it into DubSar's own vault (login.gd's exact
	#      derive_key -> encrypt_vault_blob -> vault-record shape) ─────
	var derived = bridge.derive_key(_login_key_root(BOOTSTRAP_IDENTITY, BOOTSTRAP_PASSPORT_TYPE), BOOTSTRAP_PASSPHRASE)
	if not derived.get("ok", false):
		_fail("derive_key", derived.get("error", "?"))
		return

	var sealed = bridge.encrypt_vault_blob(derived["key"], passport_json.to_utf8_buffer())
	if not sealed.get("ok", false):
		_fail("encrypt_vault_blob", sealed.get("error", "?"))
		return

	var vault: Array = []
	if FileAccess.file_exists(LOGIN_VAULT_PATH):
		var rf = FileAccess.open(LOGIN_VAULT_PATH, FileAccess.READ)
		var parsed = JSON.parse_string(rf.get_as_text())
		rf.close()
		if parsed is Array:
			vault = parsed

	var new_record = {
		"identity": BOOTSTRAP_IDENTITY,
		"passport_type": BOOTSTRAP_PASSPORT_TYPE,
		"sealed_b64": Marshalls.raw_to_base64(sealed["sealed"]),
		# FIXED (2026-07-27): derive_key() mints a fresh RANDOM salt every
		# call (kupru::sargon_kdf::SargonKdf::new) -- this MUST be
		# persisted alongside the ciphertext, or no later call can ever
		# re-derive the same key. This exact gap is what the round-trip
		# check below caught: it was the first thing to actually call
		# derive_key a second time and discover login.gd's real login
		# flow had the same bug, unfixed, before this script existed.
		"salt_b64": Marshalls.raw_to_base64(derived["salt"]),
	}
	var replaced := false
	for i in range(vault.size()):
		if vault[i].get("identity") == BOOTSTRAP_IDENTITY and vault[i].get("passport_type") == BOOTSTRAP_PASSPORT_TYPE:
			vault[i] = new_record
			replaced = true
			break
	if not replaced:
		vault.append(new_record)

	var wf = FileAccess.open(LOGIN_VAULT_PATH, FileAccess.WRITE)
	wf.store_string(JSON.stringify(vault))
	wf.close()

	# ── 3. Round-trip verification -- read the vault back, decrypt, and
	#      run the SAME verify_passport() check _on_login_pressed does,
	#      so this proves the whole mint->import->decrypt->verify chain
	#      actually works, not just that files got written. Uses the
	#      SAME salt via derive_key_with_salt, exactly as the (now also
	#      fixed) _on_login_pressed does. ───────────────────────────────
	var check_salt: PackedByteArray = Marshalls.base64_to_raw(new_record["salt_b64"])
	var check_derived = bridge.derive_key_with_salt(_login_key_root(BOOTSTRAP_IDENTITY, BOOTSTRAP_PASSPORT_TYPE), BOOTSTRAP_PASSPHRASE, check_salt)
	if not check_derived.get("ok", false):
		_fail("roundtrip derive_key_with_salt", check_derived.get("error", "?"))
		return
	var resealed: PackedByteArray = Marshalls.base64_to_raw(new_record["sealed_b64"])
	var decrypted = bridge.decrypt_vault_blob(check_derived["key"], resealed)
	if not decrypted.get("ok", false):
		_fail("roundtrip decrypt_vault_blob", decrypted.get("error", "?"))
		return
	var roundtrip_json: String = decrypted["plaintext"].get_string_from_utf8()
	var verified = bridge.verify_passport(roundtrip_json)
	if not verified.get("ok", false) or not verified.get("valid", false):
		_fail("roundtrip verify_passport", verified.get("error", "seal check failed"))
		return

	print("ISIMUD-BOOTSTRAP OK: minted + imported + round-trip-verified a gardener Passport.")
	print("  identity=\"%s\"  type=\"%s\"  passphrase=\"%s\"" % [BOOTSTRAP_IDENTITY, BOOTSTRAP_PASSPORT_TYPE, BOOTSTRAP_PASSPHRASE])
	print("  Exported passport JSON: %s (54h TTL -- re-run this script once it expires)." % EXPORT_PATH)
	print("  DubSar login gate will now accept the identity/type/passphrase above with zero further setup.")
	quit(0)
