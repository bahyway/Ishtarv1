#!/usr/bin/env python3
"""EnkiDB v4.0 -- 7-Type CQRS smoke test.

Confirms all 7 EnkiDB types actually respond, using the fleet's real
length-prefixed wire protocol -- not just "is the container Up" (podman ps
only proves the process exists, not that it's accepting and answering
requests).

Write side: TCP-connect only. Write servers only understand SEED/FLUSH
(SEED mutates real data), so this script deliberately never sends either --
it only proves the listener is there and accepting connections.

Read side: a real QUERY:, sent and parsed per the fleet's binary frame
([1B tag] 0x00=success+[u32 LE row_count]+rows, 0x01=error+[u32 LE len]+msg
-- see bin/enkidb-read-server/src/main.rs's own doc comment). The probe
attribute/value are made up on purpose (`_smoke_test_attr` /
`_smoke_test_probe`) -- a query-level ERROR response still proves the
server's request loop is alive and answering, so it counts as UP, same as
a 0-row success would.

USAGE:
    python3 scripts/enkidb_smoke_test.py
    python3 scripts/enkidb_smoke_test.py --write-host 192.168.122.111 --read-host 192.168.122.112

Writes a timestamped JSON + text report to ./enkidb_smoke_reports/ (or
--out-dir) and exits 0 only if all 7 types are fully up on both sides.
"""
import argparse
import json
import socket
import struct
import sys
import time
from datetime import datetime, timezone
from pathlib import Path

# (type_name, write_port, read_port, write_container, read_container)
# EnkiODB/EnkiQDB's write side is intentionally NOT a separate container --
# enkisdb-write-server owns EnkiSDB + EnkiODB + EnkiQDB together (see
# playbook_222's own header comment), so all three share port 7013 and the
# enkisdb-write container name on the write side only.
TYPES = [
    ("EnkiDDB", 7101, 7102, "enkiddb-write", "enkiddb-read"),
    ("EnkiMDB", 7201, 7202, "enkimdb-write", "enkimdb-read"),
    ("EnkiDB",  7011, 7001, "enkidb-write",  "enkidb-read"),
    ("EnkiSDB", 7013, 7003, "enkisdb-write", "enkisdb-read"),
    ("EnkiODB", 7013, 7004, "enkisdb-write", "enkiodb-read"),
    ("EnkiQDB", 7013, 7005, "enkisdb-write", "enkiqdb-read"),
    ("EnkiDW",  7012, 7002, "enkidw-write",  "enkidw-read"),
]

SMOKE_QUERY = (
    'QUERY:WHO T.E\n'
    'WHAT E[_smoke_test_attr]\n'
    'WHERE E[_smoke_test_attr] = "_smoke_test_probe"\n'
    'HOW_MUCH LIMIT 1'
)


def recv_exact(sock, n, deadline):
    buf = b""
    while len(buf) < n:
        remaining = deadline - time.monotonic()
        if remaining <= 0:
            raise TimeoutError(f"timed out waiting for {n - len(buf)} more bytes")
        sock.settimeout(remaining)
        chunk = sock.recv(n - len(buf))
        if not chunk:
            raise EOFError("connection closed before expected bytes arrived")
        buf += chunk
    return buf


def tcp_probe(host, port, timeout=5.0):
    """Bare TCP connect -- proves the listener exists, nothing about the app."""
    t0 = time.monotonic()
    try:
        with socket.create_connection((host, port), timeout=timeout):
            pass
        return True, (time.monotonic() - t0) * 1000, "listener accepted connection"
    except OSError as e:
        return False, (time.monotonic() - t0) * 1000, f"{type(e).__name__}: {e}"


def query_probe(host, port, timeout=8.0):
    """Real QUERY: round trip -- proves the server's request loop is alive."""
    t0 = time.monotonic()
    deadline = t0 + timeout
    try:
        with socket.create_connection((host, port), timeout=timeout) as s:
            payload = SMOKE_QUERY.encode("utf-8")
            s.sendall(struct.pack("<I", len(payload)) + payload)
            (flen,) = struct.unpack("<I", recv_exact(s, 4, deadline))
            body = recv_exact(s, flen, deadline)
            elapsed_ms = (time.monotonic() - t0) * 1000
            tag = body[0]
            if tag == 0x00:
                row_count = struct.unpack_from("<I", body, 1)[0]
                return True, elapsed_ms, f"OK (row_count={row_count})"
            elif tag == 0x01:
                mlen = struct.unpack_from("<I", body, 1)[0]
                msg = body[5:5 + mlen].decode("utf-8", "replace")
                return True, elapsed_ms, f"OK (server answered with ERROR: {msg})"
            else:
                return False, elapsed_ms, f"unexpected tag byte 0x{tag:02x}"
    except Exception as e:
        return False, (time.monotonic() - t0) * 1000, f"{type(e).__name__}: {e}"


def main():
    ap = argparse.ArgumentParser(description="EnkiDB v4.0 7-type CQRS smoke test")
    ap.add_argument("--write-host", default="192.168.122.111", help="uruk-node-write IP")
    ap.add_argument("--read-host", default="192.168.122.112", help="uruk-node-read IP")
    ap.add_argument("--out-dir", default="enkidb_smoke_reports", help="report output directory")
    args = ap.parse_args()

    ts = datetime.now(timezone.utc).strftime("%Y%m%dT%H%M%SZ")
    out_dir = Path(args.out_dir)
    out_dir.mkdir(parents=True, exist_ok=True)

    print(f"EnkiDB v4.0 -- 7-Type CQRS Smoke Test -- {ts}")
    print(f"write-host={args.write_host}  read-host={args.read_host}\n")
    header = (f"{'TYPE':<9} {'W-PORT':<7} {'WRITE':<6} {'ms':>7}   "
              f"{'R-PORT':<7} {'READ':<6} {'ms':>7}   detail")
    print(header)
    print("-" * len(header))

    results = []
    for name, wport, rport, wcontainer, rcontainer in TYPES:
        w_ok, w_ms, w_detail = tcp_probe(args.write_host, wport)
        r_ok, r_ms, r_detail = query_probe(args.read_host, rport)

        w_status = "UP" if w_ok else "DOWN"
        r_status = "UP" if r_ok else "DOWN"
        detail = r_detail if not r_ok else (w_detail if not w_ok else r_detail)
        print(f"{name:<9} {wport:<7} {w_status:<6} {w_ms:7.1f}   "
              f"{rport:<7} {r_status:<6} {r_ms:7.1f}   {detail}")

        results.append({
            "type": name,
            "write_host": args.write_host, "write_port": wport,
            "write_container": wcontainer, "write_up": w_ok,
            "write_latency_ms": round(w_ms, 1), "write_detail": w_detail,
            "read_host": args.read_host, "read_port": rport,
            "read_container": rcontainer, "read_up": r_ok,
            "read_latency_ms": round(r_ms, 1), "read_detail": r_detail,
        })

    total = len(results)
    write_up = sum(1 for r in results if r["write_up"])
    read_up = sum(1 for r in results if r["read_up"])
    all_up = sum(1 for r in results if r["write_up"] and r["read_up"])

    print(f"\nSummary: {all_up}/{total} types fully up "
          f"(write {write_up}/{total}, read {read_up}/{total})")

    report = {
        "timestamp_utc": ts,
        "write_host": args.write_host,
        "read_host": args.read_host,
        "results": results,
        "summary": {"total_types": total, "write_up": write_up,
                    "read_up": read_up, "fully_up": all_up},
    }

    json_path = out_dir / f"enkidb_smoke_{ts}.json"
    txt_path = out_dir / f"enkidb_smoke_{ts}.txt"
    json_path.write_text(json.dumps(report, indent=2))

    with txt_path.open("w") as f:
        f.write(f"EnkiDB v4.0 -- 7-Type CQRS Smoke Test -- {ts}\n")
        f.write(f"write-host={args.write_host}  read-host={args.read_host}\n\n")
        f.write(header + "\n")
        f.write("-" * len(header) + "\n")
        for r in results:
            w_status = "UP" if r["write_up"] else "DOWN"
            r_status = "UP" if r["read_up"] else "DOWN"
            detail = r["read_detail"] if not r["read_up"] else (
                r["write_detail"] if not r["write_up"] else r["read_detail"])
            f.write(f"{r['type']:<9} {r['write_port']:<7} {w_status:<6} {r['write_latency_ms']:7.1f}   "
                    f"{r['read_port']:<7} {r_status:<6} {r['read_latency_ms']:7.1f}   {detail}\n")
        f.write(f"\nSummary: {all_up}/{total} types fully up "
                f"(write {write_up}/{total}, read {read_up}/{total})\n")

    print(f"\nReport written to:\n  {json_path}\n  {txt_path}")
    sys.exit(0 if all_up == total else 1)


if __name__ == "__main__":
    main()
