# NAMTAR - Active threat sites query
#
# NOTE: `EAV.eav_state IN [...]` is not supported by the current HeptaScript
# grammar (Op is Eq/Neq/Gt/Lt/Gte/Lte only, no IN). Using an OR-chained
# equality condition instead, which the current parser does support.

WHO T.E
WHERE E[eav_state] = "FUZZY" OR E[eav_state] = "DEAD"
HOW E[phi] ASC
LIMIT 100
