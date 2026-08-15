class_name AkkadiCompiler
extends RefCounted
# =============================================================================
# Minimal, REAL, working .akk -> HeptaScript compiler (2026-07-10).
#
# HONEST SCOPE: this is NOT the full "AkkadianAOL v4.0 compiler" (NABU)
# referenced in earlier ecosystem docs -- NABU has no real crate or
# scaffold anywhere in the repo (crates/bahyway-core/src/akkadian.rs
# declares the name only; confirmed no compiler logic exists). This is a
# new, small, working compiler for a minimal PDM-model subset of .akk
# syntax, emitting HeptaScript text confirmed grammar-correct against
# crates/heptascript/src/parser.rs's own tests:
#   NODE db_target (| db_target)*      (optional)
#   WHO TribeName.Var                  (mandatory)
#   WHAT Var[attr1, attr2, ...]        (derived from ATTR lines)
#   HOW_MUCH LIMIT n                   (optional)
#
# .akk source grammar (this compiler's own minimal PDM-model syntax):
#   MODEL <name>
#   NODE <DbTarget>[ | <DbTarget> ...]   # optional, e.g. "NODE EnkiSDB | EnkiDB"
#   WHO <TribeName>.<Var>                # mandatory
#   ATTR <attr.path>                     # zero or more
#   LIMIT <n>                            # optional
#
# Blank lines and lines starting with # are ignored. Canonicalizing this
# as a real Rust module (crates/akkadi) is a deliberate fast-follow, not
# a blocker -- this GDScript version is what the IDE actually runs today
# since no GDExtension bridge exists yet.
# =============================================================================

class CompileResult:
	var ok: bool = true
	var heptascript: String = ""
	var model_name: String = ""
	var attrs: Array = []
	var errors: Array = []

static func compile(akk_source: String) -> CompileResult:
	var result = CompileResult.new()

	var node_clause = ""
	var who_clause = ""
	var who_var = ""
	var limit_clause = ""

	for raw_line in akk_source.split("\n"):
		var line = raw_line.strip_edges()
		if line.is_empty() or line.begins_with("#"):
			continue

		if line.begins_with("MODEL "):
			result.model_name = line.substr(6).strip_edges()
		elif line.begins_with("NODE "):
			node_clause = "NODE " + line.substr(5).strip_edges()
		elif line.begins_with("WHO "):
			who_clause = line
			var rhs = line.substr(4).strip_edges()
			if not rhs.contains("."):
				result.errors.append("WHO clause must be TribeName.Var, got: '%s'" % rhs)
				result.ok = false
			else:
				var parts = rhs.split(".")
				who_var = parts[parts.size() - 1]
		elif line.begins_with("ATTR "):
			result.attrs.append(line.substr(5).strip_edges())
		elif line.begins_with("LIMIT "):
			var n = line.substr(6).strip_edges()
			if not n.is_valid_int():
				result.errors.append("LIMIT must be an integer, got: '%s'" % n)
				result.ok = false
			else:
				limit_clause = "HOW_MUCH LIMIT " + n
		else:
			result.errors.append("Unrecognized .akk line: '%s'" % line)
			result.ok = false

	if who_clause.is_empty():
		result.errors.append("WHO clause is mandatory (WHO TribeName.Var) -- every real HeptaScript query requires it.")
		result.ok = false

	if not result.ok:
		return result

	var lines: Array = []
	if not node_clause.is_empty():
		lines.append(node_clause)
	lines.append(who_clause)
	if not result.attrs.is_empty():
		var attr_str = ""
		for i in range(result.attrs.size()):
			if i > 0:
				attr_str += ", "
			attr_str += result.attrs[i]
		lines.append("WHAT %s[%s]" % [who_var, attr_str])
	if not limit_clause.is_empty():
		lines.append(limit_clause)

	var text = ""
	for i in range(lines.size()):
		if i > 0:
			text += "\n"
		text += lines[i]
	result.heptascript = text
	return result
