#!/usr/bin/env bash
# Usage: ./bundle_tiles.sh <extract.osm.pbf> <bbox: minlon,minlat,maxlon,maxlat>
# Example (Baghdad core): ./bundle_tiles.sh iraq-latest.osm.pbf 44.25,33.20,44.60,33.55
# Example (Amsterdam demo): ./bundle_tiles.sh nl-noord-holland.osm.pbf 4.78,52.32,4.98,52.42
set -euo pipefail
PBF="${1:?need .osm.pbf extract}"
BBOX="${2:?need bbox minlon,minlat,maxlon,maxlat}"
OUT="/home/bfadam/Forge/EnkiDB/playbooks/../workspace/bahyway_v4/apps/gula-mobile/assets/tiles"
command -v osmium >/dev/null || { echo "install osmium-tool (dnf install osmium-tool)"; exit 1; }
osmium extract -b "$BBOX" "$PBF" -o /tmp/gula_bbox.osm.pbf --overwrite
# Buildings (extruded in Godot city LOD) and the drivable/walkable graph
osmium tags-filter /tmp/gula_bbox.osm.pbf a/building -o /tmp/gula_buildings.osm.pbf --overwrite
osmium tags-filter /tmp/gula_bbox.osm.pbf w/highway -o /tmp/gula_roads.osm.pbf --overwrite
osmium export /tmp/gula_buildings.osm.pbf -o "$OUT/buildings.geojson" --overwrite
osmium export /tmp/gula_roads.osm.pbf -o "$OUT/roads.geojson" --overwrite
echo "Bundled: $OUT/buildings.geojson + roads.geojson"
echo "roads.geojson also feeds the pure-Rust route core (contraction-hierarchy sibling of the Wadi al-Salam navigator)."
