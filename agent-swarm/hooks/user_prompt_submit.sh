#!/usr/bin/env bash
# UserPromptSubmit → mark this agent busy (start of the busy/idle bracket).
set +e
cat >/dev/null
DIR="${CLAUDE_PROJECT_DIR:-$(git rev-parse --show-toplevel 2>/dev/null)}"
CLI="$DIR/vibeos-cli"
[ -x "$CLI" ] && bash "$CLI" swarm state busy >/dev/null 2>&1
exit 0
