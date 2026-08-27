#!/usr/bin/env python3
"""
BahyWay v4.0 · CDSE Arsenal
Fetch Copernicus products, land them compressed, give every archive a KAKI and a Kaniku receipt.

Law: GL-FLD-001 §5 (the archive is a store, the field over it is a view)
     GL-UNT-001 §3 (a shared archive carries its disclosure boundary)
Honesty: Sentinel SAFE products are ALREADY compressed (JP2/COG inside a zip).
         Re-zipping typically buys 2-5%. This tool therefore stores as-is by default
         and reports the true delta so you never pay for imaginary compression.
"""
import argparse, hashlib, json, os, sys, time, zipfile
from pathlib import Path
import urllib.request, urllib.parse

TOKEN_URL = "https://identity.dataspace.copernicus.eu/auth/realms/CDSE/protocol/openid-connect/token"
ODATA     = "https://catalogue.dataspace.copernicus.eu/odata/v1/Products"
DOWNLOAD  = "https://zipper.dataspace.copernicus.eu/odata/v1/Products({pid})/$value"

def token(user, pw):
    data = urllib.parse.urlencode({
        "client_id": "cdse-public", "username": user,
        "password": pw, "grant_type": "password"}).encode()
    with urllib.request.urlopen(urllib.request.Request(TOKEN_URL, data=data)) as r:
        return json.load(r)["access_token"]

def query(collection, wkt, start, end, cloud=None, limit=20):
    f = (f"Collection/Name eq '{collection}' and "
         f"OData.CSC.Intersects(area=geography'SRID=4326;{wkt}') and "
         f"ContentDate/Start gt {start}T00:00:00.000Z and "
         f"ContentDate/Start lt {end}T00:00:00.000Z")
    if cloud is not None:
        f += (" and Attributes/OData.CSC.DoubleAttribute/any(att:att/Name eq 'cloudCover' "
              f"and att/OData.CSC.DoubleAttribute/Value lt {cloud})")
    url = ODATA + "?$filter=" + urllib.parse.quote(f) + f"&$top={limit}&$orderby=ContentDate/Start desc"
    with urllib.request.urlopen(url) as r:
        return json.load(r)["value"]

def kaki(name, sha):
    """16-byte KAKI for an archive particle. Layout mirrors v4.0; identity only, quality in EAV."""
    h = hashlib.blake2b(f"{name}|{sha}".encode(), digest_size=16).hexdigest()
    return "·".join(h[i:i+8] for i in range(0, 32, 8))

def sha256(p, chunk=1 << 20):
    h = hashlib.sha256()
    with open(p, "rb") as f:
        for b in iter(lambda: f.read(chunk), b""):
            h.update(b)
    return h.hexdigest()

def land(pid, name, out, tok, recompress=False):
    dest = Path(out) / f"{name}.zip"
    if dest.exists():
        print(f"  · already landed: {name}"); return dest
    req = urllib.request.Request(DOWNLOAD.format(pid=pid),
                                 headers={"Authorization": f"Bearer {tok}"})
    t0 = time.time()
    with urllib.request.urlopen(req) as r, open(dest, "wb") as f:
        while True:
            b = r.read(1 << 22)
            if not b: break
            f.write(b)
    raw = dest.stat().st_size
    delta = 0.0
    if recompress:                       # measured, never assumed
        tmp = dest.with_suffix(".zst.zip")
        with zipfile.ZipFile(dest) as zin, zipfile.ZipFile(tmp, "w", zipfile.ZIP_DEFLATED, compresslevel=9) as zo:
            for i in zin.infolist():
                zo.writestr(i, zin.read(i.filename))
        new = tmp.stat().st_size
        delta = (raw - new) / raw
        if delta > 0.05: dest.unlink(); tmp.rename(dest)
        else: tmp.unlink()
    print(f"  · landed {name} {raw/1e9:.2f} GB in {time.time()-t0:.0f}s"
          f"{f' · recompression gained {delta*100:.1f}%' if recompress else ''}")
    return dest

def receipt(path, meta, out):
    sha = sha256(path)
    r = {"kaki": kaki(path.name, sha), "archive": path.name,
         "bytes": path.stat().st_size, "sha256": sha,
         "product_id": meta.get("Id"), "sensed": meta.get("ContentDate", {}).get("Start"),
         "collection": meta.get("Collection", {}).get("Name") if isinstance(meta.get("Collection"), dict) else None,
         "landed": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
         "licence": "Copernicus - free and open; attribution required on any derived product",
         "epistemic": "MEASURED"}
    (Path(out) / "receipts").mkdir(exist_ok=True)
    (Path(out) / "receipts" / f"{path.stem}.kaniku.json").write_text(json.dumps(r, indent=2))
    return r

def main():
    a = argparse.ArgumentParser()
    a.add_argument("--collection", default="SENTINEL-2")
    a.add_argument("--wkt", required=True, help="POLYGON((...)) in EPSG:4326")
    a.add_argument("--start", required=True); a.add_argument("--end", required=True)
    a.add_argument("--cloud", type=float, default=30.0)
    a.add_argument("--limit", type=int, default=10)
    a.add_argument("--out", default="./landing_zone")
    a.add_argument("--recompress", action="store_true", help="try zip -9 and keep only if >5% gained")
    a.add_argument("--dry-run", action="store_true")
    args = a.parse_args()

    Path(args.out).mkdir(parents=True, exist_ok=True)
    prods = query(args.collection, args.wkt, args.start, args.end, args.cloud, args.limit)
    print(f"catalogue · {len(prods)} products match")
    est = sum(p.get("ContentLength", 0) for p in prods)
    print(f"estimated landing size · {est/1e9:.1f} GB")
    if args.dry_run:
        for p in prods: print(f"  - {p['Name']}  {p.get('ContentLength',0)/1e9:.2f} GB")
        return
    u, pw = os.environ.get("CDSE_USER"), os.environ.get("CDSE_PASS")
    if not (u and pw): sys.exit("set CDSE_USER and CDSE_PASS")
    tok = token(u, pw)
    manifest = []
    for p in prods:
        f = land(p["Id"], p["Name"], args.out, tok, args.recompress)
        manifest.append(receipt(f, p, args.out))
    Path(args.out, "ARSENAL.json").write_text(json.dumps(
        {"generated": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
         "count": len(manifest), "total_bytes": sum(m["bytes"] for m in manifest),
         "archives": manifest}, indent=2))
    print(f"arsenal · {len(manifest)} archives · "
          f"{sum(m['bytes'] for m in manifest)/1e12:.3f} TB · ARSENAL.json written")

if __name__ == "__main__":
    main()
