#!/usr/bin/env python3
"""cqrs_node_optimize.py -- real, data-driven vCPU/memory sizing proposal
for enkidb-node-write / enkidb-node-read, for PB-224
(playbook_224_cqrs_node_reconfigure.yml).

Same design as the earlier (untracked, host-local) vm_optimize.py this
session built for the 3-VM fleet -- ported into this repo, scoped to just
the two CQRS nodes, so it's git-pullable instead of living only on one
host's disk.

Real, not guessed: memory proposals for a RUNNING VM are sized from that
VM's own measured RSS (virsh dommemstat's real "rss" field), times a
safety margin, rounded up to the nearest GiB, capped at the CURRENT
ceiling (never raised automatically -- growing memory is a deliberate
--grow-mem-to override, not something this script decides on its own).
vCPU proposal is capped at (host_cores - host_core_reserve), never
raised automatically either.

--apply makes PERSISTENT (--config) changes only. Nothing is applied
live, nothing is rebooted -- each VM picks up new limits on its next
restart.
"""
import argparse
import json
import math
import re
import subprocess
import sys


def run(uri, *args):
    result = subprocess.run(
        ["virsh", "-c", uri, *args],
        capture_output=True, text=True, check=False,
    )
    return result.stdout


def kib_to_gib(kib: int) -> float:
    return kib / (1024 * 1024)


def gib_to_kib(gib: float) -> int:
    return int(round(gib * 1024 * 1024))


def parse_dominfo(text: str) -> dict:
    info = {}
    for line in text.splitlines():
        if ":" not in line:
            continue
        key, val = line.split(":", 1)
        info[key.strip()] = val.strip()
    return info


def parse_current_vcpus(text: str) -> int | None:
    live = None
    config = None
    for line in text.splitlines():
        parts = line.split()
        if len(parts) < 3 or parts[0] != "current":
            continue
        if parts[1] == "live":
            live = int(parts[2])
        elif parts[1] == "config":
            config = int(parts[2])
    return live if live is not None else config


def parse_rss_kib(text: str) -> int | None:
    for line in text.splitlines():
        parts = line.split()
        if len(parts) == 2 and parts[0] == "rss":
            try:
                return int(parts[1])
            except ValueError:
                return None
    return None


def gather_one(
    uri: str, name: str, ram_margin: float, host_cores: int, host_core_reserve: int,
    grow_mem_gib: float | None,
) -> dict:
    dominfo = parse_dominfo(run(uri, "dominfo", name))
    state = dominfo.get("State", "unknown")
    max_mem_kib_match = re.search(r"(\d+)", dominfo.get("Max memory", "0"))
    current_mem_kib = int(max_mem_kib_match.group(1)) if max_mem_kib_match else 0

    current_vcpus = parse_current_vcpus(run(uri, "vcpucount", name)) or 0

    has_live_data = False
    proposed_mem_kib = current_mem_kib  # default: no change without a reason to change
    if grow_mem_gib is not None:
        # Explicit ask to grow -- never shrink below the current ceiling
        # even if requested lower, since that's a different, riskier
        # operation this script doesn't attempt.
        proposed_mem_kib = max(current_mem_kib, gib_to_kib(grow_mem_gib))
    elif state == "running":
        rss_kib = parse_rss_kib(run(uri, "dommemstat", name))
        if rss_kib:
            has_live_data = True
            sized_kib = gib_to_kib(math.ceil(kib_to_gib(rss_kib * ram_margin)))
            proposed_mem_kib = min(sized_kib, current_mem_kib)

    target_vcpu_ceiling = max(1, host_cores - host_core_reserve) if host_cores else current_vcpus
    proposed_vcpus = min(current_vcpus, target_vcpu_ceiling) if current_vcpus else current_vcpus

    return {
        "name": name,
        "state": state,
        "has_live_data": has_live_data,
        "current_vcpus": current_vcpus,
        "proposed_vcpus": proposed_vcpus,
        "current_mem_gb": round(kib_to_gib(current_mem_kib), 1),
        "proposed_mem_gb": round(kib_to_gib(proposed_mem_kib), 1),
        "proposed_mem_kib": proposed_mem_kib,
        "savings_vcpus": max(0, current_vcpus - proposed_vcpus),
        "savings_mem_gb": round(kib_to_gib(current_mem_kib) - kib_to_gib(proposed_mem_kib), 1),
        "growth_mem_gb": round(kib_to_gib(proposed_mem_kib) - kib_to_gib(current_mem_kib), 1),
    }


def format_report(plans: list, ram_margin: float, host_cores: int, host_core_reserve: int) -> str:
    lines = []
    lines.append("=" * 60)
    lines.append(" CQRS node vCPU/memory -- current vs proposed")
    lines.append(f" (host logical CPUs detected: {host_cores}, reserving {host_core_reserve} for the host)")
    lines.append("=" * 60)
    for vm in plans:
        lines.append("")
        lines.append(f"VM: {vm['name']}  (state: {vm['state']})")
        vcpu_note = f"(-{vm['savings_vcpus']})" if vm["savings_vcpus"] > 0 else "(no change)"
        lines.append(f"  vCPUs:  {vm['current_vcpus']} -> {vm['proposed_vcpus']}  {vcpu_note}")
        if vm["growth_mem_gb"] > 0:
            mem_note = f"(+{vm['growth_mem_gb']}G, explicit grow request)"
        elif not vm["has_live_data"]:
            mem_note = "(no change -- not running, no live usage to size from)"
        elif vm["savings_mem_gb"] > 0:
            mem_note = f"(-{vm['savings_mem_gb']}G, sized from real measured RSS x{ram_margin} margin)"
        else:
            mem_note = "(already right-sized)"
        lines.append(f"  Memory: {vm['current_mem_gb']}G -> {vm['proposed_mem_gb']}G  {mem_note}")
    lines.append("")
    lines.append("-" * 60)
    lines.append("NOTE: --config (persistent) changes only. NOTHING applied live,")
    lines.append("NOTHING rebooted. Each VM picks up new limits on its next restart.")
    lines.append("-" * 60)
    return "\n".join(lines)


def apply_one(uri: str, plan: dict) -> list[str]:
    log = []
    name = plan["name"]
    if plan["savings_vcpus"] > 0:
        for cmd in (
            ["setvcpus", name, str(plan["proposed_vcpus"]), "--config", "--maximum"],
            ["setvcpus", name, str(plan["proposed_vcpus"]), "--config"],
        ):
            out = run(uri, *cmd)
            log.append(f"$ virsh -c {uri} {' '.join(cmd)}\n{out}".strip())
    if plan["savings_mem_gb"] > 0 or plan["growth_mem_gb"] > 0:
        mem_k = f"{plan['proposed_mem_kib']}K"
        for cmd in (
            ["setmaxmem", name, mem_k, "--config"],
            ["setmem", name, mem_k, "--config"],
        ):
            out = run(uri, *cmd)
            log.append(f"$ virsh -c {uri} {' '.join(cmd)}\n{out}".strip())
    if plan["savings_vcpus"] == 0 and plan["savings_mem_gb"] == 0 and plan["growth_mem_gb"] == 0:
        log.append(f"{name}: no change proposed, nothing applied.")
    return log


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--uri", default="qemu:///system")
    ap.add_argument("--vm", action="append", dest="vms")
    ap.add_argument("--ram-margin", type=float, default=1.3)
    ap.add_argument("--host-cores", type=int, default=0)
    ap.add_argument("--host-core-reserve", type=int, default=1)
    ap.add_argument(
        "--grow-mem-to",
        type=float,
        default=None,
        help="GiB to grow EVERY --vm's memory ceiling to (explicit ask, never auto-decided from RSS).",
    )
    ap.add_argument("--apply", action="store_true")
    ap.add_argument("--report", action="store_true")
    ap.add_argument("--plan-file", help="Persist/read the computed plan as JSON so --report and --apply agree exactly.")
    args = ap.parse_args()

    if args.apply and args.plan_file:
        try:
            with open(args.plan_file) as f:
                plans = json.load(f)
        except FileNotFoundError:
            print(f"NOTE: --plan-file {args.plan_file} not found -- re-querying virsh live.", file=sys.stderr)
            if not args.vms:
                ap.error("--vm is required when --plan-file is missing/unset")
            plans = [
                gather_one(args.uri, vm, args.ram_margin, args.host_cores, args.host_core_reserve, args.grow_mem_to)
                for vm in args.vms
            ]
    else:
        if not args.vms:
            ap.error("--vm is required unless --apply is reading an existing --plan-file")
        plans = [
            gather_one(args.uri, vm, args.ram_margin, args.host_cores, args.host_core_reserve, args.grow_mem_to)
            for vm in args.vms
        ]
        if args.plan_file:
            with open(args.plan_file, "w") as f:
                json.dump(plans, f)

    if args.report:
        print(format_report(plans, args.ram_margin, args.host_cores, args.host_core_reserve))
        return
    if not args.apply:
        print(json.dumps(plans))
        return

    for plan in plans:
        for line in apply_one(args.uri, plan):
            print(line)
        print("---")


if __name__ == "__main__":
    sys.exit(main())
