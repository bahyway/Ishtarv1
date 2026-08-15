import json
from itertools import combinations

# ============================================================
# REAL formal context: microbiology cross-table.
# These are genuine textbook microbiological properties.
# (Static, transparently — this sandbox cannot reach medical APIs.)
# TB is deliberately left gram-unclassified (acid_fast) — a real fact
# that will produce an honest, slightly awkward concept. We keep it.
# ============================================================
OBJECTS = ["E.coli","Salmonella","Pseudomonas","Klebsiella","Neisseria",
           "Haemophilus","M.tuberculosis","S.aureus","S.pneumoniae","C.difficile"]
ATTRS = ["gram_pos","gram_neg","rod","coccus","aerobe","anaerobe","facultative",
         "motile","spore","catalase","oxidase","lactose_ferm","acid_fast","encapsulated"]

# incidence[obj] = set of attributes that hold (real values)
I = {
 "E.coli":       {"gram_neg","rod","facultative","motile","catalase","lactose_ferm"},
 "Salmonella":   {"gram_neg","rod","facultative","motile","catalase","encapsulated"},
 "Pseudomonas":  {"gram_neg","rod","aerobe","motile","catalase","oxidase","encapsulated"},
 "Klebsiella":   {"gram_neg","rod","facultative","catalase","lactose_ferm","encapsulated"},
 "Neisseria":    {"gram_neg","coccus","aerobe","catalase","oxidase","encapsulated"},
 "Haemophilus":  {"gram_neg","rod","facultative","encapsulated"},
 "M.tuberculosis":{"rod","aerobe","catalase","acid_fast"},           # acid-fast, NOT gram-classified
 "S.aureus":     {"gram_pos","coccus","facultative","catalase"},
 "S.pneumoniae": {"gram_pos","coccus","facultative","encapsulated"}, # catalase-negative
 "C.difficile":  {"gram_pos","rod","anaerobe","motile","spore"},     # catalase-negative
}

Oset=set(OBJECTS)
def extent(B): return frozenset(o for o in OBJECTS if set(B) <= I[o])
def intent(A):
    if not A: return frozenset(ATTRS)
    s=set(ATTRS)
    for o in A: s &= I[o]
    return frozenset(s)

# ---- compute ALL formal concepts (intents = closure of intersections of object-intents) ----
obj_intents={frozenset(I[o]) for o in OBJECTS}
all_intents=set(obj_intents); all_intents.add(frozenset(ATTRS))
changed=True
while changed:
    changed=False
    for a,b in combinations(list(all_intents),2):
        inter=a&b
        if inter not in all_intents:
            all_intents.add(inter); changed=True
# bottom intent (common to all) too:
all_intents.add(intent(Oset))

concepts=[]
seen=set()
for B in all_intents:
    A=extent(B); Bc=intent(A)   # close
    key=(A,Bc)
    if key not in seen:
        seen.add(key); concepts.append((A,Bc))

# order & covering (Hasse): c1 < c2  iff extent(c1) subset extent(c2)
def leq(c1,c2): return c1[0] <= c2[0]
n=len(concepts)
# covering relation
covers=[]
for i in range(n):
    for j in range(n):
        if i==j: continue
        if concepts[i][0] < concepts[j][0]:
            # j covers i if no k strictly between
            between=False
            for k in range(n):
                if k in (i,j): continue
                if concepts[i][0] < concepts[k][0] < concepts[j][0]:
                    between=True; break
            if not between: covers.append((i,j))

# ---- attribute implications (real): a -> closure({a}) minus a ----
implications=[]
for a in ATTRS:
    cl=intent(extent({a}))
    if extent({a}):  # attribute actually occurs
        consequent=set(cl)-{a}
        if consequent:
            implications.append((a, sorted(consequent)))

# ---- SYMBOLIC-ICON resolution RULES (each attribute/concept resolves to a symbol by rule) ----
def attr_symbol(a):
    e=extent({a}); k=len(e)
    if k==len(OBJECTS): return ("TOP","⊤","universal: holds for every object")
    if k==1: return ("DISC","◆","discriminator: unique to one object")
    if k==0: return ("VOID","∅","never occurs")
    return ("SHARED","∩","shared by %d objects (overlap)"%k)

def concept_symbol(A,B):
    k=len(A)
    if k==len(OBJECTS): return ("ROOT","⊤","root: extent = all objects")
    if k==0: return ("BOTTOM","⊥","bottom: no object has all these attributes")
    if k==1: return ("AGENT","⬡","agent/object concept: a single organism")
    return ("COMPOUND","⧉","compound: meet of %d organisms (a real shared concept)"%k)

attr_syms={a:attr_symbol(a) for a in ATTRS}
concept_out=[]
for idx,(A,B) in enumerate(concepts):
    kind,sym,why=concept_symbol(A,B)
    concept_out.append({
        "id":idx,
        "extent":sorted(A),
        "intent":sorted(B),
        "size":len(A),
        "symbol":sym,"kind":kind,"why":why
    })

# ---- DERIVE the M2 metamodel FROM the data (not hardcoded) ----
metamodel={
  "ObjectType":{"count":len(OBJECTS),"instances":OBJECTS},
  "AttributeType":{"count":len(ATTRS),
     "roles":{a:attr_syms[a][0] for a in ATTRS}},
  "ConceptType":{"count":len(concepts),
     "by_symbol":{}},
  "OrderRelation":{"covering_edges":len(covers)},
  "ImplicationRules":{"count":len(implications)}
}
from collections import Counter
cc=Counter(c["kind"] for c in concept_out)
metamodel["ConceptType"]["by_symbol"]=dict(cc)

out={
  "source":"static real microbiology cross-table (10 bacteria x 14 properties). NOT a live API — sandbox cannot reach medical APIs. Everything below is COMPUTED from this table.",
  "objects":OBJECTS,"attributes":ATTRS,
  "incidence":{o:sorted(I[o]) for o in OBJECTS},
  "attr_symbols":{a:{"symbol":attr_syms[a][1],"kind":attr_syms[a][0],"why":attr_syms[a][2],
                     "extent":sorted(extent({a}))} for a in ATTRS},
  "concepts":concept_out,
  "covers":covers,
  "implications":[{"if":a,"then":c} for a,c in implications],
  "metamodel":metamodel,
}
print(json.dumps(out,indent=1))
print("\n---SUMMARY---")
print("objects:",len(OBJECTS)," attributes:",len(ATTRS)," concepts:",len(concepts)," covering-edges:",len(covers)," implications:",len(implications))
print("concept symbol distribution:",dict(cc))
