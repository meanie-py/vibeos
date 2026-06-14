#!/usr/bin/env bash
# SessionEnd → mark this agent offline.
set +e
cat >/dev/null
DIR="${CLAUDE_PROJECT_DIR:-$(git rev-parse --show-toplevel 2>/dev/null)}"
SWARM="$DIR/bin/swarm"; [ -x "$SWARM" ] || SWARM="$(command -v swarm)"
[ -n "$SWARM" ] && bash "$SWARM" offline >/dev/null 2>&1
exit 0
