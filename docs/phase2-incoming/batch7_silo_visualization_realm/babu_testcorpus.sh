#!/usr/bin/env bash
# =====================================================================
# babu_testcorpus.sh — build a layered test corpus for the Bābu gate.
# Exercises BOTH paths: clean files -> Karû, EICAR-flagged -> Ṣibittu.
# The EICAR string is the industry-standard HARMLESS anti-malware test
# pattern: it is inert (does nothing) but every AV flags it. We never
# ship real malware. Run: ./babu_testcorpus.sh /tmp/babu_test
# =====================================================================
set -euo pipefail
OUT="${1:-/tmp/babu_test}"
mkdir -p "$OUT"/{clean,flagged,nested,mixed}
cd "$OUT"

# --- EICAR standard test string (assembled at runtime so THIS script
#     itself is not flagged by your editor's scanner) --------------------
E1='X5O!P%@AP[4\PZX54(P^)7CC)7}$'
E2='EICAR-STANDARD-ANTIVIRUS-TEST-FILE!$H+H*'
printf '%s%s' "$E1" "$E2" > flagged/eicar_trigger.txt

# --- clean files: varied formats to test the SLA format test ----------
printf 'Amoxicillin 500mg — patient leaflet (clean test doc).\n' > clean/leaflet_en.txt
printf 'أموكسيسيلين ٥٠٠ ملغ — نشرة المريض (ملف اختبار نظيف).\n'   > clean/leaflet_ar.txt
printf 'batch_id,product,expiry_days,assay_pct\nBGH-1000,Amoxicillin,400,99.2\n' > clean/batches.csv
printf '{"corpus":"test","chunk":"clean json particle"}\n'       > clean/record.json
head -c 4096 /dev/urandom > clean/waveform.bin   # binary -> asset annex path

# --- clean archives (extraction rite, no threat) ----------------------
( cd clean && zip -q ../mixed/clean_bundle.zip ./*.txt ./*.csv ./*.json )
tar -czf mixed/clean_bundle.tar.gz -C clean .

# --- flagged archive: EICAR zipped (tests scan-after-extract) ---------
( cd flagged && zip -q ../mixed/flagged_bundle.zip eicar_trigger.txt )

# --- nested archive: EICAR two layers deep (tests recursive B-1 rite) --
cp flagged/eicar_trigger.txt nested/
( cd nested && zip -q inner.zip eicar_trigger.txt && zip -q outer.zip inner.zip && rm -f inner.zip eicar_trigger.txt )

# --- mixed archive: clean + flagged together (tests per-file verdict) --
mkdir -p _m && cp clean/leaflet_en.txt clean/batches.csv flagged/eicar_trigger.txt _m/
( cd _m && zip -q ../mixed/mixed_bundle.zip ./* ) && rm -rf _m

cat > MANIFEST.txt <<EOF
Bābu test corpus — generated $(date -u +%FT%TZ)
clean/           5 clean files (en/ar text, csv, json, binary)
mixed/clean_bundle.zip, clean_bundle.tar.gz   -> expect: all Karû
mixed/flagged_bundle.zip                       -> expect: Ṣibittu (1 jailed)
nested/outer.zip (EICAR 2 layers deep)         -> expect: Ṣibittu after recursive extract
mixed/mixed_bundle.zip (clean + EICAR)         -> expect: split — clean to Karû, EICAR jailed
EOF

echo "Test corpus built at $OUT"
find "$OUT" -type f | sort
echo
echo "EICAR = harmless industry-standard AV test string. No real malware present."
