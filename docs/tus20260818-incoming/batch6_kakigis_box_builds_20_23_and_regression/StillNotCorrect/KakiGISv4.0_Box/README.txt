KAKIGIS v4.0 — Box test edition
================================
HOW TO RUN (one step):
  ./run.sh
Then the browser opens http://127.0.0.1:8090 (or open it yourself).
NEVER open index.html via file:// — OSM tiles 403 without a Referer;
  the app now hard-refuses and tells you what to run.
  If run.sh is not executable after unzip, use:  bash run.sh

WHAT IT DOES:
- Vendored Leaflet 1.9.4 (BSD-2) — local, no CDN.
- Basemap: real OpenStreetMap tiles ((c) OpenStreetMap contributors,
  ODbL) — needs internet.
- REAL Wadi-us-Salaam boundary: loads data/yard.geojson if present,
  otherwise fetches OSM way 325851897 LIVE from the OSM API.
- Press "Use mask" after sizing the view: the {7,3} HeptaSpaceMap
  generates on the fly for that view, clipped to the real yard;
  after the first press it regenerates automatically on zoom/pan.
- Search a plot: Palahu veil arrival — no flight over the graves,
  exact per cm (flyTo is disabled in code).
- Plots are mock until the SilaEngine service (127.0.0.1:7010) runs.

Zero AI in this runtime.
