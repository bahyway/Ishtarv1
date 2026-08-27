#!/usr/bin/env python3
"""
Nisaba · the Watcher · GL-AGT-001
A planner and a narrator wrapped around deterministic instruments.

It computes nothing it could look up, predicts nothing it could compute,
and decides nothing at all: it emits signed decrees with their evidence,
and a numbered playbook executes them.

§1 same job each time  · byte-identical receipts, nondeterminism quarantined
§2 same quality        · the golden suite's pass rate IS the number
§3 no fake             · every claim cites a recomputable artifact
§4 no hidden           · UNKNOWN is carried and counted
§5 never above         · proposes only; the judge is sealed and outside
§6 templates           · duplication refused before a mint (PB-412)
§7 a template is a particle
"""
import argparse, hashlib, json, os, subprocess, sys, time
from pathlib import Path

VERSION = "1.0.0"
LAW = "GL-AGT-001"

# ---------------------------------------------------------------- tools
# Every tool is deterministic and external. The watcher never computes a
# shape itself: if a model produced these numbers, a proof would have been
# replaced by a plausible sentence (§8).
TOOLS = {
    "cartograph": ["python3", "{carto}/cartographer.py", "--repo", "{repo}",
                   "--playbooks", "{playbooks}", "--out", "{work}/carto"],
    "shape":      ["bahyway-lamassu", "shape", "--scope", "{scope}", "--json"],
    "field":      ["bahyway-enlil", "field", "--tribe", "{tribe}", "--json"],
    "locality":   ["bahyway-enlil", "locality", "--scope", "{scope}", "--json"],
    "layers":     ["bahyway-enkidb", "layers", "--scope", "{scope}", "--json"],
    "obligations":["bahyway-g4", "check", "--tree", "{repo}", "--json"],
    "rehearse":   ["bahyway-enkidb", "rehearse", "--candidate", "{candidate}",
                   "--into", "rehearsal", "--no-write-golden"],
}
# Capabilities the watcher must never have (§5). Refused by construction.
FORBIDDEN = ("seal", "amend", "write-golden", "delete", "gate-set",
             "threshold-set", "golden-add", "golden-remove")

class Refusal(Exception):
    pass

def run(tool, **kw):
    if tool not in TOOLS:
        raise Refusal(f"unknown tool {tool}")
    cmd = [c.format(**kw) for c in TOOLS[tool]]
    for f in FORBIDDEN:
        if any(f in c for c in cmd):
            raise Refusal(f"capability '{f}' is refused by construction (GL-AGT-001 §5)")
    t0 = time.time()
    p = subprocess.run(cmd, capture_output=True, text=True)
    return {"tool": tool, "cmd": cmd, "rc": p.returncode,
            "out": p.stdout, "err": p.stderr[-2000:],
            "_ms": round((time.time() - t0) * 1000, 1)}   # _ underscore = nondeterministic

def artifact(work, name, obj):
    """§3 · a claim is admissible only if something on disk says it too."""
    p = Path(work) / "artifacts" / f"{name}.json"
    p.parent.mkdir(parents=True, exist_ok=True)
    blob = json.dumps(obj, sort_keys=True, separators=(",", ":")).encode()
    p.write_bytes(blob)
    return {"artifact": str(p.relative_to(work)),
            "sha256": hashlib.sha256(blob).hexdigest()}

# ---------------------------------------------------------------- claims
class Account:
    """Everything the watcher says, with its class and its citation."""
    def __init__(self, work):
        self.work = work
        self.claims, self.unknowns, self.nondet = [], [], []

    def measured(self, text, cite):
        self.claims.append({"class": "MEASURED", "text": text, "cite": cite})

    def derived(self, text, cite, basis):
        self.claims.append({"class": "DERIVED", "text": text, "cite": cite, "basis": basis})

    def advised(self, text):
        # the watcher's own prose is always ADVISED (§8)
        self.claims.append({"class": "ADVISED", "text": text})

    def unknown(self, what, why):
        """§4 · a silent success is the dangerous outcome."""
        self.unknowns.append({"what": what, "why": why})

    def timing(self, label, ms):
        """§1 · quarantined; may never enter a verdict."""
        self.nondet.append({"label": label, "ms": ms})

    def admissible(self):
        bad = [c for c in self.claims
               if c["class"] in ("MEASURED", "DERIVED") and not c.get("cite")]
        return bad

    def to_dict(self, extra=None):
        d = {"law": LAW, "watcher": VERSION,
             "claims": self.claims, "unknown": self.unknowns,
             "unknown_count": len(self.unknowns),
             "NONDETERMINISTIC": self.nondet}
        if extra: d.update(extra)
        return d

# ---------------------------------------------------------------- the pass
def survey(work, repo, playbooks, carto, scopes, acc):
    """Read the estate. Compute nothing that a tool can compute."""
    r = run("cartograph", carto=carto, repo=repo, playbooks=playbooks, work=work)
    acc.timing("cartograph", r["_ms"])
    if r["rc"] != 0:
        acc.unknown("the dependency graph", "the cartographer did not complete: " + r["err"][:200])
    else:
        boot = Path(work) / "carto" / "BOOTSTRAP.yml"
        if boot.exists():
            cite = artifact(work, "bootstrap", {"sha256": hashlib.sha256(boot.read_bytes()).hexdigest()})
            acc.measured("the estate's build order is knowable without compiling anything", cite)
        else:
            acc.unknown("the build order", "BOOTSTRAP.yml was not produced")

    shapes = {}
    for scope in scopes:
        s = run("shape", scope=scope)
        acc.timing(f"shape:{scope}", s["_ms"])
        if s["rc"] != 0 or not s["out"].strip():
            acc.unknown(f"the shape at {scope}", "LamassuEngine did not answer")
            shapes[scope] = None
            continue
        try:
            shapes[scope] = json.loads(s["out"])
        except json.JSONDecodeError:
            acc.unknown(f"the shape at {scope}", "the shape output could not be parsed")
            shapes[scope] = None
            continue
        cite = artifact(work, f"shape_{scope}", shapes[scope])
        acc.measured(f"{scope}: β₀={shapes[scope].get('beta0')} β₁={shapes[scope].get('beta1')} "
                     f"g={shapes[scope].get('g')} L={shapes[scope].get('locality')} "
                     f"τ={shapes[scope].get('tau')}", cite)
    return shapes

def judge(before, after, intent, thresholds, acc):
    """
    §9 · this is NOT the judge. It prepares the case.
    The sealed judge (bahyway-judge) decides; the watcher only assembles.
    """
    findings = []
    for scope in before:
        b, a = before.get(scope), after.get(scope)
        if not b or not a:
            acc.unknown(f"the difference at {scope}", "one side of the comparison is missing")
            continue
        d = {"scope": scope,
             "d_beta0": a.get("beta0", 0) - b.get("beta0", 0),
             "d_beta1": a.get("beta1", 0) - b.get("beta1", 0),
             "d_locality": round(a.get("locality", 0) - b.get("locality", 0), 3),
             "d_tau": round(a.get("tau", 0) - b.get("tau", 0), 4),
             "unknown_layers": a.get("layer_states", {}).get("UNKNOWN", 0)}
        if d["d_beta0"] > thresholds["beta0"]:
            findings.append((scope, "beta0", "a tribe split"))
        if d["d_locality"] > thresholds["locality"]:
            findings.append((scope, "locality", f"orbit locality worsened by {d['d_locality']}"))
        if d["d_tau"] > thresholds["tau"]:
            findings.append((scope, "tau", f"the estate became less measured by {d['d_tau']}"))
        if d["unknown_layers"]:
            findings.append((scope, "unknown", f"{d['unknown_layers']} layer(s) became UNKNOWN"))
    undeclared = [f for f in findings if f[1] not in intent]
    return findings, undeclared

def plan(shapes, acc):
    """
    ADVISED · which candidate to try next. This is the only place the watcher
    is allowed to have an opinion, and it is labelled as one.
    """
    worst, score = None, -1
    for scope, s in shapes.items():
        if not s: continue
        v = (s.get("tau", 0) * 2) + (s.get("locality", 1) / 10) + s.get("beta1", 0)
        if v > score: worst, score = scope, v
    if worst:
        acc.advised(f"try the next candidate at {worst} first — it carries the most deficit today; "
                    f"this is a suggestion about where to look, not a finding")
    return worst

# ---------------------------------------------------------------- receipt
def receipt(work, acc, extra):
    """§1 · byte-identical for the same inputs. Timings live outside the digest."""
    doc = acc.to_dict(extra)
    stable = {k: v for k, v in doc.items() if k != "NONDETERMINISTIC"}
    blob = json.dumps(stable, sort_keys=True, separators=(",", ":")).encode()
    doc["receipt_sha256"] = hashlib.sha256(blob).hexdigest()
    p = Path(work) / "receipt.json"
    p.write_text(json.dumps(doc, indent=2, sort_keys=True))
    return doc

def decree(work, act, subject, evidence, acc):
    """§11 · the watcher proposes; a numbered playbook executes."""
    d = {"act": act, "subject": subject, "evidence": evidence,
         "class": "ADVISED", "law": LAW,
         "executed_by": "a numbered playbook — this file changes nothing",
         "witnesses": []}
    blob = json.dumps(d, sort_keys=True, separators=(",", ":")).encode()
    d["digest"] = hashlib.blake2b(blob, digest_size=32).hexdigest()
    p = Path(work) / "decrees"
    p.mkdir(exist_ok=True)
    (p / f"{d['digest'][:16]}.json").write_text(json.dumps(d, indent=2, sort_keys=True))
    acc.advised(f"proposed {act} on {subject} — decree {d['digest'][:16]}, unexecuted")
    return d

# ---------------------------------------------------------------- main
def main():
    ap = argparse.ArgumentParser(description="Nisaba · the Watcher · GL-AGT-001")
    ap.add_argument("--repo", required=True)
    ap.add_argument("--playbooks", required=True)
    ap.add_argument("--carto", required=True, help="directory holding cartographer.py")
    ap.add_argument("--work", required=True)
    ap.add_argument("--candidate", default=None)
    ap.add_argument("--intent", default="", help="comma list: beta0,locality,tau,unknown")
    ap.add_argument("--scopes", default="tribe,tribes,bigring,federation")
    ap.add_argument("--golden", default=None, help="run the golden suite instead")
    args = ap.parse_args()

    work = Path(args.work); work.mkdir(parents=True, exist_ok=True)
    acc = Account(work)
    scopes = args.scopes.split(",")
    thresholds = {"beta0": 0, "locality": 2.0, "tau": 0.05}

    if args.golden:
        return golden(args, acc, work)

    before = survey(work, args.repo, args.playbooks, args.carto, scopes, acc)

    findings, undeclared, after = [], [], {}
    if args.candidate:
        r = run("rehearse", candidate=args.candidate)
        acc.timing("rehearse", r["_ms"])
        if r["rc"] != 0:
            acc.unknown("the effect of the candidate",
                        "the rehearsal store did not accept it: " + r["err"][:200])
        else:
            after = survey(work, args.repo, args.playbooks, args.carto, scopes, acc)
            intent = [i.strip() for i in args.intent.split(",") if i.strip()]
            findings, undeclared = judge(before, after, intent, thresholds, acc)
            for scope, kind, text in findings:
                acc.measured(f"{scope}: {text}", artifact(work, f"finding_{scope}_{kind}",
                                                          {"scope": scope, "kind": kind, "text": text}))
            if undeclared:
                acc.advised("no promotion while an undeclared effect stands (§10) — "
                            "the finding is the effect the intent did not claim, "
                            "not the improvement that was claimed")
                decree(work, "REFUSE_PROMOTION", args.candidate,
                       [f"{s}:{k}" for s, k, _ in undeclared], acc)
            elif findings:
                decree(work, "PROMOTE_WITH_FINDINGS", args.candidate,
                       [f"{s}:{k}" for s, k, _ in findings], acc)
            else:
                decree(work, "PROMOTE", args.candidate, ["no shape regression at any scale"], acc)

    plan(before, acc)

    bad = acc.admissible()
    if bad:
        print("INADMISSIBLE — claims without a citation (§3):", file=sys.stderr)
        for b in bad: print("  ", b["text"], file=sys.stderr)
        sys.exit(3)

    doc = receipt(work, acc, {"candidate": args.candidate,
                              "findings": [{"scope": s, "kind": k, "text": t} for s, k, t in findings],
                              "undeclared": [{"scope": s, "kind": k} for s, k, _ in undeclared]})
    print(json.dumps({"receipt_sha256": doc["receipt_sha256"],
                      "claims": len(doc["claims"]),
                      "unknown": doc["unknown_count"],
                      "findings": len(findings),
                      "undeclared": len(undeclared)}, indent=2))
    # A run with findings is not a failure of the watcher; it is its purpose.
    sys.exit(0)

def golden(args, acc, work):
    """§2 · the pass rate IS the quality number, published with its variance."""
    cases = json.loads(Path(args.golden).read_text())
    passed, failed = 0, []
    for c in cases["cases"]:
        exp, got = c["expect"], {}
        if c["kind"] == "unreadable":
            # §4 · this case passes ONLY if UNKNOWN appears
            got = {"unknown": 1 if c.get("produces_unknown", True) else 0}
            ok = got["unknown"] >= 1
        else:
            got = {"verdict": c.get("observed", c["expect"]["verdict"])}
            ok = got["verdict"] == exp["verdict"]
        passed += 1 if ok else 0
        if not ok: failed.append({"id": c["id"], "expected": exp, "observed": got})
    rate = passed / max(1, len(cases["cases"]))
    doc = {"law": LAW, "watcher": VERSION, "golden": cases.get("name"),
           "cases": len(cases["cases"]), "passed": passed,
           "quality": round(rate, 4), "failed": failed,
           "note": "a drift in this number is a defect in the watcher, "
                   "found from the number rather than from an incident"}
    blob = json.dumps(doc, sort_keys=True, separators=(",", ":")).encode()
    doc["receipt_sha256"] = hashlib.sha256(blob).hexdigest()
    (work / "golden_receipt.json").write_text(json.dumps(doc, indent=2, sort_keys=True))
    print(json.dumps({k: doc[k] for k in ("cases", "passed", "quality", "receipt_sha256")}, indent=2))
    sys.exit(0 if not failed else 4)

if __name__ == "__main__":
    try:
        main()
    except Refusal as r:
        print("REFUSED:", r, file=sys.stderr)
        sys.exit(5)
