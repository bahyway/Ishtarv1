class_name AkkadiSafeBridge
# =============================================================================
# WIZ-001 — Bridge to AkkadiSafeEngine (sovereign credential vault,
# Argon2id KDF) and AkkadianSeal (Ed25519 profile sealing).
#
# LAW: the wizard NEVER implements its own cryptography. The previous
# XOR-with-machine-ID scheme is abolished. If the vault bridge is not yet
# wired, secrets are simply NOT persisted — a missing feature is lawful,
# a homebrew cipher is not.
#
# TODO(v4.0 wiring): connect to the real crates/kupru (AkkadianCipher /
# SargonKdf / AkkadianSeal) via GDExtension or local socket. Until then
# all calls return UNAVAILABLE.
# =============================================================================

enum Result { OK, UNAVAILABLE, DENIED }

static func vault_available() -> bool:
	return false  # flips true once the GDExtension/socket bridge to kupru lands

static func store_secret(_profile_key: String, _passphrase: String) -> Result:
	if not vault_available():
		return Result.UNAVAILABLE
	return Result.OK

static func seal_profile(_profile: Dictionary) -> String:
	# Returns an Ed25519 AkkadianSeal over the profile when wired.
	if not vault_available():
		return "UNSEALED-DEV"  # explicit, greppable, honest
	return ""
