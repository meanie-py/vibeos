#!/usr/bin/env bash
# Device environment adapter; staging policy is implemented in native Rust.
set -euo pipefail
HERE=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)
REPO=$(cd "$HERE/../.." && pwd)
. "$HERE/env.sh"
exec "$REPO/vibeos-cli" pi5 stage "$@"
