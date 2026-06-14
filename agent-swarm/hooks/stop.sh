#!/usr/bin/env bash
# Stop → mark this agent idle (end of the busy/idle bracket). Idle delivery now routes
# through the MCP channel.
set +e
cat >/dev/null
DIR="${CLAUDE_PROJECT_DIR:-$(git rev-parse --show-toplevel 2>/dev/null)}"
SWARM="$DIR/bin/swarm"; [ -x "$SWARM" ] || SWARM="$(command -v swarm)"
[ -n "$SWARM" ] && bash "$SWARM" state idle >/dev/null 2>&1
exit 0
