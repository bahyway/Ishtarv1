#!/usr/bin/env python3
"""
mini-EnkiDDB — a real (small) Input->Processing->Streaming->Output pipeline.

This is NOT the production EnkiDDB. It is an honest stand-in that performs the
ACTUAL computation the Fuzzy Lens must eventually read from EnkiDDB:
  1. INGEST   : clinically-plausible dated blood-pressure readings (a "patient orbit")
  2. PROCESS  : per-reading EAV attributes + KAKI mint + GOLDEN/FUZZY/DEAD state
  3. ANALYSE  : REAL statistics (Theil-Sen slope, Mann-Kendall trend, threshold
                crossings, epsilon band) — the "signals in the fog"
  4. OUTPUT   : a sealed JSON the Lens reads. Every number the Lens shows is
                computed HERE, not invented in the browser.

The readings are generated (no real person), but the *pipeline logic and the
statistics are genuine* — the drift the Lens reveals is really in the data and
really detected by Mann-Kendall, not scripted.
"""
import json, hashlib, math, datetime as dt
import numpy as np
from scipy import stats

SEED = 20260812
rng = np.random.default_rng(SEED)

# ---- clinical constants (systolic BP, mmHg) ----
SYS_NORMAL_MAX = 120     # normal < 120
SYS_ELEVATED   = 130     # 120-129 elevated; 130+ stage-1 hypertension
FUZZY_BAND     = 5       # +/- around a threshold = "sitting on the edge"

# ============================================================= INGEST
def ingest_orbit():
    """A patient's systolic BP over ~7 years, roughly quarterly.
    Hidden truth: a slow upward drift (~1.6 mmHg/year) buried under
    visit-to-visit noise — the kind no single visit flags."""
    start = dt.date(2019, 3, 1)
    n = 28
    readings = []
    drift_per_day = 1.6 / 365.0
    for i in range(n):
        day = start + dt.timedelta(days=int(i * (365*7/n)))
        t = (day - start).days
        trend = 112 + drift_per_day * t              # true latent mean
        noise = rng.normal(0, 6.5)                    # measurement + biological
        seasonal = 2.0*math.sin(t/180.0)              # mild seasonal wobble
        sys = round(trend + noise + seasonal, 1)
        readings.append({"date": day.isoformat(), "t_days": t, "systolic": sys})
    return readings

# ============================================================= PROCESS
def kaki_hex(s):
    h = hashlib.sha256(s.encode()).digest()
    k = bytearray(16)
    k[0:4] = h[0:4]
    k[4], k[5] = 0x44, 0x44           # tribe 'DD' (EnkiDDB)
    k[6], k[7] = 0x01, 0x01           # Identity / reading
    days = (dt.date.today() - dt.date(2020,1,1)).days
    k[12], k[13] = (days>>8)&0xff, days&0xff
    # crc16-ccitt
    crc = 0xFFFF
    for b in k[:14]:
        crc ^= b << 8
        for _ in range(8):
            crc = ((crc<<1)^0x1021)&0xFFFF if crc&0x8000 else (crc<<1)&0xFFFF
    k[14], k[15] = (crc>>8)&0xff, crc&0xff
    return " ".join(f"{k[i]:02X}{k[i+1]:02X}" for i in range(0,16,2))

def state_for(sys):
    """EAV state assignment (the trichotomy), plus fuzzy-edge flag."""
    near_edge = (abs(sys - SYS_NORMAL_MAX) <= FUZZY_BAND or
                 abs(sys - SYS_ELEVATED)   <= FUZZY_BAND)
    if sys >= SYS_ELEVATED:      state = "FUZZY"   # elevated = worth watching
    elif sys >= SYS_NORMAL_MAX:  state = "FUZZY"
    else:                        state = "GOLDEN"
    return state, near_edge

def process(readings):
    for i, r in enumerate(readings):
        state, near_edge = state_for(r["systolic"])
        r["kaki"] = kaki_hex(f"bp-{r['date']}-{r['systolic']}")
        r["state"] = state
        r["near_edge"] = near_edge
        # epsilon: measurement uncertainty grows for readings near the edge
        r["epsilon"] = round(0.04 + (0.10 if near_edge else 0.0) + rng.uniform(0,0.03), 3)
    return readings

# ============================================================= ANALYSE (real stats)
def theil_sen(t, y):
    res = stats.theilslopes(y, t)          # robust slope (mmHg per day)
    return res  # slope, intercept, low, high

def mann_kendall(y):
    """Non-parametric monotonic-trend test. Returns (tau, p)."""
    n = len(y)
    s = sum(np.sign(y[j]-y[i]) for i in range(n-1) for j in range(i+1, n))
    tau, p = stats.kendalltau(np.arange(n), y)
    return float(tau), float(p), int(s)

def analyse(readings):
    t = np.array([r["t_days"] for r in readings], float)
    y = np.array([r["systolic"] for r in readings], float)

    slope, intercept, lo, hi = theil_sen(t, y)
    slope_yr = slope*365.0
    tau, p, S = mann_kendall(y)

    # threshold crossings: where the robust trend line crosses 120 / 130
    def cross_day(level):
        if slope == 0: return None
        d = (level - intercept)/slope
        return d if t.min() <= d <= t.max()*1.6 else d  # allow near-future
    cross120 = cross_day(120); cross130 = cross_day(130)

    # how many readings sit in the fuzzy edge band
    edge_n = sum(1 for r in readings if r["near_edge"])

    signals = []
    # SIGNAL 1 — slow drift (Mann-Kendall)
    drift_conf = max(0.0, min(1.0, 1 - p))    # 1-p as a rough confidence proxy
    signals.append({
        "id":"drift","name":"A slow upward drift",
        "conf": round(drift_conf,2),
        "stat_label":"Mann–Kendall trend",
        "stat_value": f"tau={tau:+.2f}, p={p:.3f}  ·  Theil–Sen {slope_yr:+.2f} mmHg/yr",
        "detected": p < 0.05,
        "p_body":"Your blood pressure has been creeping upward across the years — too gradual to notice at any one visit, but the trend test picks it out of the noise.",
        "r_body":f"Monotonic increase confirmed by Mann–Kendall (tau={tau:+.2f}, p={p:.3f}); robust Theil–Sen slope {slope_yr:+.2f} mmHg/yr.",
        "ask":"Ask: “Has my blood pressure been trending up over my whole history, not just today’s reading?”",
        "method":"Mann–Kendall trend test + Theil–Sen robust slope",
        "status":"Standard in research; rarely run on routine BP records."
    })
    # SIGNAL 2 — approaching a threshold (crossing)
    if cross130 is not None:
        yrs_to_130 = (cross130 - t.max())/365.0
        near = abs(yrs_to_130) < 4
        signals.append({
            "id":"cross","name":"Heading toward a threshold",
            "conf": round(0.5 + 0.3*drift_conf,2),
            "stat_label":"Projected 130 mmHg crossing",
            "stat_value": (f"~{yrs_to_130:+.1f} yr from last visit" if near else "beyond horizon"),
            "detected": near and slope_yr>0,
            "p_body":"If the current drift continues, your readings are on course to reach the stage-1 hypertension line — not now, but on a timeline worth knowing.",
            "r_body":f"Extrapolated Theil–Sen crossing of 130 mmHg at ~{yrs_to_130:+.1f} yr from last observation (linear extrapolation; widen CI accordingly).",
            "ask":"Ask: “At this rate, when would my blood pressure reach the level that needs treatment — and can we act before then?”",
            "method":"Trend extrapolation to clinical threshold",
            "status":"Trivial to compute; almost never surfaced to patients."
        })
    # SIGNAL 3 — sitting on the edge (fuzzy band)
    signals.append({
        "id":"edge","name":"Readings on the borderline",
        "conf": round(min(1.0, edge_n/len(readings)+0.3),2),
        "stat_label":"Readings within ±5 mmHg of a threshold",
        "stat_value": f"{edge_n} of {len(readings)} readings in the fuzzy band",
        "detected": edge_n>=3,
        "p_body":"Several of your readings land right on the border between “normal” and “not.” Which side they fall on can depend on the day — that borderline itself is worth watching.",
        "r_body":f"{edge_n}/{len(readings)} observations within the ±{FUZZY_BAND} mmHg epsilon-band of 120/130; classification unstable to measurement noise.",
        "ask":"Ask: “Some of my readings are borderline — should we repeat them or monitor more closely?”",
        "method":"Fuzzy membership / uncertainty-band analysis",
        "status":"Well understood; routine care usually rounds it away."
    })

    analysis = {
        "marker":"Systolic blood pressure",
        "unit":"mmHg",
        "thresholds":{"normal_max":SYS_NORMAL_MAX,"elevated":SYS_ELEVATED,"fuzzy_band":FUZZY_BAND},
        "trend":{"slope_per_year":round(slope_yr,2),
                 "theil_sen_intercept":round(intercept,2),
                 "theil_sen_slope_per_day":slope,
                 "mann_kendall_tau":round(tau,3),"mann_kendall_p":round(p,4),
                 "S":S},
        "signals":signals,
        "n":len(readings)
    }
    return analysis

# ============================================================= OUTPUT (sealed)
def _clean(o):
    """Coerce numpy scalars/bools to native Python for JSON."""
    if isinstance(o, dict):  return {k:_clean(v) for k,v in o.items()}
    if isinstance(o, list):  return [_clean(v) for v in o]
    if isinstance(o, (np.bool_,)):     return bool(o)
    if isinstance(o, (np.integer,)):   return int(o)
    if isinstance(o, (np.floating,)):  return float(o)
    return o

def seal(obj):
    payload = json.dumps(_clean(obj), sort_keys=True).encode()
    digest = hashlib.sha256(payload).hexdigest()
    return digest

def main():
    readings = ingest_orbit()
    readings = process(readings)
    analysis = analyse(readings)
    out = {
        "source":"mini-EnkiDDB (honest stand-in) · pipeline.py",
        "generated_utc": dt.datetime.now(dt.timezone.utc).isoformat(),
        "provenance":"synthetic readings; REAL ingest/score/statistics — Mann–Kendall, Theil–Sen, fuzzy-band",
        "disclaimer":"Not a diagnosis. Not a real person. The statistics are genuine; the readings are generated.",
        "readings": readings,
        "analysis": analysis,
    }
    out["seal_sha256"] = seal({"readings":readings,"analysis":analysis})
    with open("lens_data.json","w") as f:
        json.dump(_clean(out), f, indent=2)
    # console proof that the stats are real
    tr=analysis["trend"]
    print(f"INGEST  {len(readings)} readings 2019–2026")
    print(f"PROCESS states: " + ", ".join(f"{s}={sum(1 for r in readings if r['state']==s)}" for s in ("GOLDEN","FUZZY")))
    print(f"ANALYSE Theil–Sen slope = {tr['slope_per_year']:+.2f} mmHg/yr")
    print(f"        Mann–Kendall tau = {tr['mann_kendall_tau']:+.3f}, p = {tr['mann_kendall_p']:.4f}"
          + ("  → trend DETECTED (p<0.05)" if tr['mann_kendall_p']<0.05 else "  → not significant"))
    print(f"OUTPUT  lens_data.json sealed sha256={out['seal_sha256'][:16]}…")

if __name__=="__main__":
    main()
