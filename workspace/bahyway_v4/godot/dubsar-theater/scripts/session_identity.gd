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
# HONEST LIMIT (narrowed 2026-08-21): this is a session-scoped RECORD of
# what was verified at login time. theater_controller.gd's
# open_enkidb_wizard() now reads privilege_level to gate the EnkiDB
# Connector Wizard specifically (Architect-only) -- the first real
# feature-level gate, not just the login formality this comment used to
# describe. Every OTHER panel (HeptaScript editor, Grid & Orbit, Graph
# Explorer, Dashboards, Document Explorer) still reads nothing here --
# gating those, if ever wanted, is real, separate future work.
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
