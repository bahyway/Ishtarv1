class_name SessionIdentity
# =============================================================================
# SessionIdentity -- BahyWay.Ecosystem v4.0
#
# Holds the passport that actually got the current session past
# login.gd's real verify_passport() check. Plain static-var class (no
# autoload registration needed) so any scene can read
# SessionIdentity.privilege_level etc. after login without needing to be
# handed a reference.
#
# HONEST LIMIT: this is a session-scoped RECORD of what was verified at
# login time -- nothing in DubSar PDM yet READS privilege_level to
# gate or restrict any feature. That's real, separate future work; this
# only stops the login gate itself from being a pure formality.
# =============================================================================

static var valid: bool = false
static var identity: String = ""
static var passport_type: String = ""
static var realm: String = ""
static var privilege_level: int = 0
static var passport_id: String = ""
static var expires_at: int = 0

static func set_from_verified(p_identity: String, p_passport_type: String, summary: Dictionary) -> void:
	valid = true
	identity = p_identity
	passport_type = p_passport_type
	realm = summary.get("realm", "?")
	privilege_level = summary.get("privilege_level", 0)
	passport_id = summary.get("passport_id", "?")
	expires_at = summary.get("expires_at", 0)
