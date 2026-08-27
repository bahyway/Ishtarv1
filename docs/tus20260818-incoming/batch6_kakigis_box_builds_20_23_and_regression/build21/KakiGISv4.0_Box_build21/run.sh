#!/usr/bin/env bash
# KAKIGIS v4.0 Box — one-test runner. Serves over HTTP (file:// is refused).
cd "$(dirname "$0")"
( sleep 1; xdg-open http://127.0.0.1:8090 2>/dev/null || true ) &
echo "KAKIGIS Box serving at http://127.0.0.1:8090  (Ctrl+C to stop)"
python3 -m http.server 8090 --bind 127.0.0.1
