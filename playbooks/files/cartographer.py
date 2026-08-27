#!/usr/bin/env python3
"""
BahyWay v4.0 · Cartographer · PB-380
Reads every Cargo.toml and every playbook WITHOUT compiling or running anything,
and emits: ontograph.json (typed nodes/edges), hypergraph.json (n-ary relations),
BOOTSTRAP.yml (topological layers = the client-side regeneration template).

Doctrine:
  · the graph is static text; only the errors inside a layer need a compiler (GL-FLD-001 §5 spirit)
  · an undeclared dependency is a finding, not a surprise (GL-SEC-002 §4 two-witness analogue)
  · nothing here executes a playbook or builds a crate
"""
import argparse, json, os, re, sys
from collections import defaultdict, deque
from pathlib import Path
try:
    import yaml
except ImportError:
    sys.exit("pip install pyyaml")

# ---------- crate graph (from Cargo.toml only) ----------
def scan_crates(root):
    crates, deps = {}, defaultdict(set)
    tomls = [p for p in Path(root).rglob("Cargo.toml") if "target" not in p.parts]
    for t in tomls:
        txt = t.read_text(errors="ignore")
        m = re.search(r'(?m)^\s*name\s*=\s*"([^"]+)"', txt)
        if not m:
            continue
        name = m.group(1)
        crates[name] = {"path": str(t.parent), "kind": "crate"}
        # any local dependency: path = "..." or workspace = true entries
        for dm in re.finditer(r'(?m)^\s*([A-Za-z0-9_\-]+)\s*=\s*\{[^}]*\}', txt):
            entry = dm.group(0)
            if "path" in entry or "workspace" in entry:
                deps[name].add(dm.group(1))
        for dm in re.finditer(r'(?m)^\s*([A-Za-z0-9_\-]+)\.workspace\s*=\s*true', txt):
            deps[name].add(dm.group(1))
    for n in list(deps):                       # keep only workspace-internal edges
        deps[n] = {d for d in deps[n] if d in crates and d != n}
    return crates, deps

# ---------- playbook graph (from YAML only) ----------
PROVIDE_HINTS = [
    (r'systemd:\s*\{?\s*name:\s*([A-Za-z0-9_.\-]+)', 'service:{}'),
    (r'name:\s*([A-Za-z0-9_.\-]+)\s*,?\s*enabled:\s*true',  'service:{}'),
    (r'ansible\.posix\.mount', 'mount:'),
    (r'docker_container:\s*\n\s*name:\s*([A-Za-z0-9_.\-]+)', 'container:{}'),
]
REQUIRE_HINTS = [
    (r'\b(bahyway-[a-z\-]+)\b', 'binary:{}'),
    (r'cargo build --release -p ([A-Za-z0-9_\-]+)', 'crate:{}'),
    (r'cargo build\s+.*-p\s+([A-Za-z0-9_\-]+)', 'crate:{}'),
    (r'--port\s+(70[0-9]{2})', 'port:{}'),
    (r'localhost:(\d{4,5})|127\.0\.0\.1:(\d{4,5})', 'port:{}'),
]

def scan_playbooks(pbdir):
    pbs = {}
    for p in sorted(Path(pbdir).glob("**/*.yml")):
        txt = p.read_text(errors="ignore")
        try:
            docs = [d for d in yaml.safe_load_all(txt) if d]
        except Exception as e:
            docs = []
        pid = re.match(r'(PB-\d+)', p.name)
        pid = pid.group(1) if pid else p.stem
        node = {"id": pid, "file": p.name, "kind": "playbook",
                "imports": [], "roles": [], "provides": set(), "requires": set(),
                "declared_provides": set(), "declared_requires": set(), "hosts": set()}
        for d in docs:
            if isinstance(d, list):
                for play in d:
                    _play(play, node)
            elif isinstance(d, dict):
                _play(d, node)
        for rx, fmt in PROVIDE_HINTS:
            for m in re.finditer(rx, txt):
                g = next((x for x in m.groups() if x), "") if m.groups() else ""
                node["provides"].add(fmt.format(g))
        for rx, fmt in REQUIRE_HINTS:
            for m in re.finditer(rx, txt):
                g = next((x for x in m.groups() if x), "") if m.groups() else ""
                if g:
                    node["requires"].add(fmt.format(g))
        for m in re.finditer(r'import_playbook:\s*(\S+)', txt):
            im = re.match(r'(PB-\d+)', Path(m.group(1)).name)
            if im:
                node["imports"].append(im.group(1))
        pbs[pid] = node
    return pbs

def _play(play, node):
    if not isinstance(play, dict):
        return
    h = play.get("hosts")
    if h:
        node["hosts"].add(str(h))
    v = play.get("vars") or {}
    for k, key in (("bahyway_provides", "declared_provides"), ("bahyway_requires", "declared_requires")):
        val = v.get(k)
        if val:
            node[key] |= set(val if isinstance(val, list) else [val])
    for r in (play.get("roles") or []):
        node["roles"].append(r if isinstance(r, str) else r.get("role", "?"))

# ---------- graph maths ----------
def layers(nodes, edges):
    """edges: node -> set(prereqs). Returns list of layers, plus cycle members."""
    indeg = {n: len(edges.get(n, set()) & set(nodes)) for n in nodes}
    rev = defaultdict(set)
    for n, ps in edges.items():
        for p in ps:
            if p in nodes:
                rev[p].add(n)
    q = deque(sorted([n for n in nodes if indeg[n] == 0]))
    out, seen = [], set()
    while q:
        layer = sorted(q); q.clear()
        out.append(layer); seen |= set(layer)
        for n in layer:
            for m in sorted(rev[n]):
                indeg[m] -= 1
                if indeg[m] == 0:
                    q.append(m)
    cycles = sorted(set(nodes) - seen)
    return out, cycles

def longest_path(nodes, edges):
    depth, order = {}, []
    ls, _ = layers(nodes, edges)
    for L in ls:
        order += L
    for n in order:
        depth[n] = 1 + max([depth.get(p, 0) for p in edges.get(n, set()) if p in depth] or [0])
    if not depth:
        return []
    end = max(depth, key=lambda n: depth[n])
    path = [end]
    while True:
        ps = [p for p in edges.get(path[-1], set()) if p in depth]
        if not ps:
            break
        path.append(max(ps, key=lambda p: depth[p]))
    return list(reversed(path))

# ---------- main ----------
def main():
    a = argparse.ArgumentParser()
    a.add_argument("--repo", required=True, help="workspace root (Cargo.toml tree)")
    a.add_argument("--playbooks", required=True)
    a.add_argument("--out", default="./carto")
    args = a.parse_args()
    Path(args.out).mkdir(parents=True, exist_ok=True)

    crates, cdeps = scan_crates(args.repo)
    pbs = scan_playbooks(args.playbooks)

    # playbook prerequisites: explicit imports + declared requires satisfied by another PB's provides
    provider = {}
    for pid, n in pbs.items():
        for p in (n["provides"] | n["declared_provides"]):
            provider.setdefault(p, pid)
    pdeps = defaultdict(set)
    undeclared = []
    for pid, n in pbs.items():
        pdeps[pid] |= set(n["imports"])
        for r in (n["requires"] | n["declared_requires"]):
            owner = provider.get(r)
            if owner and owner != pid:
                pdeps[pid].add(owner)
                if r not in n["declared_requires"]:
                    undeclared.append({"playbook": pid, "resource": r, "provided_by": owner,
                                       "class": "DERIVED",
                                       "note": "inferred from text, not declared in bahyway_requires"})
    clayers, ccycles = layers(set(crates), cdeps)
    players, pcycles = layers(set(pbs), pdeps)
    crit = longest_path(set(crates), cdeps)

    onto = {"nodes": [{"id": k, **v} for k, v in crates.items()]
                     + [{"id": k, "kind": "playbook", "file": v["file"], "hosts": sorted(v["hosts"])} for k, v in pbs.items()],
            "edges": [{"from": n, "to": d, "rel": "depends_on", "domain": "crate"} for n, ds in cdeps.items() for d in ds]
                     + [{"from": n, "to": d, "rel": "after", "domain": "playbook"} for n, ds in pdeps.items() for d in ds]}
    # hyperedges: a playbook binds itself + the crates it builds + the services it provides
    hyper = []
    for pid, n in pbs.items():
        members = [pid] + [r.split(":", 1)[1] for r in n["requires"] if r.startswith("crate:")] \
                        + sorted(n["provides"] | n["declared_provides"])
        if len(members) >= 3:
            hyper.append({"id": "H-" + pid, "members": members, "rel": "playbook_binds",
                          "arity": len(members)})
    Path(args.out, "ontograph.json").write_text(json.dumps(onto, indent=2))
    Path(args.out, "hypergraph.json").write_text(json.dumps(
        {"hyperedges": hyper, "note": "arity>=3 relations; a filled 2-simplex or higher in the Sala bench"}, indent=2))
    boot = {"generated_by": "PB-380 cartographer",
            "crate_layers": clayers, "crate_cycles": ccycles,
            "playbook_layers": players, "playbook_cycles": pcycles,
            "critical_path_crates": crit,
            "undeclared_dependencies": undeclared,
            "regeneration_rule": "run each playbook layer in order; within a layer, order is free",
            "check_rule": "cargo check -p <crate> --keep-going per crate layer; all errors in a layer surface in one pass"}
    Path(args.out, "BOOTSTRAP.yml").write_text(yaml.safe_dump(boot, sort_keys=False, width=100))
    print(f"crates      : {len(crates)} in {len(clayers)} layers"
          f"{' · CYCLES ' + ','.join(ccycles) if ccycles else ''}")
    print(f"playbooks   : {len(pbs)} in {len(players)} layers"
          f"{' · CYCLES ' + ','.join(pcycles) if pcycles else ''}")
    print(f"hyperedges  : {len(hyper)}")
    print(f"undeclared  : {len(undeclared)} inferred dependencies not written in bahyway_requires")
    print(f"critical    : {' -> '.join(crit[:8])}{' ...' if len(crit) > 8 else ''}")
    return 1 if (ccycles or pcycles) else 0

if __name__ == "__main__":
    sys.exit(main())
