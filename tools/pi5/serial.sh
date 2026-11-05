#!/usr/bin/env bash
# Environment/argument adapter for the native leased Pico listener lifecycle.
set -euo pipefail
HERE=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)
REPO=$(cd "$HERE/../.." && pwd)
. "$HERE/env.sh"
exec "$REPO/vibeos-cli" pi5 serial "${1:-status}"
