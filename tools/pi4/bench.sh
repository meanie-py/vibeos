#!/usr/bin/env bash
# CLI adapter for native, leased Pi 4 bench services and payload staging.
# LAN HTTP/TFTP run on Windows; Pico TCP capture runs on Linux.
set -euo pipefail
HERE=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)
REPO=$(cd "$HERE/../.." && pwd)
case "${1:-status}" in
    stage) shift; exec "$REPO/vibeos-cli" pi4 stage "$@" ;;
    up|http|status|down|serial-restart)
        exec "$REPO/vibeos-cli" pi4 services "${1:-status}" ;;
    *) echo "usage: $0 up|http|status|down|serial-restart|stage" >&2; exit 2 ;;
esac
