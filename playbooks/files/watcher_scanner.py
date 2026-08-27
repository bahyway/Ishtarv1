#!/usr/bin/env python3
"""
Watcher_Scanner · GL-BND-001
Scores a stakeholder's templates against real orbits and proposes a band.

It awards nothing. It computes five measures, each from an instrument, attaches
the citation for each, subtracts the refusals, reads the band off a sealed ladder,
and emits a decree a steward signs.

  §1 five measures, weighted  · §2 a duplicate scores zero
  §3 the median, never the sum · §4 the ladder, black needs another's adoption
  §5 a band is worn, not owned · §6 the scanner proposes; a steward signs
"""
import argparse, hashlib, json, sys, time
from pathlib import Path

VERSION = "1.0.0"
LAW = "GL-BND-001"

WEIGHTS = {"fidelity": 0.30, "discrimination": 0.25, "coverage": 0.15,
           "honesty": 0.20, "originality": 0.10}          # sealed; the scanner may not change these
REFUSAL_PENALTY = 0.05
DAN_KANJI = ["初段","弐段","参段","四段","五段","六段","七段","八段","九段","十段","十一段","十二段"]
LADDER = [  # (band, kanji, median, extra requirement key)
    ("BLACK",  "黒帯", 0.88, "adopted_elsewhere_and_no_refusal"),
    ("BROWN",  "茶帯", 0.80, "survived_second_epoch"),
    ("BLUE",   "青帯", 0.72, "at_least_three"),
    ("GREEN",  "緑帯", 0.65, "no_duplicate"),
    ("ORANGE", "橙帯", 0.55, None),
    ("YELLOW", "黄帯", 0.45, None),
    ("WHITE",  "白帯", 0.00, None),
]
COSMETIC = {"name", "author", "engine", "colour", "color", "size", "label",
            "labels", "speed", "camera", "tone", "alpha", "font", "caption",
            "created", "version"}

def canon(o):
    if isinstance(o, dict):
        return {k: canon(v) for k, v in sorted(o.items()) if k not in COSMETIC}
    if isinstance(o, list):
        return sorted((canon(v) for v in o), key=lambda x: json.dumps(x, sort_keys=True))
    if isinstance(o, float):
        return round(o, 6)
    return o

def digest(c):
    return hashlib.blake2b(json.dumps(c, sort_keys=True, separators=(",", ":")).encode(),
                           digest_size=32).hexdigest()

def cite(work, name, obj):
    p = Path(work) / "artifacts" / f"{name}.json"
    p.parent.mkdir(parents=True, exist_ok=True)
    blob = json.dumps(obj, sort_keys=True, separators=(",", ":")).encode()
    p.write_bytes(blob)
    return {"artifact": p.name, "sha256": hashlib.sha256(blob).hexdigest()}

# ---------------------------------------------------------------- measures
def m_fidelity(tpl, orbits, work):
    """Can the counts be recovered from what it renders? Instrument: bahyway-enlil replay."""
    scores, worst = [], None
    for o in orbits:
        # the instrument returns the reconstruction error of the rendered field
        err = o["replay_error"].get(tpl["cid"], o["replay_error"].get("default", 0.5))
        s = max(0.0, 1.0 - err)
        scores.append(s)
        if worst is None or s < worst[1]: worst = (o["id"], s)
    v = sum(scores) / max(1, len(scores))
    return v, cite(work, f"fidelity_{tpl['cid'][:12]}", {"per_orbit": scores, "worst": worst}), \
        (f"weakest on {worst[0]}" if worst else "no orbit")

def m_discrimination(tpl, orbits, work):
    """Shown a healthy orbit and a decaying one, does it look different?"""
    pairs = [(a, b) for a in orbits if a["label"] == "healthy" for b in orbits if b["label"] != "healthy"]
    seps = []
    for a, b in pairs:
        va = a["signature"].get(tpl["mode"], 0.0)
        vb = b["signature"].get(tpl["mode"], 0.0)
        seps.append(min(1.0, abs(va - vb) * 2.0))
    v = sum(seps) / max(1, len(seps))
    return v, cite(work, f"discrim_{tpl['cid'][:12]}", {"pairs": len(pairs), "separations": seps}), \
        (f"{len(pairs)} labelled pair(s)" if pairs else "no labelled pair — cannot discriminate")

def m_coverage(tpl, orbits, work):
    """On what fraction of real orbits are its units and strata defined?"""
    need_u, need_s = set(tpl["units"]), set(tpl["strata"])
    ok = [o["id"] for o in orbits
          if need_u <= set(o["units"]) and need_s <= set(o["strata"])]
    v = len(ok) / max(1, len(orbits))
    return v, cite(work, f"coverage_{tpl['cid'][:12]}", {"defined_on": ok, "of": len(orbits)}), \
        f"defined on {len(ok)} of {len(orbits)} orbits"

def m_honesty(tpl, orbits, work):
    """Does it render confidence in the fill and draw the unknown?"""
    has = tpl.get("grammar", {})
    checks = {"fill_states_class": bool(has.get("fill_by_class")),
              "draws_unknown": bool(has.get("draws_unknown")),
              "projections_outlined": bool(has.get("projections_outlined")),
              "no_decimation": bool(has.get("no_decimation"))}
    v = sum(1 for x in checks.values() if x) / len(checks)
    missing = [k for k, x in checks.items() if not x]
    return v, cite(work, f"honesty_{tpl['cid'][:12]}", checks), \
        ("complete" if not missing else "missing: " + ", ".join(missing))

def m_originality(tpl, registry, work):
    """Distance from the nearest existing template."""
    def sim(a, b):
        sc, tot = 0.0, 1.0
        sc += 1.0 if a["mode"] == b["mode"] else 0.0
        for k in ("units", "strata"):
            tot += 1
            A, B = set(a[k]), set(b[k])
            sc += (len(A & B) / len(A | B)) if (A | B) else 1.0
        return sc / tot
    near = [(sim(tpl, r), r["name"], r["author"]) for r in registry if r["cid"] != tpl["cid"]]
    near.sort(reverse=True)
    top = near[0] if near else (0.0, "—", "—")
    v = max(0.0, 1.0 - top[0])
    return v, cite(work, f"orig_{tpl['cid'][:12]}", {"nearest": top[1], "similarity": round(top[0], 3)}), \
        f"nearest is {top[1]} by {top[2]} at {top[0]:.2f}"

# ---------------------------------------------------------------- scoring
def score_template(tpl, orbits, registry, work):
    exact = next((r for r in registry if r["cid"] == tpl["cid"] and r["author"] != tpl["author"]), None) \
         or next((r for r in registry if r["cid"] == tpl["cid"] and r.get("minted_before")), None)
    m, notes, cites = {}, {}, {}
    for key, fn in (("fidelity", m_fidelity), ("discrimination", m_discrimination),
                    ("coverage", m_coverage), ("honesty", m_honesty)):
        v, c, note = fn(tpl, orbits, work)
        m[key], cites[key], notes[key] = round(v, 4), c, note
    v, c, note = m_originality(tpl, registry, work)
    m["originality"], cites["originality"], notes["originality"] = round(v, 4), c, note

    if exact:
        total = 0.0
        notes["duplicate"] = (f"exact duplicate of {exact['name']} by {exact['author']} — "
                              f"scores 0.00 whatever the other measures say (GL-BND-001 §2)")
    else:
        total = round(sum(m[k] * w for k, w in WEIGHTS.items()), 4)

    use = tpl.get("use", {"held": 0, "led_to_decree": 0})
    return {"name": tpl["name"], "cid": tpl["cid"][:16], "author": tpl["author"],
            "measures": m, "notes": notes, "cites": cites,
            "score": total, "refused_duplicate": bool(exact),
            "USE_not_counted": use,
            "adopted_by_other_engine": bool(tpl.get("adopted_by_distinct", tpl.get("adopted_by", []))),
            "adopted_by_distinct": list(tpl.get("adopted_by_distinct", [])),
            "epochs": int(tpl.get("epochs", 1)),
            "survived_second_epoch": bool(tpl.get("epochs", 1) >= 2),
            "weakest_measure": min(m, key=m.get)}

def dan_of(scored):
    """§9 · each dan asks the same thing in a larger form: someone else chose to use it, again.
       The engines must be DISTINCT, and an engine the stakeholder maintains does not count."""
    engines, surviving = set(), 0
    for t in scored:
        for e in t.get("adopted_by_distinct", []):
            engines.add(e)
        if t.get("epochs", 1) >= 2:
            surviving += 1
    n_engines = len(engines)
    dan = 0
    while dan < 11 and n_engines >= (dan + 2) and surviving >= (dan + 2):
        dan += 1
    return dan, sorted(engines), surviving

def band_of(scored):
    """§3 the median, never the sum. §4 the ladder, with its extra conditions."""
    if not scored:
        return {"band": "—", "kanji": "", "median": 0.0, "why": "no template scored"}
    vals = sorted(s["score"] for s in scored)
    n = len(vals)
    median = vals[n // 2] if n % 2 else (vals[n // 2 - 1] + vals[n // 2]) / 2
    refusals = sum(1 for s in scored if s["refused_duplicate"])
    adjusted = round(median - REFUSAL_PENALTY * refusals, 4)
    facts = {"no_duplicate": refusals == 0,
             "at_least_three": n >= 3,
             "survived_second_epoch": any(s["survived_second_epoch"] for s in scored),
             "adopted_elsewhere_and_no_refusal": any(s["adopted_by_other_engine"] for s in scored) and refusals == 0}
    dan, engines, surviving = dan_of(scored)
    for band, kanji, need, extra in LADDER:
        if adjusted >= need and (extra is None or facts.get(extra, False)):
            blocked = None
            for b2, k2, n2, e2 in LADDER:
                if n2 > need and adjusted >= n2 and e2 and not facts.get(e2, False):
                    blocked = (b2, e2); break
            label, dan_out = band, 0
            if band == "BLACK" and dan > 0:
                label = f"BLACK+{dan}"
                kanji = DAN_KANJI[min(dan, len(DAN_KANJI) - 1)]
                dan_out = dan
                if dan >= 11:
                    label = "BLACK+11 · MASTER OF MASTERS"
            elif band == "BLACK":
                kanji = DAN_KANJI[0]
            return {"band": band, "label": label, "kanji": kanji, "dan": dan_out,
                    "median": median, "adjusted": adjusted,
                    "refusals": refusals, "templates": n, "facts": facts,
                    "distinct_engines": engines, "surviving": surviving,
                    "blocked_from": blocked,
                    "next_dan_needs": (f"{dan + 2} distinct engines and {dan + 2} templates through "
                                       f"{dan + 3} epochs — you have {len(engines)} and {surviving}")
                                      if band == "BLACK" else None,
                    "why": f"median {median:.3f} − {REFUSAL_PENALTY}×{refusals} refusal(s) = {adjusted:.3f}"}
    return {"band": "WHITE", "kanji": "白帯", "median": median, "adjusted": adjusted, "why": "below every threshold"}

def reward(band, pricing):
    """§10–§14 · the scanner reads the sealed price table; it never sets one."""
    base = pricing["discount_by_band"].get(band["band"], 0.0)
    disc = base + pricing["per_dan_above_black"] * band.get("dan", 0)
    capped = min(disc, pricing["ceiling"])
    lic, met = pricing["licence_eur_per_day"], pricing["metered_eur_per_day"]
    floor = pricing["marginal_cost_floor_eur_per_day"]
    lic_after = round(lic * (1 - capped), 2)          # §11 · the licence only
    total_after = round(lic_after + met, 2)
    floored = total_after < floor
    if floored:
        total_after = floor
        lic_after = round(floor - met, 2)
    return {"band": band.get("label", band["band"]), "kanji": band.get("kanji", ""),
            "discount_from_table": round(base, 4), "dan_bonus": round(pricing["per_dan_above_black"] * band.get("dan", 0), 4),
            "discount_applied": round(capped, 4),
            "capped_at_ceiling": disc > pricing["ceiling"],
            "licence_before": lic, "licence_after": lic_after,
            "metered_never_discounted": met,
            "total_per_day_before": round(lic + met, 2), "total_per_day_after": total_after,
            "floor_reached": floored, "marginal_cost_floor": floor,
            "applies_from": "the next billing period",
            "note": ("a discount already granted stands; a demotion is never back-billed (§12). "
                     "the discount follows the band currently held, never the sum of past awards (§10).")}

def next_step(scored, band):
    """§7 · named as a specific measure, never as encouragement."""
    if not scored: return "mint a template"
    dups = [s for s in scored if s["refused_duplicate"]]
    if dups:
        return (f"withdraw {len(dups)} duplicate claim(s) — each subtracts {REFUSAL_PENALTY} "
                f"and caps the band below GREEN")
    lowest = min(scored, key=lambda s: s["score"])
    w = lowest["weakest_measure"]
    return (f"the median rises when the middle rises: '{lowest['name']}' is weakest at "
            f"{w} = {lowest['measures'][w]:.2f} — {lowest['notes'][w]}")

# ---------------------------------------------------------------- main
def main():
    ap = argparse.ArgumentParser(description="Watcher_Scanner · GL-BND-001")
    ap.add_argument("--portfolio", required=True, help="the stakeholder's candidate templates (json)")
    ap.add_argument("--registry", required=True, help="templates already minted (json)")
    ap.add_argument("--orbits", required=True, help="real orbits to score against (json)")
    ap.add_argument("--pricing", default=None, help="the sealed price table (read-only)")
    ap.add_argument("--work", required=True)
    args = ap.parse_args()
    work = Path(args.work); work.mkdir(parents=True, exist_ok=True)

    port = json.loads(Path(args.portfolio).read_text())
    reg = json.loads(Path(args.registry).read_text())
    orbits = json.loads(Path(args.orbits).read_text())["orbits"]
    for t in port["templates"] + reg["templates"]:
        t["cid"] = digest(canon({"mode": t["mode"], "units": t["units"], "strata": t["strata"]}))
    own = {port.get("owns_engine"), port["stakeholder"]}
    for t in port["templates"]:
        # §9 · an engine the stakeholder owns or maintains does not count toward a dan
        t["adopted_by_distinct"] = [e for e in t.get("adopted_by", []) if e not in own]

    scored = [score_template(t, orbits, reg["templates"], work) for t in port["templates"]]
    band = band_of(scored)

    pricing = json.loads(Path(args.pricing).read_text()) if args.pricing else None
    rw = reward(band, pricing) if pricing else None

    decree = {"law": LAW, "scanner": VERSION, "act": "PROPOSE_BAND",
              "reward": rw,
              "stakeholder": port["stakeholder"], "band": band,
              "templates": scored, "next_step": next_step(scored, band),
              "weights": WEIGHTS, "note": "USE is reported and carries weight zero (§1)",
              "executed_by": "a steward signs; the scanner awards nothing (§6)"}
    blob = json.dumps(decree, sort_keys=True, separators=(",", ":")).encode()
    decree["receipt_sha256"] = hashlib.sha256(blob).hexdigest()
    (work / "band_decree.json").write_text(json.dumps(decree, indent=2, sort_keys=True))

    print(f"{port['stakeholder']} · {band.get('label', band['band'])} {band.get('kanji','')} · {band['why']}")
    if band.get("distinct_engines"):
        print(f"  adopted by {len(band['distinct_engines'])} distinct engine(s): {', '.join(band['distinct_engines'])}")
    for s in scored:
        flag = " · REFUSED DUPLICATE" if s["refused_duplicate"] else ""
        print(f"  {s['score']:.3f}  {s['name']:<22} weakest: {s['weakest_measure']}{flag}")
    if band.get("blocked_from"):
        print(f"  blocked from {band['blocked_from'][0]}: {band['blocked_from'][1]}")
    if band.get("next_dan_needs"):
        print(f"  next dan: {band['next_dan_needs']}")
    if rw:
        print(f"  reward · {rw['discount_applied']*100:.0f}% off the LICENCE only"
              + (" (ceiling reached)" if rw["capped_at_ceiling"] else ""))
        print(f"           licence {rw['licence_before']:.2f} → {rw['licence_after']:.2f} EUR/day · "
              f"metered {rw['metered_never_discounted']:.2f} never discounted")
        print(f"           total {rw['total_per_day_before']:.2f} → {rw['total_per_day_after']:.2f} EUR/day"
              + (" · FLOOR REACHED" if rw["floor_reached"] else ""))
        print(f"           applies from {rw['applies_from']}; a demotion is never back-billed")
    print(f"  next: {decree['next_step']}")
    print(f"  receipt {decree['receipt_sha256'][:16]}…  (unsigned — a steward signs)")

if __name__ == "__main__":
    main()
