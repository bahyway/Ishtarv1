# HeptaScript PRESENT - Cross-region ASHNAN cascade query
# Fertile Crescent (IQ + SY + JO) CMI/IRI/LMI consolidated view
# Anti-SQL: no SELECT/FROM/WHERE keyword-soup - sovereign W5H2 structure
#
# NOTE: the original playbook draft for this query used `tribe_id IN [...]`
# and `HOW: grouped by tribe_id` - neither exists in the current HeptaScript
# grammar (heptascript::query::Op is Eq/Neq/Gt/Lt/Gte/Lte only, no IN;
# HowClause supports single-attribute ordering only, no GROUP BY). Keeping
# this as the aspirational reference version below, plus a syntactically
# valid companion query using what the grammar actually supports today
# (OR-chained equality conditions, single-attribute HOW ordering).

# --- Aspirational (not yet parseable - needs IN + GROUP BY grammar support) ---
# PRESENT
#   WHO:   tribe_id IN [
#            "ashnan_diyala",
#            "ashnan_jordan_valley",
#            "ashnan_syria_euphrates",
#            "ashnan_fertile_crescent"
#          ]
#   WHAT:  EAV.cmi, EAV.iri, EAV.lmi, EAV.eav_state
#   WHEN:  last_7_days
#   WHERE: EAV.eav_state IN ["FUZZY", "DEAD"]
#   HOW:   grouped by tribe_id, ordered by EAV.cmi DESC
#   LIMIT: 500

# --- Valid today (per-tribe, OR-chained state filter, single-attr ordering) ---
WHO T.E
WHERE E[eav_state] = "FUZZY" OR E[eav_state] = "DEAD"
HOW E[cmi] DESC
LIMIT 500
